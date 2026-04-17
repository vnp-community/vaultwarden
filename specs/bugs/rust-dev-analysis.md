# Vaultwarden — Phân Tích Kỹ Thuật Từ Góc Nhìn Đội Phát Triển Rust

> **Tác giả**: Phân tích bởi Rust developer  
> **Ngày**: 2026-04-11  
> **Phiên bản**: 1.0  
> **Phạm vi**: Chất lượng code, kiến trúc, hiệu năng, khả năng mở rộng, maintainability

---

## 1. Điểm mạnh kỹ thuật

### 1.1 Code Safety
- `#![forbid(unsafe_code)]` và `#![forbid(non_ascii_idents)]` được enforce ở workspace level
- Không có `unwrap()` nguy hiểm tại code path quan trọng — hầu hết dùng `?` operator với `MapResult` trait
- Clippy deny-list nghiêm ngặt ở workspace level giúp phát hiện nhiều anti-pattern

### 1.2 Async Architecture
- Tokio multi-threaded runtime được sử dụng đúng cách
- Background jobs chạy trong dedicated OS thread tách biệt khỏi HTTP handler pool
- `DashMap` cho WebSocket session state: lock-free concurrent access, hiệu năng cao

### 1.3 Type Safety
- Newtype wrappers cho các ID (`UserId`, `CipherId`, `OrganizationId`, v.v.) ngăn nhầm lẫn kiểu
- Enum cho device types, membership roles, token types
- Diesel ORM compile-time SQL query validation

---

## 2. Điểm yếu & Giới hạn Kỹ Thuật

### 2.1 Macro Hell trong Config System

**File**: [src/config.rs:58-100](src/config.rs#L58)

```rust
macro_rules! make_config {
    // 100+ dòng macro expansion
}
```

**Vấn đề**:
- `make_config!` macro tạo ra toàn bộ config struct, builder, deserializer, display logic
- Code được generate không visible trong IDE — không có autocomplete, không có "go to definition"
- Rất khó debug khi config validation fail (error messages từ macro expansion khó đọc)
- Thêm config field mới yêu cầu hiểu macro DSL phức tạp, không thân thiện với contributor mới
- Không có typed validation cho config values (ví dụ: URL format, email format) — chỉ validate ở runtime

**Đề xuất**: Chuyển sang `serde` + custom `Deserialize` implementation + `validator` crate. Hoặc ít nhất document macro DSL rõ ràng.

---

### 2.2 Global State qua LazyLock

**File**: [src/auth.rs:35-48](src/auth.rs#L35), [src/ratelimit.rs:9-19](src/ratelimit.rs#L9)

```rust
static JWT_LOGIN_ISSUER: LazyLock<String> = LazyLock::new(|| format!("{}|login", CONFIG.domain_origin()));
static LIMITER_LOGIN: LazyLock<Limiter> = LazyLock::new(|| { ... });
```

**Vấn đề**:
- Toàn bộ cấu hình được đọc một lần vào static LazyLock, không thể reload mà không restart
- `CONFIG` global là một `LazyLock<Config>` — không có hot-reload config support
- Khó test unit: tests cần mock CONFIG nhưng global state không thể swap
- Rate limiter state (`LIMITER_LOGIN`) là global static — không thể reset giữa các test cases

**Đề xuất**: Dependency injection pattern; truyền `Arc<Config>` qua Rocket state management thay vì global static.

---

### 2.3 Blocking Code trong Async Context

**File**: [src/config.rs:37-54](src/config.rs#L37)

```rust
pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap_or_else(...);
        rt.block_on(Config::load())...
    }).join()...
});
```

**Vấn đề**:
- Config loading spawn một OS thread riêng để chạy Tokio runtime — không cần thiết phức tạp
- Pattern này nguy hiểm: nếu được gọi trong async context, sẽ block executor thread
- `std::thread::spawn` + `block_on` trong LazyLock là workaround cho "chicken-and-egg" problem nhưng tạo ra overhead khởi động

---

### 2.4 Diesel với Ba Database Backend — Feature Flag Complexity

**File**: [src/db/mod.rs](src/db/mod.rs)

**Vấn đề**:
- Ba backend (SQLite, PostgreSQL, MySQL) được compile với `#[cfg(sqlite)]`, `#[cfg(postgresql)]`, `#[cfg(mysql)]`
- Mỗi query phải được viết ba lần trong nhiều trường hợp (macro expansion hoặc feature-gated blocks)
- MySQL bị pinned ở `diesel 2.3.3` vì compatibility issues — technical debt rõ ràng
- Không thể test toàn bộ ba backends trong một test run
- Schema migration phải được maintain ở ba thư mục riêng biệt: `migrations/sqlite/`, `migrations/postgresql/`, `migrations/mysql/`

**Đề xuất**: Xem xét chuyển sang `sqlx` với query macros — supports cả ba backend với unified API và compile-time query checking.

---

### 2.5 Error Handling: `err!` Macro Che Giấu Stack Trace

**File**: [src/error.rs](src/error.rs)

```rust
macro_rules! err {
    ($msg:expr) => { return Err(Error::new($msg, $msg)) };
}
```

**Vấn đề**:
- `err!` macro tạo `Error` nhưng không capture stack trace (không dùng `anyhow` hoặc `thiserror`)
- Tất cả errors đều có cùng type `Error`, không có rich error hierarchy
- Khó distinguish giữa user-facing errors và internal errors trong logging
- `err_code!` macro mix HTTP status codes vào error type — coupling giữa business logic và HTTP layer

---

### 2.6 RSA Key Generation tại Runtime với OpenSSL

**File**: [src/auth.rs:74-76](src/auth.rs#L74)

```rust
let rsa_key = Rsa::generate(2048)?;
let priv_key_buffer = rsa_key.private_key_to_pem()?;
operator.write(&rsa_key_filename, priv_key_buffer.clone()).await?;
```

**Vấn đề**:
- Private key được write vào storage dưới dạng **unencrypted PEM**
- Dùng `openssl` crate (C FFI) cho RSA generation trong khi codebase đã có `ring` crate (pure Rust)
- Không có key rotation — key tạo một lần và dùng vĩnh viễn
- RSA-2048 key generation chậm (~200ms) nhưng chỉ xảy ra một lần

**Đề xuất**: Chuyển sang `ring::signature::Ed25519KeyPair` hoặc `ring::signature::EcdsaKeyPair` (EdDSA/ECDSA) để thống nhất crypto backend và cải thiện performance.

---

### 2.7 Job Scheduler — Single-Threaded, Không Có Error Recovery

**File**: [src/main.rs](src/main.rs) (job scheduler setup)

**Vấn đề**:
- `job_scheduler_ng` chạy trong một OS thread duy nhất
- Nếu một job panic, toàn bộ scheduler thread crash — không có automatic restart
- Không có job execution monitoring, alerting khi job fail
- Jobs không idempotent đảm bảo: nếu server restart giữa chừng, job có thể chạy dở

---

### 2.8 WebSocket State — Memory Leak Potential

**File**: [src/api/notifications.rs:22-32](src/api/notifications.rs#L22)

```rust
pub static WS_USERS: LazyLock<Arc<WebSocketUsers>> = LazyLock::new(|| {
    Arc::new(WebSocketUsers {
        map: Arc::new(dashmap::DashMap::new()),
    })
});
```

**Vấn đề**:
- `DashMap<UserId, Vec<(uuid, Sender<Message>)>>` không có TTL hoặc max-entries limit
- Nếu nhiều users connect rồi disconnect nhanh, entries trong map có thể tăng trưởng không giới hạn trước khi `Drop` được gọi
- Không có monitoring cho số lượng active WebSocket connections
- `WS_ANONYMOUS_SUBSCRIPTIONS` cho anonymous connections cũng không có giới hạn

---

### 2.9 Regex Compilation tại Runtime

**File**: [src/api/icons.rs:79](src/api/icons.rs#L79), [src/http_client.rs:85-98](src/http_client.rs#L85)

```rust
static ICON_SIZE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?x)(\d+)\D*(\d+)").unwrap());

// Trong http_client.rs — regex được re-compile khi config thay đổi:
static COMPILED_REGEX: Mutex<Option<(String, Regex)>> = Mutex::new(None);
```

**Vấn đề**:
- `http_request_block_regex` được compile lại mỗi khi config thay đổi, dùng `Mutex<Option<(String, Regex)>>`
- Pattern này tạo lock contention trên hot path (mỗi icon request)
- `unwrap()` trên Regex::new trong LazyLock — nếu pattern invalid, server panic khi khởi động thay vì fail gracefully

---

### 2.10 Test Coverage — Không Có Integration Tests

**Vấn đề**:
- Codebase không có visible integration test suite (không thấy `tests/` directory)
- Unit tests ở mức module rất ít
- Không có test fixtures hoặc test database setup
- Việc test toàn bộ auth flow yêu cầu running instance — không thể test offline

**Đề xuất**: Thêm integration tests sử dụng `testcontainers` hoặc `sqlx::test` macros.

---

## 3. Technical Debt Đáng Chú Ý

| ID | Mô tả | File | Mức độ |
|----|-------|------|--------|
| TD-01 | MySQL pinned tại Diesel 2.3.3 | Cargo.toml | Cao |
| TD-02 | `openssl` + `ring` cùng tồn tại | auth.rs, crypto.rs | Trung bình |
| TD-03 | Config macro DSL khó maintain | config.rs | Cao |
| TD-04 | Không có hot-reload config | config.rs | Trung bình |
| TD-05 | Không có structured logging (JSON) | util.rs | Thấp |
| TD-06 | `panic!` trong `encode_jwt` | auth.rs:96 | Cao |
| TD-07 | Bitwarden Legacy Duo iframe flow | two_factor/duo.rs | Thấp |
| TD-08 | Ba migration directories riêng biệt | migrations/ | Trung bình |

### TD-06: Panic trong Production Code Path

**File**: [src/auth.rs:94-97](src/auth.rs#L94)

```rust
pub fn encode_jwt<T: Serialize>(claims: &T) -> String {
    match jsonwebtoken::encode(&JWT_HEADER, claims, PRIVATE_RSA_KEY.wait()) {
        Ok(token) => token,
        Err(e) => panic!("Error encoding jwt {e}"),  // ← PANIC trong production!
    }
}
```

**Vấn đề**: `encode_jwt` **panic** thay vì trả về `Result`. Nếu JWT encoding fail (lý thuyết là không thể nhưng...), toàn bộ Rocket worker thread crash. Nên trả về `Result<String, Error>`.

---

## 4. Dependency Risk Assessment

| Crate | Version | Rủi ro |
|-------|---------|--------|
| `diesel` | 2.3.3 (MySQL pinned) | Cao — không thể upgrade MySQL backend |
| `rocket` | 0.5.1 | Trung bình — còn active nhưng slow release cycle |
| `openidconnect` | 4.0.1 | Thấp |
| `webauthn-rs` | 0.5.3 | Trung bình — deprecated API trong 0.5.x, đang migrate lên 0.6 |
| `job_scheduler_ng` | 2.4 | Cao — ít maintainer, không phổ biến |
| `yubico_ng` | 0.14 | Cao — fork của `yubico`, ít maintenance |
| `openssl` | 0.10 | Trung bình — C FFI, dependency phức tạp |

---

## 5. Khuyến Nghị Ưu Tiên

### Ngắn hạn (< 1 tháng)
1. **Fix `panic!` trong `encode_jwt`** — đổi sang `Result` return type
2. **Loại bỏ query param JWT auth** cho WebSocket — chỉ dùng Authorization header
3. **Document config macro DSL** với ví dụ rõ ràng

### Trung hạn (1–3 tháng)
4. **Migrate từ openssl sang ring** cho RSA operations để giảm C FFI dependency
5. **Thêm per-account rate limiting** bên cạnh per-IP
6. **Viết integration test suite** với testcontainers

### Dài hạn (3–6 tháng)
7. **Xem xét migrate sang `sqlx`** để thống nhất database backend
8. **Dependency injection cho Config** — loại bỏ global static, dễ test hơn
9. **Triển khai key rotation** cho RSA JWT signing keys
10. **Structured logging** (JSON format) để tích hợp với log aggregation systems

---

*End of Document*
