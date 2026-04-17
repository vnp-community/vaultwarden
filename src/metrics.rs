// SOL-010: Complete Prometheus Metrics Integration

use std::sync::{Arc, LazyLock, RwLock};
use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::{
        counter::Counter,
        family::Family,
        gauge::Gauge,
        histogram::{exponential_buckets, Histogram},
    },
    registry::Registry,
};

/// Labels for HTTP requests
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HttpLabels {
    pub method: String,
    pub path: String,
    pub status: String,
}

/// Labels for login events
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct LoginLabels {
    pub result: String, // "success" | "failure"
    pub method: String, // "password" | "sso" | "api_key" | "2fa"
}

/// Labels for email events
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct EmailLabels {
    pub event: String, // "sent" | "failed"
}

/// Labels for background jobs
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct JobLabels {
    pub job: String,
}

/// Labels for rate limiting
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct RateLimitLabels {
    pub route: String,
}

/// All Vaultwarden metrics in one struct
pub struct VaultwardenMetrics {
    // HTTP
    pub http_requests_total: Family<HttpLabels, Counter>,
    pub http_request_duration_seconds: Family<HttpLabels, Histogram>,

    // Auth
    pub login_attempts_total: Family<LoginLabels, Counter>,
    pub active_sessions: Gauge,
    pub failed_logins_total: Family<LoginLabels, Counter>,

    // WebSocket
    pub websocket_connections: Gauge,
    pub websocket_events_sent: Counter,
    pub websocket_events_failed: Counter,

    // DB pool
    pub db_pool_size: Gauge,
    pub db_pool_idle: Gauge,

    // Email
    pub email_total: Family<EmailLabels, Counter>,

    // Counts
    pub users_total: Gauge,
    pub ciphers_total: Gauge,
    pub orgs_total: Gauge,

    // Rate limiting
    pub rate_limit_triggers: Family<RateLimitLabels, Counter>,

    // Background jobs
    pub job_runs_total: Family<JobLabels, Counter>,
    pub job_duration_seconds: Family<JobLabels, Histogram>,
    pub job_failures_total: Family<JobLabels, Counter>,
}

pub static METRICS: LazyLock<Arc<VaultwardenMetrics>> = LazyLock::new(|| {
    let http_requests_total = Family::<HttpLabels, Counter>::default();
    let http_request_duration_seconds = Family::<HttpLabels, Histogram>::new_with_constructor(|| {
        Histogram::new(exponential_buckets(0.005, 2.0, 12))
    });
    let login_attempts_total = Family::<LoginLabels, Counter>::default();
    let active_sessions: Gauge = Gauge::default();
    let failed_logins_total = Family::<LoginLabels, Counter>::default();
    let websocket_connections: Gauge = Gauge::default();
    let websocket_events_sent: Counter = Counter::default();
    let websocket_events_failed: Counter = Counter::default();
    let db_pool_size: Gauge = Gauge::default();
    let db_pool_idle: Gauge = Gauge::default();
    let email_total = Family::<EmailLabels, Counter>::default();
    let users_total: Gauge = Gauge::default();
    let ciphers_total: Gauge = Gauge::default();
    let orgs_total: Gauge = Gauge::default();
    let rate_limit_triggers = Family::<RateLimitLabels, Counter>::default();
    let job_runs_total = Family::<JobLabels, Counter>::default();
    let job_duration_seconds = Family::<JobLabels, Histogram>::new_with_constructor(|| {
        Histogram::new(exponential_buckets(0.01, 2.0, 10))
    });
    let job_failures_total = Family::<JobLabels, Counter>::default();

    if crate::CONFIG.metrics_enabled() {
        let mut registry = REGISTRY.write().expect("REGISTRY poisoned");

        registry.register("vaultwarden_http_requests_total", "Total HTTP requests handled", http_requests_total.clone());
        registry.register("vaultwarden_http_request_duration_seconds", "HTTP request duration in seconds", http_request_duration_seconds.clone());
        registry.register("vaultwarden_login_attempts_total", "Total login attempts", login_attempts_total.clone());
        registry.register("vaultwarden_active_sessions", "Number of active sessions", active_sessions.clone());
        registry.register("vaultwarden_failed_logins_total", "Total failed login attempts", failed_logins_total.clone());
        registry.register("vaultwarden_websocket_connections", "Current WebSocket connections", websocket_connections.clone());
        registry.register("vaultwarden_websocket_events_sent_total", "Total WebSocket events sent successfully", websocket_events_sent.clone());
        registry.register("vaultwarden_websocket_events_failed_total", "Total WebSocket events that failed", websocket_events_failed.clone());
        registry.register("vaultwarden_db_pool_size", "Database connection pool size", db_pool_size.clone());
        registry.register("vaultwarden_db_pool_idle", "Database connection pool idle connections", db_pool_idle.clone());
        registry.register("vaultwarden_emails_total", "Total emails by event type", email_total.clone());
        registry.register("vaultwarden_users_total", "Total number of users", users_total.clone());
        registry.register("vaultwarden_ciphers_total", "Total number of vault items", ciphers_total.clone());
        registry.register("vaultwarden_organizations_total", "Total number of organizations", orgs_total.clone());
        registry.register("vaultwarden_rate_limit_triggers_total", "Total rate limit events triggered", rate_limit_triggers.clone());
        registry.register("vaultwarden_job_runs_total", "Total background job runs", job_runs_total.clone());
        registry.register("vaultwarden_job_duration_seconds", "Background job duration in seconds", job_duration_seconds.clone());
        registry.register("vaultwarden_job_failures_total", "Total background job failures", job_failures_total.clone());
    }

    Arc::new(VaultwardenMetrics {
        http_requests_total,
        http_request_duration_seconds,
        login_attempts_total,
        active_sessions,
        failed_logins_total,
        websocket_connections,
        websocket_events_sent,
        websocket_events_failed,
        db_pool_size,
        db_pool_idle,
        email_total,
        users_total,
        ciphers_total,
        orgs_total,
        rate_limit_triggers,
        job_runs_total,
        job_duration_seconds,
        job_failures_total,
    })
});

pub static REGISTRY: LazyLock<RwLock<Registry>> = LazyLock::new(|| RwLock::new(Registry::default()));

/// Normalize URL path metrics by replacing UUIDs with `{id}` placeholder
pub fn normalize_metric_path(path: &str) -> String {
    use std::sync::LazyLock;
    use regex::Regex;
    static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap()
    });
    UUID_RE.replace_all(path, "{id}").into_owned()
}

pub fn get_metrics_snapshot() -> String {
    let mut buffer = String::new();
    let registry = REGISTRY.read().expect("REGISTRY poisoned");
    let _ = prometheus_client::encoding::text::encode(&mut buffer, &registry);
    buffer
}
