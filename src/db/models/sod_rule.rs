use crate::util;
use crate::db::schema::sod_rules;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = sod_rules)]
#[diesel(primary_key(uuid))]
pub struct SodRule {
    pub uuid: String,
    pub org_uuid: String,
    pub role_a_uuid: String,
    pub role_b_uuid: String,
    pub enforcement: String, // "soft" or "hard"
}

impl SodRule {
    pub fn new(org_uuid: String, role_a_uuid: String, role_b_uuid: String, enforcement: String) -> Self {
        Self {
            uuid: util::get_uuid(),
            org_uuid,
            role_a_uuid,
            role_b_uuid,
            enforcement,
        }
    }
}

impl SodRule {
    pub async fn save(&self, conn: &DbConn) -> EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(sod_rules::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(sod_rules::table)
                            .filter(sod_rules::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving sod rule")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving sod rule")
            }
            postgresql {
                diesel::insert_into(sod_rules::table)
                    .values(self)
                    .on_conflict(sod_rules::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving sod rule")
            }
        }
    }

    pub async fn delete(self, conn: &DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(sod_rules::table.filter(sod_rules::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting sod rule")
        }}
    }

    pub async fn find_by_uuid(uuid: &str, conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            sod_rules::table
                .filter(sod_rules::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn find_by_org(org_uuid: &str, conn: &DbConn) -> Vec<Self> {
        db_run! { conn: {
            sod_rules::table
                .filter(sod_rules::org_uuid.eq(org_uuid))
                .load::<Self>(conn)
                .expect("Error loading sod rules for org")
        }}
    }
}
