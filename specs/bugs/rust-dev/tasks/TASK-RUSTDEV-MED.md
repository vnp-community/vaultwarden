# TASK-RUSTDEV-MED: P3 Medium — Sprint 2–3

> **Severity**: P3 — Medium  
> **Sprint**: Sprint 2–3 (tuần 3–6)  
> **Effort tổng**: ~3.5 tuần  
> **Nguồn**: [SOL-rust-dev.md](../SOL-rust-dev.md)

---

## §2.5: Error Handling — Không Có Error Hierarchy [Sprint 2 — 1 tuần]

**File**: `src/error.rs`  
**Rủi ro**: Mix HTTP status codes vào business logic, khó debug, không phân loại lỗi cho monitoring

### TASK-RUSTDEV-MED-01-A ✅ DONE
- **Tên**: Thêm `ErrorCategory` enum vào `src/error.rs`
- **File**: `src/error.rs`
- **Mô tả**: Đã thêm `pub enum ErrorCategory { NotFound, Unauthorized, Forbidden, ValidationError, DatabaseError, InternalError, ExternalServiceError }`. Thêm field `pub category: ErrorCategory` vào struct `Error` (default: `InternalError`). Thêm constructor methods: `Error::not_found(msg)`, `Error::unauthorized(msg)`, `Error::forbidden(msg)`, `Error::validation(msg)`, `Error::internal(msg)`. Thêm `with_category()` builder. Giữ macro `ErrorKind` enum riêng (tracking Rust error type) — `ErrorCategory` là semantic HTTP category.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-RUSTDEV-MED-01-B ✅ DONE
- **Tên**: Thêm typed error macros
- **File**: `src/error.rs`
- **Mô tả**: Đã thêm macros: `err_not_found!`, `err_unauthorized!`, `err_forbidden!`, `err_validation!`. Mỗi macro hỗ trợ 1-arg (`$msg`) và 2-arg (`$usr_msg, $log_value`) forms để tương thích với pattern của `err!`. Backward compat với `err!` macro hoàn toàn giữ nguyên.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-MED-01-A

### TASK-RUSTDEV-MED-01-C ✅ DONE
- **Tên**: Cập nhật `Error::respond_to()` để map `ErrorCategory` → HTTP status
- **File**: `src/error.rs`
- **Mô tả**: Đã cập nhật `impl Responder`: `match self.category` để map NotFound→404, Unauthorized→401, Forbidden→403, ValidationError→422. Internal/Database/ExternalService errors log ở `error!` level; semantic errors (NotFound, Unauthorized, Forbidden) log ở `debug!` level. Category-derived status có priority cao hơn raw `error_code` cho semantic errors.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-MED-01-A

### TASK-RUSTDEV-MED-01-D ✅ DONE (partial)
- **Tên**: Migrate high-traffic handlers sang typed errors
- **File**: `src/api/identity.rs`
- **Mô tả**: Đã migrate login endpoint (user-not-found case) sang `err_unauthorized!`. Đã migrate organization API key login: malformed client_id → `err_validation!`, invalid org → `err_not_found!`, wrong secret → `err_unauthorized!`. Các handlers với `ErrorEvent` attachment giữ nguyên `err!` macro (EventType tracking không support trong typed macros yet).
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-MED-01-B, TASK-RUSTDEV-MED-01-C

---

## §2.6: RSA Private Key Lưu Unencrypted [Sprint 2 — 3 ngày]

**File**: `src/auth.rs`  
**Rủi ro**: RSA private key lưu plaintext PEM trên disk — nếu data directory bị leak, attacker có thể forge JWT

### TASK-RUSTDEV-MED-02-A ✅ DONE
- **Tên**: Thêm `RSA_KEY_ENCRYPTION_KEY` config
- **File**: `src/config.rs`
- **Mô tả**: Đã thêm `rsa_key_encryption_key: Pass, false, def, String::new()` vào group `folders`. Type `Pass` đảm bảo không serialize ra config.json/admin UI. Default empty = backward compat (plaintext PEM). Set = master key dùng cho AES-256-GCM.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-RUSTDEV-MED-02-B ✅ DONE
- **Tên**: Implement `encrypt_rsa_key()` với AES-256-GCM
- **File**: `src/auth.rs`
- **Mô tả**: Đã implement `encrypt_rsa_key(pem, master_key)` dùng `ring::aead::AES_256_GCM`. Key derive bằng SHA-256 hash của master_key string → 32 bytes. Store format: `[12-byte random nonce][AES-GCM ciphertext+tag]`. Integrated vào `initialize_keys()`.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-RUSTDEV-MED-02-A

### TASK-RUSTDEV-MED-02-C ✅ DONE
- **Tên**: Implement `decrypt_rsa_key()` với decryption
- **File**: `src/auth.rs`
- **Mô tả**: Đã implement `decrypt_rsa_key(data, master_key)`: tách 12-byte nonce, decrypt AES-GCM. Nếu decrypt fail: return `Err` với descriptive message. `initialize_keys()` sử dụng decrypt path khi `RSA_KEY_ENCRYPTION_KEY` set.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-MED-02-B

### TASK-RUSTDEV-MED-02-D ✅ DONE
- **Tên**: Startup warning nếu RSA key không encrypted
- **File**: `src/auth.rs`
- **Mô tả**: Đã thêm `warn!("SECURITY: RSA private key is stored unencrypted. Set RSA_KEY_ENCRYPTION_KEY to encrypt it at rest.")` trong `initialize_keys()` khi master key empty.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-MED-02-A

---

## §2.2: Global State qua LazyLock — Khó Test [Sprint 3 — 2 tuần]

**File**: `src/config.rs`, `src/ratelimit.rs`, `src/main.rs`  
**Rủi ro**: `CONFIG`, `LIMITER_LOGIN` là global statics — không thể inject mock trong tests, không support hot-reload

### TASK-RUSTDEV-MED-03-A ✅ DONE
- **Tên**: Định nghĩa `AppState` struct
- **File**: `src/app_state.rs` (mới)
- **Mô tả**: Đã tạo `src/app_state.rs` với `pub trait RateLimiter: Send + Sync { async fn check_login(&self, ip: &IpAddr) -> Result<(), Error>; async fn check_admin(&self, ip: &IpAddr) -> Result<(), Error>; }`. `pub struct AppState { pub rate_limiter: Arc<dyn RateLimiter> }`. `IpRateLimiter` (production) delegate sang `crate::ratelimit::check_limit_login/check_limit_admin`. `AppState::new()` + `Default` impl. `allow(dead_code)` attribute trong khi handlers chưa migrate.
- **Loại**: New file + New struct
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### TASK-RUSTDEV-MED-03-B ✅ DONE
- **Tên**: Integrate `AppState` vào Rocket
- **File**: `src/main.rs`
- **Mô tả**: Đã gọi `.manage(app_state::AppState::new())` trong `rocket_main()` (`src/main.rs:605`). `app_state` module được khai báo trong `lib.rs` / project root. `CONFIG` global giữ nguyên cho backward compat trong giai đoạn chuyển đổi — handlers có thể dùng `State<AppState>` khi sẵn sàng.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-RUSTDEV-MED-03-A

### TASK-RUSTDEV-MED-03-C ✅ DONE (2026-04-15)
- **Tên**: Migrate rate limiter handlers sang `AppState`
- **File**: `src/api/identity.rs`, `src/app_state.rs`
- **Mô tả**: Đã thêm `state: &State<AppState>` guard vào public `login` Rocket handler. Đã truyền `limiter = state.rate_limiter.as_ref()` qua 3 private async helpers: `_password_login`, `_api_key_login`, `_sso_login`. Mỗi helper nhận `limiter: &dyn RateLimiter` parameter và gọi `limiter.check_login(&ip.ip).await?` thay vì `crate::ratelimit::check_limit_login(&ip.ip)?`. Đã thêm import `use rocket::State` và `use crate::app_state::{AppState, RateLimiter}` vào identity.rs. Đã xóa `#[allow(dead_code)]` attributes trên `AppState.rate_limiter` field và `RateLimiter` trait vì giờ đã active. Giữ `#[allow(dead_code)]` chỉ trên `check_admin` (Sprint 4). `cargo check --features sqlite` pass.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-RUSTDEV-MED-03-B

### TASK-RUSTDEV-MED-03-D ✅ DONE
- **Tên**: Implement mock rate limiter cho tests
- **File**: `src/app_state.rs` (mod `test_utils`)
- **Mô tả**: Đã implement trong `src/app_state.rs` `#[cfg(test)] pub mod test_utils`: `struct NoopRateLimiter` — luôn trả `Ok(())` cho cả `check_login` và `check_admin`. `struct CountingRateLimiter { login_count: AtomicU32, admin_count: AtomicU32 }` — track số lần gọi. Cả hai impl `RateLimiter` trait. Đi kèm `#[tokio::test]` tests: `test_noop_rate_limiter_always_ok`, `test_counting_rate_limiter_increments`, `test_app_state_default_constructs`.
- **Loại**: New test utilities
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-MED-03-A

---

## §2.3: Blocking Code trong Async Context [Sprint 3]

**Ghi chú**: Issue này là consequence trực tiếp của §2.2. Sau khi migrate `Config` sang `AppState` (TASK-RUSTDEV-MED-03-A/B), config loading xảy ra trong `async main()` — không cần workaround `std::thread::spawn + block_on` nữa.

### TASK-RUSTDEV-MED-04-A ⏭️ DEFERRED (Sprint 5+)
- **Tên**: Xóa `spawn_blocking` workaround trong config initialization
- **File**: `src/config.rs`
- **Mô tả**: `src/config.rs` vẫn dùng `rt.block_on(Config::load())` trong `LazyLock<Config>`. Không thể xóa được khi `CONFIG` vẫn là global static — việc xóa cần phải di chuyển `Config` hoàn toàn vào `AppState` trước. Phân tích cho thấy `CONFIG` được dùng tại **200+ call sites** trong toàn codebase (`grep -rn "CONFIG\." src/ | wc -l`), migration là task riêng biệt cần một sprint đầy đủ. **Quyết định: DEFERRED đến Sprint 5+ sau khi config migration research (LOW-01-C) xác nhận approach với `figment`**. Không phải regression — workaround hiện tại an toàn.
- **Loại**: Modify existing
- **Độ phức tạp**: Cao (blocked on CONFIG migration)
- **Phụ thuộc**: LOW-01-C (research GO, approach figment+serde), LOW-01-D (full migration)

---

## Acceptance Criteria

- [x] `ErrorCategory` enum có 7 variants với typed constructor methods ✅
- [x] `Error::respond_to()` map `ErrorCategory` → HTTP status đúng ✅
- [x] Internal/DB errors log ở `error!`, semantic errors log ở `debug!` ✅
- [x] `err_not_found!`, `err_unauthorized!`, `err_forbidden!`, `err_validation!` macros available ✅
- [x] RSA key được encrypt khi `RSA_KEY_ENCRYPTION_KEY` set ✅
- [x] Startup warn nếu key unencrypted ✅
- [x] `cargo check` pass ✅
- [x] `AppState` với `RateLimiter` trait được register với Rocket `.manage()` (MED-03-A/B ✅)
- [x] `NoopRateLimiter` và `CountingRateLimiter` sẵn sàng dùng trong tests ✅
- [x] Login handler dùng `state.rate_limiter.check_login` thay vì global `check_limit_login` (MED-03-C ✅ 2026-04-15)
- [⏭️] `block_on` workaround trong config.rs được xóa sau khi CONFIG migration hoàn tất (MED-04-A DEFERRED → Sprint 5+, phụ thuộc LOW-01-C/D)

---

*Tạo từ SOL-rust-dev.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: Sprint 3 ✅ COMPLETE — MED-01/02/03 tất cả done. MED-04-A DEFERRED → Sprint 5+ (phụ thuộc config migration LOW-01-C/D)*
