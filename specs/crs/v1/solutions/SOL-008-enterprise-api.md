# SOL-008: Giải Pháp Thực Hiện — Enterprise API Management & Developer Portal

> **Giải pháp cho**: CR-008  
> **Ngày**: 2026-04-12  
> **Trạng thái**: Draft  
> **Kiến trúc thay đổi**: Tối thiểu — mở rộng API key model, thêm webhook system

---

## 1. Tổng Quan Giải Pháp

Vaultwarden đã có `OrganizationApiKey` trong `src/db/models/organization.rs`. Giải pháp **mở rộng** model này và thêm:

1. **Enhanced API Keys**: Scoped permissions, IP whitelist, rate limiting per-key
2. **Webhook System**: Event-driven HTTP callbacks với HMAC signing
3. **Secrets API**: Subset of vault items tagged as secrets
4. **Usage Analytics**: Per-key request tracking

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/api/core/api_keys.rs` | Enhanced API key management routes |
| `src/api/core/webhooks.rs` | Webhook CRUD + delivery |
| `src/api/core/secrets.rs` | Secrets API (thin layer over ciphers) |
| `src/webhook_delivery.rs` | Background webhook delivery task |
| `src/db/models/api_key_v2.rs` | Enhanced API key model |
| `src/db/models/webhook.rs` | Webhook model + delivery log |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/db/models/organization.rs` | Deprecate old `OrganizationApiKey` → new model |
| `src/auth.rs` | Thêm API key scope validation |
| `src/api/identity.rs` | Trigger webhook events từ auth events |
| `src/api/core/ciphers.rs` | Trigger webhook events từ vault operations |
| `src/config.rs` | Thêm WEBHOOK_* config keys |
| `src/main.rs` | Khởi động webhook delivery background task |

### 2.3 Database Migrations

```sql
-- migrations/postgresql/YYYYMMDD_api_keys_v2/up.sql

-- Enhanced API keys (replaces old org_api_keys)
CREATE TABLE api_keys_v2 (
    uuid                VARCHAR(40) PRIMARY KEY,
    org_uuid            VARCHAR(40) NOT NULL REFERENCES organizations(uuid) ON DELETE CASCADE,
    name                VARCHAR(200) NOT NULL,
    description         TEXT,
    created_by_uuid     VARCHAR(40) NOT NULL,
    
    -- Credentials
    client_id           VARCHAR(40) NOT NULL UNIQUE,    -- For OAuth client_credentials
    client_secret_hash  VARCHAR(64) NOT NULL,           -- SHA-256 of secret
    
    -- Access Control
    scopes              TEXT[] NOT NULL DEFAULT '{}',   -- ['VaultRead', 'OrgRead']
    allowed_ips         TEXT[] NOT NULL DEFAULT '{}',   -- CIDR ranges
    allowed_collections TEXT[] NOT NULL DEFAULT '{}',   -- collection UUIDs
    
    -- Rate Limiting
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    rate_limit_per_hour   INTEGER NOT NULL DEFAULT 1000,
    
    -- Lifecycle
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at          TIMESTAMPTZ,
    last_used_at        TIMESTAMPTZ,
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Rotation reminder
    rotate_reminder_days INTEGER
);

-- API key usage tracking
CREATE TABLE api_key_usage (
    id              BIGSERIAL PRIMARY KEY,
    api_key_uuid    VARCHAR(40) NOT NULL REFERENCES api_keys_v2(uuid) ON DELETE CASCADE,
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    endpoint        VARCHAR(500) NOT NULL,
    method          VARCHAR(10) NOT NULL,
    status_code     SMALLINT NOT NULL,
    response_ms     INTEGER
);

-- Partition by timestamp for performance
CREATE INDEX idx_api_key_usage_key_time ON api_key_usage(api_key_uuid, timestamp DESC);

-- Webhooks
CREATE TABLE webhooks (
    uuid                VARCHAR(40) PRIMARY KEY,
    org_uuid            VARCHAR(40) NOT NULL REFERENCES organizations(uuid) ON DELETE CASCADE,
    name                VARCHAR(200) NOT NULL,
    url                 TEXT NOT NULL,                  -- Must be HTTPS
    secret_hash         VARCHAR(64) NOT NULL,           -- For HMAC signing (hash stored, not plaintext)
    events              TEXT[] NOT NULL DEFAULT '{}',   -- List of event types
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    retry_count         SMALLINT NOT NULL DEFAULT 3,
    timeout_seconds     INTEGER NOT NULL DEFAULT 30,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_delivery_at    TIMESTAMPTZ,
    last_delivery_status SMALLINT
);

-- Webhook delivery log
CREATE TABLE webhook_deliveries (
    id              BIGSERIAL PRIMARY KEY,
    webhook_uuid    VARCHAR(40) NOT NULL REFERENCES webhooks(uuid) ON DELETE CASCADE,
    event_type      VARCHAR(100) NOT NULL,
    payload         JSONB NOT NULL,
    attempt_count   SMALLINT NOT NULL DEFAULT 0,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, delivered, failed
    delivered_at    TIMESTAMPTZ,
    last_error      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhook_deliveries_pending ON webhook_deliveries(status, created_at) 
    WHERE status = 'pending';

-- Secrets: just a tagged subset of ciphers
-- No new table needed — use cipher fields:
ALTER TABLE ciphers ADD COLUMN is_secret BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE ciphers ADD COLUMN secret_project VARCHAR(200);  -- For project-scoped access
```

---

## 3. Thiết Kế Chi Tiết

### 3.1 Enhanced API Key Auth

**File**: `src/auth.rs` — thêm API key guard:

```rust
pub struct ApiKeyAuth {
    pub api_key: ApiKeyV2,
    pub org_uuid: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKeyAuth {
    type Error = Error;
    
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Support Bearer token format
        let auth_header = req.headers().get_one("Authorization")
            .and_then(|h| h.strip_prefix("Bearer "));
        
        // Also support Basic auth (client_id:client_secret format — Bitwarden compat)
        let token = auth_header.or_else(|| extract_basic_auth(req));
        
        let token = match token {
            Some(t) => t,
            None => return Outcome::Error((Status::Unauthorized, Error::new("Missing auth", ""))),
        };
        
        let conn = req.guard::<DbConn>().await.unwrap();
        
        match ApiKeyV2::verify_token(token, &conn).await {
            Ok(key) => {
                // IP check
                if !key.allowed_ips.is_empty() {
                    let ip = req.client_ip().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
                    if !is_ip_in_allowlist(&ip, &key.allowed_ips) {
                        return Outcome::Error((Status::Forbidden, 
                            Error::new("IP not allowed for this API key", "")));
                    }
                }
                
                // Rate limit per API key
                let rate_key = format!("api_key:{}", key.uuid);
                if let Err(_) = check_api_key_rate_limit(&rate_key, &key).await {
                    return Outcome::Error((Status::TooManyRequests, 
                        Error::new("API key rate limit exceeded", "")));
                }
                
                // Update last_used_at
                ApiKeyV2::touch(&key.uuid, &conn).await.ok();
                
                let org_uuid = key.org_uuid.clone();
                Outcome::Success(ApiKeyAuth { api_key: key, org_uuid })
            }
            Err(e) => Outcome::Error((Status::Unauthorized, e)),
        }
    }
}

// Scope validation
pub fn require_scope(key: &ApiKeyV2, scope: &str) -> Result<(), Error> {
    if !key.scopes.iter().any(|s| s == scope) {
        err!(format!("API key missing required scope: {scope}"));
    }
    Ok(())
}
```

### 3.2 Webhook Delivery System

**File**: `src/webhook_delivery.rs`

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub async fn deliver_event(
    event_type: &str,
    org_uuid: &str,
    payload: &Value,
    conn: &DbConn,
) {
    // Tìm tất cả webhooks đăng ký event này
    let webhooks = Webhook::find_active_for_event(event_type, org_uuid, conn)
        .await.unwrap_or_default();
    
    for webhook in webhooks {
        // Tạo delivery record
        let delivery = WebhookDelivery {
            webhook_uuid: webhook.uuid.clone(),
            event_type: event_type.to_string(),
            payload: payload.clone(),
            status: "pending".to_string(),
            ..Default::default()
        };
        let delivery_id = delivery.insert(conn).await.unwrap();
        
        // Deliver async (không block caller)
        tokio::spawn(async move {
            deliver_with_retry(delivery_id, &webhook).await;
        });
    }
}

async fn deliver_with_retry(delivery_id: i64, webhook: &Webhook) {
    let pool = DB_POOL.get().expect("pool");
    let conn = pool.get().expect("conn");
    
    let delivery = WebhookDelivery::find_by_id(delivery_id, &conn).await.unwrap();
    
    for attempt in 0..webhook.retry_count {
        let result = send_webhook(webhook, &delivery.payload, attempt + 1).await;
        
        match result {
            Ok(status) => {
                WebhookDelivery::mark_delivered(delivery_id, status, &conn).await.ok();
                Webhook::update_delivery_status(&webhook.uuid, status, &conn).await.ok();
                return;
            }
            Err(e) if attempt < webhook.retry_count - 1 => {
                warn!("Webhook delivery attempt {} failed: {e}", attempt + 1);
                // Exponential backoff: 2^attempt seconds
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempt as u32))).await;
            }
            Err(e) => {
                error!("Webhook delivery failed after {} attempts: {e}", webhook.retry_count);
                WebhookDelivery::mark_failed(delivery_id, &e.to_string(), &conn).await.ok();
            }
        }
    }
}

async fn send_webhook(
    webhook: &Webhook,
    payload: &Value,
    attempt: i32,
) -> Result<u16, Error> {
    let payload_str = serde_json::to_string(payload)?;
    
    // Tính HMAC-SHA256 signature
    let signature = sign_payload(&payload_str, &webhook.secret_hash);
    
    let full_payload = json!({
        "id": format!("evt_{}", get_uuid()),
        "type": payload.get("event_type").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "timestamp": Utc::now().to_rfc3339(),
        "attempt": attempt,
        "data": payload,
        "signature": format!("sha256={signature}"),
    });
    
    let resp = get_reqwest_client()
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("X-Vaultwarden-Signature", format!("sha256={signature}"))
        .header("X-Vaultwarden-Event", payload.get("event_type")
            .and_then(|v| v.as_str()).unwrap_or(""))
        .timeout(Duration::from_secs(webhook.timeout_seconds as u64))
        .body(serde_json::to_string(&full_payload)?)
        .send()
        .await
        .map_err(|e| Error::new(&format!("Webhook request failed: {e}"), ""))?;
    
    Ok(resp.status().as_u16())
}

fn sign_payload(payload: &str, webhook_secret_hash: &str) -> String {
    // Để sign, cần original secret (không phải hash)
    // Solution: store secret encrypted (not hashed) với server key
    // Or: store as bcrypt hash cho verification only, separate signing secret
    // Thiết kế: lưu secret encrypt với AES-256-GCM dùng server master key
    let secret = decrypt_webhook_secret(webhook_secret_hash);
    
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC key size is always valid");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

### 3.3 Secrets API

**File**: `src/api/core/secrets.rs`

```rust
// GET /api/secrets?project=
#[get("/secrets?<project>")]
async fn list_secrets(
    project: Option<&str>,
    auth: ApiKeyAuth,
    conn: DbConn,
) -> JsonResult {
    require_scope(&auth.api_key, "SecretsRead")?;
    
    let secrets = Cipher::find_secrets_for_org(
        &auth.org_uuid,
        project,
        &auth.api_key.allowed_collections,
        &conn,
    ).await?;
    
    Ok(Json(json!({
        "secrets": secrets.iter().map(|s| json!({
            "id": s.uuid,
            "name": s.name,
            "project": s.secret_project,
            "created_at": s.created_at,
            "updated_at": s.updated_at,
        })).collect::<Vec<_>>()
    })))
}

// GET /api/secrets/{id} — Returns decrypted value via org key
// Note: Client-side encryption means server cannot decrypt directly.
// This endpoint returns the encrypted blob; the API client must have org key access.
#[get("/secrets/<id>")]
async fn get_secret(
    id: &str,
    auth: ApiKeyAuth,
    conn: DbConn,
) -> JsonResult {
    require_scope(&auth.api_key, "SecretsRead")?;
    
    let cipher = Cipher::find_by_uuid(id, &conn).await?
        .ok_or_else(|| Error::new("Secret not found", ""))?;
    
    // Validate belongs to org and is a secret
    if cipher.organization_uuid.as_deref() != Some(&auth.org_uuid) || !cipher.is_secret {
        err!("Secret not found");
    }
    
    // Check collection access
    if !auth.api_key.allowed_collections.is_empty() {
        let cipher_collections = CollectionCipher::find_by_cipher(id, &conn).await?;
        let allowed: HashSet<_> = auth.api_key.allowed_collections.iter().collect();
        if !cipher_collections.iter().any(|cc| allowed.contains(&cc.collection_uuid)) {
            err!("Access denied: cipher not in allowed collections");
        }
    }
    
    // Track usage
    track_api_key_usage(&auth.api_key, &conn).await;
    
    Ok(Json(cipher.to_json(&conn).await?))
}

// GET /api/secrets/export?format=env|json|dotenv
#[get("/secrets/export?<format>&<project>")]
async fn export_secrets(
    format: &str,
    project: Option<&str>,
    auth: ApiKeyAuth,
    conn: DbConn,
) -> Result<String, Error> {
    require_scope(&auth.api_key, "SecretsRead")?;
    
    // NOTE: Server trả về encrypted blobs — API client phải decrypt
    // Đây là design limitation của E2E encryption
    // Documented: "Secret values are returned encrypted; use Vaultwarden SDK to decrypt"
    let secrets = Cipher::find_secrets_for_org(&auth.org_uuid, project, &[], &conn).await?;
    
    match format {
        "env" | "dotenv" => {
            let lines: Vec<String> = secrets.iter()
                .map(|s| format!("{}={}", sanitize_env_key(&s.name), "[ENCRYPTED_BLOB]"))
                .collect();
            Ok(lines.join("\n"))
        }
        "json" => {
            Ok(serde_json::to_string_pretty(&secrets)?)
        }
        _ => err!("Unsupported format. Use env, dotenv, or json"),
    }
}
```

### 3.4 Usage Analytics

```rust
// Track usage per API key
async fn track_api_key_usage(key: &ApiKeyV2, endpoint: &str, status: u16, duration_ms: u32) {
    // Non-blocking insert
    tokio::spawn(async move {
        let pool = DB_POOL.get().expect("pool");
        let conn = pool.get().expect("conn");
        
        ApiKeyUsage {
            api_key_uuid: key.uuid.clone(),
            endpoint: endpoint.to_string(),
            method: "GET".to_string(),
            status_code: status,
            response_ms: duration_ms,
        }.insert(&conn).await.ok();
    });
}

// GET /api/admin/api-analytics?period=7d
#[get("/admin/api-analytics?<period>")]
async fn api_analytics(
    period: Option<&str>,
    _admin: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    let days = match period.unwrap_or("7d") {
        "7d"  => 7,
        "30d" => 30,
        "90d" => 90,
        _     => 7,
    };
    
    let keys = ApiKeyV2::find_all(&conn).await?;
    let mut analytics = Vec::new();
    
    for key in keys {
        let stats = ApiKeyUsage::aggregate_for_key(&key.uuid, days, &conn).await?;
        analytics.push(json!({
            "key_id": key.uuid,
            "name": key.name,
            "requests_total": stats.total,
            "error_rate": stats.error_rate,
            "top_endpoints": stats.top_endpoints,
            "last_used": key.last_used_at,
        }));
    }
    
    Ok(Json(json!({"api_keys": analytics})))
}
```

---

## 4. Webhook Event Integration

Thêm vào các handlers:

```rust
// Trong src/api/core/ciphers.rs — sau khi create/update/delete cipher
webhook_delivery::deliver_event("cipher.created", &org_uuid, &json!({
    "event_type": "cipher.created",
    "cipher_id": cipher.uuid,
    "actor": {"user_id": user.uuid, "email": user.email},
}), &conn).await;
```

---

## 5. Config Variables Mới

```bash
# Webhook delivery
WEBHOOK_ENABLED=true
WEBHOOK_WORKER_CONCURRENCY=10
WEBHOOK_MAX_RETRY_DELAY_SECONDS=300
WEBHOOK_DELIVERY_QUEUE_SIZE=1000

# API Keys
API_KEY_V2_ENABLED=true
API_KEY_DEFAULT_RATE_LIMIT_MINUTE=60
API_KEY_USAGE_TRACKING=true
API_KEY_ROTATION_REMINDER_DAYS=30   # Email reminder before expiry
```

---

## 6. API Endpoints Mới

| Method | Path | Auth | Mô tả |
|--------|------|------|-------|
| GET | `/api/organizations/{id}/api-keys` | Admin | List API keys |
| POST | `/api/organizations/{id}/api-keys` | Admin | Create API key |
| PATCH | `/api/organizations/{id}/api-keys/{kid}` | Admin | Update key |
| POST | `/api/organizations/{id}/api-keys/{kid}/rotate` | Admin | Rotate secret |
| DELETE | `/api/organizations/{id}/api-keys/{kid}` | Admin | Delete key |
| GET | `/api/organizations/{id}/api-keys/{kid}/usage` | Admin | Usage stats |
| GET | `/api/organizations/{id}/webhooks` | Admin | List webhooks |
| POST | `/api/organizations/{id}/webhooks` | Admin | Create webhook |
| POST | `/api/organizations/{id}/webhooks/{wid}/test` | Admin | Test delivery |
| GET | `/api/organizations/{id}/webhooks/{wid}/deliveries` | Admin | Delivery log |
| GET | `/api/secrets` | API Key | List secrets |
| GET | `/api/secrets/{id}` | API Key | Get secret |
| GET | `/api/secrets/export` | API Key | Export secrets |
| GET | `/api/admin/api-analytics` | Admin | Usage analytics |

---

## 7. Kế Hoạch Triển Khai

### Sprint 1–2: Enhanced API Keys
- DB migration
- Scoped permission validation
- IP allowlist + rate limiting per key

### Sprint 3–5: Webhook System
- Webhook model + CRUD API
- Delivery background task
- HMAC signing
- Retry logic

### Sprint 6–7: Secrets API
- `is_secret` flag on ciphers
- Secrets CRUD + export

### Sprint 8–9: Usage Analytics
- Usage tracking (async)
- Analytics aggregation API
- Admin dashboard

---

*Status: Draft | Ngày: 2026-04-12*
