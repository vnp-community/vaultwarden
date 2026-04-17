# SOL-001: Giải Pháp Thực Hiện — Enterprise Compliance Framework

> **Giải pháp cho**: CR-001  
> **Ngày**: 2026-04-12  
> **Trạng thái**: Draft  
> **Kiến trúc thay đổi**: Tối thiểu — additive changes only

---

## 1. Tổng Quan Giải Pháp

CR-001 yêu cầu compliance framework cho PCI DSS, SOC 2, ISO 27001, GDPR/PDPA. Giải pháp tập trung vào **thêm mới** (không sửa đổi core), sử dụng tối đa infrastructure sẵn có (Handlebars templates, Rocket routes, OpenDAL storage).

**Chiến lược**:
- Compliance Evidence API: module mới `src/api/core/compliance.rs`
- GDPR Erasure: mở rộng user deletion flow hiện có
- Data Residency: config validation tại startup + OpenDAL region check
- Security Headers: Rocket fairing mới trong `src/util.rs`
- Report Generator: Handlebars templates sẵn có (`src/static/templates/`)

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/api/core/compliance.rs` | Compliance Evidence API routes |
| `src/db/models/erasure_log.rs` | GDPR erasure audit records |
| `src/compliance/mod.rs` | Business logic cho compliance checks |
| `src/compliance/evidence.rs` | Evidence collector (thu thập dữ liệu từ DB) |
| `src/compliance/report.rs` | Report generator (PDF/CSV) |
| `src/static/templates/compliance/` | Handlebars templates cho reports |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/api/core/mod.rs` | Thêm route mount cho compliance module |
| `src/api/core/accounts.rs` | Mở rộng `delete_account` để trigger GDPR erasure pipeline |
| `src/config.rs` | Thêm config keys mới (data residency, PII encryption) |
| `src/util.rs` | Thêm `SecurityHeadersFairing` |
| `src/main.rs` | Attach fairing mới, mount compliance routes |
| `src/db/models/user.rs` | Thêm `pii_erasure_scheduled_at`, `pii_erased_at` fields |

### 2.3 Database Migrations Mới

```sql
-- migrations/postgresql/YYYYMMDD_compliance/up.sql

-- GDPR erasure tracking
CREATE TABLE erasure_logs (
    id          BIGSERIAL PRIMARY KEY,
    user_uuid   VARCHAR(40) NOT NULL,
    requested_at TIMESTAMP NOT NULL DEFAULT NOW(),
    scheduled_at TIMESTAMP NOT NULL,           -- D+30
    completed_at TIMESTAMP,
    erased_fields TEXT[] NOT NULL,             -- ['email', 'name', 'ip_logs']
    erased_by    VARCHAR(40),                  -- NULL = self-request, UUID = admin
    legal_basis  VARCHAR(100) NOT NULL,        -- 'GDPR Art. 17', 'User Request'
    audit_hash   VARCHAR(64) NOT NULL,         -- SHA-256 của entry (tamper-evident)
    prev_hash    VARCHAR(64)                   -- Chain với erasure_logs trước
);

-- Chỉ INSERT, không DELETE (append-only via DB policy)
ALTER TABLE erasure_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY erasure_log_insert_only ON erasure_logs FOR DELETE USING (false);

-- Data Processing Register
CREATE TABLE data_processing_register (
    id           SERIAL PRIMARY KEY,
    data_category VARCHAR(100) NOT NULL,       -- 'email', 'vault_items', 'ip_addresses'
    purpose       TEXT NOT NULL,
    legal_basis   VARCHAR(100) NOT NULL,
    retention_days INTEGER NOT NULL,
    location      VARCHAR(50) NOT NULL,        -- 'database', 's3-ap-southeast-1'
    updated_at    TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Thêm vào bảng users
ALTER TABLE users ADD COLUMN pii_erasure_scheduled_at TIMESTAMP;
ALTER TABLE users ADD COLUMN pii_erased_at TIMESTAMP;
```

---

## 3. Thiết Kế Chi Tiết

### 3.1 Security Headers Fairing

Thêm vào `src/util.rs`:

```rust
pub struct SecurityHeadersFairing;

#[rocket::async_trait]
impl Fairing for SecurityHeadersFairing {
    fn info(&self) -> Info {
        Info { name: "Security Headers", kind: Kind::Response }
    }
    
    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        // HSTS: 1 năm, include subdomains
        res.set_raw_header("Strict-Transport-Security", 
            "max-age=31536000; includeSubDomains");
        
        // CSP: restrict sources
        res.set_raw_header("Content-Security-Policy",
            "default-src 'self'; script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
             connect-src 'self' wss:; frame-ancestors 'none'");
        
        // Other security headers
        res.set_raw_header("X-Frame-Options", "DENY");
        res.set_raw_header("X-Content-Type-Options", "nosniff");
        res.set_raw_header("X-XSS-Protection", "1; mode=block");
        res.set_raw_header("Referrer-Policy", "strict-origin-when-cross-origin");
        res.set_raw_header("Permissions-Policy", 
            "accelerometer=(), camera=(), geolocation=(), microphone=()");
    }
}
```

Mount trong `src/main.rs`:
```rust
.attach(SecurityHeadersFairing)
```

### 3.2 Compliance Evidence API

**Route**: `src/api/core/compliance.rs`

```rust
#[get("/compliance/evidence?<standard>&<from>&<to>")]
async fn get_evidence(
    standard: &str,         // "pci_dss", "soc2", "iso27001", "gdpr"
    from: Option<&str>,
    to: Option<&str>,
    _admin: AdminHeaders,   // Require admin auth
    conn: DbConn,
) -> JsonResult {
    let evidence = match standard {
        "pci_dss" => collect_pci_dss_evidence(&conn, from, to).await?,
        "soc2"    => collect_soc2_evidence(&conn, from, to).await?,
        "iso27001" => collect_iso27001_evidence(&conn, from, to).await?,
        "gdpr"    => collect_gdpr_evidence(&conn, from, to).await?,
        _         => err!("Unknown compliance standard"),
    };
    Ok(Json(evidence))
}

#[get("/compliance/evidence/export?<standard>&<format>")]
async fn export_evidence_report(
    standard: &str,
    format: &str,           // "pdf", "csv", "json"
    _admin: AdminHeaders,
    conn: DbConn,
) -> Result<Vec<u8>, Error> {
    // Generate report using Handlebars
    let evidence = collect_evidence(standard, &conn).await?;
    match format {
        "csv"  => generate_csv_report(evidence),
        "json" => generate_json_report(evidence),
        _      => err!("Unsupported format. Use csv or json"),
    }
}

#[get("/compliance/data-register")]
async fn get_data_register(
    _admin: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    // Return Data Processing Register (GDPR Art. 30)
    let register = DataProcessingRegister::get_all(&conn).await?;
    Ok(Json(register))
}

// .well-known/security.txt
#[get("/well-known/security.txt")]
fn security_txt() -> &'static str {
    "Contact: security@vaultwarden.example.com\n\
     Expires: 2027-01-01T00:00:00.000Z\n\
     Preferred-Languages: en, vi\n"
}
```

**Evidence Collector** (`src/compliance/evidence.rs`):

```rust
pub struct PciDssEvidence {
    // Req 7: Access Control
    pub total_users: i64,
    pub users_with_2fa: i64,
    pub mfa_enforcement_enabled: bool,
    
    // Req 8: Auth Management  
    pub password_policy_enabled: bool,
    pub min_password_length: u32,
    pub account_lockout_enabled: bool,
    
    // Req 10: Audit Logs
    pub audit_log_enabled: bool,
    pub audit_log_retention_days: u32,
    pub audit_events_last_30_days: i64,
    pub hash_chain_verified: bool,
    
    // Req 2: No Default Passwords
    pub admin_token_hashed: bool,     // Argon2id, not plaintext
    
    pub generated_at: DateTime<Utc>,
    pub period_from: DateTime<Utc>,
    pub period_to: DateTime<Utc>,
}

async fn collect_pci_dss_evidence(conn: &DbConn, from: Option<&str>, to: Option<&str>) 
    -> Result<PciDssEvidence, Error> 
{
    let total_users = User::count_all(conn).await?;
    let users_with_2fa = TwoFactor::count_enabled_users(conn).await?;
    let audit_events = AuditEntry::count_in_period(conn, from, to).await?;
    let hash_valid = verify_hash_chain_fast(conn, from, to).await?;
    
    Ok(PciDssEvidence {
        total_users,
        users_with_2fa,
        mfa_enforcement_enabled: CONFIG.require_2fa(),
        audit_log_enabled: CONFIG.audit_log_enabled(),
        audit_log_retention_days: CONFIG.audit_retention_days(),
        audit_events_last_30_days: audit_events,
        hash_chain_verified: hash_valid,
        admin_token_hashed: CONFIG.admin_token().starts_with("$argon2"),
        generated_at: Utc::now(),
        ..Default::default()
    })
}
```

### 3.3 GDPR Right to Erasure Pipeline

Mở rộng `src/api/core/accounts.rs` — function `delete_account`:

```rust
pub async fn delete_account_gdpr(
    user_uuid: &str,
    conn: &DbConn,
    requester: Option<&str>,  // None = self, Some = admin UUID
) -> Result<(), Error> {
    // Phase 1: Immediate — revoke all sessions
    Device::delete_all_by_user(user_uuid, conn).await?;
    
    // Phase 2: Schedule PII erasure within 30 days
    let scheduled_at = Utc::now() + Duration::days(30);
    User::schedule_pii_erasure(user_uuid, scheduled_at, conn).await?;
    
    // Phase 3: Log erasure request (tamper-evident, cannot be deleted)
    ErasureLog::create(ErasureLog {
        user_uuid: user_uuid.to_string(),
        requested_at: Utc::now(),
        scheduled_at,
        legal_basis: "GDPR Art. 17 — Right to Erasure".to_string(),
        erased_by: requester.map(|s| s.to_string()),
        ..Default::default()
    }, conn).await?;
    
    // Phase 4: Mark user as pending erasure (soft-disable)
    User::mark_pending_erasure(user_uuid, conn).await?;
    
    Ok(())
}

// Background job: chạy daily, thực hiện actual PII erasure khi đến hạn
pub async fn execute_scheduled_erasures(conn: &DbConn) -> Result<(), Error> {
    let pending = User::find_erasure_due(conn).await?;
    for user in pending {
        // Xóa PII fields: email → hashed_uuid@erased.invalid, name → [ERASED]
        User::erase_pii(&user.uuid, conn).await?;
        // Xóa audit log IP addresses cho user này
        AuditEntry::anonymize_ip_for_user(&user.uuid, conn).await?;
        // Cập nhật erasure log với completion timestamp
        ErasureLog::mark_completed(&user.uuid, conn).await?;
    }
    Ok(())
}
```

### 3.4 Data Residency Controls

Thêm vào `src/config.rs` (trong `make_config!` macro):
```
data_residency_region:          String, false, def, "";
data_residency_enforce:         bool,   false, def, false;
pii_encryption_key_id:          String, false, def, "";  // Masked in display
```

Validation trong `src/api/core/ciphers.rs` và attachment upload:
```rust
fn validate_storage_region(destination: &str) -> Result<(), Error> {
    if !CONFIG.data_residency_enforce() || CONFIG.data_residency_region().is_empty() {
        return Ok(());
    }
    let allowed_region = CONFIG.data_residency_region();
    if !destination.contains(&allowed_region) {
        err!(format!("Storage destination violates data residency policy. \
                      Required region: {allowed_region}"));
    }
    Ok(())
}
```

### 3.5 Penetration Test Mode

Thêm config:
```
pen_test_mode:  bool, false, def, false;
pen_test_token: String, false, def, "";  // Masked
```

Fairing để set read-only headers khi pen test mode bật:
```rust
// Khi pen_test_mode=true: block tất cả write operations (POST/PUT/PATCH/DELETE)
// Return 403 với header X-PenTest-Mode: active
```

---

## 4. Config Variables Mới

```bash
# Data Residency
DATA_RESIDENCY_REGION=VN            # Restrict storage to this region code
DATA_RESIDENCY_ENFORCE=false        # Reject uploads violating policy

# GDPR
GDPR_ERASURE_DELAY_DAYS=30          # Days to complete erasure after request
GDPR_EXPORT_ENABLED=true            # Allow user data export (Art. 20)

# Security Headers (override defaults)
SECURITY_HEADERS_ENABLED=true       # Enable application-layer security headers
CSP_OVERRIDE=""                     # Custom CSP if provided overrides default

# Compliance
COMPLIANCE_REPORT_ENABLED=true      # Enable /api/compliance/* endpoints
PEN_TEST_MODE=false                 # Read-only mode for security assessors
PEN_TEST_TOKEN=""                   # Token required to access in pen test mode

# Security contact
SECURITY_TXT_CONTACT=""             # security@example.com
SECURITY_TXT_EXPIRES=""             # RFC3339 datetime
```

---

## 5. API Endpoints Mới

| Method | Path | Auth | Mô tả |
|--------|------|------|-------|
| GET | `/api/compliance/evidence?standard=pci_dss` | Admin | PCI DSS evidence report |
| GET | `/api/compliance/evidence?standard=soc2` | Admin | SOC 2 evidence report |
| GET | `/api/compliance/evidence?standard=gdpr` | Admin | GDPR evidence report |
| GET | `/api/compliance/evidence/export?format=csv` | Admin | Export report |
| GET | `/api/compliance/data-register` | Admin | Data Processing Register |
| POST | `/api/accounts/delete-gdpr` | User | GDPR erasure request |
| GET | `/api/accounts/export-data` | User | GDPR data portability export |
| GET | `/.well-known/security.txt` | Public | Security disclosure contact |

---

## 6. Phụ Thuộc Mới

| Crate | Phiên bản | Lý do |
|-------|-----------|-------|
| `csv` | 1.x | CSV report generation |

> **Không cần** thêm crate PDF: sử dụng CSV/JSON thay vì PDF để tránh dependency nặng. Nếu PDF cần thiết, sử dụng external tool (wkhtmltopdf) qua command line.

---

## 7. Kế Hoạch Triển Khai

### Sprint 1 (2 tuần): Security Headers + Config
- Triển khai `SecurityHeadersFairing`
- Thêm `.well-known/security.txt` endpoint
- Thêm config variables

### Sprint 2 (2 tuần): GDPR Erasure Pipeline
- DB migration cho `erasure_logs`, `pii_erasure_scheduled_at`
- Implement `delete_account_gdpr()` và background job
- User data export endpoint

### Sprint 3–4 (4 tuần): Compliance Evidence API
- `src/compliance/evidence.rs` — collector cho PCI DSS và SOC 2
- `src/api/core/compliance.rs` — REST endpoints
- CSV export format

### Sprint 5 (2 tuần): Data Residency + Testing
- Data residency validation cho S3 uploads
- Integration tests
- Documentation

---

## 8. Acceptance Criteria Mapping

| Criterion | Giải pháp |
|-----------|----------|
| PCI DSS Req 10 evidence report | `GET /api/compliance/evidence?standard=pci_dss` → JSON report |
| GDPR erasure < 30 ngày + audit receipt | Background job + `erasure_logs` table |
| Data residency reject non-compliant regions | `validate_storage_region()` trong upload path |
| Security headers trên mọi response | `SecurityHeadersFairing` |
| PII fields trong data register | `data_processing_register` table, pre-populated |

---

*Status: Draft | Ngày: 2026-04-12*
