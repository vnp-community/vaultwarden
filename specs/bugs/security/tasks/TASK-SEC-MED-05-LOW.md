# TASK-SEC-MED-05 và SEC-LOW: Medium/Low Priority Security Fixes

> **Severity**: P3–P4  
> **Sprint**: Sprint 4–5

---

## SEC-MED-05: Push Relay Metadata Leak [Sprint 4 — 1 ngày]

**File**: `src/api/push.rs`  
**Rủi ro**: Privacy leak — usage patterns gửi tới bên thứ ba (Bitwarden push server)

### TASK-SEC-MED-05-A ✅ DONE (2026-04-15 — code review verified)
- **Tên**: Minimize metadata trong push relay requests
- **File**: `src/api/push.rs`
- **Mô tả**: Code review xác nhận: push relay requests chỉ gửi `device_push_id` (opaque push token), event `type` (int), và payload size. `user_uuid` và `org_uuid` không bao giờ được gửi ra relay. Tình trạng hiện tại đã tugoân theo GDPR minimization — không cần thay đổi code.
- **Loại**: Verify — no change needed
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-SEC-MED-05-B ✅ DONE (2026-04-15)
- **Tên**: Thêm privacy warning vào config documentation
- **File**: `src/config.rs` — `PUSH_RELAY_URI` doc string
- **Mô tả**: Privacy note được document inline trong config key: only push token + event type sent; no vault data. Self-host guidance referenced in key comment. Nếu `PUSH_RELAY_URI` trỏng (default) thì feature tắt hoàn toàn — quân y quản lý privacy tốt nhất.
- **Loại**: Documentation (inline config)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-MED-05-A

---

## SEC-LOW-01: RSA-2048 Không Future-Proof [Sprint 5 — 1 tuần]

**Rủi ro**: NIST khuyến nghị post-quantum migration trước 2035

### TASK-SEC-LOW-01-A ✅ DONE (2026-04-15)
- **Tên**: Implement `rotate_jwt_signing_key()` admin action
- **File**: `src/auth.rs`, `src/api/admin.rs`
- **Mô tả**: Đổi `PRIVATE_RSA_KEY`/`PUBLIC_RSA_KEY` từ `OnceLock` sang `RwLock<Option<...>>` — cho phép hot-swap không cần restart. `rotate_jwt_signing_key()`: (1) archive key cũ với timestamp suffix `{filename}.{YYYYMMDDTHHMMSSz}.bak` (best-effort), (2) generate RSA-2048 mới, (3) persist (encrypt nếu `RSA_KEY_ENCRYPTION_KEY` được set), (4) hot-swap cả hai statics dưới write lock. Admin endpoint `POST /admin/rotate-jwt-key`: gọi `rotate_jwt_signing_key()` + reset `security_stamp` toàn bộ user (buộc re-login). Trả JSON với new public key PEM và số sessions bị invalidate.
- **Loại**: New function + admin endpoint
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### TASK-SEC-LOW-01-B ✅ DONE (2026-04-15)
- **Tên**: Thêm `JWT_KEY_ROTATION_SCHEDULE` config key + scheduler job
- **File**: `src/config.rs` — `jobs {}` section; `src/main.rs` — `schedule_jobs()`
- **Mô tả**: Đã thêm `jwt_key_rotation_schedule: String, false, def, String::new()`. Khi non-empty: đăng ký job với cron schedule — job gọi `rotate_jwt_signing_key()` rồi reset toàn bộ user security_stamp. Cảnh báo rõ: mỗi rotation buộc tất cả user re-authenticate. Empty = manual only (default, an toàn cho mội người).
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-LOW-01-A

### TASK-SEC-LOW-01-C ✅ DONE (2026-04-15 — research only, no implementation)
- **Tên**: Research ES256 (ECDSA P-256) support
- **File**: `docs/research_es256_jwt.md` (NEW)
- **Mô tả**: Đã tạo research document. Kết luận: KHAI TRIỂN ES256 không phù hợp tại thời điểm này. Biếrden clients không verify JWT signature client-side — liợi thế hiệu suất không đáng kể. Thay đổi algorithm có rủi ro breaking changes khi Bitwarden clients thêm strict validation trong tương lai. ES256 cũng không post-quantum safe. Nên xem xét lại khi NIST ML-DSA được `jsonwebtoken` hỗ trợ (~2030+).
- **Loại**: Research
- **Độ phức tạp**: Cao
- **Phụ thuộc**: Không

---

## SEC-LOW-02: Anonymous WebSocket Rate Limiting [Sprint 4 — 1 ngày]

**File**: `src/api/notifications.rs`  
**Rủi ro**: DoS via connection exhaustion trên anonymous WebSocket endpoint

### TASK-SEC-LOW-02-A ✅ DONE (2026-04-15)
- **Tên**: Thêm per-IP rate limiting cho `/notifications/anonymous-hub`
- **File**: `src/ratelimit.rs`, `src/api/notifications.rs`
- **Mô tả**: Đã thêm `LIMITER_ANON_WS: LazyLock<Limiter>` trong `ratelimit.rs` — window 60 giây, burst configurable qua `WS_ANON_RATELIMIT_BURST` env var (default 10). `check_limit_anon_ws(&ip)` gọi trong `anonymous_websockets_hub()` trước khi tăng `WS_ANON_ACTIVE` counter — IP bị reject trước khi tiêu thụ slot. Trả `429 Too Many Requests` khi exceeded.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-SEC-LOW-02-B ✅ DONE — FULL (2026-04-15)
- **Tên**: Thêm max concurrent anonymous connections — migrated to `make_config!` key
- **File**: `src/config.rs` — `ws {}` section; `src/api/notifications.rs`
- **Mô tả**: Đã nâng cấp từ raw env var sang proper config key `ws_anon_max_connections: usize, false, def, 100`. `WS_ANON_MAX` LazyLock giờ là `LazyLock::new(|| CONFIG.ws_anon_max_connections())` thay vì đọc `std::env::var("WS_ANON_MAX_CONNECTIONS")` thủ công. Backward compatible: `WS_ANON_MAX_CONNECTIONS` env var vẫn hoạt động do config system tự đọc env vars. Operator có thể cấu hình qua admin UI.
- **Loại**: Modify existing (upgrade partial to full)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-LOW-02-A

---

## SEC-LOW-03: KDF Iterations Không Enforce [Sprint 4 — 1 ngày]

**File**: `src/db/models/user.rs:128`  
**Rủi ro**: User có thể set KDF params cực thấp, weakening master password protection

### TASK-SEC-LOW-03-A ✅ DONE (2026-04-15)
- **Tên**: Thêm minimum KDF config keys
- **File**: `src/config.rs`
- **Mô tả**: Đã thêm 3 config keys vào section user account: `min_pbkdf2_iterations: u32, true, def, 600_000` (OWASP/NIST 2024 recommendation), `min_argon2_memory_kb: u32, true, def, 65_536` (64 MB OWASP minimum), `enforce_min_kdf: bool, true, def, true` (true = reject, false = warn-only for migration). Các key này có doc string giải thích lý do cần minimum.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-SEC-LOW-03-B ✅ DONE (2026-04-15)
- **Tên**: Implement validate_kdf_config() trong account registration và password change
- **File**: `src/api/core/accounts.rs` — `set_kdf_data()`
- **Mô tả**: Đã nâng cấp `set_kdf_data()`: (1) PBKDF2: giữ hard floor 100k; thêm configurable floor từ `CONFIG.min_pbkdf2_iterations()`: nếu violation và `enforce_min_kdf()=true` → err!, nếu false → warn!. (2) Argon2id: giữ hard mem range 15–1024 MB; thêm configurable floor từ `CONFIG.min_argon2_memory_kb()` (KB unit so sánh với m*1024): tương tự enforce/warn pattern. Áp dụng cho cả registration và password change (set_kdf_data gọi từ cả hai path).
- **Loại**: Modify existing (enhanced)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-LOW-03-A

---

## SEC-LOW-04: SQLite Backup Exposure [Sprint 4 — 0.5 ngày]

**File**: `src/db/mod.rs`  
**Rủi ro**: SQLite backup file có thể accessible qua web nếu nằm trong data folder

### TASK-SEC-LOW-04-A ✅ DONE — FULLY WIRED (2026-04-15)
- **Tên**: Cảnh báo nếu backup folder trong data folder
- **File**: `src/config.rs` — `check_backup_location()`; `src/main.rs` — startup
- **Mô tả**: Đã implement `check_backup_location()` trong `src/config.rs`: dùng `Path::canonicalize()` + `Path::starts_with()` để detect nếu `backup_folder` là subdirectory của `data_folder`. Nếu đúng: in cảnh báo rõ ràng với link tới `docs/nginx_backup_block.md`. Goọi ở `src/main.rs` trong startup sequence (sau `check_config_file_permissions()`). Đây là non-blocking warning — không fail startup.
- **Loại**: New function + startup integration
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-SEC-LOW-04-B ✅ DONE (2026-04-15)
- **Tên**: Thêm `BACKUP_FOLDER` config key
- **File**: `src/config.rs` — `folders` section
- **Mô tả**: Đã thêm `backup_folder: String, false, auto, |c| format!("{}/backups", c.data_folder)`. Default `{DATA_FOLDER}/backups`. Doc string cảnh báo operator phải block web access tới thư mục này (reference `docs/nginx_backup_block.md`).
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-SEC-LOW-04-C ✅ DONE (2026-04-15)
- **Tên**: Thêm nginx config template để block backup files
- **File**: `docs/nginx_backup_block.md` (NEW)
- **Mô tả**: Đã tạo `docs/nginx_backup_block.md` với: Option A (URL path `deny all`), Option B (filesystem permissions), full nginx server block example với cả backup block và SQLite extension block (`*.sqlite3|*.db|*.bak|*.sql|*.dump`), env var reference table, verification checklist.
- **Loại**: Documentation (new file)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

---

## SEC-LOW-05: CSP Không Được Enforce [Sprint 1]

**Ghi chú**: Đã được cover trong **SOL-001** (CR-001 — `SecurityHeadersFairing`). Xem [TASKS-SOL-001.md](../../crs/v1/solutions/tasks/TASKS-SOL-001.md), task TASK-001-001 và TASK-001-002.

Không cần task riêng — implement như một phần của SOL-001.

---

*Tạo từ SOL-security.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: MED-05-A ✅ (verified), MED-05-B ✅, LOW-01-A/B/C ✅ DONE, LOW-02-A/B ✅ DONE (full), LOW-03-A/B ✅ DONE, LOW-04-A/B/C ✅ DONE*
