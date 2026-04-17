# TASKS-SOL-005: High Availability & Horizontal Scaling

> **Giải pháp**: SOL-005  
> **CR**: CR-005  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 17

---

## Phase 1 — Redis Support Optional (v2.0 beta)

### [x] TASK-005-001
- **Tên**: Thêm Redis feature flag vào Cargo.toml
- **File**: `Cargo.toml`
- **Mô tả**: Thêm feature `redis = ["deadpool-redis", "redis"]`. Thêm optional dependencies: `deadpool-redis = { version = "0.18", optional = true }`, `redis = { version = "0.27", features = ["tokio-comp", "connection-manager"], optional = true }`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-005-002
- **Tên**: Implement `CacheBackend` trait
- **File**: `src/cache.rs` (mới)
- **Mô tả**: `async_trait CacheBackend`: `get()`, `set()`, `delete()`, `increment()`, `is_healthy()`. Error type `CacheError`.
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-005-003
- **Tên**: Implement `InMemoryCache`
- **File**: `src/cache.rs`
- **Mô tả**: DashMap-based cache với TTL. Background cleanup task mỗi 60 giây. Atomic increment qua DashMap entry API. `is_healthy()` luôn trả `true`.
- **Loại**: New code
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-005-002

### [x] TASK-005-004
- **Tên**: Implement `RedisCache` (#[cfg(feature = "redis")])
- **File**: `src/cache.rs`
- **Mô tả**: deadpool-redis connection pool. `get()` via Redis GET, `set()` via SET EX, `increment()` via atomic INCR+EXPIRE pipeline, `delete()` via DEL, `is_healthy()` via PING. Key prefix support.
- **Loại**: New code (feature-gated)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-005-001, TASK-005-002

### [x] TASK-005-005
- **Tên**: Thêm REDIS_* và CLUSTER_MODE config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `redis_enabled`, `redis_url`, `redis_tls`, `redis_password` (masked), `redis_pool_size`, `redis_key_prefix`, `redis_connect_timeout_seconds`, `cluster_mode`, `instance_id`, `shutdown_timeout_seconds`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-005-006
- **Tên**: Khởi tạo global `CACHE` instance trong `main.rs`
- **File**: `src/main.rs`, `src/cache.rs`
- **Mô tả**: `LazyLock<Arc<dyn CacheBackend>>` — khởi tạo `RedisCache` nếu `redis_enabled`, ngược lại `InMemoryCache`. Cleanup task cho InMemoryCache.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-005-003, TASK-005-004, TASK-005-005

---

## Phase 2 — Cluster Testing: Rate Limiter + OIDC Cache Migration

### [x] TASK-005-007
- **Tên**: Migrate rate limiter sang `CacheBackend`
- **File**: `src/ratelimit.rs`
- **Mô tả**: Thay `governor` in-memory bằng `CACHE.increment()` khi `CLUSTER_MODE=true`. Giữ `governor` cho single-instance mode. Function `check_rate_limit(key, identifier, max, window_secs)`.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-005-006

### [x] TASK-005-008
- **Tên**: Migrate OIDC state cache sang `CacheBackend`
- **File**: `src/sso.rs`
- **Mô tả**: Thay `mini-moka` `AC_CACHE` bằng `CACHE.set/get()` với key `oidc:{state}`, TTL 600 giây. `cache_set_oidc_state()`, `cache_get_oidc_state()`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-005-006

---

## Phase 2 — WebSocket Redis Pub/Sub

### [x] TASK-005-009
- **Tên**: Implement Redis pub/sub publish trong notification handler
- **File**: `src/api/notifications.rs`
- **Mô tả**: Trong `send_notification_to_user()`: publish event lên Redis channel `{prefix}ws:user:{uuid}` khi `CLUSTER_MODE=true`, sau khi đã gửi tới local WS connections.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-005-001, TASK-005-005

### [x] TASK-005-010
- **Tên**: Implement Redis pub/sub subscriber background task
- **File**: `src/api/notifications.rs`
- **Mô tả**: `redis_pubsub_listener()` (feature = "redis"): subscribe pattern `{prefix}ws:user:*`, nhận messages, extract user_uuid từ channel name, forward tới local WS connections của instance này.
- **Loại**: New function (feature-gated)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-005-009

---

## Phase 2 — Health Endpoints

### [x] TASK-005-011
- **Tên**: Implement health check endpoints
- **File**: `src/api/health.rs` (mới)
- **Mô tả**: `GET /health` (200 nếu DB ok), `GET /health/ready` (Kubernetes readiness), `GET /health/live` (luôn 200), `GET /health/detailed` (DB + Redis status, version, instance_id). Auth optional cho `/health/detailed`.
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-005-006

### [x] TASK-005-012
- **Tên**: Mount health routes trong `main.rs`
- **File**: `src/main.rs`
- **Mô tả**: Mount `/health` routes. Không yêu cầu auth (load balancer cần truy cập tự do).
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-005-011

---

## Phase 2 — Graceful Shutdown

### [x] TASK-005-013
- **Tên**: Implement graceful shutdown support
- **File**: `src/main.rs`
- **Mô tả**: Cấu hình Rocket `shutdown.ctrlc=true`, `shutdown.grace=SHUTDOWN_TIMEOUT_SECONDS`, `shutdown.mercy=5`. Spawn SIGTERM handler với `tokio::signal::unix`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-005-005

---

## Phase 3 — PostgreSQL Read Replica

### [x] TASK-005-014
- **Tên**: Thêm DATABASE_READ_URL config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `database_read_url`, `database_read_pool_size`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-005-015
- **Tên**: Implement read replica connection pool
- **File**: `src/db/mod.rs`
- **Mô tả**: `LazyLock<Option<DbPool>>` READ_POOL. `get_read_conn()`: dùng read replica nếu available, fallback về write pool. Áp dụng cho routes read-only (list, sync operations).
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-005-014

---

## Phase 4 — Kubernetes & Load Testing

### [x] TASK-005-016
- **Tên**: Tạo Helm chart cho 3-replica deployment
- **File**: `deploy/helm/` (mới directory)
- **Mô tả**: Helm chart với: Deployment (3 replicas), Service, Ingress, ConfigMap, HPA (min 2, max 10, CPU 70%). Resource limits. Liveness/readiness probes dùng `/health/live` và `/health/ready`.
- **Loại**: New files
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-005-011, TASK-005-013

### [x] TASK-005-017
- **Tên**: Load testing và acceptance criteria verification
- **File**: Tests + documentation
- **Mô tả**: Test 3 instances: kill một instance → không gây lỗi client. WebSocket event từ Instance A → User trên Instance B. Rate limit shared. OIDC flow cross-instance. Zero downtime restart với rolling update.
- **Loại**: Testing
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-005-007 → TASK-005-013

---

## Tóm Tắt

| Phase | Tasks | Kết quả |
|-------|-------|---------|
| Phase 1 | TASK-005-001 → 006 | Redis abstraction layer |
| Phase 2 (rate/OIDC) | TASK-005-007 → 008 | Cache migration |
| Phase 2 (WS) | TASK-005-009 → 010 | Cross-instance WebSocket |
| Phase 2 (health) | TASK-005-011 → 012 | K8s health probes |
| Phase 2 (shutdown) | TASK-005-013 | Graceful shutdown |
| Phase 3 | TASK-005-014 → 015 | Read replica |
| Phase 4 | TASK-005-016 → 017 | K8s Helm + load testing |

---

*Tạo từ SOL-005 | Ngày: 2026-04-13*
