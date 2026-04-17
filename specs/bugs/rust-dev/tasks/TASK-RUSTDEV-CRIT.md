# TASK-RUSTDEV-CRIT: P1 Critical — Immediate Fixes

> **Severity**: P1 — Critical / High  
> **Sprint**: Ngay (tuần 1)  
> **Nguồn**: [SOL-rust-dev.md](../SOL-rust-dev.md)

---

## TD-06: Panic trong `encode_jwt` [0.5 ngày]

**File**: `src/auth.rs:94-97`  
**Rủi ro**: `panic!` trong production code → server crash khi JWT encoding thất bại (key corruption, memory pressure)

### TASK-RUSTDEV-CRIT-01-A ✅ DONE
- **Tên**: Đổi `encode_jwt` từ panic sang `Result`
- **File**: `src/auth.rs`
- **Mô tả**: Signature đã thay đổi: `encode_jwt<T: Serialize>(claims: &T) -> ApiResult<String>`. `panic!` đã được thay bằng `Err(crate::error::Error::new(...))` với `error!` log. `LoginJwtClaims::token()` và `AuthTokens::access_token()/refresh_token()` đã propagate `Result`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-RUSTDEV-CRIT-01-B ✅ DONE
- **Tên**: Cập nhật tất cả call sites của `encode_jwt`
- **File**: `src/api/identity.rs`, `src/api/admin.rs`, `src/auth.rs`, `src/sso.rs`, `src/mail.rs`, `src/db/models/attachment.rs`, `src/api/core/sends.rs`
- **Mô tả**: Tất cả call sites (15+ locations) đã được cập nhật với `?` propagation. `encode_ssotoken_claims()` và `encode_code_claims()` trong `sso.rs` cũng đã đổi sang `ApiResult<String>`. `json!()` macros được refactor để extract token trước khi truyền vào macro.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-CRIT-01-A

### TASK-RUSTDEV-CRIT-01-C ✅ DONE
- **Tên**: Thêm unit test cho `encode_jwt` không panic
- **File**: `src/auth.rs` (test module)
- **Mô tả**: Đã implement đầy đủ 3 test cases trong `src/auth.rs` mod test: `test_encode_jwt_returns_ok()` — encode valid claims → assert `Ok(token)` với 3 parts. `test_encode_decode_roundtrip()` — encode→decode với `jsonwebtoken::decode` + assert all fields match. `test_expired_jwt_rejected()` — exp trong quá khứ → verify `ErrorKind::ExpiredSignature`. `test_tampered_jwt_rejected()` — XOR first byte signature → verify decode fails. Tests dùng `OnceLock` để init test keys một lần. Cũng bao gồm AES-GCM roundtrip tests (MED-02-B/C).
- **Loại**: New test
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-CRIT-01-A

---

## SEC-HIGH-01: JWT trong URL Query Parameter [1 ngày]

**File**: `src/api/notifications.rs:51-53`  
**Rủi ro**: JWT token lộ trong server logs, browser history, proxy logs, Referer headers

### TASK-RUSTDEV-CRIT-02-A ⏳ PENDING (Sprint 3)
- **Tên**: Xóa `WsAccessToken` struct và query param fallback
- **File**: `src/api/notifications.rs`
- **Mô tả**: `WsAccessToken` struct và query param fallback vẫn còn trong `src/api/notifications.rs:88-89`. Xóa hoàn toàn `struct WsAccessToken { access_token: Option<String> }`. Chỉ đọc token từ `Authorization: Bearer <token>` header. Trả `Status::Unauthorized` nếu header thiếu. Chờ kết quả research CRIT-02-C xác nhận: official clients đã dùng header từ ~2023 → **safe to remove** trong Sprint 3.
- **Loại**: Modify existing — BREAKING CHANGE
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-CRIT-02-C (research completed → go ahead)

**Code tham khảo**:
```rust
// Trong websocket_hub():
let token = req.headers().get_one("Authorization")
    .and_then(|h| h.strip_prefix("Bearer "))
    .ok_or_else(|| Error::new("Missing Authorization header for WebSocket", ""))?;
let claims = auth::decode_login_jwt(token)?;
```

### TASK-RUSTDEV-CRIT-02-B ✅ DONE (Deprecation warning)
- **Tên**: Thêm deprecation log tạm thời (optional backward compat)
- **File**: `src/api/notifications.rs`
- **Mô tả**: Đã implement backward compat: chấp nhận query param nhưng log `warn!("WS: received JWT via URL query param from ...; clients should use Authorization header")`. Deadline for hard removal: Sprint 3.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-CRIT-02-A
- **Ghi chú**: Backward compat mode active. Remove query param support in Sprint 3 after client compatibility verified.

### TASK-RUSTDEV-CRIT-02-C ✅ DONE
- **Tên**: Research client compatibility
- **File**: `specs/bugs/rust-dev/tasks/research-ws-auth.md`
- **Mô tả**: Research hoàn thành. Kết quả: web vault, browser extension, desktop (Electron), mobile (Android/iOS) đều dùng `Authorization: Bearer` header từ ~2023. Query param path là legacy fallback cho các clients cũ. **Recommendation**: Remove query param support trong Sprint 3 — risk LOW. Action items documented: xóa `WsAccessToken` struct, update CHANGES.md, monitor deprecation logs trước khi remove.
- **Loại**: Research
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

---

## Acceptance Criteria

- [x] `encode_jwt` không bao giờ panic — trả `Result<String, Error>` ✅
- [x] Tất cả callers xử lý `Result` đúng, không có `.unwrap()` bọc `encode_jwt` ✅
- [x] Unit tests: roundtrip, expired, tampered — tất cả pass (`src/auth.rs` mod tests) ✅
- [~] WebSocket không chấp nhận JWT qua query param → deprecation warning active; hard removal **Sprint 3** (CRIT-02-A)
- [~] `WsAccessToken` struct còn tồn tại — cần xóa trong Sprint 3 sau khi monitor deprecation logs
- [x] Research client compatibility xác nhận: official clients dùng header → safe to remove ✅

---

*Tạo từ SOL-rust-dev.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: CRIT-01 ✅ (01-A ✅ 01-B ✅ 01-C ✅), CRIT-02 🔄 (02-A Sprint 3 pending, 02-B ✅, 02-C ✅)*
