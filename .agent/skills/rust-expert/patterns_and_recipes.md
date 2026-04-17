# Patterns & Recipes — Vaultwarden Project

## 1. Thêm Model Mới (End-to-End)

### Bước 1: Tạo migration
```sql
-- migrations/2026-04-16-000000_create_access_schedule/up.sql
CREATE TABLE IF NOT EXISTS access_schedules (
    uuid            TEXT    NOT NULL PRIMARY KEY,
    org_uuid        TEXT    NOT NULL REFERENCES organizations(uuid) ON DELETE CASCADE,
    name            TEXT    NOT NULL,
    schedule_cron   TEXT    NOT NULL,
    timezone        TEXT    NOT NULL DEFAULT 'UTC',
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_access_schedules_org ON access_schedules(org_uuid);
```

```sql
-- migrations/2026-04-16-000000_create_access_schedule/down.sql
DROP TABLE IF EXISTS access_schedules;
```

### Bước 2: Update schema.rs
```bash
# Schema auto-generated — chạy diesel print-schema sau migration
diesel print-schema > src/db/schema.rs
```

### Bước 3: Tạo Model
```rust
// src/db/models/access_schedule.rs
use crate::db::schema::access_schedules;
use crate::db::DbConn;
use crate::error::Error;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

db_object! {
    #[derive(Identifiable, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
    #[diesel(table_name = access_schedules)]
    pub struct AccessSchedule {
        pub uuid: String,
        pub org_uuid: String,
        pub name: String,
        pub schedule_cron: String,
        pub timezone: String,
        pub enabled: bool,
        pub created_at: NaiveDateTime,
    }
}

impl AccessSchedule {
    pub fn new(org_uuid: String, name: String, schedule_cron: String) -> Self {
        Self {
            uuid: crate::util::format_uuid(&uuid::Uuid::new_v4()),
            org_uuid,
            name,
            schedule_cron,
            timezone: "UTC".to_string(),
            enabled: true,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
    
    // CRUD Operations
    pub async fn save(&self, conn: &mut DbConn) -> crate::error::EmptyResult {
        db_run! { conn:
            diesel::replace_into(access_schedules::table)
                .values(self)
                .execute(conn)
                .map_res("Error saving AccessSchedule")
        }
    }
    
    pub async fn delete(self, conn: &mut DbConn) -> crate::error::EmptyResult {
        db_run! { conn:
            diesel::delete(
                access_schedules::table.filter(access_schedules::uuid.eq(&self.uuid))
            )
            .execute(conn)
            .map_res("Error deleting AccessSchedule")
        }
    }
    
    pub async fn find_by_uuid(uuid: &str, conn: &mut DbConn) -> Result<Self, Error> {
        db_run! { conn:
            access_schedules::table
                .filter(access_schedules::uuid.eq(uuid))
                .first::<AccessScheduleDb>(conn)
                .map_res("AccessSchedule not found")
        }
    }
    
    pub async fn find_by_org(org_uuid: &str, conn: &mut DbConn) -> Result<Vec<Self>, Error> {
        db_run! { conn:
            access_schedules::table
                .filter(access_schedules::org_uuid.eq(org_uuid))
                .order(access_schedules::created_at.desc())
                .load::<AccessScheduleDb>(conn)
                .map_res("Error loading AccessSchedules")
        }
    }
    
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "Id": self.uuid,
            "OrgId": self.org_uuid,
            "Name": self.name,
            "ScheduleCron": self.schedule_cron,
            "Timezone": self.timezone,
            "Enabled": self.enabled,
            "CreatedAt": self.created_at,
            "Object": "accessSchedule",
        })
    }
}
```

### Bước 4: Register trong db/models/mod.rs
```rust
// src/db/models/mod.rs
pub mod access_schedule;
pub use access_schedule::AccessSchedule;
```

### Bước 5: API Endpoints
```rust
// src/api/core/organizations.rs
use crate::db::models::AccessSchedule;

#[get("/organizations/<org_id>/access-schedules")]
async fn list_schedules(
    org_id: &str,
    auth: UserAuth,
    mut conn: DbConn,
) -> JsonResult {
    let schedules = AccessSchedule::find_by_org(org_id, &mut conn).await?;
    let json: Vec<_> = schedules.iter().map(|s| s.to_json()).collect();
    Ok(Json(json!({ "Data": json, "Object": "list", "ContinuationToken": null })))
}

#[post("/organizations/<org_id>/access-schedules", data = "<data>")]
async fn create_schedule(
    org_id: &str,
    data: Json<CreateScheduleData>,
    auth: AdminAuth,
    mut conn: DbConn,
) -> JsonResult {
    let schedule = AccessSchedule::new(
        org_id.to_string(),
        data.name.clone(),
        data.schedule_cron.clone(),
    );
    schedule.save(&mut conn).await?;
    Ok(Json(schedule.to_json()))
}

// Route registration
pub fn routes() -> Vec<Route> {
    routes![list_schedules, create_schedule]
}
```

---

## 2. Request Guard Pattern

```rust
// Tạo custom request guard
pub struct TenantContext {
    pub tenant_uuid: String,
    pub org_uuid: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TenantContext {
    type Error = Error;
    
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Get tenant from header
        let tenant_id = match request.headers().get_one("X-Tenant-ID") {
            Some(id) => id,
            None => return Outcome::Error((Status::BadRequest, Error::BadRequest("Missing X-Tenant-ID".into()))),
        };
        
        // Validate & load from DB
        let conn = match DbConn::from_request(request).await {
            Outcome::Success(c) => c,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };
        
        // DB lookup...
        match load_tenant(tenant_id, &mut conn.into_inner()).await {
            Ok(ctx) => Outcome::Success(ctx),
            Err(e) => Outcome::Error((Status::Unauthorized, e)),
        }
    }
}
```

---

## 3. Background Job với tokio-cron-scheduler

```rust
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn setup_scheduler() -> Result<JobScheduler, Error> {
    let scheduler = JobScheduler::new().await
        .map_err(|e| Error::Internal(format!("Scheduler init: {e}")))?;
    
    // Cleanup expired sessions mỗi giờ
    scheduler.add(
        Job::new_async("0 0 * * * *", |_, _| Box::pin(async {
            if let Err(e) = cleanup_expired_sessions().await {
                error!("Session cleanup failed: {e:?}");
            }
        })).map_err(|e| Error::Internal(format!("Job creation: {e}")))?
    ).await.map_err(|e| Error::Internal(format!("Job add: {e}")))?;
    
    // Hourly backup nếu được config
    if CONFIG.backup_enabled() {
        let cron = CONFIG.backup_cron_schedule();
        scheduler.add(
            Job::new_async(&cron, |_, _| Box::pin(async {
                if let Err(e) = run_backup().await {
                    error!("Backup failed: {e:?}");
                }
            })).map_err(|e| Error::Internal(format!("Backup job: {e}")))?
        ).await.map_err(|e| Error::Internal(format!("Backup job add: {e}")))?;
    }
    
    scheduler.start().await
        .map_err(|e| Error::Internal(format!("Scheduler start: {e}")))?;
    
    Ok(scheduler)
}
```

---

## 4. Config Field Pattern

```rust
// Trong src/config.rs — thêm config field mới
// Sử dụng macro make_config!

make_config! {
    // ... existing fields ...
    
    enterprise: {
        /// Maximum number of custom roles per organization
        _max_custom_roles: u32, true, def, 50;
        
        /// Enable IP allowlist enforcement
        _enable_ip_allowlist: bool, true, def, false;
        
        /// Access schedule enforcement mode (strict/permissive/disabled)
        _access_schedule_mode: String, true, def, "disabled";
    }
}

// Usage
if CONFIG.enable_ip_allowlist() {
    enforce_ip_rules(&request)?;
}
```

---

## 5. Webhook Delivery Pattern

```rust
// src/webhook_delivery.rs
use reqwest::Client;
use serde_json::Value;

pub async fn deliver_webhook(
    url: &str,
    payload: &Value,
    secret: Option<&str>,
) -> Result<(), Error> {
    let client = crate::http_client::make_http_client()?;
    
    let mut builder = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(payload);
    
    // HMAC signature se include nếu có secret
    if let Some(secret) = secret {
        let body_bytes = serde_json::to_vec(payload)?;
        let sig = crate::crypto::hmac_sign(secret.as_bytes(), &body_bytes);
        let sig_hex = data_encoding::HEXLOWER.encode(&sig);
        builder = builder.header("X-Vaultwarden-Signature", format!("sha256={sig_hex}"));
    }
    
    let response = builder.send().await
        .map_err(|e| Error::Internal(format!("Webhook delivery failed: {e}")))?;
    
    if !response.status().is_success() {
        warn!("Webhook delivery returned {}: {url}", response.status());
    }
    
    Ok(())
}
```

---

## 6. Cache Pattern (mini-moka)

```rust
use mini_moka::sync::Cache;
use std::time::Duration;

// Type definition
type UserCache = Cache<String, Arc<User>>;

// Initialize trong AppState
let user_cache: UserCache = Cache::builder()
    .max_capacity(1000)
    .time_to_live(Duration::from_secs(300))  // 5 min TTL
    .build();

// Usage
pub async fn get_cached_user(
    uuid: &str,
    cache: &UserCache,
    conn: &mut DbConn,
) -> Result<Arc<User>, Error> {
    if let Some(user) = cache.get(uuid) {
        return Ok(user);
    }
    
    let user = User::find_by_uuid(uuid, conn).await?;
    let user = Arc::new(user);
    cache.insert(uuid.to_string(), Arc::clone(&user));
    Ok(user)
}

// Invalidate on update
pub async fn update_user(
    user: &User,
    cache: &UserCache,
    conn: &mut DbConn,
) -> Result<(), Error> {
    user.save(conn).await?;
    cache.invalidate(&user.uuid);  // Invalidate cache
    Ok(())
}
```

---

## 7. Multi-Database Query Helper

```rust
// Pattern để handle differences giữa SQLite, MySQL, PostgreSQL
// Sử dụng db_run! macro đã có trong project

// SQLite-specific: strftime
#[cfg(feature = "sqlite")]
fn filter_by_date<'a>(query: BoxedQuery<'a, Sqlite>) -> BoxedQuery<'a, Sqlite> {
    query.filter(diesel::dsl::sql("strftime('%Y-%m-%d', created_at) = date('now')"))
}

// MySQL-specific: DATE()
#[cfg(feature = "mysql")]
fn filter_by_date<'a>(query: BoxedQuery<'a, Mysql>) -> BoxedQuery<'a, Mysql> {
    query.filter(diesel::dsl::sql("DATE(created_at) = CURDATE()"))
}

// Generic approach — prefer ISO 8601 for portability
pub async fn find_recent(conn: &mut DbConn) -> Result<Vec<Entity>, Error> {
    let cutoff = (Utc::now() - chrono::Duration::hours(24))
        .naive_utc()
        .to_string();
    
    db_run! { conn:
        table::table
            .filter(table::created_at.gt(cutoff))
            .load::<EntityDb>(conn)
            .map_res("Error querying recent entities")
    }
}
```

---

## 8. Error Context Macros

```rust
// Trait extension để add context to errors (inspired by anyhow)
pub trait ErrorContext<T> {
    fn context(self, msg: &str) -> Result<T, Error>;
}

impl<T, E: std::fmt::Debug> ErrorContext<T> for Result<T, E> {
    fn context(self, msg: &str) -> Result<T, Error> {
        self.map_err(|e| {
            error!("{msg}: {e:?}");
            Error::Internal(msg.to_string())
        })
    }
}

// Usage
let data = parse_json(raw)
    .context("Failed to parse webhook payload")?;

let user = User::find_by_uuid(&id, conn).await
    .context("User lookup failed during token refresh")?;
```
