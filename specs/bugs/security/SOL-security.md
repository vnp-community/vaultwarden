# Giải Pháp Khắc Phục — Điểm Yếu & Giới Hạn Bảo Mật

> **Tham chiếu**: [specs/bugs/security-analysis.md](../security-analysis.md)  
> **Ngày**: 2026-04-12  
> **Phân loại**: P1 (Critical), P2 (High), P3 (Medium), P4 (Low)

---

## SEC-CRIT-01: Admin Token Plaintext Fallback [P1 — Làm Ngay]

**File**: [src/api/admin.rs:245](../../src/api/admin.rs)  
**Rủi ro**: Brute-force admin panel với low-entropy token.

### Giải Pháp

**Option A (Recommended)**: Từ chối khởi động nếu token không phải Argon2.

```rust
// src/api/admin.rs hoặc src/main.rs — startup validation

pub fn validate_admin_token() -> Result<(), Error> {
    let token = CONFIG.admin_token();
    
    if token.is_empty() || CONFIG.disable_admin_token() {
        return Ok(()); // Handled separately
    }
    
    if !token.starts_with("$argon2") {
        // Log CRITICAL warning
        error!(
            "SECURITY CRITICAL: ADMIN_TOKEN is not an Argon2id hash. \
             This is a serious security risk. \
             Use 'vaultwarden hash' to generate a secure token. \
             Server will refuse to start in strict mode."
        );
        
        if CONFIG.admin_token_strict_mode() {
            return Err(Error::new(
                "ADMIN_TOKEN must be an Argon2id PHC string. \
                 Run: vaultwarden hash --preset owasp", 
                ""
            ));
        }
        
        // Non-strict mode: warn but continue (backward compat)
        warn!("Admin panel will use INSECURE plaintext token comparison!");
    }
    
    Ok(())
}
```

**Config mới**:
```bash
ADMIN_TOKEN_STRICT_MODE=false  # true = reject startup nếu token không phải Argon2
                               # Sẽ đổi default thành true trong v2.1
```

**Option B (Cứng hơn)**: Luôn reject non-Argon2 token kể từ v2.0:

```rust
// Startup validation — không có fallback
if !CONFIG.admin_token().starts_with("$argon2") && !CONFIG.disable_admin_token() {
    panic!(
        "FATAL: ADMIN_TOKEN must be an Argon2id PHC string. \
         Generate one with: vaultwarden hash --preset owasp"
    );
}
```

**Recommended**: Option A với `ADMIN_TOKEN_STRICT_MODE=true` làm default từ v2.0.

---

## SEC-CRIT-02: DISABLE_ADMIN_TOKEN Không Có Safeguard [P1 — Làm Ngay]

**File**: [src/config.rs:758](../../src/config.rs)  
**Rủi ro**: Admin panel exposed to internet nếu vô tình set.

### Giải Pháp

**Giải pháp 1 (Tức thời)**: Yêu cầu biến xác nhận thứ hai:

```rust
// src/api/admin.rs — startup check
pub fn validate_disable_admin_token() -> Result<(), Error> {
    if CONFIG.disable_admin_token() {
        // Yêu cầu biến xác nhận
        let confirmed = CONFIG.disable_admin_token_confirmed();
        
        if !confirmed {
            error!(
                "SECURITY WARNING: DISABLE_ADMIN_TOKEN=true but \
                 DISABLE_ADMIN_TOKEN_CONFIRM is not set. \
                 The admin panel will be UNAUTHENTICATED. \
                 Set DISABLE_ADMIN_TOKEN_CONFIRM=true to acknowledge this risk."
            );
            return Err(Error::new(
                "Must set DISABLE_ADMIN_TOKEN_CONFIRM=true to use DISABLE_ADMIN_TOKEN=true",
                ""
            ));
        }
        
        // Log mỗi lần server restart
        warn!(
            "SECURITY NOTICE: Admin panel authentication is DISABLED. \
             Ensure access is restricted via network-level controls \
             (firewall, reverse proxy auth, IP allowlist)."
        );
        
        // Audit event mỗi lần server start với config này
        // (sẽ được ghi khi audit subsystem khởi động)
    }
    Ok(())
}
```

**Config mới**:
```bash
DISABLE_ADMIN_TOKEN=false
DISABLE_ADMIN_TOKEN_CONFIRM=false   # Phải explicit set true để activate DISABLE_ADMIN_TOKEN
```

**Giải pháp 2 (Bổ sung)**: IP allowlist enforce khi admin token disabled:

```rust
// Nếu DISABLE_ADMIN_TOKEN=true, auto-enable IP_ALLOWLIST_ADMIN_PANEL 
// và yêu cầu IP_ALLOWLIST không empty
if CONFIG.disable_admin_token() && CONFIG.ip_allowlist_admin_panel() {
    if CONFIG.ip_allowlist().is_empty() {
        error!(
            "SECURITY ERROR: DISABLE_ADMIN_TOKEN=true but IP_ALLOWLIST is empty. \
             Admin panel would be accessible from any IP."
        );
        return Err(Error::new("IP_ALLOWLIST required when DISABLE_ADMIN_TOKEN=true", ""));
    }
}
```

---

## SEC-HIGH-01: JWT trong URL Query Parameter [P2 — Sprint 1]

**File**: [src/api/notifications.rs:51-53](../../src/api/notifications.rs)  
**Rủi ro**: Session hijacking qua server log exfiltration.

### Giải Pháp

Xem chi tiết tại `specs/bugs/rust-dev/SOL-rust-dev.md` (cùng vấn đề).

**Tóm tắt**: Xóa `WsAccessToken` struct, chỉ chấp nhận `Authorization: Bearer` header.

```rust
// Xóa hoàn toàn:
// #[derive(FromForm)] struct WsAccessToken { access_token: Option<String> }

// WebSocket handler — chỉ header
async fn websocket_hub(ws: WebSocket, req: &Request<'_>, conn: DbConn) -> ... {
    let token = req.headers().get_one("Authorization")
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(Status::Unauthorized)?;
    // ...
}
```

**Migration path**: Cung cấp deprecation log cho 1 minor version trước khi xóa query param support.

---

## SEC-HIGH-02: Không Có JWT Revocation [P2 — Sprint 2]

**File**: [src/auth.rs:30-32](../../src/auth.rs)  
**Rủi ro**: Stolen refresh token valid 90 ngày.

### Giải Pháp

**Phase 1 (Nhanh)**: Giảm refresh token TTL + tăng security stamp rotation:

```rust
// src/auth.rs — giảm mobile refresh từ 90 xuống 30 ngày
pub static MOBILE_REFRESH_VALIDITY: LazyLock<TimeDelta> = 
    LazyLock::new(|| TimeDelta::try_days(30).unwrap());  // Giảm từ 90 → 30

// Thêm config để operator điều chỉnh
pub fn get_refresh_validity(device_type: DeviceType) -> TimeDelta {
    let days = match device_type {
        DeviceType::Mobile => CONFIG.mobile_refresh_validity_days(),  // Default: 30
        _                  => CONFIG.default_refresh_validity_days(), // Default: 30
    };
    TimeDelta::try_days(days.into()).unwrap_or(TimeDelta::try_days(30).unwrap())
}
```

**Phase 2 (Trung hạn)**: Token revocation list dựa trên `security_stamp`:

```rust
// Cơ chế hiện có: security_stamp trong User model đã là token revocation mechanism!
// Khi user đổi password → security_stamp thay đổi → tất cả cũ JWTs invalid

// Cần thêm: "logout all devices" endpoint
#[post("/accounts/logout-all")]
async fn logout_all_devices(user: Headers, conn: DbConn) -> EmptyResult {
    // Cập nhật security_stamp → invalidate tất cả JWTs
    User::update_security_stamp(&user.user.uuid, &conn).await?;
    // Xóa tất cả devices (push tokens)
    Device::delete_all_by_user(&user.user.uuid, &conn).await?;
    Ok(())
}
```

**Phase 3 (Dài hạn)**: DB-backed revocation list:

```sql
CREATE TABLE revoked_tokens (
    jti         VARCHAR(40) PRIMARY KEY,    -- JWT ID claim
    user_uuid   VARCHAR(40) NOT NULL,
    revoked_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL        -- Auto-cleanup sau token expiry
);

CREATE INDEX idx_revoked_tokens_exp ON revoked_tokens(expires_at);
```

```rust
// Trong JWT validation
async fn validate_jwt_not_revoked(jti: &str, conn: &DbConn) -> Result<(), Error> {
    if RevokedToken::exists(jti, conn).await? {
        err!("Token has been revoked");
    }
    Ok(())
}

// Cleanup job
async fn cleanup_expired_revoked_tokens(conn: &DbConn) {
    RevokedToken::delete_expired(conn).await.ok();
}
```

**Config mới**:
```bash
DEFAULT_REFRESH_VALIDITY_DAYS=30    # Giảm từ implicit 30d
MOBILE_REFRESH_VALIDITY_DAYS=30    # Giảm từ 90d
ACCESS_TOKEN_VALIDITY_HOURS=2      # Giữ nguyên
TOKEN_REVOCATION_ENABLED=false     # Phase 3 opt-in
```

---

## SEC-HIGH-03: SSRF / DNS Rebinding qua Icon Proxy [P2 — Sprint 1]

**File**: [src/http_client.rs:219-236](../../src/http_client.rs)  
**Rủi ro**: Attacker fetch internal resources.

### Giải Pháp

**Fix 1**: Đổi default của `http_request_block_non_global_ips` thành `true`:

```rust
// src/config.rs — trong make_config!
http_request_block_non_global_ips: bool, false, def, true;  // ĐỔI TỪ false → true
```

**Fix 2**: Prevent DNS Rebinding — verify IP sau khi resolve:

```rust
// src/http_client.rs

struct SecurityAwareConnector {
    inner: HttpsConnector<HttpConnector>,
}

// Custom connector: verify resolved IP trước khi connect
impl Service<Uri> for SecurityAwareConnector {
    async fn call(&mut self, uri: Uri) -> Result<Self::Response, Self::Error> {
        let host = uri.host().ok_or("Missing host")?;
        
        // Resolve hostname
        let addrs = tokio::net::lookup_host(format!("{host}:80")).await?;
        
        // Verify TẤT CẢ resolved IPs (chống DNS rebinding)
        for addr in addrs {
            let ip = addr.ip();
            if !ip.is_global() && CONFIG.http_request_block_non_global_ips() {
                return Err(format!("Blocked: {host} resolves to non-global IP {ip}").into());
            }
        }
        
        // Proceed với connection
        self.inner.call(uri).await
    }
}
```

**Fix 3**: Enforce block list cho icon domains:

```rust
// src/api/icons.rs — thêm domain blocklist
const BLOCKED_DOMAINS: &[&str] = &[
    "localhost",
    "metadata.google.internal",  // GCP metadata
    "169.254.169.254",           // AWS/Azure metadata IP
    "100.100.100.200",           // Alibaba Cloud metadata
];

fn validate_icon_domain(domain: &str) -> Result<(), Error> {
    if BLOCKED_DOMAINS.contains(&domain.to_lowercase().as_str()) {
        err!("Domain blocked for security reasons");
    }
    Ok(())
}
```

---

## SEC-HIGH-04: Rate Limiting Chỉ Theo IP [P2 — Sprint 2]

**File**: [src/ratelimit.rs](../../src/ratelimit.rs)  
**Rủi ro**: Distributed credential stuffing bypass.

### Giải Pháp

**Thêm per-account rate limiting**:

```rust
// src/ratelimit.rs

pub async fn check_login_rate_limit(
    ip: IpAddr,
    username: &str,
    conn: &DbConn,
) -> Result<(), Error> {
    // Check 1: Per-IP (hiện có)
    check_rate_limit("login_ip", &ip.to_string(), 
        CONFIG.login_ratelimit_max_burst(), 60).await?;
    
    // Check 2: Per-account (MỚI)
    let account_key = format!("login_account:{}", 
        // Hash username để tránh leak email trong Redis keys
        hex::encode(Sha256::digest(username.as_bytes()))
    );
    
    let account_attempts = CACHE.increment(&account_key, Duration::from_secs(900)).await
        .unwrap_or(1);
    
    if account_attempts > CONFIG.account_lockout_threshold() as i64 {
        // Log suspicious activity
        audit::emit(AuditEntry {
            event_type: AuditEventType::AccountLockoutThresholdReached,
            actor_email: Some(username.to_string()),
            ip_address: Some(ip),
            metadata: json!({
                "attempts": account_attempts,
                "window_minutes": 15,
            }),
            ..Default::default()
        });
        
        // Tùy chọn: soft lockout (không lock ngay, nhưng require CAPTCHA/email)
        err!("Too many login attempts for this account. Please try again later.");
    }
    
    // Check 3: Anomaly detection — cùng username từ nhiều IPs
    if CONFIG.account_enumeration_detection() {
        detect_credential_stuffing(username, ip, conn).await?;
    }
    
    Ok(())
}

async fn detect_credential_stuffing(
    username: &str,
    ip: IpAddr, 
    conn: &DbConn,
) -> Result<(), Error> {
    // Nếu cùng username attempt từ > 5 IPs khác nhau trong 15 phút → alert
    let key = format!("cs_detect:{}", hex::encode(Sha256::digest(username.as_bytes())));
    let ip_key = format!("{key}:ip:{ip}");
    
    // Mark IP đã thử username này
    CACHE.set(&ip_key, "1", Duration::from_secs(900)).await.ok();
    
    // Count unique IPs (approximate via counter)
    let unique_ips = CACHE.increment(&key, Duration::from_secs(900)).await.unwrap_or(1);
    
    if unique_ips > 5 {
        // Alert security team
        audit::emit(AuditEntry {
            event_type: AuditEventType::PossibleCredentialStuffing,
            severity: Severity::Critical,
            actor_email: Some(username.to_string()),
            ip_address: Some(ip),
            metadata: json!({"unique_ips_count": unique_ips}),
            ..Default::default()
        });
    }
    
    Ok(())
}
```

**Validate reverse proxy IP headers**:

```rust
// src/util.rs — trusted proxy validation

pub fn get_real_ip(req: &Request<'_>) -> IpAddr {
    if CONFIG.trusted_proxies().is_empty() {
        // Không có trusted proxy config — dùng direct connection IP
        return req.remote().map(|r| r.ip()).unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    }
    
    let direct_ip = req.remote().map(|r| r.ip()).unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    
    // Chỉ trust X-Forwarded-For nếu request đến từ trusted proxy
    if is_trusted_proxy(direct_ip) {
        req.headers().get_one("X-Forwarded-For")
            .and_then(|xff| xff.split(',').next())
            .and_then(|ip| ip.trim().parse().ok())
            .unwrap_or(direct_ip)
    } else {
        // Không tin X-Forwarded-For từ untrusted source
        direct_ip
    }
}
```

**Config mới**:
```bash
ACCOUNT_LOCKOUT_THRESHOLD=10        # Attempts before account soft-lock (per 15 min)
ACCOUNT_ENUMERATION_DETECTION=true  # Detect credential stuffing
TRUSTED_PROXIES=10.0.0.0/8         # CIDR của reverse proxy (cho X-Forwarded-For trust)
```

---

## SEC-MED-01: Password Hint Lưu Plaintext [P3 — Sprint 3]

**File**: [src/db/models/user.rs:42](../../src/db/models/user.rs)  
**Rủi ro**: Database leak → hint tiết lộ password pattern.

### Giải Pháp

**Option A (Đơn giản)**: Chỉ hiển thị hint sau khi verify email/2FA:

```rust
// src/api/identity.rs — khi trả lời prelogin
// Không trả hint trong prelogin response nếu chưa verify
// Hint chỉ được trả sau full authentication thành công

// TRƯỚC: trả hint trong prelogin (trước khi auth)
// SAU: chỉ trả hint sau login thành công

// Trong login response (sau auth):
"PasswordHint": if user.verified_at.is_some() || user.password_hint.is_none() {
    user.password_hint.clone()  // Chỉ trả nếu đã verify email
} else {
    None  // Hint bị giữ lại nếu email chưa verify
},
```

**Option B (Mạnh hơn)**: Encrypt hint với org key hoặc server key:

```rust
// Encrypt hint trước khi lưu
fn encrypt_password_hint(hint: &str) -> Result<String, Error> {
    // AES-256-GCM với server master key
    let encrypted = aes_gcm_encrypt(hint.as_bytes(), get_server_master_key())?;
    Ok(base64::encode(encrypted))
}

fn decrypt_password_hint(encrypted: &str) -> Result<String, Error> {
    let bytes = base64::decode(encrypted)?;
    let decrypted = aes_gcm_decrypt(&bytes, get_server_master_key())?;
    Ok(String::from_utf8(decrypted)?)
}
```

**Recommendation**: Option A là đơn giản và hiệu quả nhất. Hint không nên accessible trước authentication.

---

## SEC-MED-02: SSO Auto-Provisioning Bypass SIGNUPS_ALLOWED [P3 — Sprint 3]

**File**: [src/sso.rs](../../src/sso.rs)  
**Rủi ro**: Unauthorized user provisioning qua SSO.

### Giải Pháp

```rust
// src/sso.rs — thêm IdP group whitelist check

pub async fn provision_sso_user(
    claims: &IdTokenClaims,
    conn: &DbConn,
) -> Result<User, Error> {
    let email = claims.email().ok_or_else(|| Error::new("Missing email claim", ""))?;
    
    // Check SIGNUPS_ALLOWED (existing check)
    if !CONFIG.signups_allowed() && User::find_by_email(email, conn).await?.is_none() {
        // NEW: Check SSO group whitelist
        if !CONFIG.sso_allowed_groups().is_empty() {
            let user_groups: Vec<String> = claims.additional_claims()
                .get("groups")
                .and_then(|g| serde_json::from_value(g.clone()).ok())
                .unwrap_or_default();
            
            let allowed: HashSet<&str> = CONFIG.sso_allowed_groups()
                .split(',')
                .map(|s| s.trim())
                .collect();
            
            let has_allowed_group = user_groups.iter()
                .any(|g| allowed.contains(g.as_str()));
            
            if !has_allowed_group {
                audit::emit(AuditEntry {
                    event_type: AuditEventType::SsoProvisioningBlocked,
                    actor_email: Some(email.to_string()),
                    metadata: json!({
                        "reason": "not_in_allowed_groups",
                        "user_groups": user_groups,
                    }),
                    ..Default::default()
                });
                err!("Your account is not authorized to access this system. \
                      Contact your administrator.");
            }
        } else {
            // Không có group whitelist → respect SIGNUPS_ALLOWED=false
            err!("New account registration is not allowed");
        }
    }
    
    // Continue với provisioning...
}
```

**Config mới**:
```bash
SSO_ALLOWED_GROUPS=""               # Comma-separated list của allowed IdP groups
                                    # Empty = allow all IdP users (current behavior)
SSO_REQUIRE_EMAIL_DOMAIN=""         # Only allow @company.com emails
```

---

## SEC-MED-03: Emergency Access — Email Delivery Failure [P3 — Sprint 3]

**File**: [src/db/models/emergency_access.rs](../../src/db/models/emergency_access.rs)  
**Rủi ro**: Vault accessed without grantor knowledge due to email failure.

### Giải Pháp

**Thêm multi-channel notification**:

```rust
// src/api/core/emergency_access.rs — notification enhancement

async fn notify_grantor_emergency_request(
    grantor: &User,
    grantee: &User,
    access: &EmergencyAccess,
    conn: &DbConn,
) -> Result<(), Error> {
    // 1. Email notification (existing)
    let email_result = mail::send_emergency_access_request(
        &grantor.email, 
        &grantee.email,
        access.wait_time_days
    ).await;
    
    // 2. WebSocket in-app notification (NEW)
    if let Err(e) = email_result {
        warn!("Email notification failed for emergency access request: {e}");
        // Tăng mức độ retry
    }
    
    // Push in-app notification qua WebSocket
    notifications::send_notification_to_user(&grantor.uuid, &json!({
        "type": "EmergencyAccessRequest",
        "granteeEmail": grantee.email,
        "waitTimeDays": access.wait_time_days,
        "expiresAt": (Utc::now() + Duration::days(access.wait_time_days as i64)).to_rfc3339(),
        "message": format!("{} has requested emergency access to your vault. \
                           You have {} days to deny.", 
                           grantee.email, access.wait_time_days),
    })).await;
    
    // 3. Send reminders (NGÀY T-7, T-3, T-1 trước khi grant)
    // Thêm vào job scheduler — mỗi ngày check và gửi reminder nếu cần
    
    Ok(())
}

// Job hàng ngày: gửi reminder cho grantor
async fn emergency_access_reminder_job(conn: &DbConn) {
    let pending = EmergencyAccess::find_pending_requiring_reminder(conn).await
        .unwrap_or_default();
    
    for access in pending {
        let days_remaining = access.days_until_auto_grant();
        
        // Gửi reminder ở T-7, T-3, T-1
        if [7, 3, 1].contains(&days_remaining) {
            if let Some(grantor) = User::find_by_uuid(&access.grantor_uuid, conn).await.ok().flatten() {
                mail::send_emergency_access_reminder(
                    &grantor.email,
                    days_remaining,
                ).await.ok();
                
                // Also push WebSocket notification
                notifications::send_notification_to_user(&grantor.uuid, &json!({
                    "type": "EmergencyAccessReminderMessage",
                    "daysRemaining": days_remaining,
                })).await;
            }
        }
    }
}
```

---

## SEC-MED-04: config.json Chứa Secrets [P3 — Sprint 2]

**File**: [src/config.rs:20-22](../../src/config.rs)  
**Rủi ro**: Credentials exposed nếu data directory bị misconfigured.

### Giải Pháp

**Fix 1**: Không lưu sensitive fields vào config.json:

```rust
// src/config.rs — trong make_config! macro
// Đánh dấu fields là "env_only" — không serialize vào config.json

smtp_password:          String, true,  env_only, "";  // ← env_only = không lưu config.json
sso_client_secret:      String, true,  env_only, "";
admin_token:            String, true,  env_only, "";
push_relay_uri:         String, false, env_only, "";

// Khi lưu config.json, skip env_only fields
fn save_to_config_file() {
    let config = self.to_config_file_json();  // Chỉ include non-env_only fields
    // ...
}
```

**Fix 2**: Cảnh báo nếu config.json chứa sensitive data:

```rust
// Startup check
pub async fn audit_config_file_for_secrets() {
    if let Ok(content) = tokio::fs::read_to_string(CONFIG_FILE.as_str()).await {
        let config_json: Value = serde_json::from_str(&content).unwrap_or_default();
        
        const SENSITIVE_KEYS: &[&str] = &[
            "smtp_password", "sso_client_secret", "admin_token", 
            "push_relay_uri", "ldap_bind_password", "scim_token",
        ];
        
        for key in SENSITIVE_KEYS {
            if config_json.get(key).is_some() {
                warn!(
                    "SECURITY WARNING: Sensitive field '{}' found in config.json. \
                     Move this to environment variable instead.",
                    key
                );
            }
        }
    }
}
```

**Fix 3**: File permission check:

```rust
// Warn nếu config.json readable by others
#[cfg(unix)]
async fn check_config_file_permissions() {
    use std::os::unix::fs::PermissionsExt;
    
    if let Ok(meta) = tokio::fs::metadata(CONFIG_FILE.as_str()).await {
        let mode = meta.permissions().mode();
        if mode & 0o044 != 0 {  // Readable by group hoặc others
            warn!(
                "SECURITY WARNING: config.json is world/group readable (mode: {:o}). \
                 Run: chmod 600 {}",
                mode, CONFIG_FILE.as_str()
            );
        }
    }
}
```

---

## SEC-MED-05: Push Relay Metadata Leak [P3 — Sprint 4]

**File**: [src/api/push.rs](../../src/api/push.rs)  
**Rủi ro**: Privacy leak về usage patterns tới bên thứ ba.

### Giải Pháp

**Fix 1**: Minimize metadata gửi đến relay:

```rust
// Chỉ gửi push token và event type — không gửi user UUID, org UUID
let push_payload = json!({
    "device_token": device.push_token,
    "type": event_type,
    // KHÔNG bao gồm: user_uuid, org_uuid, device_uuid chi tiết
});
```

**Fix 2**: Thêm documentation rõ ràng:

```bash
# Trong config template
# PUSH_RELAY_URI: External push relay server for mobile notifications.
# WARNING: This server receives device push tokens and sync event timing.
# It does NOT receive vault data (end-to-end encrypted).
# Privacy: consider self-hosting at https://github.com/dani-garcia/vaultwarden#push
PUSH_RELAY_URI=https://push.bitwarden.com
```

**Fix 3 (Dài hạn)**: Document hướng dẫn self-host push relay.

---

## SEC-LOW-01: RSA-2048 Không Future-Proof [P4]

**Rủi ro**: NIST khuyến nghị post-quantum trước 2035.

### Giải Pháp

**Phase 1**: Thêm key rotation:

```rust
// src/auth.rs — key rotation support
pub async fn rotate_jwt_signing_key(operator: &Operator) -> Result<(), Error> {
    // Archive current key
    let current = operator.read(RSA_KEY_FILENAME).await?;
    operator.write(
        &format!("rsa_key_backup_{}.pem", Utc::now().timestamp()),
        current
    ).await?;
    
    // Generate new key
    let new_key = generate_rsa_key()?;
    operator.write(RSA_KEY_FILENAME, new_key).await?;
    
    // Force all users to re-login (update security stamps)
    // Optional: grace period where both old and new keys valid
    
    audit::emit(AuditEntry {
        event_type: AuditEventType::KeyRotation,
        severity: Severity::Warn,
        metadata: json!({"key_type": "rsa_jwt_signing"}),
        ..Default::default()
    });
    
    Ok(())
}
```

**Config mới**:
```bash
JWT_KEY_ROTATION_SCHEDULE=""        # Cron expression, empty = manual only
```

**Phase 2 (ES256)**: Support ECDSA P-256 (`ring::signature::ECDSA_P256_SHA256_ASN1_SIGNING`).  
Note: Cần verify Bitwarden client compatibility với ES256 trước khi implement.

---

## SEC-LOW-02: Anonymous WebSocket Rate Limiting [P4]

**File**: [src/api/notifications.rs](../../src/api/notifications.rs)

### Giải Pháp

```rust
// Thêm rate limiting cho anonymous WS endpoint
static ANON_WS_RATE_LIMITER: LazyLock<...> = ...;

#[get("/notifications/anonymous")]
async fn anonymous_websocket(
    ws: WebSocket,
    req: &Request<'_>,
) -> Result<Channel<'static>, Status> {
    let ip = get_real_ip(req).to_string();
    
    // Rate limit: 10 connections per minute per IP
    check_rate_limit("anon_ws", &ip, 10, 60).await
        .map_err(|_| Status::TooManyRequests)?;
    
    // Max concurrent anonymous connections
    if WS_ANONYMOUS_COUNT.load(Ordering::Relaxed) >= CONFIG.max_anonymous_ws_connections() {
        return Err(Status::TooManyRequests);
    }
    
    // ... rest
}
```

**Config mới**:
```bash
MAX_ANONYMOUS_WS_CONNECTIONS=1000   # Default
```

---

## SEC-LOW-03: KDF Iterations Không Enforce [P4]

**File**: [src/db/models/user.rs:128](../../src/db/models/user.rs)

### Giải Pháp

```rust
// src/api/core/accounts.rs — khi đăng ký và khi đổi password

fn validate_kdf_config(kdf_type: u8, iterations: u32, memory: u32) -> Result<(), Error> {
    match kdf_type {
        0 => {  // PBKDF2
            const MIN_PBKDF2_ITERATIONS: u32 = 600_000;
            if iterations < MIN_PBKDF2_ITERATIONS {
                err!(format!(
                    "KDF iterations ({iterations}) below minimum ({MIN_PBKDF2_ITERATIONS}). \
                     This weakens your master password protection."
                ));
            }
        }
        1 => {  // Argon2id
            const MIN_ARGON2_MEMORY_KB: u32 = 65_536;  // 64 MB
            const MIN_ARGON2_ITERATIONS: u32 = 3;
            
            if memory < MIN_ARGON2_MEMORY_KB {
                err!(format!(
                    "Argon2id memory ({memory} KB) below minimum ({MIN_ARGON2_MEMORY_KB} KB)"
                ));
            }
            if iterations < MIN_ARGON2_ITERATIONS {
                err!(format!(
                    "Argon2id iterations ({iterations}) below minimum ({MIN_ARGON2_ITERATIONS})"
                ));
            }
        }
        _ => err!("Unknown KDF type"),
    }
    Ok(())
}
```

**Config mới**:
```bash
MIN_PBKDF2_ITERATIONS=600000
MIN_ARGON2_MEMORY_KB=65536
ENFORCE_MIN_KDF=true                # false = warn only (backward compat)
```

---

## SEC-LOW-04: SQLite Backup Exposure [P4]

**File**: [src/db/mod.rs](../../src/db/mod.rs)

### Giải Pháp

```rust
// Backup vào directory NGOÀI web root
fn get_backup_path() -> String {
    let backup_dir = CONFIG.backup_folder();
    
    // Warn nếu backup dir nằm trong data folder (web-accessible)
    if backup_dir.starts_with(CONFIG.data_folder()) {
        warn!(
            "SQLite backup location ({}) may be web-accessible. \
             Set BACKUP_FOLDER to a directory outside DATA_FOLDER.",
            backup_dir
        );
    }
    
    backup_dir
}
```

**Config mới**:
```bash
BACKUP_FOLDER=data/backups         # Có thể set ra ngoài web root
```

Đồng thời: hướng dẫn trong nginx config để block `*.bak` files:
```nginx
location ~* \.(bak|backup|sql|sqlite3)$ {
    deny all;
}
```

---

## SEC-LOW-05: CSP Không Được Enforce [P4]

Giải pháp đã được cover trong **SOL-001** (CR-001 — `SecurityHeadersFairing`).

Tóm tắt: thêm `SecurityHeadersFairing` vào `src/main.rs` để set:
```
Content-Security-Policy: default-src 'self'; ...
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

---

## Priority Matrix & Implementation Timeline

| ID | Severity | Giải pháp | Sprint | Effort |
|----|----------|----------|--------|--------|
| SEC-CRIT-01 | Critical | Reject non-Argon2 token | **Ngay** | 2 ngày |
| SEC-CRIT-02 | Critical | Double-confirmation + audit | **Ngay** | 2 ngày |
| SEC-HIGH-01 | High | Xóa JWT query param | Sprint 1 | 1 ngày |
| SEC-HIGH-03 | High | Block non-global IPs default | Sprint 1 | 2 ngày |
| SEC-HIGH-04 | High | Per-account rate limit | Sprint 2 | 3 ngày |
| SEC-HIGH-02 | High | Token TTL reduce + logout-all | Sprint 2 | 3 ngày |
| SEC-MED-04 | Medium | env_only config fields | Sprint 2 | 2 ngày |
| SEC-MED-02 | Medium | SSO group whitelist | Sprint 3 | 2 ngày |
| SEC-MED-01 | Medium | Hint visible after auth only | Sprint 3 | 1 ngày |
| SEC-MED-03 | Medium | Multi-channel emergency alert | Sprint 3 | 3 ngày |
| SEC-MED-05 | Medium | Push relay documentation | Sprint 4 | 1 ngày |
| SEC-LOW-03 | Low | KDF minimum enforcement | Sprint 4 | 1 ngày |
| SEC-LOW-02 | Low | Anon WS rate limiting | Sprint 4 | 1 ngày |
| SEC-LOW-04 | Low | SQLite backup path | Sprint 4 | 0.5 ngày |
| SEC-LOW-01 | Low | RSA key rotation | Sprint 5 | 1 tuần |
| SEC-LOW-05 | Low | CSP headers | Sprint 1 | (CR-001) |

---

*Status: Draft | Ngày: 2026-04-12*
