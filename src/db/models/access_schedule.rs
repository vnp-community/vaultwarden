use crate::util;
use crate::db::schema::access_schedules;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;
use chrono::NaiveTime;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = access_schedules)]
#[diesel(primary_key(uuid))]
pub struct AccessSchedule {
    pub uuid: String,
    pub org_uuid: Option<String>,
    pub user_uuid: Option<String>,
    pub timezone: String,
    pub allowed_days: i32,
    pub allowed_time_from: Option<NaiveTime>,
    pub allowed_time_until: Option<NaiveTime>,
}

impl AccessSchedule {
    pub fn new(org_uuid: Option<String>, user_uuid: Option<String>) -> Self {
        Self {
            uuid: util::get_uuid(),
            org_uuid,
            user_uuid,
            timezone: "UTC".to_string(),
            allowed_days: 127, // all 7 days by default bitmask
            allowed_time_from: None,
            allowed_time_until: None,
        }
    }
}

impl AccessSchedule {
    pub async fn save(&self, conn: &DbConn) -> EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(access_schedules::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(access_schedules::table)
                            .filter(access_schedules::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving access schedule")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving access schedule")
            }
            postgresql {
                diesel::insert_into(access_schedules::table)
                    .values(self)
                    .on_conflict(access_schedules::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving access schedule")
            }
        }
    }

    pub async fn delete(self, conn: &DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(access_schedules::table.filter(access_schedules::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting access schedule")
        }}
    }

    pub async fn find_by_uuid(uuid: &str, conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            access_schedules::table
                .filter(access_schedules::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn find_by_org(org_uuid: &str, conn: &DbConn) -> Vec<Self> {
        db_run! { conn: {
            access_schedules::table
                .filter(access_schedules::org_uuid.eq(org_uuid))
                .load::<Self>(conn)
                .expect("Error loading access schedules for org")
        }}
    }

    pub async fn find_by_user(user_uuid: &str, conn: &DbConn) -> Vec<Self> {
        db_run! { conn: {
            access_schedules::table
                .filter(access_schedules::user_uuid.eq(user_uuid))
                .load::<Self>(conn)
                .expect("Error loading access schedules for user")
        }}
    }
}
