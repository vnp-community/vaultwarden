# TASKS-SOL-007: Privileged Access Management (PAM)

> **Giải pháp**: SOL-007  
> **CR**: CR-007  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 17

---

## Sprint 1 — Privileged Cipher Type (2 tuần)

### [x] TASK-007-001
- **Tên**: DB migration — PAM tables
- **File**: `migrations/postgresql/YYYYMMDD_pam/up.sql`
- **Mô tả**: Tạo: `privileged_configs` (per-cipher config: requires_approval, max_checkout_duration, rotation config), `checkouts` (checkout records với justification, ITSM ticket, access_count, rotation status), `rotation_history`. Thêm cột `is_privileged` và `privileged_config_uuid` vào `ciphers`.
- **Loại**: New migration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### [x] TASK-007-002
- **Tên**: Thêm `is_privileged` field vào Cipher model
- **File**: `src/db/models/cipher.rs`
- **Mô tả**: Thêm `is_privileged: bool`, `privileged_config_uuid: Option<String>` vào `Cipher` struct. Update Diesel schema. Thêm methods: `count_privileged()`, `flag_rotation_pending()`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-007-001

### [x] TASK-007-003
- **Tên**: Implement `PrivilegedConfig` model
- **File**: `src/db/models/privileged_config.rs` (mới)
- **Mô tả**: Struct `PrivilegedConfig` với tất cả fields từ migration. Methods: `find_by_cipher()`, `save()`. Struct `RotationTargetConfig` (serde JSON).
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-007-001

### [x] TASK-007-004
- **Tên**: Thêm PAM config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `pam_enabled`, `pam_rotation_enabled`, `pam_rotation_worker_concurrency`, `pam_rotation_timeout_seconds`, `pam_rotation_ssh_key_path`, `pam_checkout_expiry_check_interval_seconds`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

---

## Sprint 2–4 — Checkout System (6 tuần)

### [x] TASK-007-005
- **Tên**: Implement `Checkout` model
- **File**: `src/db/models/checkout.rs` (mới)
- **Mô tả**: Struct `Checkout`. Methods: `find_expired_active()`, `count_active_for_cipher()`, `find_active_for_resource(user, cipher)`, `mark_checked_in()`, `mark_expired()`, `count_active()`, `count_expired_unhandled()`.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-007-001

### [x] TASK-007-006
- **Tên**: Implement `CheckoutManager::request_checkout()`
- **File**: `src/pam/checkout.rs` (mới, module `src/pam/`)
- **Mô tả**: Validate: cipher is privileged, concurrent limit, ITSM ticket (nếu required), approval status (reuse CR-004 ApprovalRequest). Tạo `Checkout` record. Emit audit event `PrivilegedCheckout`. Return `CheckoutResult::Success | PendingApproval`.
- **Loại**: New file
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-007-003, TASK-007-005

### [x] TASK-007-007
- **Tên**: Implement `CheckoutManager::checkin()`
- **File**: `src/pam/checkout.rs`
- **Mô tả**: Validate ownership. `mark_checked_in()`. Nếu `auto_rotate_after_checkout`: spawn rotation task async. Emit audit event `PrivilegedCheckin` với duration và access_count.
- **Loại**: New code
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-007-006

### [x] TASK-007-008
- **Tên**: Integrate checkout flow vào cipher access
- **File**: `src/api/core/ciphers.rs`
- **Mô tả**: Trong cipher GET handler: nếu `cipher.is_privileged`, gọi `CheckoutManager::request_checkout()`. Nếu không có active checkout: trả error. Increment `access_count` trên active checkout.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-007-006

### [x] TASK-007-009
- **Tên**: Implement checkout API endpoints
- **File**: `src/api/core/pam.rs` (mới)
- **Mô tả**: Routes: `POST /api/ciphers/{id}/checkout`, `POST /api/ciphers/{id}/checkin`, `GET /api/ciphers/{id}/checkouts`, `DELETE /api/checkouts/{id}` (force check-in), `GET /api/checkouts?active=true`.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-007-006, TASK-007-007

### [x] TASK-007-010
- **Tên**: Implement `expire_checkouts_job` background job
- **File**: `src/pam/checkout.rs`, `src/main.rs`
- **Mô tả**: Background job chạy mỗi `PAM_CHECKOUT_EXPIRY_CHECK_INTERVAL_SECONDS` giây: tìm expired active checkouts, mark_expired, trigger rotation nếu configured, emit `CheckoutExpired` audit event (Severity::Warn).
- **Loại**: New function + scheduler integration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-007-005

---

## Sprint 5–8 — Auto-Rotation Engine (8 tuần)

### [x] TASK-007-011
- **Tên**: Implement `RotationHistory` model
- **File**: `src/db/models/checkout.rs` (hoặc riêng)
- **Mô tả**: Struct `RotationHistory`. Methods: `insert()`, `mark_success()`, `mark_failed()`, `count_failed_last_24h()`.
- **Loại**: New code
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-007-001

### [x] TASK-007-012
- **Tên**: Implement `RotationEngine::rotate_credential()`
- **File**: `src/pam/rotation.rs` (mới)
- **Mô tả**: Dispatch theo `rotation_target_type`. Tạo `RotationHistory` record (running). On success: `flag_rotation_pending()` trên cipher, mark history success, emit audit. On failure: mark_failed, alert email.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-007-003, TASK-007-011

### [x] TASK-007-013
- **Tên**: Implement SSH rotation
- **File**: `src/pam/rotation.rs`
- **Mô tả**: `rotate_ssh()`: generate 32-char secure password, connect via `ssh` CLI với key auth (`PAM_ROTATION_SSH_KEY_PATH`), run `chpasswd`. Timeout `PAM_ROTATION_TIMEOUT_SECONDS`. Return new password.
- **Loại**: New function
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-007-012

### [x] TASK-007-014
- **Tên**: Implement MySQL và PostgreSQL rotation
- **File**: `src/pam/rotation.rs`
- **Mô tả**: `rotate_mysql()`: gọi `mysql` CLI với `ALTER USER ... IDENTIFIED BY`. `rotate_postgres()`: gọi `psql` CLI với `ALTER USER ... PASSWORD`. Cả hai dùng admin credentials từ `rotation_target_config`.
- **Loại**: New functions
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-007-012

---

## Sprint 9–10 — ITSM + Dashboard (4 tuần)

### [x] TASK-007-015
- **Tên**: Implement `IstmClient` với ServiceNow validation
- **File**: `src/pam/itsm.rs` (mới)
- **Mô tả**: `validate_ticket()`: dispatch theo `ITSM_TYPE`. `validate_servicenow_ticket()`: query ServiceNow `/api/now/table/incident` với Basic auth, verify ticket exists và không ở state Resolved/Closed.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### [x] TASK-007-016
- **Tên**: Thêm ITSM config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `itsm_enabled`, `itsm_type`, `itsm_servicenow_instance`, `itsm_servicenow_user`, `itsm_servicenow_password` (masked), `itsm_ticket_required`, `itsm_ticket_validation`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-007-017
- **Tên**: Implement PAM Dashboard API
- **File**: `src/api/core/pam.rs`
- **Mô tả**: `GET /api/admin/pam/dashboard`: parallel queries cho active_checkouts, overdue_checkouts, rotations_pending, rotations_failed_24h, privileged_ciphers_count, approval_requests_pending. `POST /api/admin/pam/ciphers/{id}/rotate` (manual rotation trigger).
- **Loại**: New routes
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-007-005, TASK-007-011

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1 | TASK-007-001 → 004 | 1–2 | Privileged cipher type |
| Sprint 2–4 | TASK-007-005 → 010 | 3–8 | Checkout system |
| Sprint 5–8 | TASK-007-011 → 014 | 9–16 | Rotation engine |
| Sprint 9–10 | TASK-007-015 → 017 | 17–20 | ITSM + Dashboard |

---

*Tạo từ SOL-007 | Ngày: 2026-04-13*
