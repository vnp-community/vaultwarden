# SOL-002: Giải Pháp Thực Hiện — System-Wide Tamper-Evident Audit Log & SIEM Integration

> **Giải pháp cho**: CR-002  
> **Ngày**: 2026-04-12  
> **Trạng thái**: Draft  
> **Kiến trúc thay đổi**: Trung bình — thêm audit subsystem mới, không thay đổi core API

---

## 1. Tổng Quan Giải Pháp

Vaultwarden hiện có `src/db/models/event.rs` và `src/api/core/events.rs` cho org-level events. Giải pháp **mở rộng** hệ thống này thay vì tạo mới hoàn toàn:

1. **Tách audit subsystem**: Module mới `src/audit.rs` với async channel emitter
2. **Mở rộng event model**: Thêm system-level events bên cạnh org events
3. **Hash chain**: Mỗi entry có SHA-256 của entry trước
4. **SIEM forward**: Background task dùng `reqwest` (đã có sẵn)
5. **Append-only enforcement**: DB-level policy không cho DELETE

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/audit.rs` | AuditEmitter, AuditEntry model, hash chain logic |
| `src/siem.rs` | SIEM forwarder (Splunk HEC, Syslog, Sentinel) |
| `src/api/core/audit.rs` | REST API cho audit log queries |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/api/identity.rs` | Emit audit events cho login success/failure |
| `src/api/admin.rs` | Emit audit events cho admin actions |
| `src/api/core/accounts.rs` | Emit events cho password change, 2FA changes |
| `src/api/core/ciphers.rs` | Emit events cho file download, bulk ops |
| `src/api/notifications.rs` | Emit events cho WebSocket sessions |
| `src/config.rs` | Thêm AUDIT_* config keys |
| `src/main.rs` | Khởi động audit emitter, SIEM forwarder |
| `src/api/core/mod.rs` | Mount audit API routes |

### 2.3 Database Migration

```sql
-- migrations/postgresql/YYYYMMDD_audit_log/up.sql

CREATE TABLE audit_entries (
    id              BIGSERIAL PRIMARY KEY,              -- Monotonic, không thể skip
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type      VARCHAR(100) NOT NULL,
    severity        VARCHAR(20) NOT NULL DEFAULT 'INFO', -- INFO, WARN, CRITICAL
    actor_user_uuid VARCHAR(40),
    actor_email     VARCHAR(255),                       -- Denormalized để preserve history
    target_resource VARCHAR(500),
    ip_address      INET,
    user_agent      TEXT,
    org_uuid        VARCHAR(40),
    metadata        JSONB NOT NULL DEFAULT '{}',
    
    -- Tamper-evident hash chain
    prev_hash       BYTEA NOT NULL,                     -- SHA-256 của entry trước (zero bytes cho entry đầu)
    entry_hash      BYTEA NOT NULL,                     -- SHA-256 của toàn bộ entry này
    
    -- SIEM delivery tracking
    siem_delivered  BOOLEAN NOT NULL DEFAULT FALSE,
    siem_delivered_at TIMESTAMPTZ,
    siem_attempts   SMALLINT NOT NULL DEFAULT 0
);

-- Index cho queries thường gặp
CREATE INDEX idx_audit_timestamp ON audit_entries(timestamp DESC);
CREATE INDEX idx_audit_event_type ON audit_entries(event_type);
CREATE INDEX idx_audit_actor ON audit_entries(actor_user_uuid);
CREATE INDEX idx_audit_org ON audit_entries(org_uuid);
CREATE INDEX idx_audit_siem ON audit_entries(siem_delivered, timestamp) WHERE siem_delivered = FALSE;

-- Append-only policy: không ai có thể DELETE
ALTER TABLE audit_entries ENABLE ROW LEVEL SECURITY;
CREATE POLICY audit_no_delete ON audit_entries FOR DELETE USING (false);
CREATE POLICY audit_no_update ON audit_entries FOR UPDATE USING (false);

-- Chỉ INSERT allowed (gán quyền tối thiểu cho app user nếu cần)
-- GRANT INSERT, SELECT ON audit_entries TO vaultwarden_app;

-- Archival table cho entries cũ hơn retention period
CREATE TABLE audit_entries_archive (
    LIKE audit_entries INCLUDING ALL
);
```

---

## 3. Thiết Kế Chi Tiết

### 3.1 AuditEmitter — Async Channel Architecture

**File**: `src/audit.rs`

```rust
use tokio::sync::mpsc;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub event_type: AuditEventType,
    pub severity: Severity,
    pub actor_user_uuid: Option<String>,
    pub actor_email: Option<String>,
    pub target_resource: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub org_uuid: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AuditEventType {
    // Authentication
    LoginSuccess,
    LoginFailurePassword,
    LoginFailure2FA,
    LoginFailureRateLimit,
    AccountLockout,
    TokenRefresh,
    Logout,
    
    // Admin Panel
    AdminLoginSuccess,
    AdminLoginFailure,
    AdminConfigChanged { field: String },
    AdminUserManagement { action: String },
    AdminBackupTriggered,
    
    // Session
    SessionCreated,
    SessionExpired,
    SessionRevoked,
    
    // File Operations
    AttachmentUploaded,
    AttachmentDownloaded,
    SendCreated,
    SendAccessed,
    SendDeleted,
    
    // Security Events
    RateLimitTriggered { endpoint: String },
    SuspiciousIpDetected,
    EmergencyAccessRequested,
    EmergencyAccessGranted,
    
    // System
    ServerStarted,
    ServerStopped,
    MigrationApplied { version: String },
    KeyRotation,
}

// Global sender — chia sẻ giữa handlers
pub static AUDIT_TX: LazyLock<Option<mpsc::Sender<AuditEntry>>> = 
    LazyLock::new(|| None);  // Được khởi tạo trong main.rs

pub fn emit(entry: AuditEntry) {
    if let Some(tx) = AUDIT_TX.as_ref() {
        // Fire-and-forget: không block HTTP handler
        let _ = tx.try_send(entry);
    }
}

// Audit writer task — chạy trong Tokio background task
pub async fn audit_writer_task(
    mut rx: mpsc::Receiver<AuditEntry>,
    db_pool: DbPool,
) {
    let mut batch: Vec<AuditEntry> = Vec::with_capacity(100);
    let mut interval = tokio::time::interval(Duration::from_millis(1000));
    
    loop {
        tokio::select! {
            Some(entry) = rx.recv() => {
                batch.push(entry);
                if batch.len() >= 100 {
                    flush_batch(&mut batch, &db_pool).await;
                }
            }
            _ = interval.tick() => {
                if !batch.is_empty() {
                    flush_batch(&mut batch, &db_pool).await;
                }
            }
        }
    }
}

async fn flush_batch(batch: &mut Vec<AuditEntry>, pool: &DbPool) {
    let conn = pool.get().expect("DB connection");
    for entry in batch.drain(..) {
        if let Err(e) = write_audit_entry(entry, &conn).await {
            // Log error but don't crash — audit failure is non-fatal for app
            error!("Failed to write audit entry: {e}");
        }
    }
}
```

### 3.2 Hash Chain Implementation

```rust
async fn write_audit_entry(entry: AuditEntry, conn: &DbConn) -> Result<(), Error> {
    // Lấy hash của entry cuối cùng (serialized as atomic DB transaction)
    let prev_hash = AuditEntryDb::get_last_hash(conn).await?
        .unwrap_or([0u8; 32]);  // Genesis: zeros
    
    // Tính hash cho entry mới
    let mut hasher = Sha256::new();
    hasher.update(&prev_hash);
    hasher.update(entry.timestamp.to_rfc3339().as_bytes());
    hasher.update(entry.event_type.to_string().as_bytes());
    hasher.update(entry.actor_user_uuid.as_deref().unwrap_or("").as_bytes());
    hasher.update(entry.ip_address.map(|ip| ip.to_string())
        .unwrap_or_default().as_bytes());
    hasher.update(serde_json::to_vec(&entry.metadata).unwrap_or_default());
    let entry_hash: [u8; 32] = hasher.finalize().into();
    
    // Insert vào DB — trong transaction để đảm bảo atomic
    AuditEntryDb {
        prev_hash: prev_hash.to_vec(),
        entry_hash: entry_hash.to_vec(),
        event_type: entry.event_type.to_string(),
        severity: entry.severity.to_string(),
        actor_user_uuid: entry.actor_user_uuid,
        target_resource: entry.target_resource,
        ip_address: entry.ip_address.map(|ip| ip.to_string()),
        org_uuid: entry.org_uuid,
        metadata: entry.metadata,
        ..Default::default()
    }.insert(conn).await
}
```

### 3.3 Hash Chain Verification

```rust
// GET /api/audit/verify-chain?from=&to=
pub async fn verify_chain(
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    _admin: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    let entries = AuditEntryDb::list_ordered(from, to, &conn).await?;
    
    let mut prev_hash = [0u8; 32];
    let mut broken_at: Option<i64> = None;
    
    for entry in &entries {
        // Tính lại hash và so sánh
        let expected = compute_entry_hash(&entry, &prev_hash);
        if expected != entry.entry_hash.as_slice() {
            broken_at = Some(entry.id);
            break;
        }
        prev_hash.copy_from_slice(&entry.entry_hash);
    }
    
    Ok(Json(json!({
        "valid": broken_at.is_none(),
        "entries_checked": entries.len(),
        "broken_at_id": broken_at,
        "period_from": from,
        "period_to": to,
    })))
}
```

### 3.4 SIEM Integration

**File**: `src/siem.rs`

```rust
#[derive(Debug, Clone)]
pub enum SiemFormat {
    SplunkHec,
    SyslogRfc5424,
    JsonLines,
    MicrosoftSentinel,
}

pub struct SiemForwarder {
    client: reqwest::Client,   // Reuse existing HTTP client pattern
    config: SiemConfig,
}

impl SiemForwarder {
    // Chạy trong background task — check mỗi 5 giây cho undelivered entries
    pub async fn run_delivery_loop(&self, pool: DbPool) {
        let mut interval = tokio::time::interval(
            Duration::from_millis(CONFIG.audit_siem_flush_interval_ms())
        );
        loop {
            interval.tick().await;
            self.deliver_pending(&pool).await;
        }
    }
    
    async fn deliver_pending(&self, pool: &DbPool) {
        let conn = pool.get().expect("DB connection");
        let batch = AuditEntryDb::get_undelivered(
            CONFIG.audit_siem_batch_size(), &conn
        ).await.unwrap_or_default();
        
        if batch.is_empty() { return; }
        
        let payload = self.format_batch(&batch);
        
        for attempt in 0..CONFIG.audit_siem_retry_count() {
            match self.send_to_siem(&payload).await {
                Ok(_) => {
                    AuditEntryDb::mark_delivered(&batch, &conn).await.ok();
                    break;
                }
                Err(e) if attempt < CONFIG.audit_siem_retry_count() - 1 => {
                    warn!("SIEM delivery attempt {attempt} failed: {e}. Retrying...");
                    tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                }
                Err(e) => {
                    error!("SIEM delivery failed after {} attempts: {e}", 
                           CONFIG.audit_siem_retry_count());
                }
            }
        }
    }
    
    fn format_batch(&self, entries: &[AuditEntryDb]) -> String {
        match self.config.format {
            SiemFormat::SplunkHec => {
                entries.iter().map(|e| {
                    json!({
                        "time": e.timestamp.timestamp(),
                        "source": "vaultwarden",
                        "sourcetype": "vaultwarden:audit",
                        "event": {
                            "event_type": e.event_type,
                            "actor": e.actor_user_uuid,
                            "ip": e.ip_address,
                            "metadata": e.metadata,
                        }
                    }).to_string()
                }).collect::<Vec<_>>().join("\n")
            }
            SiemFormat::SyslogRfc5424 => {
                entries.iter().map(|e| {
                    format!(
                        "<134>1 {} vaultwarden - AUDIT - [event_type=\"{}\" actor=\"{}\"] {}",
                        e.timestamp.to_rfc3339(),
                        e.event_type,
                        e.actor_user_uuid.as_deref().unwrap_or("-"),
                        serde_json::to_string(&e.metadata).unwrap_or_default()
                    )
                }).collect::<Vec<_>>().join("\n")
            }
            SiemFormat::JsonLines => {
                entries.iter()
                    .map(|e| serde_json::to_string(e).unwrap_or_default())
                    .collect::<Vec<_>>().join("\n")
            }
            SiemFormat::MicrosoftSentinel => {
                // Azure Monitor Data Collection API format
                let events: Vec<_> = entries.iter().map(|e| json!({
                    "TimeGenerated": e.timestamp.to_rfc3339(),
                    "EventType": e.event_type,
                    "ActorUserUuid": e.actor_user_uuid,
                    "IpAddress": e.ip_address,
                    "Properties": e.metadata,
                })).collect();
                serde_json::to_string(&events).unwrap_or_default()
            }
        }
    }
}
```

### 3.5 Integration vào HTTP Handlers

Pattern nhất quán: mỗi handler emit audit event. Ví dụ trong `src/api/identity.rs`:

```rust
// Trong login handler — trường hợp thất bại
Err(e) => {
    audit::emit(AuditEntry {
        event_type: AuditEventType::LoginFailurePassword,
        severity: Severity::Warn,
        actor_email: Some(login_request.username.clone()),
        ip_address: Some(remote_addr),
        user_agent: Some(user_agent.to_string()),
        metadata: json!({
            "reason": "invalid_password",
            "username_exists": user_exists,
        }),
        ..Default::default()
    });
    Err(e)
}

// Trường hợp thành công
Ok(token) => {
    audit::emit(AuditEntry {
        event_type: AuditEventType::LoginSuccess,
        severity: Severity::Info,
        actor_user_uuid: Some(user.uuid.clone()),
        actor_email: Some(user.email.clone()),
        ip_address: Some(remote_addr),
        user_agent: Some(user_agent.to_string()),
        metadata: json!({
            "device_type": device.atype,
            "2fa_method": used_2fa_method,
        }),
        ..Default::default()
    });
    Ok(token)
}
```

### 3.6 Log Retention Policy

Background job (thêm vào job scheduler hiện có trong `src/main.rs`):

```rust
// Chạy daily — archive entries cũ hơn retention period
async fn audit_retention_job(conn: &DbConn) {
    let retention_days = CONFIG.audit_retention_days() as i64;
    let cutoff = Utc::now() - Duration::days(retention_days);
    
    // Move to archive table thay vì DELETE
    let archived = AuditEntryDb::archive_older_than(cutoff, conn).await
        .unwrap_or(0);
    
    if archived > 0 {
        info!("Archived {} audit entries older than {} days", archived, retention_days);
    }
}
```

---

## 4. Audit API Routes

**File**: `src/api/core/audit.rs`

```
GET  /api/audit/events?from=&to=&type=&user=&org=&page=&limit=
GET  /api/audit/events/{id}
GET  /api/audit/verify-chain?from=&to=
GET  /api/audit/export?format=csv|json&from=&to=
```

**Auth**: Tất cả routes yêu cầu Admin token + Protected Action re-verification cho export.

---

## 5. Config Variables Mới

```bash
# Core
AUDIT_LOG_ENABLED=true
AUDIT_RETENTION_DAYS=2555           # 7 năm (banking default)
AUDIT_RETENTION_MINIMUM_DAYS=365    # Admin không thể giảm xuống dưới này
AUDIT_DB_URL=""                     # Nếu empty: dùng cùng DB với app

# SIEM
AUDIT_SIEM_ENABLED=false
AUDIT_SIEM_ENDPOINT=""
AUDIT_SIEM_TOKEN=""                 # Masked trong display
AUDIT_SIEM_FORMAT=json_lines        # splunk_hec|syslog_rfc5424|json_lines|microsoft_sentinel
AUDIT_SIEM_RETRY_COUNT=3
AUDIT_SIEM_BATCH_SIZE=100
AUDIT_SIEM_FLUSH_INTERVAL_MS=5000
AUDIT_SIEM_TLS_VERIFY=true

# Channel buffer
AUDIT_CHANNEL_BUFFER_SIZE=10000     # Buffer cho async channel
```

---

## 6. Phụ Thuộc Mới

| Crate | Phiên bản | Lý do |
|-------|-----------|-------|
| `sha2` | 0.10 | SHA-256 hash chain |

> `reqwest` đã có sẵn cho SIEM HTTP delivery.  
> `tokio` đã có sẵn cho async channel.

---

## 7. Kế Hoạch Triển Khai

### Sprint 1–2: Core Audit Infrastructure
- DB migration `audit_entries`
- `src/audit.rs` — emitter, hash chain, writer task
- Integration vào `main.rs`

### Sprint 3: Extended Event Types
- Integration vào identity.rs, admin.rs, accounts.rs, ciphers.rs
- Tất cả events trong CR-002 §2.2

### Sprint 4: SIEM Integration
- `src/siem.rs` — Splunk HEC + Syslog
- Background delivery task

### Sprint 5: Additional SIEM + API
- Microsoft Sentinel format
- Audit API routes
- Hash chain verification endpoint

### Sprint 6: Retention Policy + Testing
- Archival job
- Load test với 10,000 entries
- Verify chain test

---

## 8. Acceptance Criteria Mapping

| Criterion | Giải pháp |
|-----------|----------|
| Failed login logged với IP, timestamp | `AuditEventType::LoginFailurePassword` trong identity.rs |
| Admin config changes logged | `AuditEventType::AdminConfigChanged` trong admin.rs |
| Hash chain verify sau 10,000 entries | `GET /api/audit/verify-chain` — tính lại SHA-256 sequence |
| Xóa entry breaks chain | Entry hash include `prev_hash`, bất kỳ modification nào phá chain |
| SIEM delivery < 10 giây | SIEM flush interval mặc định 5 giây, batch size 100 |
| Log retention enforced | Background job archive (không delete) cũ hơn retention period |

---

*Status: Draft | Ngày: 2026-04-12*
