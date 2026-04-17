# TASKS-SOL-001: Enterprise Compliance Framework

> **Giải pháp**: SOL-001  
> **CR**: CR-001  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 16

---

## Sprint 1 — Security Headers + Config (2 tuần)

### [x] TASK-001-001
- **Tên**: Implement `SecurityHeadersFairing`
- **File**: `src/util.rs`
- **Mô tả**: Thêm Rocket Fairing tự động gắn security headers (HSTS, CSP, X-Frame-Options, X-Content-Type-Options, X-XSS-Protection, Referrer-Policy, Permissions-Policy) vào mọi HTTP response.
- **Loại**: New code
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-001-002
- **Tên**: Attach `SecurityHeadersFairing` vào Rocket instance
- **File**: `src/main.rs`
- **Mô tả**: Gọi `.attach(SecurityHeadersFairing)` trong chuỗi builder của Rocket. Thêm config key `SECURITY_HEADERS_ENABLED`, `CSP_OVERRIDE`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-001-001

### [x] TASK-001-003
- **Tên**: Thêm config keys cho Security Headers
- **File**: `src/config.rs`
- **Mô tả**: Thêm vào `make_config!` macro: `security_headers_enabled`, `csp_override`, `security_txt_contact`, `security_txt_expires`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-001-004
- **Tên**: Thêm endpoint `/.well-known/security.txt`
- **File**: `src/api/core/compliance.rs` (mới)
- **Mô tả**: Route GET trả nội dung `security.txt` theo RFC 9116. Nội dung lấy từ config `SECURITY_TXT_CONTACT`, `SECURITY_TXT_EXPIRES`.
- **Loại**: New file, new route
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-001-003

---

## Sprint 2 — GDPR Erasure Pipeline (2 tuần)

### [x] TASK-001-005
- **Tên**: DB migration — GDPR erasure tables
- **File**: `migrations/postgresql/YYYYMMDD_compliance/up.sql`
- **Mô tả**: Tạo bảng `erasure_logs` (append-only với RLS policy), `data_processing_register`. Thêm cột `pii_erasure_scheduled_at`, `pii_erased_at` vào `users`.
- **Loại**: New migration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### [x] TASK-001-006
- **Tên**: Implement model `ErasureLog`
- **File**: `src/db/models/erasure_log.rs` (mới)
- **Mô tả**: Struct `ErasureLog` với Diesel schema mapping. Phương thức `create()`, `mark_completed()`, `get_last_hash()`. Hash chain: SHA-256 của entry liên kết với entry trước (`prev_hash`).
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-001-005

### [x] TASK-001-007
- **Tên**: Implement `delete_account_gdpr()` pipeline
- **File**: `src/api/core/accounts.rs`
- **Mô tả**: Hàm xử lý GDPR erasure: revoke sessions, schedule PII erasure D+30, ghi `ErasureLog`, mark user pending. Endpoint `POST /api/accounts/delete-gdpr`.
- **Loại**: New function + route
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-001-006

### [x] TASK-001-008
- **Tên**: Implement background job `execute_scheduled_erasures`
- **File**: `src/api/core/accounts.rs` + `src/main.rs`
- **Mô tả**: Job chạy daily: tìm users đến hạn erasure, xóa PII fields (email → hashed@erased.invalid, name → [ERASED]), ẩn danh hóa IP logs, cập nhật `erasure_log`.
- **Loại**: New function, scheduler integration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-001-007

### [x] TASK-001-009
- **Tên**: Implement GDPR data export endpoint
- **File**: `src/api/core/accounts.rs`
- **Mô tả**: `GET /api/accounts/export-data` — thu thập tất cả dữ liệu của user (profile, vault items, audit logs), trả về JSON (GDPR Art. 20 data portability).
- **Loại**: New route
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-001-006

---

## Sprint 3–4 — Compliance Evidence API (4 tuần)

### [x] TASK-001-010
- **Tên**: Implement `src/compliance/evidence.rs`
- **File**: `src/compliance/evidence.rs` (mới)
- **Mô tả**: Evidence collectors cho từng standard: `collect_pci_dss_evidence()`, `collect_soc2_evidence()`, `collect_iso27001_evidence()`, `collect_gdpr_evidence()`. Truy vấn DB để lấy stats (user count, 2FA rate, audit events, hash chain status).
- **Loại**: New file
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-001-006

### [x] TASK-001-011
- **Tên**: Implement Compliance Evidence API routes
- **File**: `src/api/core/compliance.rs`
- **Mô tả**: Routes: `GET /api/compliance/evidence?standard=`, `GET /api/compliance/evidence/export?format=`, `GET /api/compliance/data-register`. Tất cả yêu cầu Admin auth.
- **Loại**: New routes
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-001-010

### [x] TASK-001-012
- **Tên**: Implement CSV export cho compliance reports
- **File**: `src/compliance/report.rs` (mới)
- **Mô tả**: Sử dụng crate `csv` để generate CSV report từ evidence data. Function `generate_csv_report()`, `generate_json_report()`.
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-001-010
- **Dependency mới**: crate `csv = "1.x"`

### [x] TASK-001-013
- **Tên**: Mount compliance routes vào Rocket
- **File**: `src/api/core/mod.rs`, `src/main.rs`
- **Mô tả**: Thêm `compliance.rs` vào module tree, mount routes dưới `/api/compliance/`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-001-011

---

## Sprint 5 — Data Residency + Testing (2 tuần)

### [x] TASK-001-014
- **Tên**: Thêm config keys cho Data Residency
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `data_residency_region`, `data_residency_enforce`, `pii_encryption_key_id`, `gdpr_erasure_delay_days`, `compliance_report_enabled`, `pen_test_mode`, `pen_test_token`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-001-015
- **Tên**: Implement `validate_storage_region()` cho attachment upload
- **File**: `src/api/core/ciphers.rs`
- **Mô tả**: Hàm kiểm tra region của storage destination so với `DATA_RESIDENCY_REGION`. Block upload nếu `DATA_RESIDENCY_ENFORCE=true` và region không khớp.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-001-014

### [x] TASK-001-016
- **Tên**: Integration tests cho Compliance Framework
- **File**: `tests/compliance_tests.rs` (mới)
- **Mô tả**: Test cases: security headers trên mọi response, GDPR erasure flow end-to-end, compliance evidence API, data residency rejection, CSV export format.
- **Loại**: New test file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-001-001 → TASK-001-015
- **Hoàn thành**: 2026-04-16 — 8 tests pass (CSV format, security.txt RFC9116, GDPR erasure hash chain, data residency, auth guards)

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1 | TASK-001-001 → 004 | 1–2 | Security headers, security.txt |
| Sprint 2 | TASK-001-005 → 009 | 3–4 | GDPR erasure pipeline |
| Sprint 3–4 | TASK-001-010 → 013 | 5–8 | Compliance Evidence API |
| Sprint 5 | TASK-001-014 → 016 | 9–10 | Data residency, testing |

---

*Tạo từ SOL-001 | Ngày: 2026-04-13*
