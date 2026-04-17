# TASKS-SOL-010: Enterprise Monitoring, Observability & Alerting

> **Giải pháp**: SOL-010  
> **CR**: CR-010  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 17

---

## Sprint 1–2 — Prometheus Core Metrics (4 tuần)

### [x] TASK-010-001
- **Tên**: Thêm `prometheus-client` dependency
- **File**: `Cargo.toml`
- **Mô tả**: Thêm `prometheus-client = "0.22"`. Thêm `tracing = "0.1"`, `tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-010-002
- **Tên**: Implement `VaultwardenMetrics` struct với tất cả metric definitions
- **File**: `src/metrics.rs`
- **Mô tả**: Định nghĩa tất cả metrics: login_attempts, active_sessions, http_requests/duration, websocket_connections, db_pool_size/idle, email_total, users_total/ciphers_total/orgs_total, job_runs/duration/failures, rate_limit_triggers, failed_logins. Global `METRICS: LazyLock<Arc<VaultwardenMetrics>>`, `REGISTRY: LazyLock<RwLock<Registry>>`.
- **Loại**: New file (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-010-001

### [x] TASK-010-003
- **Tên**: Implement `MetricsFairing`
- **File**: `src/util.rs`
- **Mô tả**: Rocket Fairing (Kind::Request | Kind::Response). `on_request`: cache `MetricsStart(Instant::now())`. `on_response`: measure duration, `normalize_metric_path()` (replace UUIDs với `{id}`), increment `http_requests`, observe `http_duration` histogram.
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-010-002

### [x] TASK-010-004
- **Tên**: Implement `/metrics` endpoint
- **File**: `src/api/metrics.rs`
- **Mô tả**: `GET /metrics`: Bearer token auth (constant-time compare với `METRICS_TOKEN`). Prometheus text format via `prometheus_client::encoding::text::encode()`.
- **Loại**: New file (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-010-002

### [x] TASK-010-005
- **Tên**: Thêm METRICS_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `metrics_enabled`, `metrics_token` (masked), `metrics_allowed_ips`, `metrics_port`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-010-006
- **Tên**: Attach `MetricsFairing` và mount metrics route
- **File**: `src/main.rs`
- **Mô tả**: `.attach(MetricsFairing)`. Mount `/metrics` route. Khởi tạo `METRICS` và `REGISTRY` static variables.
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-010-003, TASK-010-004

---

## Sprint 3 — All Metric Categories (2 tuần)

### [x] TASK-010-007
- **Tên**: Integrate login metrics vào `identity.rs`
- **File**: `src/api/identity.rs`
- **Mô tả**: Emit `METRICS.login_attempts_total` với LoginLabels (result: success/failure, method: grant_type). `METRICS.active_sessions.inc()` khi login success. `METRICS.failed_logins_total`.
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-010-002

### [x] TASK-010-008
- **Tên**: Integrate WebSocket và DB pool metrics
- **File**: `src/api/notifications.rs`
- **Mô tả**: `METRICS.websocket_connections.inc()` on connect (authenticated + anonymous guards) in both `websockets_hub` and `anonymous_websockets_hub`. `METRICS.websocket_connections.dec()` in `Drop` impls of `WSEntryMapGuard` and `WSAnonymousEntryMapGuard`. `METRICS.websocket_events_sent/failed` tracked in both `send_update_local` hot paths. DB pool metrics already exported via the existing `update_db_pool_metrics()` call in the scheduler.
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-010-002

### [x] TASK-010-009
- **Tên**: Integrate rate limit và job metrics
- **File**: `src/ratelimit.rs`, `src/main.rs` (job scheduler)
- **Mô tả**: ratelimit.rs: `METRICS.rate_limit_triggers` khi triggered (login + admin routes).
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-010-002

---

## Sprint 4 — Enhanced Health Check (2 tuần)

### [x] TASK-010-010
- **Tên**: Mở rộng health check endpoint từ SOL-005
- **File**: `src/api/health.rs`
- **Mô tả**: Mở rộng `/health/detailed`: thêm metrics info (active_sessions, ws_connections), DB connectivity check. `/health/alive` và `/health/ready` lightweight probes.
- **Loại**: New file (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-010-002

---

## Sprint 5 — JSON Structured Logging (2 tuần)

### [x] TASK-010-011
- **Tên**: Implement JSON structured logging setup
- **File**: `src/main.rs`
- **Mô tả**: In `init_logging()`: when `LOG_FORMAT=json`, bypasses fern and installs `tracing_subscriber::fmt().json()` with `EnvFilter`, `with_current_span(true)`, `with_span_list(true)`. `LOG_INCLUDE_TRACE_ID=true` adds thread IDs. Installs `tracing_log::LogTracer` bridge so existing `log::` macros are captured. Falls through to the existing fern path for the default `"text"` format (full backward compatibility).
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-010-001

### [x] TASK-010-012
- **Tên**: Thêm LOG_FORMAT config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `log_format` (default "text"), `log_include_trace_id`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

---

## Sprint 6–7 — OpenTelemetry (Optional Feature) (4 tuần)

### [x] TASK-010-013
- **Tên**: Thêm `otel` feature flag và dependencies
- **File**: `Cargo.toml`, `src/config.rs`
- **Mô tả**: Feature `otel` flag. Config: `otel_enabled`, `otel_exporter`, `otel_endpoint`, `otel_service_name`, `otel_sample_rate`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-010-001

### [x] TASK-010-014
- **Tên**: Implement OpenTelemetry setup (`#[cfg(feature = "otel")]`)
- **File**: `src/tracing.rs`
- **Mô tả**: `setup_otel()` public entry point — no-op when `OTEL_ENABLED=false` or when `otel` feature is not compiled. `setup_otel_inner()` (feature-gated): builds OTLP gRPC exporter via `opentelemetry-otlp`, uses `TraceIdRatioBased` sampler (or `AlwaysOn` at 1.0), sets `service.name` + `service.version` resource, attaches `OpenTelemetryLayer` to `tracing_subscriber::registry()`. Provider is leaked to `'static` so Tokio tasks continue submitting spans. Cargo.toml: added `otel` feature flag with optional `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry` dependencies.
- **Loại**: New file (implemented — feature-gated)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-010-013

---

## Sprint 8 — Security Alerting + Grafana (2 tuần)

### [x] TASK-010-015
- **Tên**: Implement `SecurityAlertingEngine`
- **File**: `src/alerting.rs`
- **Mô tả**: Sliding window counters cho failed_logins/rate_limits per minute. Background task mỗi 10 giây: check thresholds vs `ALERT_FAILED_LOGINS_PER_MINUTE` và `ALERT_RATE_LIMIT_PER_MINUTE`. Slack webhook dispatch.
- **Loại**: New file (implemented)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-010-002

### [x] TASK-010-016
- **Tên**: Thêm SECURITY_ALERTS_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `security_alerts_enabled`, `security_alert_email`, `security_alert_webhook_url`, `alert_failed_logins_per_minute`, `alert_rate_limit_per_minute`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-010-017
- **Tên**: Tạo Grafana dashboard template + API endpoint
- **File**: `src/static/grafana-dashboard.json`, `src/api/admin.rs`
- **Mô tả**: Pre-built Grafana dashboard JSON với panels: login rate, active sessions, HTTP duration p99, error rate, background jobs, WebSocket connections, rate limit triggers, DB pool health, entity counts. `GET /api/admin/grafana-dashboard` trả JSON file content.
- **Loại**: New static file + route (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-010-002

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1–2 | TASK-010-001 → 006 | 1–4 | Prometheus metrics core |
| Sprint 3 | TASK-010-007 → 009 | 5–6 | All metric integrations |
| Sprint 4 | TASK-010-010 | 7–8 | Enhanced health check |
| Sprint 5 | TASK-010-011 → 012 | 9–10 | JSON structured logging |
| Sprint 6–7 | TASK-010-013 → 014 | 11–14 | OpenTelemetry (optional) |
| Sprint 8 | TASK-010-015 → 017 | 15–16 | Alerting + Grafana |

---

*Tạo từ SOL-010 | Ngày: 2026-04-13*
