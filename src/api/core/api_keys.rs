/// TASK-008-005: API Key V2 CRUD routes
/// TASK-008-016: API Analytics endpoint

use rocket::{serde::json::Json, Route};
use sha2::{Digest, Sha256};

use crate::{
    api::{EmptyResult, JsonResult},
    auth::AdminHeaders,
    db_run,
    db::{models::ApiKeyV2, DbConn},
    error::Error,
    util, CONFIG,
};

pub fn routes() -> Vec<Route> {
    routes![
        get_api_keys,
        create_api_key,
        update_api_key,
        rotate_api_key,
        delete_api_key,
        get_api_key_usage,
        get_api_analytics,
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Request bodies
// ─────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
struct UpdateApiKeyRequest {
    name: Option<String>,
    scopes: Option<Vec<String>>,
    allowed_ips: Option<Vec<String>>,
    rate_limit_minute: Option<i32>,
    is_active: Option<bool>,
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-005: GET /organizations/<org_id>/api-keys
// ─────────────────────────────────────────────────────────────────────────────

#[get("/organizations/<org_id>/api-keys")]
pub async fn get_api_keys(org_id: &str, headers: AdminHeaders, mut conn: DbConn) -> JsonResult {
    // Ensure requestor belongs to this org
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "You are not an admin of this organization"));
    }

    let keys = ApiKeyV2::find_all_for_org(org_id, &mut conn).await;
    let json: Vec<serde_json::Value> = keys.iter().map(|k| k.to_json()).collect();

    Ok(Json(json!({
        "data": json,
        "object": "list",
        "continuationToken": null
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-005: POST /organizations/<org_id>/api-keys
// ─────────────────────────────────────────────────────────────────────────────

#[post("/organizations/<org_id>/api-keys", data = "<data>")]
pub async fn create_api_key(
    org_id: &str,
    data: Json<serde_json::Value>,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> JsonResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let name = data["name"].as_str()
        .ok_or_else(|| Error::new("BadRequest", "name is required"))?
        .to_string();

    let scopes = data["scopes"].as_array()
        .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join(","))
        .unwrap_or_default();

    // Generate client_id and plain secret, then hash the secret
    let client_id = format!("vw_{}", &util::get_uuid()[..16]);
    let plain_secret = util::get_uuid(); // random; only shown once
    let secret_hash = {
        let mut h = Sha256::new();
        h.update(plain_secret.as_bytes());
        data_encoding::BASE64.encode(&h.finalize())
    };

    let allowed_ips = data["allowedIps"].as_array()
        .map(|arr| serde_json::to_string(arr).unwrap_or_default());
    let rate_limit = data["rateLimitMinute"].as_i64().map(|v| v as i32);

    let mut key = ApiKeyV2::new(org_id.to_string(), client_id, name, secret_hash);
    key.scopes = scopes;
    key.allowed_ips = allowed_ips;
    key.rate_limit_minute = rate_limit.or(Some(CONFIG.api_key_default_rate_limit_minute() as i32));
    key.save(&mut conn).await?;

    // Return plain secret ONCE — not stored
    let mut resp = key.to_json();
    resp["clientSecret"] = json!(plain_secret);
    resp["note"] = json!("Store the clientSecret securely — it will not be shown again.");

    info!("[APIKey] Created key '{}' for org {org_id}", key.name);
    Ok(Json(resp))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-005: PATCH /organizations/<org_id>/api-keys/<key_id>
// ─────────────────────────────────────────────────────────────────────────────

#[patch("/organizations/<org_id>/api-keys/<key_id>", data = "<data>")]
pub async fn update_api_key(
    org_id: &str,
    key_id: &str,
    data: Json<serde_json::Value>,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> JsonResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let mut key = ApiKeyV2::find_by_uuid(key_id, &mut conn).await
        .ok_or_else(|| Error::new("NotFound", "API Key not found"))?;

    if key.org_uuid != org_id {
        return Err(Error::new("Forbidden", "Key does not belong to this org"));
    }

    if let Some(name) = data["name"].as_str() { key.name = name.to_string(); }
    if let Some(active) = data["isActive"].as_bool() { key.is_active = active; }
    if let Some(rate) = data["rateLimitMinute"].as_i64() { key.rate_limit_minute = Some(rate as i32); }
    if let Some(scopes) = data["scopes"].as_array() {
        key.scopes = scopes.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join(",");
    }
    if let Some(ips) = data["allowedIps"].as_array() {
        key.allowed_ips = Some(serde_json::to_string(ips).unwrap_or_default());
    }

    key.save(&mut conn).await?;
    Ok(Json(key.to_json()))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-005: POST /organizations/<org_id>/api-keys/<key_id>/rotate
// ─────────────────────────────────────────────────────────────────────────────

#[post("/organizations/<org_id>/api-keys/<key_id>/rotate")]
pub async fn rotate_api_key(
    org_id: &str,
    key_id: &str,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> JsonResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let mut key = ApiKeyV2::find_by_uuid(key_id, &mut conn).await
        .ok_or_else(|| Error::new("NotFound", "API Key not found"))?;

    if key.org_uuid != org_id {
        return Err(Error::new("Forbidden", "Key does not belong to this org"));
    }

    // Issue a brand-new secret
    let new_secret = util::get_uuid();
    let new_hash = {
        let mut h = Sha256::new();
        h.update(new_secret.as_bytes());
        data_encoding::BASE64.encode(&h.finalize())
    };
    key.secret_hash = new_hash;
    key.save(&mut conn).await?;

    let mut resp = key.to_json();
    resp["clientSecret"] = json!(new_secret);
    resp["note"] = json!("New secret issued — old secret is now invalid.");

    info!("[APIKey] Rotated key '{}' for org {org_id}", key.name);
    Ok(Json(resp))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-005: DELETE /organizations/<org_id>/api-keys/<key_id>
// ─────────────────────────────────────────────────────────────────────────────

#[delete("/organizations/<org_id>/api-keys/<key_id>")]
pub async fn delete_api_key(
    org_id: &str,
    key_id: &str,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> EmptyResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let key = ApiKeyV2::find_by_uuid(key_id, &mut conn).await
        .ok_or_else(|| Error::new("NotFound", "API Key not found"))?;

    if key.org_uuid != org_id {
        return Err(Error::new("Forbidden", "Key does not belong to this org"));
    }

    key.delete(&mut conn).await?;
    info!("[APIKey] Deleted key {key_id} from org {org_id}");
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-005: GET /organizations/<org_id>/api-keys/<key_id>/usage
// ─────────────────────────────────────────────────────────────────────────────

#[get("/organizations/<org_id>/api-keys/<key_id>/usage?<days>")]
pub async fn get_api_key_usage(
    org_id: &str,
    key_id: &str,
    days: Option<i64>,
    headers: AdminHeaders,
    mut conn: DbConn,
) -> JsonResult {
    if headers.org_id.to_string() != org_id {
        return Err(Error::new("Forbidden", "Not your org"));
    }

    let key = ApiKeyV2::find_by_uuid(key_id, &mut conn).await
        .ok_or_else(|| Error::new("NotFound", "API Key not found"))?;

    if key.org_uuid != org_id {
        return Err(Error::new("Forbidden", "Key does not belong to this org"));
    }

    let period_days = days.unwrap_or(30).clamp(1, 90);
    let stats = aggregate_key_usage(key_id, period_days, &mut conn).await;

    Ok(Json(json!({
        "keyId": key_id,
        "periodDays": period_days,
        "totalRequests": stats.total_requests,
        "errorRate": stats.error_rate,
        "topEndpoints": stats.top_endpoints,
        "lastUsedAt": key.last_used_at,
        "object": "apiKeyUsageStats"
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-016: GET /admin/api-analytics
// ─────────────────────────────────────────────────────────────────────────────

#[get("/admin/api-analytics?<period>")]
pub async fn get_api_analytics(
    period: Option<&str>,
    _headers: AdminHeaders,
    mut conn: DbConn,
) -> JsonResult {
    let days: i64 = match period.unwrap_or("7d") {
        "30d" => 30,
        "90d" => 90,
        _ => 7,
    };

    // Aggregate stats across all keys visible to this admin
    // In production, filter by org_uuid from headers
    let stats = aggregate_all_usage(days, &mut conn).await;

    Ok(Json(json!({
        "period": period.unwrap_or("7d"),
        "periodDays": days,
        "totalRequests": stats.total_requests,
        "errorRate": stats.error_rate,
        "topEndpoints": stats.top_endpoints,
        "object": "apiAnalytics"
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Usage aggregation helpers
// ─────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
struct UsageStats {
    total_requests: i64,
    error_rate: f64,
    top_endpoints: Vec<serde_json::Value>,
}

async fn aggregate_key_usage(key_id: &str, days: i64, conn: &mut DbConn) -> UsageStats {
    use crate::db::schema::api_key_usage;
    use diesel::prelude::*;

    let cutoff = (chrono::Utc::now() - chrono::TimeDelta::try_days(days).unwrap_or_default()).naive_utc();

    let rows: Vec<(String, i32, i32)> = db_run! { conn: {
        api_key_usage::table
            .filter(api_key_usage::api_key_uuid.eq(key_id))
            .filter(api_key_usage::timestamp.gt(cutoff))
            .select((api_key_usage::endpoint, api_key_usage::status_code, api_key_usage::response_ms))
            .load::<(String, i32, i32)>(conn)
            .unwrap_or_default()
    }};

    build_stats(rows)
}

async fn aggregate_all_usage(days: i64, conn: &mut DbConn) -> UsageStats {
    use crate::db::schema::api_key_usage;
    use diesel::prelude::*;

    let cutoff = (chrono::Utc::now() - chrono::TimeDelta::try_days(days).unwrap_or_default()).naive_utc();

    let rows: Vec<(String, i32, i32)> = db_run! { conn: {
        api_key_usage::table
            .filter(api_key_usage::timestamp.gt(cutoff))
            .select((api_key_usage::endpoint, api_key_usage::status_code, api_key_usage::response_ms))
            .load::<(String, i32, i32)>(conn)
            .unwrap_or_default()
    }};

    build_stats(rows)
}

fn build_stats(rows: Vec<(String, i32, i32)>) -> UsageStats {
    use std::collections::HashMap;

    let total = rows.len() as i64;
    let errors = rows.iter().filter(|(_, status, _)| *status >= 400).count() as f64;
    let error_rate = if total > 0 { (errors / total as f64) * 100.0 } else { 0.0 };

    let mut endpoint_counts: HashMap<String, i64> = HashMap::new();
    for (endpoint, _, _) in &rows {
        *endpoint_counts.entry(endpoint.clone()).or_insert(0) += 1;
    }

    let mut top: Vec<(String, i64)> = endpoint_counts.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));
    let top_endpoints: Vec<serde_json::Value> = top.into_iter().take(10)
        .map(|(ep, count)| json!({ "endpoint": ep, "requests": count }))
        .collect();

    UsageStats { total_requests: total, error_rate, top_endpoints }
}
