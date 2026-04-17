# SOL-010: Giải Pháp Thực Hiện — Enterprise Monitoring, Observability & Alerting

> **Giải pháp cho**: CR-010  
> **Ngày**: 2026-04-12  
> **Trạng thái**: Draft  
> **Kiến trúc thay đổi**: Tối thiểu — additive, không thay đổi core logic

---

## 1. Tổng Quan Giải Pháp

Giải pháp hoàn toàn **additive** — thêm metrics, structured logging, health endpoints. Không thay đổi business logic.

1. **Prometheus metrics**: `src/metrics.rs` với `prometheus-client` crate
2. **JSON structured logging**: Thay `fern` text output bằng JSON format
3. **Enhanced health checks**: Mở rộng endpoint (sẵn có từ CR-005)
4. **OpenTelemetry tracing**: Optional feature flag
5. **Security alerting**: Background task monitor

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/metrics.rs` | Prometheus metrics registry + metric definitions |
| `src/api/metrics.rs` | `/metrics` Rocket route |
| `src/tracing.rs` | OpenTelemetry setup (optional feature) |
| `src/alerting.rs` | Security alerting engine |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/util.rs` | Thêm `MetricsFairing` (count HTTP requests/durations) |
| `src/main.rs` | Setup metrics registry, structured logging, alerting task |
| `src/config.rs` | Thêm METRICS_*, LOG_FORMAT, OTEL_*, SECURITY_ALERT_* config keys |
| `src/api/identity.rs` | Increment login metrics |
| `src/api/notifications.rs` | WebSocket connection metrics |
| `src/db/mod.rs` | DB pool metrics |

---

## 3. Thiết Kế Chi Tiết

### 3.1 Prometheus Metrics Registry

**File**: `src/metrics.rs`

**Phụ thuộc mới**:
```toml
[dependencies]
prometheus-client = "0.22"
```

```rust
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::Histogram};
use prometheus_client::registry::Registry;

pub struct VaultwardenMetrics {
    // Authentication
    pub login_attempts: Family<LoginLabels, Counter>,
    pub active_sessions: Gauge,
    pub token_refreshes: Counter,
    pub two_fa_completions: Family<TwoFaLabels, Counter>,
    
    // HTTP
    pub http_requests: Family<HttpLabels, Counter>,
    pub http_duration: Family<HttpPathLabels, Histogram>,
    pub http_active_connections: Gauge,
    
    // WebSocket
    pub websocket_connections: Gauge,
    pub websocket_events_sent: Counter,
    pub websocket_events_failed: Counter,
    
    // Database
    pub db_pool_size: Gauge,
    pub db_pool_idle: Gauge,
    pub db_query_duration: Family<DbOperationLabels, Histogram>,
    pub db_errors: Family<DbOperationLabels, Counter>,
    
    // Email
    pub email_sent: Family<EmailTypeLabels, Counter>,
    pub email_failed: Family<EmailTypeLabels, Counter>,
    
    // Vault
    pub users_total: Gauge,
    pub ciphers_total: Gauge,
    pub organizations_total: Gauge,
    pub attachments_size_bytes: Gauge,
    
    // Background jobs
    pub job_runs: Family<JobLabels, Counter>,
    pub job_duration: Family<JobLabels, Histogram>,
    pub job_failures: Family<JobLabels, Counter>,
    
    // Security
    pub rate_limit_triggers: Family<EndpointLabels, Counter>,
    pub failed_logins: Family<FailureReasonLabels, Counter>,
    pub admin_actions: Family<AdminActionLabels, Counter>,
    
    // Info
    pub info: Family<InfoLabels, Gauge>,
}

#[derive(Clone, Hash, PartialEq, Eq, EncodeLabelSet, Debug)]
pub struct LoginLabels {
    pub result: String,  // "success", "failure", "rate_limited"
}

#[derive(Clone, Hash, PartialEq, Eq, EncodeLabelSet, Debug)]
pub struct HttpLabels {
    pub method: String,
    pub path: String,
    pub status: String,
}

pub static METRICS: LazyLock<Arc<VaultwardenMetrics>> = LazyLock::new(|| {
    Arc::new(VaultwardenMetrics::new())
});

pub static REGISTRY: LazyLock<Arc<RwLock<Registry>>> = LazyLock::new(|| {
    let mut registry = Registry::default();
    METRICS.register_all(&mut registry);
    Arc::new(RwLock::new(registry))
});

impl VaultwardenMetrics {
    pub fn new() -> Self {
        // Khởi tạo tất cả metrics với default values
        let m = Self {
            login_attempts: Family::default(),
            active_sessions: Gauge::default(),
            // ... tất cả metrics khác
            info: Family::default(),
        };
        
        // Set version info metric
        m.info.get_or_create(&InfoLabels {
            version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").unwrap_or("unknown").to_string(),
        }).set(1);
        
        m
    }
}
```

### 3.2 Metrics Endpoint

**File**: `src/api/metrics.rs`

```rust
#[get("/metrics")]
pub async fn metrics_endpoint(
    req: &Request<'_>,
) -> Result<String, Status> {
    // Auth check: Bearer token
    if CONFIG.metrics_enabled() {
        let token = req.headers().get_one("Authorization")
            .and_then(|h| h.strip_prefix("Bearer "))
            .or_else(|| req.query_value::<&str>("token").and_then(|r| r.ok()));
        
        match token {
            Some(t) if crate::crypto::ct_eq(t, CONFIG.metrics_token()) => {}
            _ => return Err(Status::Unauthorized),
        }
        
        // IP allowlist
        if !CONFIG.metrics_allowed_ips().is_empty() {
            let ip = req.client_ip().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
            if !is_ip_allowed(ip, CONFIG.metrics_allowed_ips()) {
                return Err(Status::Forbidden);
            }
        }
    }
    
    // Collect latest DB stats trước khi encode
    update_db_pool_metrics().await;
    update_vault_count_metrics().await;
    
    let registry = REGISTRY.read().await;
    let mut output = String::new();
    prometheus_client::encoding::text::encode(&mut output, &registry)
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(output)
}

async fn update_db_pool_metrics() {
    if let Ok(pool) = DB_POOL.try_get() {
        let state = pool.state();
        METRICS.db_pool_size.set(state.connections as i64);
        METRICS.db_pool_idle.set(state.idle_connections as i64);
    }
}

async fn update_vault_count_metrics() {
    // Chạy queries đơn giản để cập nhật gauge metrics
    if let Ok(pool) = DB_POOL.try_get() {
        let conn = pool.get().unwrap();
        if let Ok(count) = User::count_all(&conn).await {
            METRICS.users_total.set(count);
        }
    }
}
```

### 3.3 HTTP Metrics Fairing

Thêm vào `src/util.rs`:

```rust
pub struct MetricsFairing;

#[rocket::async_trait]
impl Fairing for MetricsFairing {
    fn info(&self) -> Info {
        Info { name: "Metrics", kind: Kind::Request | Kind::Response }
    }
    
    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        req.local_cache(|| RequestStart(Instant::now()));
        METRICS.http_active_connections.inc();
    }
    
    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        METRICS.http_active_connections.dec();
        
        let start = req.local_cache(|| RequestStart(Instant::now()));
        let duration = start.0.elapsed().as_secs_f64();
        
        // Normalize path để tránh cardinality explosion
        let path = normalize_metric_path(req.uri().path().as_str());
        let method = req.method().as_str().to_string();
        let status = res.status().code.to_string();
        
        METRICS.http_requests.get_or_create(&HttpLabels {
            method: method.clone(),
            path: path.clone(),
            status,
        }).inc();
        
        METRICS.http_duration.get_or_create(&HttpPathLabels {
            method,
            path,
        }).observe(duration);
    }
}

fn normalize_metric_path(path: &str) -> String {
    // Thay UUIDs bằng {id} để tránh high cardinality
    let uuid_regex = regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
        .unwrap();
    uuid_regex.replace_all(path, "{id}").to_string()
}
```

### 3.4 Integration Points cho Business Metrics

```rust
// src/api/identity.rs — sau login attempt
METRICS.login_attempts.get_or_create(&LoginLabels {
    result: if success { "success" } else { "failure" }.to_string(),
}).inc();

if success {
    METRICS.active_sessions.inc();
}

// src/ratelimit.rs — khi rate limit triggered
METRICS.rate_limit_triggers.get_or_create(&EndpointLabels {
    endpoint: endpoint.to_string(),
}).inc();

// src/api/notifications.rs — WebSocket connections
// Khi WS connected:
METRICS.websocket_connections.inc();
// Khi WS disconnected:
METRICS.websocket_connections.dec();
```

### 3.5 Structured JSON Logging

Thay thế `fern` setup trong `src/main.rs`:

**Phụ thuộc mới**:
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
# Giữ nguyên fern cho text mode (backward compat)
```

```rust
pub fn setup_logging() {
    if CONFIG.log_format() == "json" {
        // JSON structured logging
        use tracing_subscriber::{fmt, EnvFilter};
        
        fmt::Subscriber::builder()
            .json()
            .with_env_filter(EnvFilter::new(CONFIG.log_level()))
            .with_current_span(true)
            .with_span_list(true)
            .init();
    } else {
        // Giữ nguyên fern text logging (backward compatible)
        setup_fern_logging();
    }
}

// Custom log macro wrapper để include contextual fields
macro_rules! log_event {
    ($level:expr, $msg:expr, $($key:expr => $val:expr),*) => {
        match $level {
            "INFO"  => tracing::info!(message = $msg, $($key = ?$val),*),
            "WARN"  => tracing::warn!(message = $msg, $($key = ?$val),*),
            "ERROR" => tracing::error!(message = $msg, $($key = ?$val),*),
            _       => tracing::debug!(message = $msg, $($key = ?$val),*),
        }
    }
}
```

### 3.6 OpenTelemetry Tracing (Optional Feature)

**Feature flag**: `otel` trong Cargo.toml

```toml
[features]
otel = [
    "opentelemetry",
    "opentelemetry-otlp",
    "tracing-opentelemetry",
]
```

```rust
#[cfg(feature = "otel")]
pub fn setup_otel() {
    use opentelemetry_otlp::WithExportConfig;
    use tracing_opentelemetry::OpenTelemetryLayer;
    
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(CONFIG.otel_endpoint())
        )
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(
                    opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(CONFIG.otel_sample_rate())
                )
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", CONFIG.otel_service_name()),
                    opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("OTel tracer setup failed");
    
    tracing_subscriber::registry()
        .with(OpenTelemetryLayer::new(tracer))
        .init();
}
```

### 3.7 Security Alerting

**File**: `src/alerting.rs`

```rust
pub struct SecurityAlertingEngine {
    // Sliding window counters
    failed_logins_per_minute: Arc<Mutex<SlidingWindowCounter>>,
    rate_limits_per_minute: Arc<Mutex<SlidingWindowCounter>>,
}

impl SecurityAlertingEngine {
    pub fn start(self: Arc<Self>) {
        let engine = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                engine.check_thresholds().await;
            }
        });
    }
    
    async fn check_thresholds(&self) {
        // Đọc từ Prometheus metrics
        let failed_logins = METRICS.failed_logins.get_or_create(
            &FailureReasonLabels { reason: "all".to_string() }
        ).get();
        
        let rate_limits = METRICS.rate_limit_triggers.get_or_create(
            &EndpointLabels { endpoint: "all".to_string() }
        ).get();
        
        // Brute-force detection
        let failed_per_min = self.failed_logins_per_minute.lock().await
            .add_and_get(failed_logins as u64);
        
        if failed_per_min > CONFIG.alert_failed_logins_per_minute() as u64 {
            self.send_alert(
                "Possible brute-force attack",
                &format!("{failed_per_min} failed logins in last minute"),
                Severity::Critical,
            ).await;
        }
        
        // Rate limit spike
        let rl_per_min = self.rate_limits_per_minute.lock().await
            .add_and_get(rate_limits as u64);
        
        if rl_per_min > CONFIG.alert_rate_limit_per_minute() as u64 {
            self.send_alert(
                "Rate limiting spike detected",
                &format!("{rl_per_min} rate limit events in last minute"),
                Severity::Warn,
            ).await;
        }
    }
    
    async fn send_alert(&self, title: &str, message: &str, severity: Severity) {
        // Email alert
        if !CONFIG.security_alert_email().is_empty() {
            mail::send_security_alert(
                CONFIG.security_alert_email(), title, message
            ).await.ok();
        }
        
        // Webhook alert (e.g., Slack)
        if !CONFIG.security_alert_webhook_url().is_empty() {
            let payload = json!({
                "text": format!("*{title}*\n{message}"),
                "severity": severity.to_string(),
            });
            
            get_reqwest_client()
                .post(CONFIG.security_alert_webhook_url())
                .json(&payload)
                .send()
                .await
                .ok();
        }
        
        // Audit log
        audit::emit(AuditEntry {
            event_type: AuditEventType::SecurityAlertTriggered,
            severity,
            metadata: json!({"title": title, "message": message}),
            ..Default::default()
        });
    }
}
```

---

## 4. Grafana Dashboard

Tạo template JSON và expose qua API:

```rust
// GET /api/admin/grafana-dashboard
#[get("/admin/grafana-dashboard")]
async fn grafana_dashboard(_admin: AdminHeaders) -> Json<Value> {
    // Trả về pre-built Grafana dashboard JSON
    let dashboard: Value = serde_json::from_str(
        include_str!("../static/grafana-dashboard.json")
    ).unwrap();
    Json(dashboard)
}
```

`src/static/grafana-dashboard.json` được build sẵn với panels:
- Login rate (success vs failure)
- Active sessions
- DB query time histogram
- Error rate
- Memory + CPU
- Background job runs
- Security events timeline
- WebSocket connections

---

## 5. Config Variables Mới

```bash
# Metrics
METRICS_ENABLED=false
METRICS_TOKEN=""                        # Masked — required để access /metrics
METRICS_ALLOWED_IPS=""                  # CIDR whitelist for scraper
METRICS_PORT=9090                       # Separate port (optional)

# Structured Logging
LOG_FORMAT=text                         # 'text' (default, backward compat) | 'json'
LOG_INCLUDE_TRACE_ID=false

# OpenTelemetry
OTEL_ENABLED=false
OTEL_EXPORTER=otlp                      # otlp|jaeger|zipkin
OTEL_ENDPOINT=http://jaeger:4317
OTEL_SERVICE_NAME=vaultwarden
OTEL_SAMPLE_RATE=0.1                    # 10% sampling

# Security Alerting
SECURITY_ALERTS_ENABLED=false
SECURITY_ALERT_EMAIL=""
SECURITY_ALERT_WEBHOOK_URL=""           # Slack/Teams webhook
ALERT_FAILED_LOGINS_PER_MINUTE=50
ALERT_RATE_LIMIT_PER_MINUTE=100
ALERT_NEW_ADMIN_LOGIN_NOTIFY=true
ALERT_EMERGENCY_ACCESS_NOTIFY=true
ALERT_CONFIG_CHANGE_NOTIFY=true
```

---

## 6. Phụ Thuộc Mới

| Crate | Phiên bản | Feature Flag | Lý do |
|-------|-----------|--------------|-------|
| `prometheus-client` | 0.22 | default | Metrics registry |
| `tracing` | 0.1 | default | Structured logging |
| `tracing-subscriber` | 0.3 | default | Log formatting (JSON) |
| `opentelemetry` | 0.27 | `otel` | Distributed tracing |
| `opentelemetry-otlp` | 0.27 | `otel` | OTLP exporter |
| `tracing-opentelemetry` | 0.28 | `otel` | Tracing bridge |

---

## 7. Kế Hoạch Triển Khai

### Sprint 1–2: Prometheus Core Metrics
- `src/metrics.rs` — registry + metric definitions
- `MetricsFairing` — HTTP request counting
- `/metrics` endpoint với auth

### Sprint 3: All Metric Categories
- Integration vào identity.rs, notifications.rs, db/mod.rs
- Background job metrics

### Sprint 4: Enhanced Health Check
- Reuse từ CR-005 `/health/detailed`
- Kubernetes probes

### Sprint 5: JSON Structured Logging
- `tracing-subscriber` JSON formatter
- Backward compatible với `LOG_FORMAT=text`

### Sprint 6–7: OpenTelemetry
- Optional `otel` feature
- OTLP + Jaeger support
- HTTP request spans

### Sprint 8: Security Alerting + Grafana
- Alerting engine
- Slack webhook
- Grafana dashboard template

---

*Status: Draft | Ngày: 2026-04-12*
