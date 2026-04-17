use crate::util;
use crate::db::schema::ip_allowlists;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = ip_allowlists)]
#[diesel(primary_key(uuid))]
pub struct IpAllowlist {
    pub uuid: String,
    pub org_uuid: Option<String>,
    pub cidr_ranges: String,
}

impl IpAllowlist {
    pub fn new(org_uuid: Option<String>, cidr_ranges: String) -> Self {
        Self {
            uuid: util::get_uuid(),
            org_uuid,
            cidr_ranges,
        }
    }
}

impl IpAllowlist {
    pub async fn save(&self, conn: &DbConn) -> EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(ip_allowlists::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(ip_allowlists::table)
                            .filter(ip_allowlists::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving ip allowlist")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving ip allowlist")
            }
            postgresql {
                diesel::insert_into(ip_allowlists::table)
                    .values(self)
                    .on_conflict(ip_allowlists::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving ip allowlist")
            }
        }
    }

    pub async fn delete(self, conn: &DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(ip_allowlists::table.filter(ip_allowlists::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting ip allowlist")
        }}
    }

    pub async fn find_by_uuid(uuid: &str, conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            ip_allowlists::table
                .filter(ip_allowlists::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn find_by_org(org_uuid: &str, conn: &DbConn) -> Vec<Self> {
        db_run! { conn: {
            ip_allowlists::table
                .filter(ip_allowlists::org_uuid.eq(org_uuid))
                .load::<Self>(conn)
                .expect("Error loading ip allowlists for org")
        }}
    }

    pub async fn find_globals(conn: &DbConn) -> Vec<Self> {
        db_run! { conn: {
            ip_allowlists::table
                .filter(ip_allowlists::org_uuid.is_null())
                .load::<Self>(conn)
                .expect("Error loading global ip allowlists")
        }}
    }
}
