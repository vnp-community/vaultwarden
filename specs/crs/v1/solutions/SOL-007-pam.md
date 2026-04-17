# SOL-007: Giải Pháp Thực Hiện — Privileged Access Management (PAM)

> **Giải pháp cho**: CR-007  
> **Ngày**: 2026-04-12  
> **Trạng thái**: ✅ Implemented  
> **Kiến trúc thay đổi**: Trung bình — mở rộng Cipher model, thêm checkout/rotation engine  
> **Cập nhật**: 2026-04-17 — Verified full implementation in codebase

---

## 1. Tổng Quan Giải Pháp

PAM là extension của password manager core. Giải pháp tối ưu tận dụng:
- **Cipher model** hiện có → thêm `is_privileged`, `privileged_config` fields
- **Approval workflow** từ CR-004 → reuse cho privileged access
- **Audit log** từ CR-002 → credential access recording
- **Reqwest HTTP client** → ITSM integration

**Phạm vi v2.1**: Credential checkout + rotation + ITSM. SSH proxy là future scope.

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/pam/mod.rs` | PAM module tổng quan |
| `src/pam/checkout.rs` | Checkout lifecycle management |
| `src/pam/rotation.rs` | Automated password rotation engine |
| `src/pam/itsm.rs` | ITSM integration (ServiceNow, Jira) |
| `src/api/core/pam.rs` | PAM REST API routes |
| `src/db/models/checkout.rs` | Checkout records |
| `src/db/models/privileged_config.rs` | Per-cipher privileged config |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/db/models/cipher.rs` | Thêm `is_privileged`, `privileged_config_uuid` fields |
| `src/api/core/ciphers.rs` | Check privileged flag, trigger checkout flow |
| `src/config.rs` | Thêm PAM_* config keys |
| `src/main.rs` | Thêm checkout expiry + rotation jobs |

### 2.3 Database Migrations

```sql
-- migrations/postgresql/YYYYMMDD_pam/up.sql

-- Privileged cipher configuration
CREATE TABLE privileged_configs (
    uuid                        VARCHAR(40) PRIMARY KEY,
    cipher_uuid                 VARCHAR(40) NOT NULL UNIQUE REFERENCES ciphers(uuid) ON DELETE CASCADE,
    requires_approval           BOOLEAN NOT NULL DEFAULT FALSE,
    approval_group_uuid         VARCHAR(40),
    max_checkout_duration_minutes INTEGER NOT NULL DEFAULT 60,
    auto_rotate_after_checkout  BOOLEAN NOT NULL DEFAULT FALSE,
    rotation_target_type        VARCHAR(20),    -- 'ssh', 'rdp', 'mysql', 'postgres', 'custom'
    rotation_target_config      JSONB,          -- Connection details (encrypted)
    session_recording_enabled   BOOLEAN NOT NULL DEFAULT FALSE,
    view_count_limit            INTEGER,        -- NULL = unlimited
    concurrent_access_limit     INTEGER,        -- NULL = unlimited
    requires_itsm_ticket        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Credential checkouts
CREATE TABLE checkouts (
    uuid                VARCHAR(40) PRIMARY KEY,
    cipher_uuid         VARCHAR(40) NOT NULL REFERENCES ciphers(uuid),
    user_uuid           VARCHAR(40) NOT NULL REFERENCES users(uuid),
    org_uuid            VARCHAR(40),
    
    -- Checkout details
    checked_out_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at          TIMESTAMPTZ NOT NULL,
    checked_in_at       TIMESTAMPTZ,
    status              VARCHAR(20) NOT NULL DEFAULT 'active',  -- active, expired, checked_in
    
    -- Justification
    justification       TEXT NOT NULL,
    itsm_ticket         VARCHAR(100),           -- ServiceNow/Jira ticket number
    approval_request_uuid VARCHAR(40),          -- Reference to approval (CR-004)
    
    -- Access tracking
    ip_address          INET,
    access_count        INTEGER NOT NULL DEFAULT 0,  -- How many times credential was viewed
    
    -- Rotation
    rotation_status     VARCHAR(20),            -- pending, completed, failed, not_required
    rotation_completed_at TIMESTAMPTZ,
    rotation_error      TEXT
);

CREATE INDEX idx_checkouts_cipher ON checkouts(cipher_uuid);
CREATE INDEX idx_checkouts_user ON checkouts(user_uuid);
CREATE INDEX idx_checkouts_status ON checkouts(status, expires_at);

-- Rotation history
CREATE TABLE rotation_history (
    id              BIGSERIAL PRIMARY KEY,
    cipher_uuid     VARCHAR(40) NOT NULL,
    checkout_uuid   VARCHAR(40),
    trigger_type    VARCHAR(20) NOT NULL,       -- 'checkout', 'schedule', 'manual'
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    status          VARCHAR(20) NOT NULL,       -- success, failed
    error_message   TEXT
);

-- Thêm vào ciphers
ALTER TABLE ciphers ADD COLUMN is_privileged BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE ciphers ADD COLUMN privileged_config_uuid VARCHAR(40) 
    REFERENCES privileged_configs(uuid);
```

---

## 3. Thiết Kế Chi Tiết

### 3.1 Checkout Flow

**File**: `src/pam/checkout.rs`

```rust
pub struct CheckoutManager;

impl CheckoutManager {
    pub async fn request_checkout(
        cipher_uuid: &str,
        user: &User,
        request: &CheckoutRequest,
        conn: &DbConn,
    ) -> Result<CheckoutResult, Error> {
        let cipher = Cipher::find_by_uuid(cipher_uuid, conn).await?
            .ok_or_else(|| Error::new("Cipher not found", ""))?;
        
        // 1. Validate cipher is privileged
        if !cipher.is_privileged {
            err!("This credential does not require checkout");
        }
        
        let priv_config = PrivilegedConfig::find_by_cipher(cipher_uuid, conn).await?
            .ok_or_else(|| Error::new("Privileged config not found", ""))?;
        
        // 2. Check concurrent access limit
        if let Some(limit) = priv_config.concurrent_access_limit {
            let active = Checkout::count_active_for_cipher(cipher_uuid, conn).await?;
            if active as u32 >= limit {
                err!(format!("Maximum concurrent access limit ({limit}) reached for this credential"));
            }
        }
        
        // 3. ITSM ticket validation
        if priv_config.requires_itsm_ticket {
            let ticket = request.itsm_ticket.as_deref()
                .ok_or_else(|| Error::new("ITSM ticket number required", ""))?;
            IstmClient::validate_ticket(ticket).await?;
        }
        
        // 4. Approval check (reuse CR-004 workflow)
        let approval_uuid = if priv_config.requires_approval {
            Some(ensure_approval(cipher_uuid, user, request, &priv_config, conn).await?)
        } else {
            None
        };
        
        if let Some(ref appr_uuid) = approval_uuid {
            // Check nếu approval đã được granted
            let approval = ApprovalRequest::find_by_uuid(appr_uuid, conn).await?
                .ok_or_else(|| Error::new("Approval request not found", ""))?;
            
            if approval.status != "approved" {
                return Ok(CheckoutResult::PendingApproval { 
                    approval_request_uuid: appr_uuid.clone() 
                });
            }
        }
        
        // 5. Tạo checkout record
        let duration = std::cmp::min(
            request.requested_duration_minutes.unwrap_or(60),
            priv_config.max_checkout_duration_minutes,
        );
        
        let checkout = Checkout {
            uuid: get_uuid(),
            cipher_uuid: cipher_uuid.to_string(),
            user_uuid: user.uuid.clone(),
            checked_out_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(duration as i64),
            justification: request.justification.clone(),
            itsm_ticket: request.itsm_ticket.clone(),
            approval_request_uuid: approval_uuid,
            ip_address: request.ip_address,
            status: "active".to_string(),
            ..Default::default()
        };
        checkout.save(conn).await?;
        
        // 6. Audit (credential access)
        audit::emit(AuditEntry {
            event_type: AuditEventType::PrivilegedCheckout,
            actor_user_uuid: Some(user.uuid.clone()),
            target_resource: Some(cipher_uuid.to_string()),
            metadata: json!({
                "checkout_uuid": checkout.uuid,
                "duration_minutes": duration,
                "justification": request.justification,
                "itsm_ticket": request.itsm_ticket,
                "expires_at": checkout.expires_at,
            }),
            ..Default::default()
        });
        
        Ok(CheckoutResult::Success { checkout })
    }
    
    pub async fn checkin(
        checkout_uuid: &str,
        user_uuid: &str,
        conn: &DbConn,
    ) -> Result<(), Error> {
        let checkout = Checkout::find_by_uuid(checkout_uuid, conn).await?
            .ok_or_else(|| Error::new("Checkout not found", ""))?;
        
        // Validate ownership
        if checkout.user_uuid != user_uuid {
            err!("You can only check in your own checkouts");
        }
        
        Checkout::mark_checked_in(checkout_uuid, conn).await?;
        
        // Trigger rotation nếu configured
        let priv_config = PrivilegedConfig::find_by_cipher(&checkout.cipher_uuid, conn).await?;
        if let Some(config) = priv_config {
            if config.auto_rotate_after_checkout {
                tokio::spawn(async move {
                    let pool = DB_POOL.get().expect("pool");
                    let conn = pool.get().expect("conn");
                    RotationEngine::rotate_credential(&checkout.cipher_uuid, "checkout", &conn)
                        .await
                        .unwrap_or_else(|e| error!("Rotation failed: {e}"));
                });
            }
        }
        
        audit::emit(AuditEntry {
            event_type: AuditEventType::PrivilegedCheckin,
            actor_user_uuid: Some(user_uuid.to_string()),
            target_resource: Some(checkout.cipher_uuid.clone()),
            metadata: json!({
                "checkout_uuid": checkout_uuid,
                "checked_out_duration_minutes": 
                    (Utc::now() - checkout.checked_out_at).num_minutes(),
                "access_count": checkout.access_count,
            }),
            ..Default::default()
        });
        
        Ok(())
    }
}

// Background job: expire stale checkouts
pub async fn expire_checkouts_job(conn: &DbConn) {
    let expired = Checkout::find_expired_active(conn).await.unwrap_or_default();
    
    for checkout in expired {
        Checkout::mark_expired(&checkout.uuid, conn).await.ok();
        
        // Trigger rotation nếu configured
        let priv_config = PrivilegedConfig::find_by_cipher(&checkout.cipher_uuid, conn).await
            .ok().flatten();
        if let Some(config) = priv_config {
            if config.auto_rotate_after_checkout {
                RotationEngine::rotate_credential(&checkout.cipher_uuid, "expiry", conn)
                    .await.ok();
            }
        }
        
        audit::emit(AuditEntry {
            event_type: AuditEventType::CheckoutExpired,
            severity: Severity::Warn,
            target_resource: Some(checkout.cipher_uuid.clone()),
            metadata: json!({
                "checkout_uuid": checkout.uuid,
                "user_uuid": checkout.user_uuid,
                "expired_at": Utc::now(),
            }),
            ..Default::default()
        });
    }
}
```

### 3.2 Rotation Engine

**File**: `src/pam/rotation.rs`

```rust
pub struct RotationEngine;

impl RotationEngine {
    pub async fn rotate_credential(
        cipher_uuid: &str,
        trigger: &str,
        conn: &DbConn,
    ) -> Result<(), Error> {
        let config = PrivilegedConfig::find_by_cipher(cipher_uuid, conn).await?
            .ok_or_else(|| Error::new("No privileged config", ""))?;
        
        let rotation_config: RotationTargetConfig = serde_json::from_value(
            config.rotation_target_config.unwrap_or_default()
        ).map_err(|e| Error::new(&format!("Invalid rotation config: {e}"), ""))?;
        
        // Tạo rotation history record
        let history = RotationHistory {
            cipher_uuid: cipher_uuid.to_string(),
            trigger_type: trigger.to_string(),
            status: "running".to_string(),
            ..Default::default()
        };
        let history_id = history.insert(conn).await?;
        
        let result = match config.rotation_target_type.as_deref() {
            Some("ssh")      => self.rotate_ssh(&rotation_config, conn).await,
            Some("mysql")    => self.rotate_mysql(&rotation_config, conn).await,
            Some("postgres") => self.rotate_postgres(&rotation_config, conn).await,
            Some("custom")   => self.rotate_custom(&rotation_config, conn).await,
            t => Err(Error::new(&format!("Unknown rotation target: {t:?}"), "")),
        };
        
        match result {
            Ok(new_password) => {
                // Cập nhật cipher với password mới (encrypt client-side format)
                // Note: Server không thể tự encrypt vì không có user's symmetric key
                // Thay vào đó: flag cipher là "rotation_pending" → user phải update manually
                // Hoặc: dùng org key nếu cipher thuộc org với org key access
                Cipher::flag_rotation_pending(cipher_uuid, conn).await?;
                RotationHistory::mark_success(history_id, conn).await?;
                
                audit::emit(AuditEntry {
                    event_type: AuditEventType::CredentialRotated,
                    target_resource: Some(cipher_uuid.to_string()),
                    metadata: json!({
                        "trigger": trigger,
                        "target_type": config.rotation_target_type,
                    }),
                    ..Default::default()
                });
                
                Ok(())
            }
            Err(e) => {
                RotationHistory::mark_failed(history_id, &e.to_string(), conn).await?;
                
                // Alert
                mail::send_rotation_failure_alert(cipher_uuid, &e.to_string()).await.ok();
                
                Err(e)
            }
        }
    }
    
    async fn rotate_ssh(&self, config: &RotationTargetConfig, _conn: &DbConn) 
        -> Result<String, Error> 
    {
        use tokio::process::Command;
        
        // Tạo new password
        let new_password = generate_secure_password(32);
        
        // Connect và change password via SSH
        // Dùng ssh command với key auth
        let ssh_key_path = CONFIG.pam_rotation_ssh_key_path();
        
        let change_cmd = format!(
            "echo '{}:{}' | chpasswd",
            config.username,
            new_password
        );
        
        let result = Command::new("ssh")
            .arg("-i").arg(ssh_key_path)
            .arg("-o").arg("StrictHostKeyChecking=accept-new")
            .arg("-o").arg(format!("ConnectTimeout={}", CONFIG.pam_rotation_timeout_seconds()))
            .arg(format!("{}@{}", config.username, config.host))
            .arg(&change_cmd)
            .output()
            .await?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            err!(format!("SSH rotation failed: {stderr}"));
        }
        
        Ok(new_password)
    }
    
    async fn rotate_mysql(&self, config: &RotationTargetConfig, _conn: &DbConn)
        -> Result<String, Error>
    {
        let new_password = generate_secure_password(32);
        
        // Kết nối MySQL và thay đổi password
        // Dùng tokio::process::Command với mysql CLI
        let result = tokio::process::Command::new("mysql")
            .arg(format!("-h{}", config.host))
            .arg(format!("-P{}", config.port.unwrap_or(3306)))
            .arg(format!("-u{}", config.admin_username))
            .arg(format!("-p{}", config.admin_password))
            .arg("-e")
            .arg(format!(
                "ALTER USER '{}'@'%' IDENTIFIED BY '{}'; FLUSH PRIVILEGES;",
                config.username, new_password
            ))
            .output()
            .await?;
        
        if !result.status.success() {
            err!("MySQL rotation failed");
        }
        
        Ok(new_password)
    }
}
```

### 3.3 ITSM Integration

**File**: `src/pam/itsm.rs`

```rust
pub struct IstmClient {
    client: reqwest::Client,   // Reuse existing reqwest client pattern
}

impl IstmClient {
    pub async fn validate_ticket(ticket_number: &str) -> Result<(), Error> {
        if !CONFIG.itsm_enabled() || !CONFIG.itsm_ticket_validation() {
            return Ok(()); // ITSM not configured, skip validation
        }
        
        match CONFIG.itsm_type() {
            "servicenow" => Self::validate_servicenow_ticket(ticket_number).await,
            "jira"       => Self::validate_jira_ticket(ticket_number).await,
            t => Err(Error::new(&format!("Unknown ITSM type: {t}"), "")),
        }
    }
    
    async fn validate_servicenow_ticket(ticket: &str) -> Result<(), Error> {
        let url = format!(
            "{}/api/now/table/incident?sysparm_query=number%3D{}&sysparm_fields=state,number",
            CONFIG.itsm_servicenow_instance(),
            ticket
        );
        
        let resp = get_reqwest_client()
            .get(&url)
            .basic_auth(
                CONFIG.itsm_servicenow_user(),
                Some(CONFIG.itsm_servicenow_password()),
            )
            .send()
            .await
            .map_err(|e| Error::new(&format!("ITSM request failed: {e}"), ""))?;
        
        if !resp.status().is_success() {
            err!("Failed to verify ticket with ServiceNow");
        }
        
        let body: Value = resp.json().await
            .map_err(|e| Error::new(&format!("Invalid ITSM response: {e}"), ""))?;
        
        let result = body.get("result").and_then(|r| r.as_array())
            .ok_or_else(|| Error::new("Ticket not found", ""))?;
        
        if result.is_empty() {
            err!("Ticket not found in ServiceNow");
        }
        
        // State 6 = Resolved, 7 = Closed — reject closed/resolved tickets
        let state = result[0].get("state").and_then(|s| s.as_str()).unwrap_or("7");
        if state == "6" || state == "7" {
            err!("Cannot check out credential: ticket is resolved/closed");
        }
        
        Ok(())
    }
}
```

### 3.4 PAM Dashboard API

```rust
// GET /api/admin/pam/dashboard
#[get("/admin/pam/dashboard")]
async fn pam_dashboard(_admin: AdminHeaders, conn: DbConn) -> JsonResult {
    let (active, expired, pending_rotation, failed_rotations) = tokio::try_join!(
        Checkout::count_active(&conn),
        Checkout::count_expired_unhandled(&conn),
        Checkout::count_pending_rotation(&conn),
        RotationHistory::count_failed_last_24h(&conn),
    )?;
    
    let privileged_count = Cipher::count_privileged(&conn).await?;
    let pending_approvals = ApprovalRequest::count_pending_pam(&conn).await?;
    
    Ok(Json(json!({
        "active_checkouts": active,
        "overdue_checkouts": expired,
        "rotations_pending": pending_rotation,
        "rotations_failed_24h": failed_rotations,
        "privileged_ciphers_count": privileged_count,
        "approval_requests_pending": pending_approvals,
    })))
}
```

---

## 4. Config Variables Mới

```bash
# PAM Core
PAM_ENABLED=false
PAM_ROTATION_ENABLED=false
PAM_ROTATION_WORKER_CONCURRENCY=5
PAM_ROTATION_TIMEOUT_SECONDS=60
PAM_ROTATION_SSH_KEY_PATH=data/rotation_key
PAM_CHECKOUT_EXPIRY_CHECK_INTERVAL_SECONDS=60

# ITSM
ITSM_ENABLED=false
ITSM_TYPE=servicenow               # servicenow|jira
ITSM_SERVICENOW_INSTANCE=""
ITSM_SERVICENOW_USER=""
ITSM_SERVICENOW_PASSWORD=""        # Masked
ITSM_TICKET_REQUIRED=false
ITSM_TICKET_VALIDATION=true
```

---

## 5. API Endpoints Mới

| Method | Path | Auth | Mô tả |
|--------|------|------|-------|
| POST | `/api/ciphers/{id}/checkout` | User | Checkout privileged credential |
| POST | `/api/ciphers/{id}/checkin` | User | Check in credential |
| GET | `/api/ciphers/{id}/checkouts` | Admin/Owner | Active checkouts |
| DELETE | `/api/checkouts/{id}` | Admin | Force check-in |
| GET | `/api/checkouts?active=true` | Admin | All active checkouts |
| GET | `/api/admin/pam/dashboard` | Admin | PAM summary dashboard |
| POST | `/api/admin/pam/ciphers/{id}/rotate` | Admin | Manual rotation trigger |

---

## 6. Kế Hoạch Triển Khai

### Sprint 1: Privileged Cipher Type
- DB migration
- `is_privileged` field + `privileged_configs` table
- UI flag trong cipher CRUD

### Sprint 2–4: Checkout System
- `src/pam/checkout.rs`
- Checkout API endpoints
- Expiry background job

### Sprint 5–8: Auto-Rotation Engine
- SSH rotation
- MySQL/PostgreSQL rotation
- Rotation history tracking

### Sprint 9–10: ITSM + Dashboard
- ServiceNow validation
- PAM dashboard API

---

*Status: ✅ Implemented | Ngày cập nhật: 2026-04-17*

## Implementation Notes
- `src/pam/mod.rs`, `src/pam/checkout.rs` (99 lines), `src/pam/rotation.rs` (114 lines), `src/pam/itsm.rs` — Full PAM module
- `src/api/core/pam.rs` — PAM REST API (checkout, checkin, dashboard, manual rotate)
- `src/db/models/pam.rs` — Checkout, PrivilegedConfig, RotationHistory models
- DB migration: `2026-04-15-000007_sol_007_pam` — privileged_configs, checkouts, rotation_history tables
- `ciphers.is_privileged` + `ciphers.privileged_config_uuid` fields added
- SSH, MySQL, PostgreSQL rotation engines implemented
- ServiceNow + Jira ticket validation integrated
- Checkout expiry background job + auto-rotation on checkin/expiry
- Reuses CR-004 approval workflow for privileged access
