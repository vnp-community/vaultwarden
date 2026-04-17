/// TASK-008-010: Webhook CRUD routes + test endpoint + delivery log

use rocket::{serde::json::Json, Route};

use crate::{
    api::{EmptyResult, JsonResult},
    auth::AdminHeaders,
    db::{models::Webhook, DbConn},
    error::Error,
    util, CONFIG,
};

pub fn routes() -> Vec<Route> {
    routes![
        get_webhooks,
        create_webhook,
        update_webhook,
        test_webhook,
        delete_webhook,
        get_webhook_deliveries,
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /organizations/<org_id>/webhooks
// ─────────────────────────────────────────────────────────────────────────────

#[get("/organizations/<org_id>/webhooks")]
pub async fn get_webhooks(org_id: &str, headers: AdminHeaders, mut conn: DbConn) -> JsonResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let hooks = Webhook::find_all_for_org(org_id, &mut conn).await;
    let json: Vec<serde_json::Value> = hooks.iter().map(|w| w.to_json()).collect();

    Ok(Json(json!({
        "data": json,
        "object": "list",
        "continuationToken": null
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /organizations/<org_id>/webhooks
// ─────────────────────────────────────────────────────────────────────────────

#[post("/organizations/<org_id>/webhooks", data = "<data>")]
pub async fn create_webhook(
    org_id: &str,
    data: Json<serde_json::Value>,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> JsonResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let url = data["url"].as_str()
        .ok_or_else(|| Error::new("BadRequest", "url is required"))?
        .to_string();

    // Enforce HTTPS in production
    if CONFIG.domain().starts_with("https") && !url.starts_with("https://") {
        return Err(Error::new("BadRequest", "Webhook URL must use HTTPS"));
    }

    let name = data["name"].as_str().unwrap_or("Unnamed Webhook").to_string();
    let events: Vec<String> = data["events"].as_array()
        .map(|arr| arr.iter().filter_map(|e| e.as_str().map(str::to_string)).collect())
        .unwrap_or_default();

    // Generate a fresh webhook secret (returned once, stored as hash)
    let plain_secret = util::get_uuid();
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(plain_secret.as_bytes());
    let secret_hash = data_encoding::BASE64.encode(&h.finalize());

    let mut webhook = Webhook::new(org_id.to_string(), name, url, secret_hash, events);
    webhook.save(&mut conn).await?;

    let mut resp = webhook.to_json();
    resp["secret"] = json!(plain_secret);
    resp["note"] = json!("Store the secret securely — it will not be shown again.");

    info!("[Webhook] Created webhook '{}' for org {org_id}", webhook.name);
    Ok(Json(resp))
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /organizations/<org_id>/webhooks/<webhook_id>
// ─────────────────────────────────────────────────────────────────────────────

#[patch("/organizations/<org_id>/webhooks/<webhook_id>", data = "<data>")]
pub async fn update_webhook(
    org_id: &str,
    webhook_id: &str,
    data: Json<serde_json::Value>,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> JsonResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let mut webhook = Webhook::find_by_uuid(webhook_id, &mut conn).await
        .ok_or_else(|| Error::new("NotFound", "Webhook not found"))?;

    if webhook.org_uuid != org_id {
        return Err(Error::new("Forbidden", "Webhook does not belong to this org"));
    }

    if let Some(name) = data["name"].as_str() { webhook.name = name.to_string(); }
    if let Some(active) = data["isActive"].as_bool() { webhook.is_active = active; }
    if let Some(events) = data["events"].as_array() {
        let ev: Vec<String> = events.iter().filter_map(|e| e.as_str().map(str::to_string)).collect();
        webhook.events = serde_json::to_string(&ev).unwrap_or_default();
    }

    webhook.save(&mut conn).await?;
    Ok(Json(webhook.to_json()))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /organizations/<org_id>/webhooks/<webhook_id>/test
// ─────────────────────────────────────────────────────────────────────────────

#[post("/organizations/<org_id>/webhooks/<webhook_id>/test")]
pub async fn test_webhook(
    org_id: &str,
    webhook_id: &str,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> EmptyResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let webhook = Webhook::find_by_uuid(webhook_id, &mut conn).await
        .ok_or_else(|| Error::new("NotFound", "Webhook not found"))?;

    if webhook.org_uuid != org_id {
        return Err(Error::new("Forbidden", "Webhook does not belong to this org"));
    }

    let ping_payload = json!({
        "type": "ping",
        "orgUuid": org_id,
        "webhookId": webhook_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let url = webhook.url.clone();
    let secret = webhook.secret_hash.clone();
    let payload_str = serde_json::to_string(&ping_payload).unwrap_or_default();
    let delivery_id = util::get_uuid();

    // Fire-and-forget ping delivery (no retry for test pings)
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let sig = crate::webhook_delivery::sign_payload(&payload_str, &secret);
        let _unused = client
            .post(&url)
            .header("X-Vaultwarden-Signature", sig)
            .header("X-Vaultwarden-Delivery", delivery_id)
            .header("Content-Type", "application/json")
            .body(payload_str)
            .send()
            .await;
    });

    info!("[Webhook] Test ping sent for webhook {webhook_id}");
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /organizations/<org_id>/webhooks/<webhook_id>
// ─────────────────────────────────────────────────────────────────────────────

#[delete("/organizations/<org_id>/webhooks/<webhook_id>")]
pub async fn delete_webhook(
    org_id: &str,
    webhook_id: &str,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> EmptyResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let webhook = Webhook::find_by_uuid(webhook_id, &mut conn).await
        .ok_or_else(|| Error::new("NotFound", "Webhook not found"))?;

    if webhook.org_uuid != org_id {
        return Err(Error::new("Forbidden", "Webhook does not belong to this org"));
    }

    webhook.delete(&mut conn).await?;
    info!("[Webhook] Deleted webhook {webhook_id} from org {org_id}");
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /organizations/<org_id>/webhooks/<webhook_id>/deliveries
// ─────────────────────────────────────────────────────────────────────────────

#[get("/organizations/<org_id>/webhooks/<webhook_id>/deliveries")]
pub async fn get_webhook_deliveries(
    org_id: &str,
    webhook_id: &str,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> JsonResult {
    use crate::db::schema::webhook_deliveries;
    use crate::db_run;
    use diesel::prelude::*;

    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    // Verify webhook belongs to org
    let webhook = Webhook::find_by_uuid(webhook_id, &mut conn).await
        .ok_or_else(|| Error::new("NotFound", "Webhook not found"))?;

    if webhook.org_uuid != org_id {
        return Err(Error::new("Forbidden", "Webhook does not belong to this org"));
    }

    let deliveries: Vec<(String, String, String, Option<chrono::NaiveDateTime>)> = db_run! { conn: {
        webhook_deliveries::table
            .filter(webhook_deliveries::webhook_uuid.eq(webhook_id))
            .select((
                webhook_deliveries::uuid,
                webhook_deliveries::event_type,
                webhook_deliveries::status,
                webhook_deliveries::last_attempt_at,
            ))
            .order_by(webhook_deliveries::uuid.desc())
            .limit(50)
            .load::<(String, String, String, Option<chrono::NaiveDateTime>)>(conn)
            .unwrap_or_default()
    }};

    let data: Vec<serde_json::Value> = deliveries.into_iter()
        .map(|(id, event_type, status, delivered_at)| json!({
            "id": id,
            "eventType": event_type,
            "status": status,
            "deliveredAt": delivered_at,
        }))
        .collect();

    Ok(Json(json!({
        "data": data,
        "object": "list",
        "continuationToken": null
    })))
}
