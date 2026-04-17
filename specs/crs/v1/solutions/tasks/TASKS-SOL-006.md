# TASKS-SOL-006: Disaster Recovery & Business Continuity

> **Giải pháp**: SOL-006  
> **CR**: CR-006  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 15

---

## Sprint 1–2 — Core Backup (pg_dump + S3) (4 tuần)

### [x] TASK-006-001
- **Tên**: DB migration — bảng `backup_runs`
- **File**: `migrations/postgresql/YYYYMMDD_backup/up.sql`
- **Mô tả**: Tạo bảng `backup_runs`: id (VARCHAR primary key dạng `bkp-YYYYMMDD-HHMMSS`), started_at, completed_at, status, backup_type, destination, size_bytes, sha256, manifest_json (JSONB), error_message, verified_at, verification_status, verification_error. Index trên started_at DESC.
- **Loại**: New migration
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-006-002
- **Tên**: Thêm BACKUP_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `backup_enabled`, `backup_type`, `backup_destination`, `backup_s3_region`, `backup_schedule`, `backup_retention_days`, `backup_encryption_key_id`, `backup_verify_enabled`, `backup_verify_schedule`, `backup_verify_alert_email`, `backup_verify_timeout_seconds`, `backup_cross_region_enabled`, `backup_secondary_destination`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-006-003
- **Tên**: Implement `BackupManager` struct
- **File**: `src/backup.rs` (mới)
- **Mô tả**: Struct `BackupManager { config, storage: Operator }`. Method `run_backup()`: tạo backup_run record, dispatch theo `backup_type`, tính SHA-256, upload via OpenDAL, update record. On failure: mark_failed, alert email, SIEM event.
- **Loại**: New file
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-006-001, TASK-006-002

### [x] TASK-006-004
- **Tên**: Implement `run_pg_dump()`
- **File**: `src/backup.rs`
- **Mô tả**: Gọi `tokio::process::Command::new("pg_dump")` với `--format=custom --compress=6`. Capture stdout. Optional encrypt nếu `BACKUP_ENCRYPTION_KEY_ID` được set. Return `Vec<u8>`.
- **Loại**: New code
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-006-003

### [x] TASK-006-005
- **Tên**: Implement SQLite và MySQL backup methods
- **File**: `src/backup.rs`
- **Mô tả**: `run_sqlite_copy()`: dùng SQLite online backup API. `run_mysqldump()`: gọi `mysqldump` CLI. `run_pg_basebackup()`: cho WAL-level backup.
- **Loại**: New code
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-006-003

### [x] TASK-006-006
- **Tên**: Implement `create_manifest()` và backup upload
- **File**: `src/backup.rs`
- **Mô tả**: `create_manifest()`: query record counts (users, ciphers, organizations), lấy schema version, ký manifest với RSA key. `upload_backup()` và `upload_manifest()` via OpenDAL Operator.
- **Loại**: New code
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-006-003

### [x] TASK-006-007
- **Tên**: Implement Admin Backup API
- **File**: `src/api/admin/backup.rs` (mới)
- **Mô tả**: Routes: `POST /api/admin/backup/trigger` (run backup), `GET /api/admin/backup/status` (latest + verified backup info, next schedule, retention), `POST /api/admin/backup/verify` (trigger verification). Mount trong `src/api/admin.rs`.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-006-003

### [x] TASK-006-008
- **Tên**: Đăng ký backup job vào scheduler
- **File**: `src/main.rs`
- **Mô tả**: Nếu `BACKUP_ENABLED=true`: thêm cron job với `BACKUP_SCHEDULE`. Spawn `BackupManager::run_backup()` trong Tokio task. Tương tự cho verification job.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-006-003

---

## Sprint 3–4 — WAL Archiving (4 tuần)

### [x] TASK-006-009
- **Tên**: Thêm WAL archive config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `backup_wal_archive_enabled`, `backup_wal_archive_destination`, `backup_pitr_enabled`, `backup_pitr_retention_hours`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-006-010
- **Tên**: Documentation: PostgreSQL WAL archive setup
- **File**: `docs/disaster-recovery.md` (mới hoặc trong specs)
- **Mô tả**: Hướng dẫn cấu hình `archive_command` trong postgresql.conf để upload WAL segments lên S3. PITR restore procedure. Tích hợp với `BACKUP_WAL_ARCHIVE_DESTINATION`.
- **Loại**: Documentation
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-006-009

---

## Sprint 5–7 — Verification Pipeline (6 tuần)

### [x] TASK-006-011
- **Tên**: Implement `verify_backup()` — checksum + restore test
- **File**: `src/backup.rs`
- **Mô tả**: 1) Download backup từ S3 via OpenDAL. 2) Verify SHA-256. 3) Restore vào ephemeral PostgreSQL schema/SQLite in-memory. 4) Query record counts và compare với manifest. 5) Cleanup ephemeral DB. 6) Update `BackupRun.verification_status`. Gửi alert email nếu failed.
- **Loại**: New function
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-006-006

### [x] TASK-006-012
- **Tên**: Implement backup cross-region replication
- **File**: `src/backup.rs`
- **Mô tả**: `replicate_to_secondary()`: copy backup từ primary destination sang `BACKUP_SECONDARY_DESTINATION` (khác region). Dùng OpenDAL với secondary region config.
- **Loại**: New function
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-006-006

---

## Sprint 8 — Manifest + Signing (2 tuần)

### [x] TASK-006-013
- **Tên**: Implement manifest digital signature
- **File**: `src/backup.rs`
- **Mô tả**: `sign_manifest()`: dùng `openssl`/`ring` (đã có) để ký SHA-256 hash của manifest với server RSA private key. Verify signature khi verify backup. Generate/load RSA keypair từ `data/backup_signing_key.pem`.
- **Loại**: New function
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-006-006

---

## Sprint 9 — DR Runbook (2 tuần)

### [x] TASK-006-014
- **Tên**: Implement DR Runbook generator API
- **File**: `src/api/admin/backup.rs`
- **Mô tả**: `GET /api/admin/dr-runbook?format=json|html`: generate runbook từ current config và latest backup info. JSON/HTML format. Include: deployment type, backup location, encryption status, latest backup details, restore steps.
- **Loại**: New route
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-006-006

### [x] TASK-006-015
- **Tên**: Integration tests cho backup pipeline
- **File**: `tests/backup_tests.rs` (mới)
- **Mô tả**: Test: pg_dump thành công → SHA-256 correct → manifest valid. Verification pipeline với test DB. Cross-region replication mock. Alert email khi backup fail. Backup API endpoints.
- **Loại**: New test file
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-006-003, TASK-006-011, TASK-006-014

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1–2 | TASK-006-001 → 008 | 1–4 | Core backup, S3 upload, admin API |
| Sprint 3–4 | TASK-006-009 → 010 | 5–8 | WAL archiving docs |
| Sprint 5–7 | TASK-006-011 → 012 | 9–14 | Verification pipeline |
| Sprint 8 | TASK-006-013 | 15–16 | Manifest signing |
| Sprint 9 | TASK-006-014 → 015 | 17–18 | DR Runbook, testing |

---

*Tạo từ SOL-006 | Ngày: 2026-04-13*
