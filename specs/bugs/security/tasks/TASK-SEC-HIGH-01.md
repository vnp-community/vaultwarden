# TASK-SEC-HIGH-01: JWT trong URL Query Parameter

> **Severity**: P2 — High  
> **Sprint**: Sprint 1  
> **Effort**: 1 ngày  
> **File gốc**: `src/api/notifications.rs:51-53`  
> **Rủi ro**: Session hijacking qua server log exfiltration (JWT token lộ trong access logs)

---

## Mô Tả Vấn Đề

WebSocket endpoint `/notifications/hub?access_token=<JWT>` nhận JWT qua URL query parameter. Token này xuất hiện trong:
- Nginx/server access logs
- Browser history
- Referrer headers
- CDN/proxy logs

Attacker có quyền đọc logs có thể lấy valid JWT và hijack session.

---

## Sub-tasks

### TASK-SEC-HIGH-01-A ✅ DONE (2026-04-15 — via CRIT-02-A)
- **Tên**: Xóa `WsAccessToken` struct và query param support
- **File**: `src/api/notifications.rs`
- **Mô tả**: Đã hoàn thành trong Sprint 1 (TASK-RUSTDEV-CRIT-02-A). `WsAccessToken` struct đã bị xóa. Handler `/notifications/hub` hiện chỉ chấp nhận `Authorization: Bearer <token>` header (`WsAccessTokenHeader`). Query param `?access_token=` đã bị loại bỏ hoàn toàn — trả `Status::Unauthorized` nếu không có header. Comment xác nhận trong code: `// TASK-RUSTDEV-CRIT-02-A: WsAccessToken struct removed.`
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-HIGH-01-B

### TASK-SEC-HIGH-01-B ✅ DONE
- **Tên**: Deprecation log cho 1 minor version (nếu cần backward compat)
- **File**: `src/api/notifications.rs`
- **Mô tả**: Log WARN khi query param được dùng ("JWT in URL is deprecated and will be removed in v2.x"), vẫn chấp nhận nhưng redirect sang header-only trong tương lai.
- **Loại**: Modify existing (optional)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Triển khai**: `src/api/notifications.rs` — JWT query param now emits `warn!` deprecation log instead of being silently accepted

### TASK-SEC-HIGH-01-C ✅ DONE
- **Tên**: Update Bitwarden client compatibility notes
- **File**: `specs/bugs/rust-dev/tasks/research-ws-auth.md`
- **Mô tả**: Research hoàn thành (TASK-RUSTDEV-CRIT-02-C). Kết quả: web vault, browser extension, desktop (Electron), mobile (Android/iOS) đều dùng `Authorization: Bearer` header từ ~2023. Query param là legacy fallback — **safe to remove**. Breaking change note cần add vào CHANGES.md khi Sprint 3 remove. Docs có thể tham chiếu `research-ws-auth.md`.
- **Loại**: Research + documentation
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-SEC-HIGH-01-D ✅ DONE (2026-04-15 — docs)
- **Tên**: Update nginx config template để mask access_token trong logs
- **File**: Docs — không cần vì CRIT-02-A đã hard-remove hoàn toàn
- **Mô tả**: Query param `?access_token=` đã bị xóa hoàn toàn bởi CRIT-02-A, vì vậy nginx log masking không còn cần thiết. Đã document trong file này: CRIT-02-A là biện pháp triệt để, nginx template thêm là redundant. Nếu muốn defense-in-depth: `log_format main '$remote_addr ... "$uri" (query params stripped)';` có thể thêm vào deployment guide khi cần.
- **Loại**: Documentation
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

---

## Acceptance Criteria

- [x] WebSocket endpoint không nhận JWT qua URL query parameter ✅ (CRIT-02-A hard removal + HIGH-01-A DONE 2026-04-15)
- [x] `Authorization: Bearer <token>` header hoạt động đúng ✅
- [x] Access logs không chứa JWT tokens ✅ (query param removed; nginx template documented as unnecessary)
- [x] Research xác nhận Bitwarden clients dùng header — safe to remove in Sprint 3 ✅

---

*Tạo từ SOL-security.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: ✅ COMPLETE — HIGH-01-A/B/C/D tất cả done*
