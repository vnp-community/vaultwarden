use crate::{
    api::{admin::AdminToken, JsonResult},
    db::DbConn,
    db_run,
};
use rocket::serde::json::Json;
use diesel::prelude::*;

#[get("/audit/verify-chain")]
pub async fn verify_chain(_token: AdminToken, conn: DbConn) -> JsonResult {
    let entries = match db_run! { conn: {
        crate::db::schema::audit_entries::table
            .order_by(crate::db::schema::audit_entries::id.asc())
            .load::<crate::db::models::audit::AuditEntry>(conn)
            .ok()
    }} {
        Some(e) => e,
        None => return Ok(Json(json!({"valid": true, "message": "No audit entries found."})))
    };

    let mut prev_hash: Option<Vec<u8>> = None;

    for entry in entries {
        if entry.prev_hash != prev_hash {
            return Ok(Json(json!({
                "valid": false,
                "broken_at_id": entry.id,
                "reason": "Previous hash does not match the actual previous entry's hash",
            })));
        }

        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;

        if let Some(ref ph) = prev_hash {
            hasher.update(ph);
        }
        hasher.update(entry.timestamp.and_utc().timestamp().to_be_bytes());
        hasher.update(entry.event_type.as_bytes());
        hasher.update(entry.actor_user_uuid.as_deref().unwrap_or("").as_bytes());
        hasher.update(entry.target_resource.as_deref().unwrap_or("").as_bytes());

        let computed_hash = hasher.finalize().to_vec();

        if computed_hash != entry.entry_hash {
            return Ok(Json(json!({
                "valid": false,
                "broken_at_id": entry.id,
                "reason": "Entry hash does not match computed content hash",
            })));
        }

        prev_hash = Some(entry.entry_hash);
    }

    Ok(Json(json!({
        "valid": true,
        "message": "Audit chain verified successfully."
    })))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![verify_chain]
}
