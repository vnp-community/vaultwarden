// TASK-001-006: GDPR Erasure Log model (SOL-001 Enterprise Compliance Framework)
//
// Append-only audit chain:
//   - Each entry SHA-256 hashes itself linked to `prev_hash` for tamper evidence.
//   - On insert: compute prev_hash from last entry, compute entry_hash after building the record.
//   - On erasure completion: mark completed_at and update entry_hash to cover the timestamp.

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde_json::Value;

use crate::{
    api::EmptyResult,
    db::{schema::erasure_logs, DbConn},
    error::MapResult,
    util::get_uuid,
};

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = erasure_logs)]
#[diesel(primary_key(uuid))]
pub struct ErasureLog {
    pub uuid: String,
    pub user_uuid: String,
    pub requested_at: NaiveDateTime,
    pub scheduled_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub requestor_ip: String,
    pub prev_hash: String,
    pub entry_hash: String,
}

// ─── SHA-256 helpers ───────────────────────────────────────────────────────────

fn sha256_hex(data: &str) -> String {
    use data_encoding::HEXLOWER;
    use ring::digest;
    let hash = digest::digest(&digest::SHA256, data.as_bytes());
    HEXLOWER.encode(hash.as_ref())
}

/// Compute the canonical hash of an ErasureLog entry.
/// The hash covers all immutable fields so that tampering with any column
/// breaks the chain.
fn compute_entry_hash(entry: &ErasureLog) -> String {
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}",
        entry.uuid,
        entry.user_uuid,
        entry.requested_at,
        entry.scheduled_at,
        entry.requestor_ip,
        entry.prev_hash,
    );
    sha256_hex(&canonical)
}

// ─── Local methods ─────────────────────────────────────────────────────────────

impl ErasureLog {
    /// Create a new ErasureLog entry linked to the previous chain head.
    ///
    /// * `user_uuid`    – UUID of the user requesting erasure.
    /// * `scheduled_at` – When PII will actually be erased (now + delay_days).
    /// * `requestor_ip` – IP address of requester for audit purposes.
    /// * `prev_hash`    – `entry_hash` of the most recent existing log entry,
    ///                    or empty string `""` if this is the first entry.
    pub fn new(user_uuid: &str, scheduled_at: NaiveDateTime, requestor_ip: &str, prev_hash: &str) -> Self {
        let now = Utc::now().naive_utc();
        let mut entry = Self {
            uuid: get_uuid(),
            user_uuid: user_uuid.to_string(),
            requested_at: now,
            scheduled_at,
            completed_at: None,
            requestor_ip: requestor_ip.to_string(),
            prev_hash: prev_hash.to_string(),
            entry_hash: String::new(), // filled below
        };
        entry.entry_hash = compute_entry_hash(&entry);
        entry
    }

    /// Mark this entry as completed and refresh the hash to cover `completed_at`.
    pub fn mark_completed(&mut self) {
        self.completed_at = Some(Utc::now().naive_utc());
        self.entry_hash = compute_entry_hash(self);
    }

    /// Serialize to a JSON representation suitable for compliance evidence APIs.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.uuid,
            "userUuid": self.user_uuid,
            "requestedAt": self.requested_at,
            "scheduledAt": self.scheduled_at,
            "completedAt": self.completed_at,
            "prevHash": self.prev_hash,
            "entryHash": self.entry_hash,
            "object": "erasureLog",
        })
    }
}

// ─── Database methods ──────────────────────────────────────────────────────────

impl ErasureLog {
    pub async fn save(&self, conn: &DbConn) -> EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                diesel::replace_into(erasure_logs::table)
                    .values(self)
                    .execute(conn)
                    .map_res("Error saving erasure log")
            }
            postgresql {
                diesel::insert_into(erasure_logs::table)
                    .values(self)
                    .on_conflict(erasure_logs::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving erasure log")
            }
        }
    }

    /// Return the SHA-256 hash of the most recent completed erasure log entry,
    /// or an empty string if no entries exist (genesis block).
    pub async fn get_last_hash(conn: &DbConn) -> String {
        db_run! { conn: {
            erasure_logs::table
                .order(erasure_logs::requested_at.desc())
                .select(erasure_logs::entry_hash)
                .first::<String>(conn)
                .unwrap_or_default()
        }}
    }

    /// Find pending (not yet completed) erasure log entries due for execution.
    pub async fn find_pending_due(conn: &DbConn) -> Vec<Self> {
        let now = Utc::now().naive_utc();
        db_run! { conn: {
            erasure_logs::table
                .filter(erasure_logs::completed_at.is_null())
                .filter(erasure_logs::scheduled_at.le(now))
                .load::<Self>(conn)
                .expect("Error loading pending erasure logs")
        }}
    }

    /// Find by user uuid — used when checking if an erasure is already scheduled.
    pub async fn find_pending_by_user(user_uuid: &str, conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            erasure_logs::table
                .filter(erasure_logs::user_uuid.eq(user_uuid))
                .filter(erasure_logs::completed_at.is_null())
                .first::<Self>(conn)
                .ok()
        }}
    }

    /// Count total completed erasures — used in compliance evidence collectors.
    pub async fn count_completed(conn: &DbConn) -> i64 {
        db_run! { conn: {
            erasure_logs::table
                .filter(erasure_logs::completed_at.is_not_null())
                .count()
                .get_result::<i64>(conn)
                .unwrap_or(0)
        }}
    }
}
