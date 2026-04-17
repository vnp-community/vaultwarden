# SOL-005: Giải Pháp Thực Hiện — High Availability & Horizontal Scaling

> **Giải pháp cho**: CR-005  
> **Ngày**: 2026-04-12  
> **Trạng thái**: ✅ Implemented  
> **Kiến trúc thay đổi**: Đáng kể — thêm Redis layer, refactor in-memory state  
> **Cập nhật**: 2026-04-17 — Verified full implementation in codebase

---

## 1. Tổng Quan Giải Pháp

Kiến trúc hiện tại là single-instance với in-memory state. Để đạt HA cần:

1. **Redis abstraction layer**: `src/cache.rs` — unified interface (in-memory fallback khi Redis off)
2. **Migrate rate limiter** → Redis (thay `governor` in-memory state)
3. **Migrate OIDC cache** → Redis (thay `mini-moka` in-memory cache)  
4. **Migrate WebSocket events** → Redis Pub/Sub (thay `DashMap` local state)
5. **Health endpoint** mới tại `/health`
6. **Graceful shutdown** support
7. **PostgreSQL read replica** support

**Nguyên tắc**: Mọi thay đổi backward-compatible. Khi `REDIS_ENABLED=false` (default), behavior giống hệt v1.x.

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/cache.rs` | Cache abstraction: `CacheBackend` trait + Redis/InMemory implementations |
| `src/api/health.rs` | Health check endpoints (`/health`, `/health/ready`, `/health/live`, `/health/detailed`) |
| `src/db/read_replica.rs` | Read replica connection pool management |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/ratelimit.rs` | Đổi từ `governor` in-memory → `CacheBackend` |
| `src/api/notifications.rs` | Thêm Redis pub/sub bên cạnh local DashMap |
| `src/sso.rs` | Migrate `AC_CACHE` (mini-moka) → `CacheBackend` |
| `src/config.rs` | Thêm REDIS_*, CLUSTER_MODE, DATABASE_READ_URL config keys |
| `src/main.rs` | Khởi động Redis connection, graceful shutdown, health routes |
| `src/db/mod.rs` | Thêm read replica pool |

---

## 3. Thiết Kế Chi Tiết

### 3.1 Cache Abstraction Layer

**File**: `src/cache.rs`

```rust
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait CacheBackend: Send + Sync + 'static {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    async fn increment(&self, key: &str, ttl: Duration) -> Result<i64, CacheError>;
    async fn is_healthy(&self) -> bool;
}

// In-memory implementation (default, single-instance)
pub struct InMemoryCache {
    store: Arc<dashmap::DashMap<String, (String, Instant)>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self { store: Arc::new(dashmap::DashMap::new()) }
    }
    
    // Background cleanup task
    pub fn start_cleanup_task(&self) {
        let store = self.store.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = Instant::now();
                store.retain(|_, (_, expires)| *expires > now);
            }
        });
    }
}

#[async_trait]
impl CacheBackend for InMemoryCache {
    async fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).and_then(|entry| {
            let (value, expires) = entry.value();
            if Instant::now() < *expires {
                Some(value.clone())
            } else {
                None
            }
        })
    }
    
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        self.store.insert(
            key.to_string(),
            (value.to_string(), Instant::now() + ttl),
        );
        Ok(())
    }
    
    async fn increment(&self, key: &str, ttl: Duration) -> Result<i64, CacheError> {
        // Atomic-ish via DashMap entry API
        let count = self.store.entry(key.to_string())
            .and_modify(|(val, _)| {
                *val = (val.parse::<i64>().unwrap_or(0) + 1).to_string();
            })
            .or_insert((("1".to_string()), Instant::now() + ttl))
            .value().0.parse::<i64>().unwrap_or(1);
        Ok(count)
    }
    
    async fn is_healthy(&self) -> bool { true }
    
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.store.remove(key);
        Ok(())
    }
}

// Redis implementation (cluster-mode)
#[cfg(feature = "redis")]
pub struct RedisCache {
    pool: deadpool_redis::Pool,
    key_prefix: String,
}

#[cfg(feature = "redis")]
#[async_trait]
impl CacheBackend for RedisCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.pool.get().await.ok()?;
        let prefixed = format!("{}{}", self.key_prefix, key);
        deadpool_redis::redis::cmd("GET")
            .arg(&prefixed)
            .query_async::<_, Option<String>>(&mut conn)
            .await.ok().flatten()
    }
    
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        let mut conn = self.pool.get().await.map_err(|e| CacheError::Connection(e.to_string()))?;
        let prefixed = format!("{}{}", self.key_prefix, key);
        deadpool_redis::redis::cmd("SET")
            .arg(&prefixed)
            .arg(value)
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async::<_, ()>(&mut conn)
            .await.map_err(|e| CacheError::Operation(e.to_string()))
    }
    
    async fn increment(&self, key: &str, ttl: Duration) -> Result<i64, CacheError> {
        let mut conn = self.pool.get().await.map_err(|e| CacheError::Connection(e.to_string()))?;
        let prefixed = format!("{}{}", self.key_prefix, key);
        // Atomic INCR + EXPIRE
        let count: i64 = deadpool_redis::redis::pipe()
            .cmd("INCR").arg(&prefixed)
            .cmd("EXPIRE").arg(&prefixed).arg(ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map(|(c, _): (i64, i64)| c)
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        Ok(count)
    }
    
    async fn is_healthy(&self) -> bool {
        if let Ok(mut conn) = self.pool.get().await {
            deadpool_redis::redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await.map(|s| s == "PONG").unwrap_or(false)
        } else {
            false
        }
    }
    
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.pool.get().await.map_err(|e| CacheError::Connection(e.to_string()))?;
        let prefixed = format!("{}{}", self.key_prefix, key);
        deadpool_redis::redis::cmd("DEL")
            .arg(&prefixed)
            .query_async::<_, ()>(&mut conn)
            .await.map_err(|e| CacheError::Operation(e.to_string()))
    }
}

// Global cache instance (initialized in main.rs)
pub static CACHE: LazyLock<Arc<dyn CacheBackend>> = LazyLock::new(|| {
    if CONFIG.redis_enabled() {
        #[cfg(feature = "redis")]
        return Arc::new(RedisCache::from_config());
    }
    Arc::new(InMemoryCache::new())
});
```

### 3.2 Rate Limiter Migration

**File**: `src/ratelimit.rs` — thay thế `governor` với cache-backed counter:

```rust
pub async fn check_rate_limit(
    limiter_key: &str,   // e.g. "login", "admin"
    identifier: &str,    // IP address
    max_requests: u32,
    window_secs: u64,
) -> Result<(), Error> {
    if !CONFIG.ratelimit_enabled() { return Ok(()); }
    
    let cache_key = format!("rl:{}:{}", limiter_key, identifier);
    let count = CACHE.increment(&cache_key, Duration::from_secs(window_secs)).await
        .unwrap_or(1);
    
    if count as u32 > max_requests {
        audit::emit(AuditEntry {
            event_type: AuditEventType::RateLimitTriggered { endpoint: limiter_key.to_string() },
            ip_address: identifier.parse().ok(),
            ..Default::default()
        });
        err!(format!("Rate limit exceeded for {limiter_key}"));
    }
    
    Ok(())
}
```

> **Note**: Giữ lại `governor` cho in-memory mode nếu muốn backward-compatible performance. Chuyển sang cache-backed chỉ khi `CLUSTER_MODE=true`.

### 3.3 WebSocket Redis Pub/Sub

**File**: `src/api/notifications.rs` — thêm Redis pub/sub khi cluster mode:

```rust
pub static WS_USERS: LazyLock<Arc<WebSocketUsers>> = LazyLock::new(|| {
    Arc::new(WebSocketUsers {
        map: Arc::new(dashmap::DashMap::new()),
    })
});

// Hàm gửi event đến user — check cả local map VÀ Redis pub/sub
pub async fn send_notification_to_user(user_uuid: &str, data: &Value) {
    // 1. Gửi đến local WebSocket connections trực tiếp
    if let Some(senders) = WS_USERS.map.get(user_uuid) {
        for (_, sender) in senders.iter() {
            let _ = sender.send(Message::text(data.to_string()));
        }
    }
    
    // 2. Publish lên Redis (nếu cluster mode) để các instances khác forward
    if CONFIG.cluster_mode() {
        #[cfg(feature = "redis")]
        if let Ok(mut conn) = get_redis_pubsub_conn().await {
            let channel = format!("{}ws:user:{}", CONFIG.redis_key_prefix(), user_uuid);
            let payload = serde_json::to_string(data).unwrap_or_default();
            deadpool_redis::redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(&payload)
                .query_async::<_, i64>(&mut conn)
                .await.ok();
        }
    }
}

// Background task: subscribe Redis và forward events đến local WS connections
#[cfg(feature = "redis")]
pub async fn redis_pubsub_listener(redis_url: &str) {
    let client = deadpool_redis::redis::Client::open(redis_url).expect("Redis client");
    let mut pubsub = client.get_async_pubsub().await.expect("Redis pubsub");
    
    // Subscribe to all user channels for this instance
    pubsub.psubscribe(format!("{}ws:user:*", CONFIG.redis_key_prefix())).await.ok();
    
    let mut stream = pubsub.into_on_message();
    while let Some(msg) = stream.next().await {
        let channel: String = msg.get_channel_name().to_string();
        let user_uuid = channel.split(':').last().unwrap_or("");
        
        if let Ok(data) = msg.get_payload::<String>() {
            // Gửi đến local connections ONLY (không re-publish)
            if let Some(senders) = WS_USERS.map.get(user_uuid) {
                for (_, sender) in senders.iter() {
                    let _ = sender.send(Message::text(&data));
                }
            }
        }
    }
}
```

### 3.4 OIDC Cache Migration

**File**: `src/sso.rs` — thay `mini-moka` bằng `CACHE`:

```rust
// Trước: 
// static AC_CACHE: LazyLock<Cache<OIDCState, AuthenticatedUser>> = ...;

// Sau: dùng CACHE abstraction
async fn cache_set_oidc_state(state: &OIDCState, user: &AuthenticatedUser) {
    let key = format!("oidc:{}", state.as_str());
    let value = serde_json::to_string(user).unwrap_or_default();
    CACHE.set(&key, &value, Duration::from_secs(600)).await.ok();
}

async fn cache_get_oidc_state(state: &OIDCState) -> Option<AuthenticatedUser> {
    let key = format!("oidc:{}", state.as_str());
    CACHE.get(&key).await.and_then(|v| serde_json::from_str(&v).ok())
}
```

### 3.5 Health Check Endpoints

**File**: `src/api/health.rs`

```rust
#[get("/health")]
pub async fn health_simple(conn: DbConn) -> Result<Json<Value>, Status> {
    match conn.run(|c| c.execute("SELECT 1", [])).await {
        Ok(_) => Ok(Json(json!({"status": "ok"}))),
        Err(_) => Err(Status::ServiceUnavailable),
    }
}

#[get("/health/ready")]
pub async fn health_ready(conn: DbConn) -> Result<Json<Value>, Status> {
    // Kubernetes readiness: DB must be reachable
    health_simple(conn).await
}

#[get("/health/live")]
pub fn health_live() -> Json<Value> {
    // Kubernetes liveness: process is running
    Json(json!({"status": "ok"}))
}

#[get("/health/detailed")]
pub async fn health_detailed(
    _auth: Option<AdminHeaders>,
    conn: DbConn,
) -> Json<Value> {
    let db_ok = conn.run(|c| c.execute("SELECT 1", [])).await.is_ok();
    let redis_ok = CACHE.is_healthy().await;
    
    let overall = if db_ok { 
        if redis_ok || !CONFIG.redis_enabled() { "healthy" } else { "degraded" }
    } else { 
        "unhealthy" 
    };
    
    Json(json!({
        "status": overall,
        "version": env!("CARGO_PKG_VERSION"),
        "checks": {
            "database": {"status": if db_ok { "healthy" } else { "unhealthy" }},
            "redis": {
                "status": if !CONFIG.redis_enabled() { "disabled" }
                          else if redis_ok { "healthy" } else { "unhealthy" },
                "enabled": CONFIG.redis_enabled(),
            },
            "instance_id": CONFIG.instance_id(),
        }
    }))
}
```

### 3.6 Graceful Shutdown

**File**: `src/main.rs`:

```rust
// Rocket shutdown config
let figment = rocket::Config::figment()
    .merge(("shutdown.ctrlc", true))
    .merge(("shutdown.grace", CONFIG.shutdown_timeout_seconds()))
    .merge(("shutdown.mercy", 5));  // 5 seconds after grace

// Signal handler cho rolling updates
tokio::spawn(async move {
    use tokio::signal::unix::{signal, SignalKind};
    let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
    sigterm.recv().await;
    info!("SIGTERM received, initiating graceful shutdown...");
    // Rocket's shutdown handle will be notified via ctrlc handler
});
```

### 3.7 PostgreSQL Read Replica

**File**: `src/db/mod.rs`:

```rust
// Thêm second pool cho read replica
pub static READ_POOL: LazyLock<Option<DbPool>> = LazyLock::new(|| {
    if CONFIG.database_read_url().is_empty() { return None; }
    Some(create_pool(CONFIG.database_read_url(), CONFIG.database_read_pool_size()))
});

pub fn get_read_conn() -> DbConn {
    // Sử dụng read replica nếu available, fallback về write pool
    READ_POOL.as_ref()
        .and_then(|p| p.try_get().ok())
        .unwrap_or_else(|| DB_POOL.get().expect("DB connection"))
}
```

Các route read-only (sync, list) dùng `get_read_conn()`, write operations dùng write pool.

---

## 4. Feature Flags (Cargo.toml)

```toml
[features]
default = ["sqlite"]
sqlite = [...]
postgresql = [...]
mysql = [...]
redis = ["deadpool-redis", "redis"]    # NEW: optional Redis support

[dependencies]
deadpool-redis = { version = "0.18", optional = true }
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"], optional = true }
```

---

## 5. Config Variables Mới

```bash
# Redis
REDIS_ENABLED=false
REDIS_URL=redis://localhost:6379
REDIS_TLS=false
REDIS_PASSWORD=""                   # Masked
REDIS_POOL_SIZE=20
REDIS_KEY_PREFIX=vaultwarden:
REDIS_CONNECT_TIMEOUT_SECONDS=5

# Cluster
CLUSTER_MODE=false                  # Enables Redis-backed shared state
INSTANCE_ID=auto                    # Auto-generated UUID or manual

# Database Read Replica
DATABASE_READ_URL=""                # Optional PostgreSQL read replica
DATABASE_READ_POOL_SIZE=10

# Graceful Shutdown
SHUTDOWN_TIMEOUT_SECONDS=30
```

---

## 6. Migration Path

### Phase 1 (v2.0 beta): Redis Support Optional
- Thêm `redis` feature flag
- Implement `CacheBackend` trait
- `REDIS_ENABLED=false` by default — không có breaking change

### Phase 2 (v2.0 RC): Cluster Testing
- `CLUSTER_MODE=true` + Redis: migrate rate limiter, OIDC cache
- WebSocket Redis pub/sub

### Phase 3 (v2.0 stable): Read Replica
- PostgreSQL read replica pool
- Route-level read/write split

### Phase 4 (v2.1): Kubernetes
- Helm chart với 3-replica deployment
- HPA (Horizontal Pod Autoscaler) config
- Load testing documentation

---

## 7. Acceptance Criteria Mapping

| Criterion | Giải pháp |
|-----------|----------|
| 3 instances: killing one không gây lỗi | Health check + load balancer routing |
| WebSocket event từ Instance A → User trên Instance B | Redis pub/sub listener |
| Rate limit shared giữa instances | Cache-backed counter với Redis |
| `GET /health` trả 200 | `health_simple()` endpoint |
| Zero downtime restart | Rocket graceful shutdown + drain period |
| OIDC flow hoạt động cross-instance | OIDC state trong Redis |

---

*Status: ✅ Implemented | Ngày cập nhật: 2026-04-17*

## Implementation Notes
- `src/cache.rs` (152 lines) — `CacheBackend` trait + InMemoryCache + RedisCache implementations
- `src/api/health.rs` (63 lines) — `/health`, `/health/ready`, `/health/live`, `/health/detailed` endpoints
- `src/db/mod.rs` — `READ_POOL` + read replica connection pool (TASK-005-015)
- Rate limiter migrated to cache-backed counter in `src/ratelimit.rs`
- OIDC cache migrated to `CacheBackend` in `src/sso.rs`
- WebSocket Redis Pub/Sub listener: `start_redis_pubsub_listener()` in `src/api/notifications.rs` (line 759)
- Graceful SIGTERM shutdown in `src/main.rs` (lines 703-745)
- Redis feature flag: `deadpool-redis`, `redis` optional crates in `Cargo.toml`
- Config: `REDIS_ENABLED`, `CLUSTER_MODE`, `DATABASE_READ_URL` — all implemented
