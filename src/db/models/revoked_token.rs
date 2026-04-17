// TASK-SEC-HIGH-02-F: RevokedToken model for opt-in JWT revocation.
// Each row stores the JTI (JWT ID) of a token that has been explicitly revoked.
// Entries are cleaned up daily by the revoked_token_cleanup_job (HIGH-02-G).
//
// This model is only used when CONFIG.token_revocation_enabled() == true.

use chrono::{NaiveDateTime, Utc};

use crate::api::EmptyResult;
use crate::db::schema::revoked_tokens;
use crate::db::DbConn;
use crate::error::MapResult;
use diesel::prelude::*;

pub type UserId = crate::db::models::user::UserId;

#[derive(Identifiable, Queryable, Insertable)]
#[diesel(table_name = revoked_tokens)]
#[diesel(primary_key(jti))]
pub struct RevokedToken {
    /// The JWT ID claim — a UUID string issued when the JWT was created.
    pub jti: String,
    /// The user this token belongs to. Cascade-deleted when user is deleted.
    pub user_uuid: UserId,
    /// Timestamp when the token was explicitly revoked.
    pub revoked_at: NaiveDateTime,
    /// Expiry from the JWT exp claim. Used by cleanup job to bound table size.
    pub expires_at: NaiveDateTime,
}

impl RevokedToken {
    /// Create a new revocation record.
    pub fn new(jti: String, user_uuid: UserId, expires_at: NaiveDateTime) -> Self {
        RevokedToken {
            jti,
            user_uuid,
            revoked_at: Utc::now().naive_utc(),
            expires_at,
        }
    }
}

/// Database methods
impl RevokedToken {
    /// Insert a revocation record. Silently succeeds if jti already exists (idempotent).
    pub async fn insert(jti: &str, user_uuid: &UserId, expires_at: NaiveDateTime, conn: &DbConn) -> EmptyResult {
        let record = RevokedToken::new(jti.to_string(), user_uuid.clone(), expires_at);
        db_run! { conn:
            sqlite, mysql {
                diesel::replace_into(revoked_tokens::table)
                    .values(&record)
                    .execute(conn)
                    .map_res("Error inserting revoked token")
            }
            postgresql {
                diesel::insert_into(revoked_tokens::table)
                    .values(&record)
                    .on_conflict(revoked_tokens::jti)
                    .do_nothing()
                    .execute(conn)
                    .map_res("Error inserting revoked token")
            }
        }
    }

    /// Returns true if the given jti is in the revocation list.
    pub async fn exists(jti: &str, conn: &DbConn) -> bool {
        db_run! { conn: {
            revoked_tokens::table
                .filter(revoked_tokens::jti.eq(jti))
                .count()
                .first::<i64>(conn)
                .unwrap_or(0) > 0
        }}
    }

    /// Delete all expired revocation entries (where expires_at < now).
    /// Called by the daily cleanup job (HIGH-02-G).
    pub async fn delete_expired(conn: &DbConn) -> EmptyResult {
        let now = Utc::now().naive_utc();
        db_run! { conn: {
            diesel::delete(revoked_tokens::table.filter(revoked_tokens::expires_at.lt(now)))
                .execute(conn)
                .map_res("Error deleting expired revoked tokens")
        }}
    }

    /// Revoke all tokens for a user (e.g., on password change or logout-all).
    /// In practice, security_stamp rotation already invalidates all JWTs for a user.
    /// This method is provided for explicit mass-revocation during the opt-in phase.
    pub async fn revoke_all_for_user(user_uuid: &UserId, conn: &DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(revoked_tokens::table.filter(revoked_tokens::user_uuid.eq(user_uuid)))
                .execute(conn)
                .map_res("Error deleting revoked tokens for user")
        }}
    }
}
