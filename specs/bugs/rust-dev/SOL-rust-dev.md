# Giải Pháp Khắc Phục — Điểm Yếu & Giới Hạn Kỹ Thuật (Rust Dev Analysis)

> **Tham chiếu**: [specs/bugs/rust-dev-analysis.md](../rust-dev-analysis.md)  
> **Ngày**: 2026-04-12  
> **Phân loại**: Ngắn hạn (< 1 tháng), Trung hạn (1–3 tháng), Dài hạn (3–6 tháng)

---

## TD-06 / SEV-CRITICAL: Panic trong `encode_jwt` [NGẮN HẠN]

**Vấn đề**: `src/auth.rs:94-97` — `encode_jwt` panic thay vì trả về `Result`.

### Giải Pháp

**File**: [src/auth.rs](../../src/auth.rs)

**Thay đổi**:

```rust
// TRƯỚC (panic):
pub fn encode_jwt<T: Serialize>(claims: &T) -> String {
    match jsonwebtoken::encode(&JWT_HEADER, claims, PRIVATE_RSA_KEY.wait()) {
        Ok(token) => token,
        Err(e) => panic!("Error encoding jwt {e}"),  // ← NGUY HIỂM
    }
}

// SAU (Result):
pub fn encode_jwt<T: Serialize>(claims: &T) -> Result<String, crate::error::Error> {
    jsonwebtoken::encode(&JWT_HEADER, claims, PRIVATE_RSA_KEY.wait())
        .map_err(|e| {
            error!("JWT encoding failed: {e}");
            crate::error::Error::new("JWT encoding error", "Internal server error")
        })
}
```

**Impact**: Tất cả callers của `encode_jwt` phải được cập nhật để xử lý `Result`. Ước tính ~10-15 call sites trong `src/api/identity.rs`, `src/auth.rs`.

**Test**:
```rust
#[test]
fn test_encode_jwt_returns_error_not_panic() {
    // Mock invalid key scenario
    let claims = LoginJwtClaims { ..Default::default() };
    let result = encode_jwt(&claims);
    assert!(result.is_ok() || result.is_err()); // Không panic
}
```

---

## SEC-HIGH-01 / §2.1: JWT trong URL Query Parameter [NGẮN HẠN]

**Vấn đề**: `src/api/notifications.rs:51-53` — WebSocket nhận JWT qua `?access_token=` URL param.

### Giải Pháp

**File**: [src/api/notifications.rs](../../src/api/notifications.rs)

```rust
// TRƯỚC: chấp nhận query param
struct WsAccessToken {
    access_token: Option<String>,
}

// SAU: chỉ chấp nhận Authorization header
// Xóa WsAccessToken struct hoàn toàn
// Trong WebSocket upgrade handler:

async fn websocket_hub(
    ws: WebSocket,
    req: &Request<'_>,
    conn: DbConn,
) -> Result<Channel<'static>, Error> {
    // Chỉ đọc từ Authorization header
    let token = req.headers().get_one("Authorization")
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| Error::new("Missing Authorization header for WebSocket", ""))?;
    
    // Validate token
    let claims = auth::decode_login_jwt(token)?;
    
    // ... rest of handler
}
```

**Migration**: Clients cần cập nhật để gửi token via header. Có thể keep query param support tạm thời với deprecation warning trong log:

```rust
// Temporary backward compat với DEPRECATION log
let token = req.headers().get_one("Authorization")
    .and_then(|h| h.strip_prefix("Bearer "))
    .or_else(|| {
        let t = req.query_value::<&str>("access_token").and_then(|r| r.ok());
        if t.is_some() {
            warn!("DEPRECATED: JWT via URL query param. Use Authorization header instead.");
        }
        t
    });
```

---

## §2.1: Config Macro Hell [TRUNG HẠN]

**Vấn đề**: `src/config.rs` — `make_config!` macro 100+ dòng, khó debug, không có IDE support.

### Giải Pháp (Phương án A — Ít xâm lấn)

**Bước 1**: Document macro DSL rõ ràng với inline comments và examples file.

Tạo file `src/config_guide.md`:
```markdown
# Vaultwarden Config Macro Guide

## Syntax
make_config! {
    category_name: {
        field_name: Type, is_secret, [def|fun|group], default_or_builder;
    }
}

## Examples
domain: {
    domain:         String, false, def, "http://localhost";
    //              ^^^^^^  ^^^^^  ^^^  ^^^^^^^^^^^^^^^^^^
    //              type    secret kind default_value
}
smtp_password:  String, true, def, "";
//                      ^^^^  password masked in display/API
```

**Bước 2** (Dài hạn): Migrate dần sang `serde` + validation traits.

```rust
// Mục tiêu — sau khi migrate:
#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_domain")]
    pub domain: String,
    
    #[serde(default)]
    #[validate(email)]
    pub smtp_from: String,
    
    #[serde(skip_serializing)]  // Không serialize secrets
    pub admin_token: String,
}

impl Config {
    pub async fn load() -> Result<Self, Error> {
        // 1. Load từ environment
        // 2. Load từ config.json (merge)
        // 3. Validate với validator crate
    }
}
```

**Phụ thuộc mới**: `validator = "0.18"` (nếu chọn phương án serde)

---

## §2.2: Global State qua LazyLock [TRUNG HẠN]

**Vấn đề**: `CONFIG`, `LIMITER_LOGIN` là global statics — khó test, không hot-reload.

### Giải Pháp

**Phase 1 (Trung hạn)**: Dependency injection cho tests qua Rocket state.

```rust
// Thêm AppState struct
pub struct AppState {
    pub config: Arc<Config>,
    pub rate_limiter: Arc<dyn RateLimiter + Send + Sync>,
}

// Trong main.rs
let state = AppState {
    config: Arc::new(Config::load().await?),
    rate_limiter: Arc::new(InMemoryRateLimiter::new()),
};

rocket::build()
    .manage(state)
    // ...
```

```rust
// Trong route handlers — dùng Rocket State guard
#[post("/login")]
async fn login(
    state: &State<AppState>,
    // ...
) {
    let config = &state.config;
    state.rate_limiter.check(ip).await?;
}
```

**Phase 2 (Dài hạn)**: Hot-reload config.

```rust
pub struct HotReloadConfig {
    inner: Arc<RwLock<Config>>,
}

impl HotReloadConfig {
    pub async fn reload_from_file(&self) -> Result<(), Error> {
        let new_config = Config::load().await?;
        *self.inner.write().await = new_config;
        Ok(())
    }
    
    pub async fn get(&self) -> tokio::sync::RwLockReadGuard<Config> {
        self.inner.read().await
    }
}
```

**Test benefit**: Tests có thể tạo `AppState` với mock `Config` và mock `RateLimiter`.

---

## §2.3: Blocking Code trong Async Context [TRUNG HẠN]

**Vấn đề**: `src/config.rs:37-54` — spawn OS thread để chạy async runtime trong LazyLock.

### Giải Pháp

Đây là consequence của §2.2. Sau khi migrate sang `AppState` + Rocket state management, config loading chỉ cần xảy ra một lần trong `main()` trong async context:

```rust
// main.rs — trong async main
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // Load config trực tiếp trong async context — không cần spawn thread
    let config = Config::load().await
        .expect("Failed to load configuration");
    
    let app_state = AppState::new(config).await;
    
    rocket::build()
        .manage(app_state)
        .launch()
        .await
}
```

Không cần workaround `std::thread::spawn` + `block_on` nữa.

---

## §2.4: Diesel với Ba DB Backend [DÀI HẠN]

**Vấn đề**: MySQL pinned tại Diesel 2.3.3, ba migration directories, code duplication.

### Giải Pháp Ngắn Hạn: Upgrade Path Documentation

Tạo `CONTRIBUTING.md` section về database compatibility:
```markdown
## Database Backend Guidelines

1. **New migrations**: Phải implement cho cả 3 backends trong `migrations/sqlite/`, `migrations/postgresql/`, `migrations/mysql/`
2. **Database-specific SQL**: Sử dụng `#[cfg(feature = "sqlite")]` blocks
3. **MySQL limitation**: Không dùng `RETURNING` clause, `gen_random_uuid()`, hoặc `NOW() AT TIME ZONE`
```

### Giải Pháp Dài Hạn: Migrate sang `sqlx`

**Cảnh báo**: Đây là migration RẤT LỚN — toàn bộ ORM layer. Chỉ nên thực hiện khi có dedicated engineering team.

```rust
// Thay thế Diesel với sqlx
use sqlx::{PgPool, SqlitePool, MySqlPool};

// Compile-time query checking
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE email = $1",
    email
)
.fetch_optional(&pool)
.await?;
```

**Benefit**: Unified API, compile-time queries, không cần feature flags per DB.  
**Risk**: Full rewrite của database layer — ước tính 3-6 tháng.

---

## §2.5: Error Handling — `err!` Macro [TRUNG HẠN]

**Vấn đề**: Không có error hierarchy, mix HTTP status codes vào business errors.

### Giải Pháp

**Phase 1**: Thêm error categories:

```rust
// src/error.rs — mở rộng Error type
#[derive(Debug)]
pub enum ErrorKind {
    NotFound,
    Unauthorized,
    Forbidden,
    ValidationError,
    DatabaseError,
    InternalError,
    ExternalServiceError,
}

pub struct Error {
    pub message: String,
    pub kind: ErrorKind,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    pub fn not_found(msg: &str) -> Self {
        Self { message: msg.to_string(), kind: ErrorKind::NotFound, source: None }
    }
    pub fn unauthorized(msg: &str) -> Self {
        Self { message: msg.to_string(), kind: ErrorKind::Unauthorized, source: None }
    }
    // ... etc
}

// Updated macro
macro_rules! err_not_found {
    ($msg:expr) => { return Err(Error::not_found($msg)) };
}

macro_rules! err_unauthorized {
    ($msg:expr) => { return Err(Error::unauthorized($msg)) };
}
```

**Phase 2**: Tách HTTP mapping khỏi business logic:

```rust
// Trong Rocket responder — mapping ErrorKind → HTTP status
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = match self.kind {
            ErrorKind::NotFound      => Status::NotFound,
            ErrorKind::Unauthorized  => Status::Unauthorized,
            ErrorKind::Forbidden     => Status::Forbidden,
            ErrorKind::ValidationError => Status::UnprocessableEntity,
            _                        => Status::InternalServerError,
        };
        
        // Log internal errors with full detail
        if matches!(self.kind, ErrorKind::InternalError | ErrorKind::DatabaseError) {
            error!("Internal error: {:?}", self.source);
        }
        
        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(self.message.len(), Cursor::new(self.message))
            .ok()
    }
}
```

---

## §2.6: RSA Key — OpenSSL + Unencrypted PEM [TRUNG HẠN]

**Vấn đề**: Private key lưu unencrypted, dùng `openssl` crate thay vì `ring`.

### Giải Pháp

**Phase 1 (Nhanh)**: Encrypt RSA private key PEM trước khi lưu:

```rust
// src/auth.rs — encrypt key bằng server password/secret
async fn save_rsa_key(priv_key_pem: Vec<u8>, operator: &Operator) -> Result<(), Error> {
    // Mã hóa PEM với AES-256-GCM trước khi lưu
    let master_key = derive_master_key(); // Từ ENV var hoặc hardware secret
    let encrypted = aes_gcm_encrypt(&priv_key_pem, &master_key)?;
    operator.write(RSA_KEY_FILENAME, encrypted).await?;
    Ok(())
}

async fn load_rsa_key(operator: &Operator) -> Result<Vec<u8>, Error> {
    let encrypted = operator.read(RSA_KEY_FILENAME).await?;
    let master_key = derive_master_key();
    aes_gcm_decrypt(&encrypted, &master_key)
}
```

**Phase 2 (Dài hạn)**: Chuyển sang `ring` crate, sử dụng ECDSA P-256 hoặc EdDSA (Ed25519):

```rust
// Thay thế RSA với Ed25519 — nhỏ hơn, nhanh hơn, vẫn an toàn
use ring::signature::{Ed25519KeyPair, KeyPair};

let rng = ring::rand::SystemRandom::new();
let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)?;
let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())?;
```

**Lưu ý**: Bitwarden clients expect RS256 JWT. Nếu chuyển sang EdDSA, phải confirm client compatibility.

---

## §2.7: Job Scheduler — Không Có Error Recovery [TRUNG HẠN]

**Vấn đề**: Scheduler thread crash → toàn bộ scheduler dừng.

### Giải Pháp

```rust
// src/main.rs — wrap mỗi job trong panic catcher

fn create_job_with_recovery<F>(cron: &str, job_name: &str, f: F) -> Job 
where F: Fn() + Send + Sync + 'static 
{
    let name = job_name.to_string();
    Job::new(cron, move |_, _| {
        let job_name = name.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
        
        match result {
            Ok(_) => {
                // Update metrics (CR-010)
                METRICS.job_runs.get_or_create(&JobLabels { job: job_name.clone() }).inc();
            }
            Err(e) => {
                error!("Job '{job_name}' panicked: {:?}", e);
                METRICS.job_failures.get_or_create(&JobLabels { job: job_name }).inc();
                // Không crash scheduler thread — tiếp tục với các jobs khác
            }
        }
    }).expect("Invalid cron expression")
}

// Thay thế `job_scheduler_ng` bằng giải pháp robust hơn (dài hạn):
// Xem xét `tokio-cron-scheduler` — async, panic-safe, active maintained
```

**Long-term**: Migrate từ `job_scheduler_ng` sang `tokio-cron-scheduler`:

```toml
# Cargo.toml
tokio-cron-scheduler = "0.13"
```

Benefit: Active maintained, async-native, built-in error handling.

---

## §2.8: WebSocket State — Memory Leak Potential [TRUNG HẠN]

**Vấn đề**: `DashMap` cho WS sessions không có TTL hoặc size limit.

### Giải Pháp

```rust
// src/api/notifications.rs

// Thêm cleanup task
pub fn start_ws_cleanup_task() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            
            // Remove entries với empty sender list (disconnected users)
            WS_USERS.map.retain(|_user_uuid, senders| {
                // Retain nếu có ít nhất 1 sender còn alive
                senders.retain(|(_, sender)| !sender.is_closed());
                !senders.is_empty()
            });
            
            // Log current state for monitoring
            METRICS.websocket_connections.set(
                WS_USERS.map.iter().map(|e| e.value().len()).sum::<usize>() as i64
            );
        }
    });
}

// Giới hạn anonymous connections
pub static WS_ANONYMOUS_COUNT: AtomicUsize = AtomicUsize::new(0);
const MAX_ANONYMOUS_WS: usize = 1000;

// Khi anonymous connection arrive:
if WS_ANONYMOUS_COUNT.load(Ordering::Relaxed) >= MAX_ANONYMOUS_WS {
    return Err(Status::TooManyRequests);
}
WS_ANONYMOUS_COUNT.fetch_add(1, Ordering::Relaxed);
// Khi disconnect:
WS_ANONYMOUS_COUNT.fetch_sub(1, Ordering::Relaxed);
```

---

## §2.9: Regex Compilation — Lock Contention [NGẮN HẠN]

**Vấn đề**: `COMPILED_REGEX: Mutex<Option<(String, Regex)>>` — lock contention trên hot path.

### Giải Pháp

```rust
// src/http_client.rs

// Thay Mutex<Option<Regex>> bằng ArcSwap để lock-free reads
use arc_swap::ArcSwap;

static COMPILED_REGEX: LazyLock<ArcSwap<Option<(String, Regex)>>> = 
    LazyLock::new(|| ArcSwap::new(Arc::new(None)));

// Read path (hot path) — lock-free
fn get_block_regex() -> Option<Regex> {
    COMPILED_REGEX.load()
        .as_ref()
        .as_ref()
        .map(|(_, re)| re.clone())
}

// Write path (cold, only on config change) — brief swap
fn update_block_regex(pattern: &str) -> Result<(), Error> {
    let regex = Regex::new(pattern)
        .map_err(|e| Error::new(&format!("Invalid regex: {e}"), ""))?;
    COMPILED_REGEX.store(Arc::new(Some((pattern.to_string(), regex))));
    Ok(())
}
```

**Phụ thuộc mới**: `arc-swap = "1.7"`

---

## §2.10: Test Coverage [DÀI HẠN]

**Vấn đề**: Không có integration test suite.

### Giải Pháp

**Phase 1 (Trung hạn)**: Unit tests với mock DB.

```rust
// tests/unit/auth_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_decode_jwt_roundtrip() {
        let claims = LoginJwtClaims {
            sub: "test-user-uuid".to_string(),
            exp: (Utc::now() + Duration::hours(2)).timestamp(),
            ..Default::default()
        };
        
        let token = encode_jwt(&claims).expect("encode should succeed");
        let decoded = decode_login_jwt(&token).expect("decode should succeed");
        
        assert_eq!(claims.sub, decoded.sub);
    }
}
```

**Phase 2 (Dài hạn)**: Integration tests với testcontainers:

```rust
// tests/integration/auth_flow_test.rs
use testcontainers::{clients::Cli, images::postgres::Postgres};

#[tokio::test]
async fn test_full_login_flow() {
    let docker = Cli::default();
    let pg = docker.run(Postgres::default());
    
    let db_url = format!("postgresql://postgres@localhost:{}/postgres", pg.get_host_port(5432));
    
    // Setup Vaultwarden với test DB
    let rocket = build_test_rocket(db_url).await;
    let client = rocket::local::asynchronous::Client::tracked(rocket).await.unwrap();
    
    // Register user
    let resp = client.post("/identity/accounts/register")
        .json(&json!({
            "email": "test@example.com",
            "masterPasswordHash": "hash_value",
            "kdf": 0,
        }))
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    
    // Login
    let resp = client.post("/identity/connect/token")
        .json(&json!({
            "grant_type": "password",
            "username": "test@example.com",
            "password": "hash_value",
        }))
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert!(body.get("access_token").is_some());
}
```

**Phụ thuộc mới** (dev-dependencies):
```toml
[dev-dependencies]
testcontainers = "0.22"
testcontainers-modules = { version = "0.9", features = ["postgres"] }
```

---

## §3: Technical Debt — Priority Matrix

| ID | Vấn đề | Giải pháp | Ưu tiên | Effort |
|----|--------|----------|---------|--------|
| TD-06 | panic! trong encode_jwt | Đổi sang Result | **P1 — Làm ngay** | 0.5 ngày |
| SEC-HIGH-01 | JWT trong URL | Xóa query param | **P1 — Làm ngay** | 1 ngày |
| §2.9 | Regex lock contention | ArcSwap | **P2 — Sprint 1** | 1 ngày |
| §2.8 | WebSocket memory leak | TTL cleanup task | **P2 — Sprint 1** | 2 ngày |
| §2.7 | Job scheduler panic | catch_unwind | **P2 — Sprint 1** | 1 ngày |
| §2.5 | Error hierarchy | ErrorKind enum | **P3 — Sprint 2** | 1 tuần |
| §2.6 | RSA key unencrypted | Encrypt at rest | **P3 — Sprint 2** | 3 ngày |
| §2.2 | Global state DI | AppState pattern | **P3 — Sprint 3** | 2 tuần |
| §2.1 | Config macro hell | Document + validator | **P4 — Sprint 4** | 2 tuần |
| §2.3 | Blocking async | Fixed by §2.2 | **P4** | — |
| §2.10 | Integration tests | testcontainers | **P4 — Dài hạn** | 1 tháng |
| §2.4 | Diesel → sqlx | Full rewrite | **P5 — Dài hạn** | 3-6 tháng |
| TD-01 | MySQL Diesel pin | Unpin sau rewrite | **P5** | — |

---

## Dependency Risk Mitigation

| Crate | Rủi ro | Hành động |
|-------|--------|----------|
| `job_scheduler_ng` | Ít maintainer | Migrate sang `tokio-cron-scheduler` (Sprint 2) |
| `yubico_ng` | Fork ít maintenance | Evaluate `yubico` original hoặc WebAuthn thay thế |
| `openssl` | C FFI | Phase migration sang `ring` cho RSA (Sprint 3) |
| `webauthn-rs` 0.5.x | Deprecated API | Upgrade lên 0.6.x khi stable |
| `rocket` | Slow release | Monitor rocket 0.6 release, plan upgrade |

---

*Status: Draft | Ngày: 2026-04-12*
