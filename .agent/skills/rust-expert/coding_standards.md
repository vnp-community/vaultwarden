# Coding Standards — Vaultwarden Project

## 1. Lint Rules (bắt buộc 100%)

Project enforce strict lint rules qua `workspace.lints`. Vi phạm = build FAIL.

### Forbidden (compile error)
```toml
unsafe_code = "forbid"       # Tuyệt đối không unsafe
non_ascii_idents = "forbid"  # Tên biến chỉ ASCII
```

### Denied (phải fix)
```rust
// ❌ Unused variables
let x = compute(); // error nếu x không được dùng

// ❌ Trivial casts
let x = 42u32 as u32; // trivial_casts

// ❌ Unused imports
use std::collections::HashMap; // nếu không dùng

// ❌ Single-use lifetimes
fn foo<'a>(x: &'a str) -> &'a str { x } // single_use_lifetimes
// ✅ Correct:
fn foo(x: &str) -> &str { x }
```

### Clippy Denies quan trọng
```rust
// ❌ clone_on_ref_ptr — clone Arc/Rc đúng cách
config.clone()  // nếu config: Arc<Config>
Arc::clone(&config)  // ✅ correct

// ❌ implicit_clone — phải explicit
let s: String = some_str.to_string(); // khi some_str: String → implicit clone
let s = some_str.clone(); // ✅ explicit

// ❌ needless_borrow
fn foo(s: &String) {...} // ❌
fn foo(s: &str) {...}    // ✅

// ❌ redundant_clone
let x = val.clone(); // nếu val không cần clone
use val directly     // ✅

// ❌ mem_forget
std::mem::forget(resource); // BANNED — use Drop trait

// ❌ linkedlist
use std::collections::LinkedList; // BANNED — dùng Vec

// ❌ string_add_assign
s = s + "suffix"; // BANNED
s.push_str("suffix"); // ✅

// ❌ unnecessary_join
vec.join("") // khi chỉ có 1 element — dùng trực tiếp

// ❌ unused_async
async fn foo() { // nếu không có .await bên trong
    do_sync_thing();
}
fn foo() { // ✅ remove async
    do_sync_thing();
}
```

---

## 2. Naming Conventions

```rust
// Structs: PascalCase
pub struct UserAuth { ... }
pub struct CustomRole { ... }

// Functions & methods: snake_case
pub async fn find_by_uuid(...) -> Result<...> { ... }
pub fn to_json(&self) -> Value { ... }

// Constants: SCREAMING_SNAKE_CASE
const MAX_LOGIN_ATTEMPTS: u32 = 5;
static DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// Macros: snake_case!
format_uuid!()
db_run! {}

// Feature flags: lowercase with underscores
// Trong code: #[cfg(feature = "redis")]

// UUID fields: always named "uuid" (primary key) hoặc "<entity>_uuid" (FK)
pub uuid: String,               // primary key
pub org_uuid: String,           // foreign key to organizations
pub user_uuid: String,          // foreign key to users

// Timestamps: naive_datetime suffix khi cần clarity
pub created_at: NaiveDateTime,
pub updated_at: NaiveDateTime,
```

---

## 3. Module Organization

```
src/
├── main.rs          # Entry point, Rocket setup, route mounting
├── config.rs        # App config via make_config! macro
├── auth.rs          # JWT parsing, request guards
├── error.rs         # Error types, HTTP conversions
├── util.rs          # Utilities, helpers
├── crypto.rs        # Crypto primitives
├── mail.rs          # Email sending
├── http_client.rs   # Outbound HTTP (favicons, HIBP, etc.)
├── api/
│   ├── core/        # Core Bitwarden API
│   ├── admin.rs     # Admin panel API
│   └── web.rs       # Static file serving
└── db/
    ├── mod.rs       # DB connection pool, db_run! macro
    ├── schema.rs    # Diesel schema (auto-generated)
    └── models/      # Domain models
        ├── user.rs
        ├── org.rs
        └── ...
```

### Quy tắc khi thêm file mới
1. Tạo file trong thư mục phù hợp
2. Declare module trong `main.rs` hoặc parent `mod.rs`: `mod new_module;`
3. Re-export public items nếu cần

---

## 4. Documentation

```rust
/// Brief description (dùng `///` cho public items)
///
/// Longer explanation if needed.
///
/// # Arguments
/// * `id` - UUID of the entity to find
/// * `conn` - Active database connection
///
/// # Returns
/// The found entity or `Error::NotFound`
///
/// # Errors
/// Returns `Error::Internal` if the database query fails
pub async fn find_by_uuid(id: &str, conn: &mut DbConn) -> Result<Entity, Error> {
    ...
}

// Inline comments với // (không dùng /**/)
let result = complex_operation() // This bypasses cache intentionally
    .await?;
```

---

## 5. Testing Standards

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Unit test — pure logic
    #[test]
    fn test_token_parsing() {
        let header = "Bearer eyJhbGc...";
        let token = parse_bearer_token(header).unwrap();
        assert_eq!(token, "eyJhbGc...");
    }
    
    // Async test
    #[tokio::test]
    async fn test_user_creation() {
        let user = User::new("test@example.com".into());
        assert!(user.uuid.len() > 0);
        assert!(user.enabled);
    }
    
    // Test error cases
    #[test]
    fn test_invalid_input_returns_error() {
        let result = validate_email("not-an-email");
        assert!(result.is_err());
    }
}
```

---

## 6. Logging

```rust
use log::{debug, info, warn, error};

// Levels:
// debug! — Dev info, detailed tracing
// info!  — Normal operations (user login, etc.)
// warn!  — Unexpected but recoverable (rate limit, retry)
// error! — Failures that need attention

// NEVER log sensitive data
error!("Authentication failed for user {uuid}");  // ✅ UUID ok
error!("Password was: {password}");               // ❌ NEVER

// Structured logging với key=value
info!("User logged in user_uuid={uuid} device={device_name}");

// Error logging pattern
if let Err(e) = operation().await {
    error!("Failed to send email for user {uuid}: {e:?}");
    return Err(e.into());
}
```

---

## 7. Import Ordering

```rust
// 1. Std library
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crates (alphabetical)
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use rocket::{Route, State};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// 3. Internal crates
use crate::auth::UserAuth;
use crate::db::{models::User, DbConn};
use crate::error::Error;
```

---

## 8. Git Commit Template

```
<type>(<scope>): <short description>

<body>

<footer>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`  
Scopes: `auth`, `db`, `api`, `config`, `enterprise`, `security`

Example:
```
feat(enterprise): add CustomRole CRUD with access policy enforcement

- Add CustomRole and AccessSchedule models with Diesel migrations
- Implement time-based and IP-based access guard in access_control.rs
- Integrate guards into auth.rs request processing chain

Closes #SOL-004
```
