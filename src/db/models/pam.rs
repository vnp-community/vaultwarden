use crate::db::schema::{checkouts, privileged_configs, rotation_history};
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = privileged_configs)]
#[diesel(treat_none_as_null = true)]
#[diesel(primary_key(uuid))]
pub struct PrivilegedConfig {
    pub uuid: String,
    pub cipher_uuid: String,
    pub requires_approval: bool,
    pub max_checkout_duration: Option<i32>,
    pub auto_rotate_after_checkout: bool,
    pub rotation_target_type: Option<String>,
    pub rotation_target_config: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = checkouts)]
#[diesel(treat_none_as_null = true)]
#[diesel(primary_key(uuid))]
pub struct Checkout {
    pub uuid: String,
    pub cipher_uuid: String,
    pub user_uuid: String,
    pub justification: String,
    pub itsm_ticket: Option<String>,
    pub approval_request_uuid: Option<String>,
    pub checked_out_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub checked_in_at: Option<NaiveDateTime>,
    pub access_count: i32,
    pub status: String,
    pub rotation_triggered: bool,
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = rotation_history)]
#[diesel(treat_none_as_null = true)]
#[diesel(primary_key(uuid))]
pub struct RotationHistory {
    pub uuid: String,
    pub cipher_uuid: String,
    pub checkout_uuid: Option<String>,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub status: String,
    pub error_message: Option<String>,
}

use diesel::prelude::*;
use crate::db::DbConn;
use crate::error::MapResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RotationTargetConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub database: Option<String>,
}

impl PrivilegedConfig {
    pub fn get_rotation_config(&self) -> Option<RotationTargetConfig> {
        self.rotation_target_config
            .as_ref()
            .and_then(|config_str| serde_json::from_str(config_str).ok())
    }

    pub async fn save(&self, conn: &mut DbConn) -> crate::api::EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(privileged_configs::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(privileged_configs::table)
                            .filter(privileged_configs::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving privileged config")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving privileged config")
            }
            postgresql {
                diesel::insert_into(privileged_configs::table)
                    .values(self)
                    .on_conflict(privileged_configs::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving privileged config")
            }
        }
    }

    pub async fn find_by_cipher(cipher_uuid: &str, conn: &mut DbConn) -> Option<Self> {
        let cipher_uuid = cipher_uuid.to_string();
        conn.run(move |c| {
            privileged_configs::table
                .filter(privileged_configs::cipher_uuid.eq(cipher_uuid))
                .first::<PrivilegedConfig>(c)
                .ok()
        }).await
    }
}

impl Checkout {
    pub async fn find_expired_active(conn: &mut DbConn) -> Vec<Self> {
        let now = chrono::Utc::now().naive_utc();
        conn.run(move |c| {
            checkouts::table
                .filter(checkouts::status.eq("active"))
                .filter(checkouts::expires_at.lt(now))
                .load::<Checkout>(c)
                .unwrap_or_default()
        }).await
    }

    pub async fn count_active_for_cipher(cipher_uuid: &str, conn: &mut DbConn) -> i64 {
        let cipher_uuid = cipher_uuid.to_string();
        conn.run(move |c| {
            checkouts::table
                .filter(checkouts::cipher_uuid.eq(cipher_uuid))
                .filter(checkouts::status.eq("active"))
                .count()
                .get_result::<i64>(c)
                .unwrap_or(0)
        }).await
    }

    pub async fn find_active_for_resource(user_uuid: &str, cipher_uuid: &str, conn: &mut DbConn) -> Option<Self> {
        let user_uuid = user_uuid.to_string();
        let cipher_uuid = cipher_uuid.to_string();
        conn.run(move |c| {
            checkouts::table
                .filter(checkouts::user_uuid.eq(user_uuid))
                .filter(checkouts::cipher_uuid.eq(cipher_uuid))
                .filter(checkouts::status.eq("active"))
                .first::<Checkout>(c)
                .ok()
        }).await
    }

    pub fn new(cipher_uuid: String, user_uuid: String, justification: String) -> Self {
        Self {
            uuid: crate::util::get_uuid(),
            cipher_uuid,
            user_uuid,
            justification,
            itsm_ticket: None,
            approval_request_uuid: None,
            checked_out_at: chrono::Utc::now().naive_utc(),
            expires_at: None,
            checked_in_at: None,
            access_count: 0,
            status: "active".to_string(),
            rotation_triggered: false,
        }
    }

    pub async fn save(&self, conn: &mut DbConn) -> crate::api::EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(checkouts::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(checkouts::table)
                            .filter(checkouts::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving checkout")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving checkout")
            }
            postgresql {
                diesel::insert_into(checkouts::table)
                    .values(self)
                    .on_conflict(checkouts::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving checkout")
            }
        }
    }
}

impl RotationHistory {
    pub fn new(cipher_uuid: String, checkout_uuid: Option<String>) -> Self {
        Self {
            uuid: crate::util::get_uuid(),
            cipher_uuid,
            checkout_uuid,
            started_at: chrono::Utc::now().naive_utc(),
            completed_at: None,
            status: "running".to_string(),
            error_message: None,
        }
    }

    pub async fn insert(&self, conn: &mut DbConn) -> crate::api::EmptyResult {
        db_run! { conn: {
            diesel::insert_into(rotation_history::table)
                .values(self)
                .execute(conn)
                .map_res("Error inserting rotation history")
        }}
    }

    pub async fn save(&self, conn: &mut DbConn) -> crate::api::EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(rotation_history::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(rotation_history::table)
                            .filter(rotation_history::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving rotation history")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving rotation history")
            }
            postgresql {
                diesel::insert_into(rotation_history::table)
                    .values(self)
                    .on_conflict(rotation_history::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving rotation history")
            }
        }
    }
}

