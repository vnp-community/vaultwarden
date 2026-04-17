use std::time::Duration;
use tokio::time::sleep;
use serde_json::Value;

use crate::db::DbConn;
use crate::db::models::audit::AuditEntry;
use crate::CONFIG;

pub struct SiemForwarder;

impl SiemForwarder {
    pub fn start(pool: crate::db::DbPool) {
        if !CONFIG.audit_siem_enabled() {
            return;
        }

        tokio::spawn(async move {
            info!("SIEM Forwarder background task started");
            loop {
                // Polling interval
                sleep(Duration::from_secs(30)).await;

                if let Ok(mut conn) = pool.get().await {
                    let _unused = SiemForwarder::deliver_pending(&mut conn).await;
                }
            }
        });
    }

    async fn deliver_pending(conn: &mut DbConn) -> Result<(), crate::error::Error> {
        use crate::db::schema::audit_entries;
        use diesel::prelude::*;

        let pending_entries = db_run! { conn: {
            audit_entries::table
                .filter(audit_entries::siem_delivered.eq(false))
                .filter(audit_entries::siem_attempts.lt(5))
                .order_by(audit_entries::id.asc())
                .limit(50)
                .load::<AuditEntry>(conn)
                .ok()
        }}
        .unwrap_or_default();

        if pending_entries.is_empty() {
            return Ok(());
        }

        // Just logging for now as a mock for actual Splunk HEC delivery
        debug!("SIEM: Processing {} pending audit entries", pending_entries.len());

        let mut delivered_ids = Vec::new();
        for entry in pending_entries {
            // SiemDelivery format
            let payload: Value = json!({
                "time": entry.timestamp.and_utc().timestamp(),
                "host": "vaultwarden",
                "source": "audit_log",
                "sourcetype": "_json",
                "event": {
                    "event_type": entry.event_type,
                    "severity": entry.severity,
                    "actor_uuid": entry.actor_user_uuid,
                    "actor_email": entry.actor_email,
                    "target_resource": entry.target_resource,
                    "ip_address": entry.ip_address,
                    "metadata": entry.metadata,
                }
            });

            // Mock success delivery
            if SiemForwarder::send_to_splunk(&payload).await.is_ok() {
                delivered_ids.push(entry.id);
            } else {
                // Increment attempts
                let _unused = db_run! { conn: {
                    diesel::update(audit_entries::table.filter(audit_entries::id.eq(entry.id)))
                        .set(audit_entries::siem_attempts.eq(audit_entries::siem_attempts + 1))
                        .execute(conn)
                }};
            }
        }

        if !delivered_ids.is_empty() {
            let _unused = db_run! { conn: {
                diesel::update(audit_entries::table.filter(audit_entries::id.eq_any(delivered_ids)))
                    .set(audit_entries::siem_delivered.eq(true))
                    .execute(conn)
            }};
        }

        Ok(())
    }

    async fn send_to_splunk(_payload: &Value) -> Result<(), ()> {
        // Implement HTTP POST to Splunk HEC using reqwest here.
        // For currently mock delivery, just return Ok
        Ok(())
    }
}
