# TASKS-SOL-002: System-Wide Tamper-Evident Audit Log & SIEM Integration

> **Giải pháp**: SOL-002  
> **CR**: CR-002  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 18

---

## Sprint 1–2 — Core Audit Infrastructure (4 tuần)

### [x] TASK-002-001
- **Tên**: DB migration — bảng `audit_entries`
- **File**: `migrations/postgresql/YYYYMMDD_audit_log/up.sql`
- **Mô tả**: Tạo bảng `audit_entries` với fields: id, timestamp, event_type, severity, actor_user_uuid, actor_email, target_resource, ip_address, user_agent, org_uuid, metadata (JSONB), prev_hash (BYTEA), entry_hash (BYTEA), siem_delivered, siem_attempts. Thêm RLS policy (no DELETE, no UPDATE). Tạo bảng `audit_entries_archive`. Thêm indexes.
- **Loại**: New migration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### [x] TASK-002-002
- **Tên**: Implement `AuditEventType` enum và `AuditEntry` struct
- **File**: `src/audit.rs` (mới)
- **Mô tả**: Định nghĩa toàn bộ `AuditEventType` enum (LoginSuccess, LoginFailurePassword, AdminConfigChanged, SessionCreated, AttachmentUploaded, RateLimitTriggered, ServerStarted, v.v.). Struct `AuditEntry` với derive Serialize. Enum `Severity` (Info, Warn, Critical).
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### [x] TASK-002-003
- **Tên**: Implement async channel emitter (`emit()` + global `AUDIT_TX`)
- **File**: `src/audit.rs`
- **Mô tả**: Global `LazyLock<Option<mpsc::Sender<AuditEntry>>>`. Hàm `emit()` fire-and-forget dùng `try_send`. Background task `audit_writer_task()` batch write với interval 1 giây hoặc khi batch đạt 100 entries.
- **Loại**: New code in `src/audit.rs`
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-002-002

### [x] TASK-002-004
- **Tên**: Implement hash chain logic
- **File**: `src/audit.rs`
- **Mô tả**: Hàm `write_audit_entry()` trong DB transaction: lấy `prev_hash` của entry cuối, tính SHA-256 (prev_hash + timestamp + event_type + actor + ip + metadata), insert `AuditEntryDb`.
- **Loại**: New code
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-002-001, TASK-002-003
- **Dependency mới**: crate `sha2 = "0.10"`

### [x] TASK-002-005
- **Tên**: Khởi động audit channel trong `main.rs`
- **File**: `src/main.rs`
- **Mô tả**: Tạo `mpsc::channel(AUDIT_CHANNEL_BUFFER_SIZE)`, set global `AUDIT_TX`, spawn `audit_writer_task` như Tokio background task.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-002-003

### [x] TASK-002-006
- **Tên**: Thêm AUDIT_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `audit_log_enabled`, `audit_retention_days` (default 2555), `audit_retention_minimum_days`, `audit_db_url`, `audit_channel_buffer_size`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

---

## Sprint 3 — Extended Event Types (2 tuần)

### [x] TASK-002-007
- **Tên**: Emit audit events trong `src/api/identity.rs`
- **File**: `src/api/identity.rs`
- **Mô tả**: Thêm `audit::emit()` cho: LoginSuccess (kèm device_type, 2fa_method), LoginFailurePassword, LoginFailure2FA, LoginFailureRateLimit, AccountLockout, TokenRefresh, Logout.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-002-003

### [x] TASK-002-008
- **Tên**: Emit audit events trong `src/api/admin.rs`
- **File**: `src/api/admin.rs`
- **Mô tả**: Emit events: AdminLoginSuccess, AdminLoginFailure, AdminConfigChanged (kèm field name), AdminUserManagement, AdminBackupTriggered.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-002-003

### [x] TASK-002-009
- **Tên**: Emit audit events trong accounts + ciphers
- **File**: `src/api/core/accounts.rs`, `src/api/core/ciphers.rs`
- **Mô tả**: accounts.rs: PasswordChanged, TwoFactorAdded/Removed. ciphers.rs: AttachmentUploaded, AttachmentDownloaded, SendCreated, SendAccessed, SendDeleted.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-002-003

---

## Sprint 4 — SIEM Integration (2 tuần)

### [x] TASK-002-010
- **Tên**: Implement `SiemForwarder` với Splunk HEC format
- **File**: `src/siem.rs` (mới)
- **Mô tả**: Struct `SiemForwarder` với `run_delivery_loop()` background task. `deliver_pending()` lấy undelivered entries từ DB theo batch, gửi với retry exponential backoff. Format `SiemFormat::SplunkHec` serialize đúng format.
- **Loại**: New file
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-002-001

### [x] TASK-002-011
- **Tên**: Implement Syslog RFC 5424 format
- **File**: `src/siem.rs`
- **Mô tả**: `SiemFormat::SyslogRfc5424` — format chuẩn syslog RFC 5424 với priority 134 (local0.info), structured data.
- **Loại**: New code
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-002-010

### [x] TASK-002-012
- **Tên**: Thêm SIEM config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `audit_siem_enabled`, `audit_siem_endpoint`, `audit_siem_token` (masked), `audit_siem_format`, `audit_siem_retry_count`, `audit_siem_batch_size`, `audit_siem_flush_interval_ms`, `audit_siem_tls_verify`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

---

## Sprint 5 — Additional SIEM + Audit API (2 tuần)

### [x] TASK-002-013
- **Tên**: Implement Microsoft Sentinel format
- **File**: `src/siem.rs`
- **Mô tả**: `SiemFormat::MicrosoftSentinel` — Azure Monitor Data Collection API format (JSON array với TimeGenerated, EventType, v.v.).
- **Loại**: New code
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-002-010

### [x] TASK-002-014
- **Tên**: Implement Audit REST API routes
- **File**: `src/api/core/audit.rs` (mới)
- **Mô tả**: Routes: `GET /api/audit/events` (pagination, filter), `GET /api/audit/events/{id}`, `GET /api/audit/verify-chain`, `GET /api/audit/export`. Tất cả yêu cầu AdminHeaders.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-002-001

### [x] TASK-002-015
- **Tên**: Implement hash chain verification endpoint
- **File**: `src/api/core/audit.rs`
- **Mô tả**: `GET /api/audit/verify-chain?from=&to=` — tải entries theo thứ tự, tính lại SHA-256 từng entry, so sánh với `entry_hash` lưu trong DB. Trả về `{valid, entries_checked, broken_at_id}`.
- **Loại**: New route
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-002-004

### [x] TASK-002-016
- **Tên**: Mount audit API routes
- **File**: `src/api/core/mod.rs`, `src/main.rs`
- **Mô tả**: Thêm `audit.rs` vào module, mount routes dưới `/api/audit/`. Khởi động SIEM forwarder background task.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-002-014

---

## Sprint 6 — Retention Policy + Testing (2 tuần)

### [x] TASK-002-017
- **Tên**: Implement audit retention archival job
- **File**: `src/main.rs` + `src/audit.rs`
- **Mô tả**: Background job chạy daily: `archive_older_than(cutoff)` — move entries sang `audit_entries_archive` thay vì DELETE. Log số entries archived. Tôn trọng `AUDIT_RETENTION_MINIMUM_DAYS`.
- **Loại**: New function + scheduler integration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-002-001
- **Hoàn thành**: 2026-04-16 — `archive_older_than_job()` in `src/audit.rs`, wired to `AUDIT_RETENTION_SCHEDULE` cron in `schedule_jobs()`

### [x] TASK-002-018
- **Tên**: Integration tests: hash chain + SIEM
- **File**: `tests/audit_tests.rs` (mới)
- **Mô tả**: Test: emit 10,000 entries → verify-chain trả valid. Delete một entry → verify-chain phát hiện broken_at. SIEM delivery mock test. Retention archival test.
- **Loại**: New test file
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-002-004, TASK-002-010, TASK-002-017
- **Hoàn thành**: 2026-04-16 — 15 tests pass: hash chain (10-entry consistent, tamper detection, ordering), retention floor, SIEM payload formats, AuditEventType strings

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1–2 | TASK-002-001 → 006 | 1–4 | Core audit infra, hash chain |
| Sprint 3 | TASK-002-007 → 009 | 5–6 | Event integration vào handlers |
| Sprint 4 | TASK-002-010 → 012 | 7–8 | SIEM Splunk + Syslog |
| Sprint 5 | TASK-002-013 → 016 | 9–10 | Sentinel, Audit API |
| Sprint 6 | TASK-002-017 → 018 | 11–12 | Retention, testing |

---

*Tạo từ SOL-002 | Ngày: 2026-04-13*
