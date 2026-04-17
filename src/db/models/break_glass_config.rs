use crate::util;
use crate::db::schema::break_glass_configs;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = break_glass_configs)]
#[diesel(primary_key(uuid))]
pub struct BreakGlassConfig {
    pub uuid: String,
    pub user_uuid: String,
    pub witness_uuids: String, // JSON array
    pub notification_emails: String, // JSON array
    pub session_duration_hours: i32,
}

impl BreakGlassConfig {
    pub fn new(user_uuid: String, witness_uuids: String, notification_emails: String, session_duration_hours: i32) -> Self {
        Self {
            uuid: util::get_uuid(),
            user_uuid,
            witness_uuids,
            notification_emails,
            session_duration_hours,
        }
    }
}

impl BreakGlassConfig {
    pub async fn save(&self, conn: &DbConn) -> EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(break_glass_configs::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(break_glass_configs::table)
                            .filter(break_glass_configs::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving break glass config")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving break glass config")
            }
            postgresql {
                diesel::insert_into(break_glass_configs::table)
                    .values(self)
                    .on_conflict(break_glass_configs::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving break glass config")
            }
        }
    }

    pub async fn delete(self, conn: &DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(break_glass_configs::table.filter(break_glass_configs::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting break glass config")
        }}
    }

    pub async fn find_by_uuid(uuid: &str, conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            break_glass_configs::table
                .filter(break_glass_configs::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn find_by_user(user_uuid: &str, conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            break_glass_configs::table
                .filter(break_glass_configs::user_uuid.eq(user_uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }
}
