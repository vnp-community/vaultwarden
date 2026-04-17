use serde::{Deserialize, Serialize};

use crate::util;
use crate::db::schema::custom_roles;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Permission {
    ViewCollectionItems,
    EditCollectionItems,
    InviteMembers,
    ManageOrgSettings,
    ViewPrivilegedItems,
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = custom_roles)]
#[diesel(primary_key(uuid))]
pub struct CustomRole {
    pub uuid: String,
    pub org_uuid: String,
    pub name: String,
    pub permissions: String, // Serde serialized JSON
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl CustomRole {
    pub fn new(org_uuid: String, name: String, permissions: Vec<Permission>) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            uuid: util::get_uuid(),
            org_uuid,
            name,
            permissions: serde_json::to_string(&permissions).unwrap_or_else(|_| "[]".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        if let Ok(perms) = serde_json::from_str::<Vec<Permission>>(&self.permissions) {
            perms.contains(permission)
        } else {
            false
        }
    }
}

impl CustomRole {
    pub async fn save(&self, conn: &DbConn) -> EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(custom_roles::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(custom_roles::table)
                            .filter(custom_roles::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving custom role")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving custom role")
            }
            postgresql {
                diesel::insert_into(custom_roles::table)
                    .values(self)
                    .on_conflict(custom_roles::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving custom role")
            }
        }
    }

    pub async fn delete(self, conn: &DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(custom_roles::table.filter(custom_roles::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting custom role")
        }}
    }

    pub async fn find_by_uuid(uuid: &str, conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            custom_roles::table
                .filter(custom_roles::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn find_by_org(org_uuid: &str, conn: &DbConn) -> Vec<Self> {
        db_run! { conn: {
            custom_roles::table
                .filter(custom_roles::org_uuid.eq(org_uuid))
                .load::<Self>(conn)
                .expect("Error loading custom roles for org")
        }}
    }
}
