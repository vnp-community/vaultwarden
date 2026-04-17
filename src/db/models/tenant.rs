use crate::util;
use crate::db::schema::{tenants, tenant_admins};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;

#[derive(Debug, Identifiable, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = tenants)]
#[diesel(primary_key(uuid))]
pub struct Tenant {
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub domain_restriction: Option<String>,
    pub is_active: bool,
    pub max_users: Option<i32>,
    pub max_organizations: Option<i32>,
    pub max_vault_items: Option<i32>,
    pub max_storage_bytes: Option<i64>,
    pub config_overrides: Option<String>,
    pub branding: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Identifiable, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = tenant_admins)]
#[diesel(primary_key(tenant_uuid, user_uuid))]
pub struct TenantAdmin {
    pub tenant_uuid: String,
    pub user_uuid: String,
    pub created_at: NaiveDateTime,
}

impl Tenant {
    pub fn new(name: String, slug: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            uuid: util::get_uuid(),
            name,
            slug,
            domain_restriction: None,
            is_active: true,
            max_users: None,
            max_organizations: None,
            max_vault_items: None,
            max_storage_bytes: None,
            config_overrides: None,
            branding: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "Id": self.uuid,
            "Name": self.name,
            "Slug": self.slug,
            "DomainRestriction": self.domain_restriction,
            "IsActive": self.is_active,
            "MaxUsers": self.max_users,
            "MaxOrganizations": self.max_organizations,
            "MaxVaultItems": self.max_vault_items,
            "MaxStorageBytes": self.max_storage_bytes,
            "ConfigOverrides": self.config_overrides.as_ref().and_then(|c| serde_json::from_str::<serde_json::Value>(c).ok()),
            "Branding": self.branding.as_ref().and_then(|b| serde_json::from_str::<serde_json::Value>(b).ok()),
            "CreationDate": util::format_date(&self.created_at),
            "RevisionDate": util::format_date(&self.updated_at),
            "Object": "tenant"
        })
    }

    pub async fn save(&mut self, conn: &mut DbConn) -> EmptyResult {
        self.updated_at = Utc::now().naive_utc();
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(tenants::table)
                    .values(&*self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(tenants::table)
                            .filter(tenants::uuid.eq(&self.uuid))
                            .set(&*self)
                            .execute(conn)
                            .map_res("Error saving tenant")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving tenant")
            }
            postgresql {
                diesel::insert_into(tenants::table)
                    .values(&*self)
                    .on_conflict(tenants::uuid)
                    .do_update()
                    .set(&*self)
                    .execute(conn)
                    .map_res("Error saving tenant")
            }
        }
    }

    pub async fn find_by_uuid(uuid: &str, conn: &mut DbConn) -> Option<Self> {
        db_run! { conn: {
            tenants::table
                .filter(tenants::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }
    
    pub async fn find_by_slug(slug: &str, conn: &mut DbConn) -> Option<Self> {
        db_run! { conn: {
            tenants::table
                .filter(tenants::slug.eq(slug))
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn get_all(conn: &mut DbConn) -> Vec<Self> {
        db_run! { conn: {
            tenants::table
                .load::<Self>(conn)
                .expect("Error loading tenants")
        }}
    }

    pub async fn delete(self, conn: &mut DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(tenants::table.filter(tenants::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting tenant")
        }}
    }

    /// Validate a slug: lowercase alphanumeric with hyphens
    pub fn validate_slug(slug: &str) -> bool {
        !slug.is_empty()
            && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !slug.starts_with('-')
            && !slug.ends_with('-')
    }

    pub async fn count_users_in_tenant(tenant_uuid: &str, conn: &mut DbConn) -> i64 {
        use crate::db::schema::users;
        db_run! { conn: {
            users::table
                .filter(users::tenant_uuid.eq(tenant_uuid))
                .count()
                .get_result::<i64>(conn)
                .unwrap_or(0)
        }}
    }

    pub async fn count_orgs_in_tenant(tenant_uuid: &str, conn: &mut DbConn) -> i64 {
        use crate::db::schema::organizations;
        db_run! { conn: {
            organizations::table
                .filter(organizations::tenant_uuid.eq(tenant_uuid))
                .count()
                .get_result::<i64>(conn)
                .unwrap_or(0)
        }}
    }

    pub async fn count_ciphers_in_tenant(tenant_uuid: &str, conn: &mut DbConn) -> i64 {
        // Ciphers are counted via users in the tenant
        use crate::db::schema::{users, ciphers};
        db_run! { conn: {
            ciphers::table
                .inner_join(users::table.on(ciphers::user_uuid.eq(users::uuid.nullable())))
                .filter(users::tenant_uuid.eq(tenant_uuid))
                .count()
                .get_result::<i64>(conn)
                .unwrap_or(0)
        }}
    }
}

impl TenantAdmin {
    pub async fn save(tenant_uuid: String, user_uuid: String, conn: &mut DbConn) -> EmptyResult {
        let admin = Self {
            tenant_uuid,
            user_uuid,
            created_at: Utc::now().naive_utc(),
        };
        db_run! { conn:
            sqlite, mysql {
                diesel::replace_into(tenant_admins::table)
                    .values(&admin)
                    .execute(conn)
                    .map_res("Error saving tenant admin")
            }
            postgresql {
                diesel::insert_into(tenant_admins::table)
                    .values(&admin)
                    .on_conflict((tenant_admins::tenant_uuid, tenant_admins::user_uuid))
                    .do_nothing()
                    .execute(conn)
                    .map_res("Error saving tenant admin")
            }
        }
    }

    pub async fn delete(tenant_uuid: &str, user_uuid: &str, conn: &mut DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(
                tenant_admins::table
                    .filter(tenant_admins::tenant_uuid.eq(tenant_uuid))
                    .filter(tenant_admins::user_uuid.eq(user_uuid))
            )
            .execute(conn)
            .map_res("Error deleting tenant admin")
        }}
    }

    pub async fn find_for_user(user_uuid: &str, conn: &mut DbConn) -> Vec<Self> {
        db_run! { conn: {
            tenant_admins::table
                .filter(tenant_admins::user_uuid.eq(user_uuid))
                .load::<Self>(conn)
                .expect("Error loading tenant admins")
        }}
    }
}
