use crate::util;
use crate::db::schema::{api_keys_v2, api_key_usage};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;

#[derive(Debug, Identifiable, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = api_keys_v2)]
#[diesel(primary_key(uuid))]
pub struct ApiKeyV2 {
    pub uuid: String,
    pub org_uuid: String,
    pub client_id: String,
    pub secret_hash: String,
    pub name: String,
    pub scopes: String, // JSON array
    pub allowed_ips: Option<String>, // JSON array
    pub rate_limit_minute: Option<i32>,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_used_at: Option<NaiveDateTime>,
    pub is_active: bool,
}

#[derive(Debug, Identifiable, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = api_key_usage)]
#[diesel(primary_key(id))]
pub struct ApiKeyUsage {
    pub id: String,
    pub api_key_uuid: String,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_ms: i32,
    pub timestamp: NaiveDateTime,
}

impl ApiKeyV2 {
    pub fn new(org_uuid: String, client_id: String, name: String, secret_hash: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            uuid: util::get_uuid(),
            org_uuid,
            client_id,
            secret_hash,
            name,
            scopes: "[]".to_string(),
            allowed_ips: None,
            rate_limit_minute: Some(60),
            created_at: now,
            updated_at: now,
            expires_at: None,
            last_used_at: None,
            is_active: true,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "Id": self.uuid,
            "OrganizationId": self.org_uuid,
            "ClientId": self.client_id,
            "Name": self.name,
            "Scopes": self.scopes,
            "AllowedIps": self.allowed_ips,
            "RateLimitMinute": self.rate_limit_minute,
            "ExpiresAt": self.expires_at,
            "CreationDate": util::format_date(&self.created_at),
            "RevisionDate": util::format_date(&self.updated_at),
            "LastUsedAt": self.last_used_at.as_ref().map(util::format_date),
            "IsActive": self.is_active,
            "Object": "apiKeyV2"
        })
    }

    pub fn verify_token(&self, secret_candidate: &str) -> bool {
        if let Ok(hash_bytes) = data_encoding::BASE64.decode(self.secret_hash.as_bytes()) {
            crate::crypto::verify_password_hash(
                secret_candidate.as_bytes(),
                self.client_id.as_bytes(),
                &hash_bytes,
                100_000, // Standard PBKDF2 iterations for API keys
            )
        } else {
            false
        }
    }

    pub async fn touch(&mut self, conn: &mut DbConn) -> EmptyResult {
        self.last_used_at = Some(Utc::now().naive_utc());
        self.save(conn).await
    }

    pub async fn save(&mut self, conn: &mut DbConn) -> EmptyResult {
        self.updated_at = Utc::now().naive_utc();
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(api_keys_v2::table)
                    .values(&*self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(api_keys_v2::table)
                            .filter(api_keys_v2::uuid.eq(&self.uuid))
                            .set(&*self)
                            .execute(conn)
                            .map_res("Error saving api key")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving api key")
            }
            postgresql {
                diesel::insert_into(api_keys_v2::table)
                    .values(&*self)
                    .on_conflict(api_keys_v2::uuid)
                    .do_update()
                    .set(&*self)
                    .execute(conn)
                    .map_res("Error saving api key")
            }
        }
    }

    pub async fn find_by_uuid(uuid: &str, conn: &mut DbConn) -> Option<Self> {
        db_run! { conn: {
            api_keys_v2::table
                .filter(api_keys_v2::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }
    
    pub async fn find_by_client_id(client_id: &str, conn: &mut DbConn) -> Option<Self> {
        db_run! { conn: {
            api_keys_v2::table
                .filter(api_keys_v2::client_id.eq(client_id))
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn find_all_for_org(org_uuid: &str, conn: &mut DbConn) -> Vec<Self> {
        db_run! { conn: {
            api_keys_v2::table
                .filter(api_keys_v2::org_uuid.eq(org_uuid))
                .load::<Self>(conn)
                .expect("Error loading api keys for org")
        }}
    }

    pub async fn delete(self, conn: &mut DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(api_keys_v2::table.filter(api_keys_v2::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting api key")
        }}
    }
}

impl ApiKeyUsage {
    pub async fn track_api_key_usage(
        api_key_uuid: String,
        endpoint: String,
        method: String,
        status_code: i32,
        response_ms: i32,
        conn: &mut DbConn,
    ) -> EmptyResult {
        let usage = Self {
            id: util::get_uuid(),
            api_key_uuid,
            endpoint,
            method,
            status_code,
            response_ms,
            timestamp: Utc::now().naive_utc(),
        };

        db_run! { conn: {
            diesel::insert_into(api_key_usage::table)
                .values(&usage)
                .execute(conn)
                .map_res("Error inserting API key usage tracking")
        }}
    }
}
