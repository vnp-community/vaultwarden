# TASK-SEC-CRIT-01: Admin Token Plaintext Fallback

> **Severity**: P1 — Critical  
> **Ưu tiên**: Làm Ngay  
> **Effort**: 2 ngày  
> **File gốc**: `src/api/admin.rs:245`  
> **Rủi ro**: Brute-force admin panel với low-entropy plaintext token

---

## Mô Tả Vấn Đề

Admin token cho phép dùng plaintext thay vì Argon2id hash. Nếu token yếu (vd: "admin123"), attacker có thể brute-force admin panel mà không bị giới hạn entropy.

---

## Sub-tasks

### TASK-SEC-CRIT-01-A ✅ DONE
- **Tên**: Implement `validate_admin_token()` tại startup
- **File**: `src/api/admin.rs`
- **Mô tả**: Hàm kiểm tra token tại startup: nếu token không bắt đầu bằng `$argon2`, log CRITICAL warning. Nếu `ADMIN_TOKEN_STRICT_MODE=true` (default từ v2.0): `return Err(...)` để server không khởi động. Nếu non-strict: warn và tiếp tục (backward compat).
- **Loại**: New function
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-CRIT-01-B
- **Triển khai**: `src/api/admin.rs` — `pub fn validate_admin_token() -> Result<(), Error>`

### TASK-SEC-CRIT-01-B ✅ DONE
- **Tên**: Thêm config key `ADMIN_TOKEN_STRICT_MODE`
- **File**: `src/config.rs`
- **Mô tả**: Thêm vào `make_config!`: `admin_token_strict_mode: bool, false, def, true`. Default `true` từ v2.0 (breaking change có warning). Document trong CHANGES.md.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Triển khai**: `src/config.rs` — `admin_token_strict_mode: bool, false, def, true`

### TASK-SEC-CRIT-01-C ✅ DONE
- **Tên**: Gọi `validate_admin_token()` tại startup
- **File**: `src/main.rs`
- **Mô tả**: Gọi `validate_admin_token()?` sớm trong `main()`, trước khi Rocket bắt đầu serve requests. Đảm bảo server fail fast nếu strict mode và token không đúng.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-CRIT-01-A
- **Triển khai**: `src/main.rs` — gọi `api::validate_admin_token()` sớm trong startup; exit(1) nếu lỗi

### TASK-SEC-CRIT-01-D ✅ DONE (2026-04-15)
- **Tên**: Update `vaultwarden hash` command documentation
- **File**: `src/main.rs`
- **Mô tả**: Đã cập nhật 2 chỗ trong `src/main.rs`: (1) `HELP` constant — thêm section "GENERATING AN ADMIN TOKEN" với 4 bước rõ ràng: run hash, copy output, về format `$argon2`, link docs wiki. (2) Output của `hash` command — sau khi in PHC string, in thêm "Next steps" gồm copy ADMIN_TOKEN vào .env, lý do về strict mode, restart, link docs. Bổ trung `// TASK-SEC-CRIT-01-D` comment. `cargo check` pass.
- **Loại**: Documentation + UX improvement
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-CRIT-01-A
- **Sprint**: Sprint 3

---

## Acceptance Criteria

- [x] Server không khởi động nếu `ADMIN_TOKEN_STRICT_MODE=true` và token là plaintext
- [x] Warning rõ ràng trong log khi dùng plaintext token ở non-strict mode
- [x] `vaultwarden hash` command hoạt động và output Argon2id PHC string cùng hướng dẫn next steps rõ ràng ✅ (CRIT-01-D 2026-04-15)
- [x] Config key `ADMIN_TOKEN_STRICT_MODE` đã thêm
- [x] `cargo check` pass ✅

---

*Tạo từ SOL-security.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: ✅ COMPLETE — CRIT-01-A/B/C/D tất cả done*
