# Core Rust Skills — Vaultwarden Expert

## 1. Ownership, Borrowing & Lifetimes

### Quy tắc trong project này
- Ưu tiên `&str` trong function params, trả về `String` khi cần owned value
- Dùng `Cow<'_, str>` khi cần flexibility giữa borrowed và owned
- Lifetime annotations phải rõ ràng, tránh elision khi gây confusion

```rust
// ✅ Good — clear lifetime, borrows from request
pub fn parse_token<'t>(header: &'t str) -> Option<&'t str> {
    header.strip_prefix("Bearer ")
}

// ✅ Good — owned when storing
pub struct UserContext {
    pub email: String,
    pub device_id: String,
}
```

### Common Lifetime Pitfalls trong Rocket
```rust
// ❌ Bad — cannot return reference to local data
async fn handler() -> &str {
    let s = compute_string();
    &s  // ERROR: s does not live long enough
}

// ✅ Good — return owned or use wrapper types
async fn handler() -> String {
    compute_string()
}
```

---

## 2. Error Handling

### Pattern chuẩn của Vaultwarden
Project dùng `Error` enum tự định nghĩa trong `src/error.rs`. Không dùng `anyhow` trong core (chỉ trong S3 feature).

```rust
// Propagation với ?
pub async fn find_user(id: &str, conn: &mut DbConn) -> Result<User, Error> {
    User::find_by_id(id, conn).await.map_err(|_| Error::NotFound)
}

// Conversion từ external errors
impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => Error::NotFound,
            _ => Error::Internal(e.to_string()),
        }
    }
}

// Logging error context trước khi propagate
pub async fn complex_op() -> Result<(), Error> {
    something().map_err(|e| {
        error!("Failed to do X: {e:?}");
        e
    })?;
    Ok(())
}
```

### Không bao giờ
```rust
// ❌ NEVER in production code
let user = find_user(id).unwrap(); 
let val = option.expect("msg");

// ✅ Always handle explicitly
let user = find_user(id).await?;
let val = option.ok_or(Error::NotFound)?;
```

---

## 3. Async & Concurrency

### Tokio Runtime (multi-thread)
Project dùng `tokio::main` với multi-thread runtime:

```rust
// Spawn background task
tokio::spawn(async move {
    if let Err(e) = background_job().await {
        error!("Background job failed: {e:?}");
    }
});

// Timeout pattern
use tokio::time::{timeout, Duration};
let result = timeout(Duration::from_secs(30), operation()).await
    .map_err(|_| Error::Timeout)?;

// Select để cancel hoặc race
tokio::select! {
    result = operation() => handle(result),
    _ = shutdown_signal() => { /* graceful shutdown */ }
}
```

### Shared State Patterns
```rust
// DashMap (lock-free concurrent hashmap) — dùng cho caches
use dashmap::DashMap;
let cache: DashMap<String, CachedValue> = DashMap::new();
cache.insert(key, value);
if let Some(v) = cache.get(&key) { ... }

// ArcSwap (high-read, low-write) — dùng cho config
use arc_swap::ArcSwap;
static CONFIG: ArcSwap<Config> = ArcSwap::const_empty();
CONFIG.store(Arc::new(new_config));
let cfg = CONFIG.load();

// RwLock cho multi-reader, single-writer
use tokio::sync::RwLock;
let data: Arc<RwLock<HashMap<String, Value>>> = ...;
let read = data.read().await;
let mut write = data.write().await;
```

---

## 4. Pattern Matching & Iterators

### Idiomatic Rust trong Vaultwarden
```rust
// Iterator chaining — prefer over manual loops
let active_users: Vec<_> = users.iter()
    .filter(|u| u.enabled && !u.deleted)
    .map(|u| u.to_json())
    .collect();

// ? trong closures khi dùng với iterators
use std::result::Result as StdResult;
let validated: StdResult<Vec<_>, _> = items
    .iter()
    .map(|item| validate(item))
    .collect();

// Pattern matching với enums
match auth_result {
    AuthResult::Success(user) => handle_success(user),
    AuthResult::TwoFactorRequired => require_2fa(),
    AuthResult::Failed(reason) => {
        warn!("Auth failed: {reason}");
        Err(Error::Unauthorized)
    }
}
```

---

## 5. Traits & Generics

```rust
// Request Guard pattern (Rocket-specific)
#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuth {
    type Error = Error;
    
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match validate_token(request).await {
            Ok(user) => Outcome::Success(user),
            Err(e) => Outcome::Error((Status::Unauthorized, e)),
        }
    }
}

// Generic database function
pub async fn find_by_id<T: Queryable<DefaultLoadingMode> + 'static>(
    id: &str,
    conn: &mut DbConn,
) -> Result<T, Error> {
    ...
}
```

---

## 6. Macros của Project

Project có crate `macros/` riêng với các macro:

```rust
// Database CRUD macros — sử dụng khi define model
db_object! {
    #[derive(Identifiable, Queryable, Insertable, AsChangeset)]
    #[diesel(table_name = users)]
    pub struct User {
        pub uuid: String,
        pub email: String,
        // ...
    }
}

// Config field macro
make_config! {
    config: {
        /// Description
        _field_name: String, true, def, "default_value";
    }
}
```
