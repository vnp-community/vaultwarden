# Vaultwarden Tech Stack — Cách Dùng Từng Crate

## 1. Rocket 0.5.x — Web Framework

### Route Definition
```rust
// Basic route
#[get("/users/<uuid>")]
async fn get_user(uuid: &str, auth: UserAuth, conn: DbConn) -> JsonResult {
    let user = User::find_by_uuid(uuid, &conn).await?;
    Ok(Json(user.to_json()))
}

// Route với query params
#[get("/users?<page>&<per_page>")]
async fn list_users(page: Option<i64>, per_page: Option<i64>) -> JsonResult { ... }

// POST với JSON body
#[post("/users", data = "<data>")]
async fn create_user(data: Json<CreateUserData>, auth: AdminAuth) -> JsonResult { ... }

// Mounting routes
pub fn routes() -> Vec<Route> {
    routes![get_user, list_users, create_user]
}
```

### State Management
```rust
// AppState — global shared state
// Defined in src/app_state.rs
use rocket::State;

#[get("/metrics")]
async fn metrics(state: &State<AppState>) -> String {
    state.metrics.render()
}

// DbConn — per-request DB connection
// Diesel r2d2 connection pool via Rocket fairing
#[get("/items")]
async fn list_items(mut conn: DbConn) -> JsonResult { ... }
```

### Request Guards
```rust
// Guards được chain tự động — order matters
#[get("/admin/users")]
async fn admin_list(
    _admin: AdminToken,    // Validates admin access
    _headers: Headers,    // Extracts common headers
    mut conn: DbConn,     // DB connection
) -> JsonResult { ... }
```

### Error Responses
```rust
// Sử dụng type alias JsonResult
pub type JsonResult = Result<Json<Value>, Error>;
pub type EmptyResult = Result<(), Error>;

// Error enum từ src/error.rs tự động convert sang HTTP response
Err(Error::NotFound)           // → 404
Err(Error::Unauthorized(msg))  // → 401
Err(Error::BadRequest(msg))    // → 400
Err(Error::Internal(msg))      // → 500
```

---

## 2. Diesel 2.3.x — ORM

### Kết Nối Multi-Database
Project hỗ trợ SQLite, MySQL, PostgreSQL qua feature flags:

```rust
// Connection type defined via macro in src/db/mod.rs
// Use DbConn type (abstracted) — không trực tiếp dùng diesel types

// Query pattern
use diesel::prelude::*;
use crate::db::schema::users;

pub async fn find_by_email(email: &str, conn: &mut DbConn) -> Result<User, Error> {
    db_run! { conn:
        users::table
            .filter(users::email.eq(email))
            .first::<User>(conn)
            .optional()?
            .ok_or(Error::NotFound)
    }
}
```

### Macro `db_run!`
Project dùng macro `db_run!` để abstract multi-DB:

```rust
// Pattern cơ bản
db_run! { conn:
    diesel_query_here
}

// Pattern với async
db_run! { conn: async {
    diesel_query_here
}}
```

### Migrations
```
migrations/
  ├── 2024-01-01-000000_create_users/
  │   ├── up.sql
  │   └── down.sql
```

```rust
// Chạy migrations khi startup — đã được handle trong main.rs
// Khi thêm model mới, tạo migration file với diesel CLI:
// diesel migration generate create_new_table
```

### Định Nghĩa Model (db_object! macro)
```rust
db_object! {
    #[derive(Identifiable, Queryable, Insertable, AsChangeset)]
    #[diesel(table_name = custom_roles)]
    pub struct CustomRole {
        pub uuid: String,         // Primary key — UUID v4
        pub org_uuid: String,     // Foreign key
        pub name: String,
        pub permissions: String,  // JSON serialized
        pub created_at: NaiveDateTime,
    }
}

impl CustomRole {
    pub fn new(org_uuid: String, name: String) -> Self {
        Self {
            uuid: crate::util::format_uuid(&uuid::Uuid::new_v4()),
            org_uuid,
            name,
            permissions: "[]".to_string(),
            created_at: Utc::now().naive_utc(),
        }
    }
    
    pub async fn save(&self, conn: &mut DbConn) -> EmptyResult {
        db_run! { conn:
            diesel::replace_into(custom_roles::table)
                .values(self)
                .execute(conn)
                .map_res("Error saving CustomRole")
        }
    }
    
    pub async fn find_by_uuid(uuid: &str, conn: &mut DbConn) -> Result<Self, Error> {
        db_run! { conn:
            custom_roles::table
                .filter(custom_roles::uuid.eq(uuid))
                .first::<CustomRoleDb>(conn)
                .map_res("Error finding CustomRole")
        }
    }
}
```

---

## 3. Tokio 1.x — Async Runtime

### Task Management
```rust
// Spawn detached background task
tokio::spawn(async move {
    if let Err(e) = send_email_notification(user_id).await {
        error!("Email notification failed: {e:?}");
    }
});

// Scheduled tasks via tokio-cron-scheduler
use tokio_cron_scheduler::{JobScheduler, Job};
let sched = JobScheduler::new().await?;
sched.add(Job::new_async("0 * * * * *", |_, _| Box::pin(async {
    cleanup_expired_sessions().await.ok();
}))?).await?;
sched.start().await?;
```

### Timeouts & Cancellation
```rust
use tokio::time::{timeout, Duration};

// Wrap operation with timeout
let result = timeout(Duration::from_secs(CONFIG.http_request_size_limit()), 
    fetch_remote_resource(url)
).await
.map_err(|_| Error::timeout("Request timed out"))?;
```

---

## 4. Serde — Serialization

```rust
// Standard derive cho API models
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // Match Bitwarden client expectations
pub struct UserData {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_factor_enabled: Option<bool>,
    #[serde(default)]
    pub premium_expiration: Option<DateTime<Utc>>,
}

// Custom JSON construction (dùng nhiều trong project)
fn to_json(&self) -> Value {
    json!({
        "Id": self.uuid,
        "Email": self.email,
        "TwoFactorEnabled": self.two_factor,
        "Object": "profile",
    })
}
```

---

## 5. OpenDAL 0.55 — File Storage (S3/Local)

```rust
use opendal::{Operator, services::Fs};

// Tạo operator
let op = Operator::new(Fs::default().root(&storage_path))?.finish();

// Write file
op.write(&filename, content_bytes).await?;

// Read file  
let data = op.read(&filename).await?;

// Delete
op.delete(&filename).await?;

// S3 khi enable feature "s3"
use opendal::services::S3;
let op = Operator::new(S3::default()
    .bucket(&bucket_name)
    .region(&region)
    // ...
)?.finish();
```

---

## 6. Redis (Optional Feature)

```rust
// Chỉ available khi compile với --features redis
#[cfg(feature = "redis")]
mod redis_cache {
    use deadpool_redis::{Config, Pool};
    
    pub async fn get_cached<T: DeserializeOwned>(
        pool: &Pool,
        key: &str,
    ) -> Option<T> {
        let mut conn = pool.get().await.ok()?;
        let value: String = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .ok()?;
        serde_json::from_str(&value).ok()
    }
}
```

---

## 7. Rate Limiting (governor)

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// Global rate limiter cho login endpoint
static LOGIN_LIMITER: Lazy<RateLimiter<...>> = Lazy::new(|| {
    RateLimiter::direct(Quota::per_second(NonZeroU32::new(10).unwrap()))
});
```
