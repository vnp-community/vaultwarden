use crate::util;
use crate::db::schema::{webhooks, webhook_deliveries};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;

#[derive(Debug, Identifiable, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = webhooks)]
#[diesel(primary_key(uuid))]
pub struct Webhook {
    pub uuid: String,
    pub org_uuid: String,
    pub name: String,
    pub url: String,
    pub secret_hash: String,
    pub events: String, // JSON Array
    pub is_active: bool,
    pub retry_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = webhook_deliveries)]
#[diesel(primary_key(uuid))]
pub struct WebhookDelivery {
    pub uuid: String,
    pub webhook_uuid: String,
    pub event_type: String,
    pub payload: String, // JSON
    pub status: String, // "pending", "delivered", "failed"
    pub attempt_count: i32,
    pub last_attempt_at: Option<NaiveDateTime>,
    pub next_attempt_at: Option<NaiveDateTime>,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
}

impl WebhookDelivery {
    pub fn new(webhook_uuid: String, event_type: String, payload: String) -> Self {
        Self {
            uuid: util::get_uuid(),
            webhook_uuid,
            event_type,
            payload,
            status: "pending".to_string(),
            attempt_count: 0,
            last_attempt_at: None,
            next_attempt_at: None,
            error_message: None,
            created_at: Utc::now().naive_utc(),
        }
    }

    pub async fn save(&mut self, conn: &mut DbConn) -> EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(webhook_deliveries::table)
                    .values(&*self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(webhook_deliveries::table)
                            .filter(webhook_deliveries::uuid.eq(&self.uuid))
                            .set(&*self)
                            .execute(conn)
                            .map_res("Error saving webhook delivery")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving webhook delivery")
            }
            postgresql {
                diesel::insert_into(webhook_deliveries::table)
                    .values(&*self)
                    .on_conflict(webhook_deliveries::uuid)
                    .do_update()
                    .set(&*self)
                    .execute(conn)
                    .map_res("Error saving webhook delivery")
            }
        }
    }
}

impl Webhook {
    pub fn new(org_uuid: String, name: String, url: String, secret_hash: String, events: Vec<String>) -> Self {
        let now = Utc::now().naive_utc();
        
        let events_json = serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string());
        
        Self {
            uuid: util::get_uuid(),
            org_uuid,
            name,
            url,
            secret_hash,
            events: events_json,
            is_active: true,
            retry_count: 3,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "Id": self.uuid,
            "OrganizationId": self.org_uuid,
            "Name": self.name,
            "Url": self.url,
            "Events": serde_json::from_str::<serde_json::Value>(&self.events).unwrap_or(serde_json::json!([])),
            "IsActive": self.is_active,
            "RetryCount": self.retry_count,
            "CreationDate": util::format_date(&self.created_at),
            "RevisionDate": util::format_date(&self.updated_at),
            "Object": "webhook"
        })
    }

    pub async fn save(&mut self, conn: &mut DbConn) -> EmptyResult {
        self.updated_at = Utc::now().naive_utc();
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(webhooks::table)
                    .values(&*self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(webhooks::table)
                            .filter(webhooks::uuid.eq(&self.uuid))
                            .set(&*self)
                            .execute(conn)
                            .map_res("Error saving webhook")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving webhook")
            }
            postgresql {
                diesel::insert_into(webhooks::table)
                    .values(&*self)
                    .on_conflict(webhooks::uuid)
                    .do_update()
                    .set(&*self)
                    .execute(conn)
                    .map_res("Error saving webhook")
            }
        }
    }

    pub async fn find_by_uuid(uuid: &str, conn: &mut DbConn) -> Option<Self> {
        db_run! { conn: {
            webhooks::table
                .filter(webhooks::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }
    
    pub async fn find_all_for_org(org_uuid: &str, conn: &mut DbConn) -> Vec<Self> {
        db_run! { conn: {
            webhooks::table
                .filter(webhooks::org_uuid.eq(org_uuid))
                .load::<Self>(conn)
                .expect("Error loading webhooks for org")
        }}
    }

    pub async fn find_active_for_event(org_uuid: &str, event_type: &str, conn: &mut DbConn) -> Vec<Self> {
        db_run! { conn: {
            webhooks::table
                .filter(webhooks::org_uuid.eq(org_uuid))
                .filter(webhooks::is_active.eq(true))
                // Technically we should JSON array filter 'events' matching `event_type`, but SQLite makes json arrays hard
                // Vaultwarden usually just uses basic 'LIKE' searches for generic text-array fallbacks.
                .filter(webhooks::events.like(format!("%\"{}\"%", event_type)))
                .load::<Self>(conn)
                .expect("Error loading active webhooks for event")
        }}
    }

    pub async fn delete(self, conn: &mut DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(webhooks::table.filter(webhooks::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting webhook")
        }}
    }
}
