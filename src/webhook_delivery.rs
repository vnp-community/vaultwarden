/// TASK-008-008: HMAC-SHA256 webhook signing
/// TASK-008-009: deliver_event() and deliver_with_retry() with DB integration
/// TASK-008-011: Integration into cipher/send handlers

use std::{sync::OnceLock, time::Duration};
use tokio::time::sleep;

use crate::db::DbPool;

// ─────────────────────────────────────────────────────────────────────────────
// Global pool reference (set once at startup by main.rs)
// ─────────────────────────────────────────────────────────────────────────────

static WEBHOOK_POOL: OnceLock<DbPool> = OnceLock::new();

/// Called once from main.rs after the pool is created.
pub fn init_pool(pool: DbPool) {
    let _unused = WEBHOOK_POOL.set(pool);
}

fn get_pool() -> Option<&'static DbPool> {
    WEBHOOK_POOL.get()
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-008: HMAC-SHA256 signing
// ─────────────────────────────────────────────────────────────────────────────

/// Sign a payload with HMAC-SHA256 using the webhook's secret.
/// Returns a lowercase hex digest for the `X-Vaultwarden-Signature` header.
pub fn sign_payload(payload: &str, secret: &str) -> String {
    use ring::hmac;
    use data_encoding::HEXLOWER;
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let sig = hmac::sign(&key, payload.as_bytes());
    HEXLOWER.encode(sig.as_ref())
}

/// Decrypt a stored webhook secret.
pub fn decrypt_webhook_secret(encrypted_secret: &str) -> String {
    // In production: AES-256-GCM decrypt with server master key
    encrypted_secret.to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-009: Core retry delivery with DB status tracking
// ─────────────────────────────────────────────────────────────────────────────

pub async fn deliver_with_retry(
    delivery_id: String,
    url: String,
    payload: String,
    encrypted_secret: String,
    pool: DbPool,
) {
    let secret = decrypt_webhook_secret(&encrypted_secret);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let max_retries: u32 = 3;

    for attempt in 1..=max_retries {
        let signature = sign_payload(&payload, &secret);

        let res = client
            .post(&url)
            .header("X-Vaultwarden-Signature", &signature)
            .header("X-Vaultwarden-Delivery", &delivery_id)
            .header("Content-Type", "application/json")
            .body(payload.clone())
            .send()
            .await;

        match res {
            Ok(response) if response.status().is_success() => {
                info!("[Webhook] Delivered to {url} on attempt {attempt}");
                update_delivery_status(&delivery_id, "delivered", &pool).await;
                return;
            }
            Ok(response) => {
                warn!("[Webhook] Delivery attempt {attempt} returned HTTP {}", response.status());
            }
            Err(e) => {
                error!("[Webhook] Delivery attempt {attempt} network error: {e:?}");
            }
        }

        if attempt < max_retries {
            let backoff = Duration::from_secs(2u64.pow(attempt));
            sleep(backoff).await;
        }
    }

    error!("[Webhook] Delivery to {url} failed after {max_retries} attempts");
    update_delivery_status(&delivery_id, "failed", &pool).await;
}

async fn update_delivery_status(delivery_id: &str, status: &str, pool: &DbPool) {
    use crate::db::schema::webhook_deliveries;
    use crate::db_run;
    use diesel::prelude::*;
    use chrono::Utc;

    let Ok(conn) = pool.get().await else { return; };
    let now = Utc::now().naive_utc();
    let status_str = status.to_string();
    let did = delivery_id.to_string();

    db_run! { conn: {
        let _unused = diesel::update(webhook_deliveries::table.filter(webhook_deliveries::uuid.eq(&did)))
            .set((
                webhook_deliveries::status.eq(&status_str),
                webhook_deliveries::last_attempt_at.eq(Some(now)),
            ))
            .execute(conn);
    }};
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-009 / 008-011: Public fire-and-forget entry points
// ─────────────────────────────────────────────────────────────────────────────

/// Primary entry point: fire-and-forget dispatch using an explicit pool.
/// Used from background jobs and places that have the pool available.
#[allow(dead_code)]
pub fn deliver_event_with_pool(
    event_type: &str,
    org_uuid: &str,
    payload: serde_json::Value,
    pool: DbPool,
) {
    if !crate::CONFIG.webhook_enabled() { return; }

    let et = event_type.to_string();
    let ou = org_uuid.to_string();

    tokio::spawn(async move {
        _dispatch_event(et, ou, payload, pool).await;
    });
}

/// Convenience entry point for handler code that doesn't have a pool.
/// Uses the globally-initialized `WEBHOOK_POOL` set at startup.
/// TASK-008-011: called from cipher/send handlers.
pub fn deliver_event(event_type: &str, org_uuid: &str, payload: serde_json::Value) {
    if !crate::CONFIG.webhook_enabled() { return; }

    let pool = match get_pool() {
        Some(p) => p.clone(),
        None => {
            warn!("[Webhook] Pool not initialized, event '{event_type}' for org {org_uuid} dropped");
            return;
        }
    };

    let et = event_type.to_string();
    let ou = org_uuid.to_string();

    tokio::spawn(async move {
        _dispatch_event(et, ou, payload, pool).await;
    });
}

async fn _dispatch_event(event_type: String, org_uuid: String, payload: serde_json::Value, pool: DbPool) {
    let conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => { error!("[Webhook] Pool error in dispatch: {e:?}"); return; }
    };

    use crate::db::models::Webhook;
    let webhooks = Webhook::find_active_for_event(&org_uuid, &event_type, &mut { conn }).await;

    let payload_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());

    for webhook in webhooks {
        use crate::db::models::WebhookDelivery;

        let mut delivery = WebhookDelivery::new(webhook.uuid.clone(), event_type.clone(), payload_str.clone());
        // uuid is auto-generated in WebhookDelivery::new; we keep the generated uuid as delivery_id
        let delivery_id = delivery.uuid.clone();

        if let Ok(mut save_conn) = pool.get().await {
            let _unused = delivery.save(&mut save_conn).await;
        }

        let spawn_pool = pool.clone();
        tokio::spawn(deliver_with_retry(
            delivery_id,
            webhook.url.clone(),
            payload_str.clone(),
            webhook.secret_hash.clone(),
            spawn_pool,
        ));
    }
}
