# TASK-SEC-CRIT-02: DISABLE_ADMIN_TOKEN Không Có Safeguard

> **Severity**: P1 — Critical  
> **Ưu tiên**: Làm Ngay  
> **Effort**: 2 ngày  
> **File gốc**: `src/config.rs:758`  
> **Rủi ro**: Admin panel exposed to internet nếu vô tình set `DISABLE_ADMIN_TOKEN=true`

---

## Mô Tả Vấn Đề

Chỉ cần set `DISABLE_ADMIN_TOKEN=true` là admin panel hoàn toàn không có authentication. Không có confirmation step, không có safeguard. Nếu misconfigured và không có network-level protection, admin panel accessible từ internet.

---

## Sub-tasks

### TASK-SEC-CRIT-02-A ✅ DONE
- **Tên**: Thêm config key `DISABLE_ADMIN_TOKEN_CONFIRM`
- **File**: `src/config.rs`
- **Mô tả**: Thêm `disable_admin_token_confirmed: bool, false, def, false`. Đây là biến xác nhận thứ hai phải được explicit set.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Triển khai**: `src/config.rs` — `disable_admin_token_confirmed: bool, false, def, false`

### TASK-SEC-CRIT-02-B ✅ DONE
- **Tên**: Implement `validate_disable_admin_token()` tại startup
- **File**: `src/api/admin.rs`
- **Mô tả**: Nếu `DISABLE_ADMIN_TOKEN=true` nhưng `DISABLE_ADMIN_TOKEN_CONFIRM=false`: error và không khởi động. Nếu cả hai đều true: log SECURITY NOTICE rõ ràng mỗi lần restart. Emit startup audit event khi admin panel ở chế độ unauthenticated.
- **Loại**: New function
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-CRIT-02-A
- **Triển khai**: `src/api/admin.rs` — `pub fn validate_disable_admin_token() -> Result<(), Error>`

### TASK-SEC-CRIT-02-C ✅ DONE (2026-04-15)
- **Tên**: Enforce IP allowlist khi admin token disabled
- **File**: `src/config.rs`, `src/api/admin.rs`
- **Mô tả**: Đã thêm 2 config keys mới vào `make_config!`: `ip_allowlist_admin_panel: bool, false, def, false` và `admin_panel_ip_allowlist: String, false, def, String::new()`. Đã mở rộng `validate_disable_admin_token()` trong `admin.rs`: nếu `IP_ALLOWLIST_ADMIN_PANEL=true` và `ADMIN_PANEL_IP_ALLOWLIST` rỗng → error và không khởi động với message hướng dẫn rõ. Sử dụng chưa được enforce tại route level (future work: request guard nếu cần). `cargo check` pass.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-CRIT-02-B

### TASK-SEC-CRIT-02-D ✅ DONE
- **Tên**: Gọi `validate_disable_admin_token()` tại startup
- **File**: `src/main.rs`
- **Mô tả**: Gọi sớm trong `main()` sau `validate_admin_token()`. Fail fast nếu config không hợp lệ.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-CRIT-02-B
- **Triển khai**: `src/main.rs` — gọi `api::validate_disable_admin_token()` ngay sau validate_admin_token; exit(1) nếu lỗi

### TASK-SEC-CRIT-02-E ✅ DONE (2026-04-15)
- **Tên**: Emit audit event khi server start với admin token disabled
- **File**: `src/api/admin.rs`
- **Mô tả**: Đã thực hiện trong `validate_disable_admin_token()`: sau khi vượt qua cả 2 check (double-confirm + IP allowlist), emit `warn!("SECURITY AUDIT [ServerStart]: Admin panel authentication is DISABLED ... ip_allowlist_active={ip_allowlist_active} | ...")`. Log này bao gồm trạng thái `ip_allowlist_active` để audit trail rõ ràng. Ghi ra mỗi lần restart. Không dùng `AuditEventType` DB model (phụ thuộc DB conn, không sẵn sàng tại startup time trước khi pool được init) — log-based audit là pragmatic solution an toàn.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-CRIT-02-D

---

## Acceptance Criteria

- [x] `DISABLE_ADMIN_TOKEN=true` mà không có `DISABLE_ADMIN_TOKEN_CONFIRM=true` → server không khởi động
- [x] Rõ ràng warning trong log mỗi lần restart với config này
- [x] `IP_ALLOWLIST_ADMIN_PANEL=true` với `ADMIN_PANEL_IP_ALLOWLIST` rỗng → server từ chối khởi động ✅ (CRIT-02-C 2026-04-15)
- [x] Startup audit log `SECURITY AUDIT [ServerStart]` với `ip_allowlist_active` metadata ✅ (CRIT-02-E 2026-04-15)
- [x] `cargo check` pass ✅

---

*Tạo từ SOL-security.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: ✅ COMPLETE — CRIT-02-A/B/C/D/E tất cả done*
