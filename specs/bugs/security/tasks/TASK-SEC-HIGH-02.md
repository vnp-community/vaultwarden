# TASK-SEC-HIGH-02: Không Có JWT Revocation

> **Severity**: P2 — High  
> **Sprint**: Sprint 2  
> **Effort**: 3 ngày  
> **File gốc**: `src/auth.rs:30-32`  
> **Rủi ro**: Stolen refresh token valid 90 ngày, không thể revoke

---

## Mô Tả Vấn Đề

Refresh token có TTL 90 ngày cho mobile. Nếu bị đánh cắp, attacker có quyền truy cập tới 90 ngày mà không thể block (trừ khi user đổi password, trigger security_stamp rotation).

---

## Sub-tasks

### Phase 1 — Nhanh: Giảm TTL + Configurable

#### TASK-SEC-HIGH-02-A ✅ DONE
- **Tên**: Thêm config keys cho token TTL
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `refresh_token_validity_days: u32, false, def, 30`, `mobile_refresh_token_validity_days: u32, false, def, 30` (giảm từ 90 → 30), `access_token_validity_hours: u32, false, def, 2`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Triển khai**: `src/config.rs` — ba config keys đã thêm

#### TASK-SEC-HIGH-02-B ✅ DONE
- **Tên**: Refactor `get_refresh_validity()` dùng config
- **File**: `src/auth.rs`
- **Mô tả**: Thay hardcoded `TimeDelta::try_days(90)` bằng `CONFIG.mobile_refresh_validity_days()`. Function `get_refresh_validity(device_type: DeviceType) -> TimeDelta` đọc từ config theo device type.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-HIGH-02-A
- **Triển khai**: `src/auth.rs` — `DEFAULT_REFRESH_VALIDITY`, `MOBILE_REFRESH_VALIDITY`, `DEFAULT_ACCESS_VALIDITY` đều dùng `LazyLock` đọc từ CONFIG

### Phase 2 — Trung hạn: Logout All Devices

#### TASK-SEC-HIGH-02-C ✅ DONE
- **Tên**: Implement `POST /api/accounts/logout-all` endpoint
- **File**: `src/api/core/accounts.rs`
- **Mô tả**: `logout_all_devices()`: update `security_stamp` của user (invalidate tất cả JWTs), delete tất cả `Device` records (push tokens). Yêu cầu current password re-confirmation.
- **Loại**: New route
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Triển khai**: `src/api/core/accounts.rs` — `POST /api/accounts/logout-all`; validates password/OTP, deletes all devices, resets security_stamp, sends WebSocket logout

### Phase 3 — Dài hạn: DB-backed Token Revocation (opt-in)

#### TASK-SEC-HIGH-02-D ✅ DONE (2026-04-15)
- **Tên**: DB migration — bảng `revoked_tokens`
- **File**: `migrations/{sqlite,postgresql,mysql}/2026-04-15-000001_token_revocation/up.sql`
- **Mô tả**: Đã tạo migration cho 3 DB backend: `revoked_tokens(jti PK, user_uuid FK->users, revoked_at, expires_at)` + index `idx_revoked_tokens_expires_at` trên `expires_at` cho cleanup job. FK `ON DELETE CASCADE` để auto-clean khi user bị xóa. Down migration (ó `down.sql`) drop table. Hiện ghép cặp với `TOKEN_REVOCATION_ENABLED=true` (HIGH-02-E); chưa được apply tự động — cần chạy `diesel migration run` sau khi bật feature.
- **Loại**: New migration
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-HIGH-02-E

#### TASK-SEC-HIGH-02-E ✅ DONE (2026-04-15)
- **Tên**: Thêm `TOKEN_REVOCATION_ENABLED` config key
- **File**: `src/config.rs`
- **Mô tả**: Đã thêm `token_revocation_enabled: bool, false, def, false` vào `make_config!`. Comment rõ ràng: opt-in vì có DB round-trip mỗi request; cần migration `revoked_tokens` (HIGH-02-D); `jti` claim sẽ được inject vào JWT khi bật (HIGH-02-F, Sprint 4+). Config key đã exposed qua `CONFIG.token_revocation_enabled()`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

#### TASK-SEC-HIGH-02-F ✅ DONE (2026-04-15)
- **Tên**: Implement JWT JTI validation
- **File**: `src/auth.rs`, `src/db/models/revoked_token.rs` (NEW)
- **Mô tả**: Thêm optional `jti: Option<String>` field vào `LoginJwtClaims` (serialized chỉ khi `TOKEN_REVOCATION_ENABLED=true` — `#[serde(skip_serializing_if)]`). Trong `LoginJwtClaims::new()`: inject `jti = Some(get_uuid())` nếu revocation enabled. Trong `Headers::from_request` (sau security_stamp check): nếu `TOKEN_REVOCATION_ENABLED && claims.jti.is_some()` → `RevokedToken::exists(jti, conn)` → `err_handler!("Token has been revoked")`. Tạo `RevokedToken` model với `insert()`, `exists()`, `delete_expired()`, `revoke_all_for_user()`.
- **Loại**: New code + model
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-SEC-HIGH-02-D
- **Sprint**: Sprint 4 ✅

#### TASK-SEC-HIGH-02-G ✅ DONE (2026-04-15)
- **Tên**: Background job cleanup revoked tokens
- **File**: `src/main.rs`
- **Mô tả**: Đã thêm daily job (cron `0 0 3 * * *` — 03:00 UTC) vào `schedule_jobs()` scheduler. Job chỉ đăng ký khi `TOKEN_REVOCATION_ENABLED=true`. Calls `RevokedToken::delete_expired(&conn)` — xóa entries quá `expires_at`. Lỗi được log nhưng không panic.
- **Loại**: New function + scheduler
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-HIGH-02-F
- **Sprint**: Sprint 4 ✅

---

## Acceptance Criteria

- [x] Mobile refresh token TTL giảm từ 90 xuống 30 ngày (configurable)
- [x] `POST /api/accounts/logout-all` invalidate tất cả sessions
- [x] DB migration `revoked_tokens` tạo xong cho SQLite/PG/MySQL (Sprint 4+ activation) ✅ (HIGH-02-D 2026-04-15)
- [x] `TOKEN_REVOCATION_ENABLED` config key đã thêm ✅ (HIGH-02-E 2026-04-15)
- [x] JWT JTI field + `RevokedToken` model + DB revocation check ✅ (HIGH-02-F 2026-04-15)
- [x] Revoked tokens tự động cleanup sau expiry (03:00 UTC daily job) ✅ (HIGH-02-G 2026-04-15)

---

*Tạo từ SOL-security.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: HIGH-02-A/B/C/D/E/F/G ✅ DONE*
