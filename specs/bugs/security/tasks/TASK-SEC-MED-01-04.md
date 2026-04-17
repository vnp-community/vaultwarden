# TASK-SEC-MED-01 đến SEC-MED-04: Medium Priority Security Fixes

> **Severity**: P3 — Medium  
> **Sprint**: Sprint 2–3  
> **File gốc**: Nhiều files

---

## SEC-MED-01: Password Hint Lưu Plaintext [Sprint 3 — 1 ngày]

**File**: `src/db/models/user.rs:42`, `src/api/identity.rs`  
**Rủi ro**: Database leak → hint tiết lộ password pattern

### TASK-SEC-MED-01-A ✅ DONE (2026-04-15 — verified)
- **Tên**: Ẩn password hint trước khi user authenticate xong
- **File**: `src/api/core/accounts.rs` — `_prelogin()`
- **Mô tả**: Verified by code review: hàm `_prelogin()` (lines 1262–1276) chỉ trả `kdf_type`, `kdf_iterations`, `kdf_memory`, `kdf_parallelism`. Không bao giờ trả `PasswordHint` trong prelogin response. Password hint chỉ tồn tại trong `user.password_hint` field và chỉ được gửi qua `/accounts/password-hint` endpoint (yêu cầu email). Điều này đã đáp ứng yêu cầu banning prelogin hint exposure.
- **Loại**: Verify — no change needed
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

**Code tham khảo**:
```rust
// Trong login response (sau auth thành công):
"PasswordHint": if user.verified_at.is_some() {
    user.password_hint.clone()
} else {
    None  // Giữ lại nếu email chưa verify
},
```

### TASK-SEC-MED-01-B ⏳ PENDING (Optional — Mạnh hơn)
- **Tên**: Encrypt password hint với server key
- **File**: `src/db/models/user.rs`
- **Mô tả**: `encrypt_password_hint()` dùng AES-256-GCM với server master key trước khi lưu DB. `decrypt_password_hint()` khi trả về. Migration: encrypt all existing hints.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-SEC-MED-01-A
- **Sprint**: Sprint 4+
- **Ghi chú**: Phức tạp hơn Option A. Prioritize Option A trước.

---

## SEC-MED-02: SSO Auto-Provisioning Bypass SIGNUPS_ALLOWED [Sprint 3 — 2 ngày]

**File**: `src/sso.rs`  
**Rủi ro**: Unauthorized user provisioning — user không thuộc tổ chức có thể tạo account qua SSO

### TASK-SEC-MED-02-A ✅ DONE (2026-04-15)
- **Tên**: Thêm SSO group whitelist config keys
- **File**: `src/config.rs`
- **Mô tả**: Đã thêm hai config keys vào section SSO: `sso_allowed_groups: String, true, def, String::new()` (comma-separated IdP group names), `sso_require_email_domain: String, true, def, String::new()` (block non-matching email domains at provisioning time). Cả hai có doc string rõ ràng giải thích điều kiện enforce.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-SEC-MED-02-B ✅ DONE (2026-04-15)
- **Tên**: Implement SSO group whitelist check trong provisioning
- **File**: `src/api/identity.rs`, `src/sso.rs`
- **Mô tả**: Trong `_sso_login()` new-user provisioning path (None arm): (1) thêm `groups: Vec<String>` field vào `AuthenticatedUser` và `UserInformation` structs trong `sso.rs`. (2) Populate groups bằng cách serialize `user_info` sang JSON và extract `"groups"` array (EmptyAdditionalClaims workaround). (3) Trong `identity.rs`: nếu `SIGNUPS_ALLOWED=false` và `SSO_ALLOWED_GROUPS` không empty → check user thuộc ít nhất 1 group; nếu fail → emit `error!` log + trả `UserFailedLogIn`. Case-insensitive matching.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-SEC-MED-02-A

### TASK-SEC-MED-02-C ✅ DONE (2026-04-15)
- **Tên**: Implement email domain restriction
- **File**: `src/api/identity.rs`
- **Mô tả**: Trong `_sso_login()` new-user provisioning path: nếu `SSO_REQUIRE_EMAIL_DOMAIN` không empty → check email kết thúc bằng `@{domain}` (case-insensitive, tự động normalize `domain` khỏi @ prefix). Nếu fail: emit SECURITY AUDIT error log có email + domain cấu hình, trả `UserFailedLogIn`. Được implement trước group check vì cheaper.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-MED-02-A

---

## SEC-MED-03: Emergency Access — Email Delivery Failure [Sprint 3 — 3 ngày]

**File**: `src/api/core/emergency_access.rs`  
**Rủi ro**: Vault accessed without grantor knowledge nếu email không đến

### TASK-SEC-MED-03-A ✅ DONE (2026-04-15)
- **Tên**: Thêm WebSocket in-app notification cho emergency access request
- **File**: `src/api/core/emergency_access.rs` — `initiate_emergency_access()`
- **Mô tả**: Sau khi gửi email (email failure được handle gracefully với `warn!` thay vì propagate error), gửi `push_user_update(UpdateType::SyncVault, &grantor, &None, &conn)` tới grantor user. Non-blocking: nếu push không khả dụng (push relay down, WS disabled), log warn nhưng không fail request.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Sprint**: Sprint 4 ✅

### TASK-SEC-MED-03-B ✅ DONE (2026-04-15)
- **Tên**: Implement multi-tier reminder job cho pending emergency access
- **File**: `src/api/core/emergency_access.rs` — `emergency_notification_reminder_job()`
- **Mô tả**: Thay thế logic reminder chỉ T-1 bằng multi-tier schedule: gửi reminder khi `days_remaining <= 7`, `<= 3`, `<= 1`. De-duplication guard: skip nếu đã notify trong 20 giờ gần nhất (`last_notification_at >= now - 20h`). Tier label (“7”, “3”, “1”) được pass vào email template. Cập nhật `last_notification_at` trước khi gửi email (ttránh double-send).
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-SEC-MED-03-A
- **Sprint**: Sprint 4 ✅

### TASK-SEC-MED-03-C ✅ DONE (existing template reused, 2026-04-15)
- **Tên**: Email template cho emergency access reminder
- **File**: `src/static/templates/email/emergency_access_recovery_reminder.hbs` (existing)
- **Mô tả**: Existing template `emergency_access_recovery_reminder` đã có sẵn với các variables `grantee_name`, `atype`, `days_left`. MED-03-B thành công tái sử dụng template này với `tier_label` (“7”/“3”/“1”) thay vì hardcoded “1”. Không cần template mới.
- **Loại**: Reuse existing template
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-MED-03-B
- **Sprint**: Sprint 4 ✅

---

## SEC-MED-04: config.json Chứa Secrets [Sprint 2 — 2 ngày]

**File**: `src/config.rs:20-22`  
**Rủi ro**: Credentials exposed nếu data directory bị misconfigured hoặc lộ

### TASK-SEC-MED-04-A ✅ DONE (2026-04-15)
- **Tên**: Ngăn sensitive fields ghi vào config.json
- **File**: `src/config.rs`
- **Mô tả**: Thêm `ConfigBuilder::strip_pass_fields() -> Self` trong `make_config!` macro-generated impl. Method này clone builder và set tất cả các `Pass`-typed fields về `None`: `rsa_key_encryption_key`, `push_installation_id`, `push_installation_key`, `hibp_api_key`, `admin_token`, `sso_client_secret`, `yubico_secret_key`, `duo_skey`, `_duo_akey`, `smtp_password`. Trong `update_config()`: thay vì serialize `&builder` trực tiếp, serialize `&builder.strip_pass_fields()` — chỉ bản clone stripped được ghi ra disk. `_usr` vẫn giữ full values cho runtime merge với env config (in-memory). Cũng expand `audit_config_file_for_secrets()` để cover toàn bộ danh sách Pass fields (trước đây thiếu `hibp_api_key`, `yubico_secret_key`, `duo_skey`, `push_installation_id/key`).
- **Loại**: Modify existing (macro impl + update_config)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### TASK-SEC-MED-04-B ✅ DONE
- **Tên**: Startup audit cho secrets trong config.json
- **File**: `src/config.rs`
- **Mô tả**: `audit_config_file_for_secrets()`: đọc `config.json`, check nếu chứa sensitive keys (`smtp_password`, `sso_client_secret`, v.v.). Log `warn!` cho mỗi sensitive key tìm thấy. Gọi tại startup.
- **Loại**: New function
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-MED-04-A
- **Triển khai**: `src/config.rs` — `pub fn audit_config_file_for_secrets()`; called from `src/main.rs` at startup

### TASK-SEC-MED-04-C ✅ DONE
- **Tên**: File permission check cho config.json
- **File**: `src/config.rs`
- **Mô tả**: `#[cfg(unix)]` check: nếu `config.json` readable by group hoặc others (mode & 0o044 != 0): `warn!("SECURITY WARNING: config.json is world/group readable. Run: chmod 600 {}")`.
- **Loại**: New function
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Triển khai**: `src/config.rs` — `#[cfg(unix)] pub fn check_config_file_permissions()`; called from `src/main.rs` at startup

---

## Acceptance Criteria

### SEC-MED-01
- [x] Password hint không xuất hiện trong prelogin response ✅ (MED-01-A verified 2026-04-15 — `_prelogin()` chỉ trả KDF fields)

### SEC-MED-02
- [x] SSO không auto-provision users outside allowed groups ✅ (MED-02-B 2026-04-15 — group whitelist enforced in `_sso_login()`)
- [x] SSO block email domains không match `SSO_REQUIRE_EMAIL_DOMAIN` ✅ (MED-02-C 2026-04-15)

### SEC-MED-03
- [x] Emergency access WebSocket push notification to grantor on initiation ✅ (MED-03-A 2026-04-15)
- [x] Multi-tier reminder schedule (T-7, T-3, T-1) implemented in `emergency_notification_reminder_job` ✅ (MED-03-B 2026-04-15)
- [x] Reminder template reuses existing `emergency_access_recovery_reminder` with dynamic `days_left` ✅ (MED-03-C 2026-04-15)

### SEC-MED-04
- [x] Sensitive fields KHÔNG ghi vào config.json — `strip_pass_fields()` strips all Pass fields trước khi serialize ✅ (MED-04-A 2026-04-15)
- [x] Startup warns nếu config.json chứa sensitive keys (expanded list) ✅ (MED-04-B 2026-04-15)
- [x] Startup warns nếu config.json có insecure file permissions (Unix) ✅ (MED-04-C 2026-04-15)

---

*Tạo từ SOL-security.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: **ALL DONE** ✅ — MED-01-A ✅ (verified), MED-02-A/B/C ✅, MED-03-A/B/C ✅, MED-04-A/B/C ✅ | MED-01-B optional Sprint 5+*
