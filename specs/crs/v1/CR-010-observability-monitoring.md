# CR-010: Enterprise Monitoring, Observability & Alerting

> **Change Request ID**: CR-010  
> **Title**: Prometheus Metrics, OpenTelemetry Tracing, Structured Logging & Alerting  
> **Priority**: P2 — High  
> **Target Release**: v2.0  
> **Driven By**: [specs/crs/product-market-analysis.md §2.6 Monitoring]  
> **Affects**: PRD §9.5, URD §4.8, SRS §5.5, Technical Design §14

---

## 1. Problem Statement

- Logging là text-based, không có JSON structured log
- Không có Prometheus metrics endpoint — không thể monitor qua Grafana
- Không có distributed tracing (OpenTelemetry)
- Không có health check endpoint với detailed status
- Admin panel diagnostic page là web UI — không thể integrate vào automated monitoring
- Operations team của ngân hàng không có visibility vào authentication failure rates, database performance, v.v.

---

## 2. Scope of Change

### 2.1 Prometheus Metrics Endpoint

```
GET /metrics                           # Prometheus text format
GET /metrics/json                      # JSON format
```

**Authentication**: Separate Bearer token for metrics endpoint:
```
NEW CONFIG:
METRICS_ENABLED=false
METRICS_TOKEN=<bearer-token>           # Required to access /metrics
METRICS_ALLOWED_IPS=10.0.0.0/8        # IP whitelist for Prometheus scraper
```

**Exposed Metrics**:

```prometheus
# Authentication
vaultwarden_login_attempts_total{result="success|failure|rate_limited"} counter
vaultwarden_active_sessions_total gauge
vaultwarden_token_refreshes_total counter
vaultwarden_2fa_completions_total{method="totp|webauthn|email|yubikey"} counter

# HTTP
vaultwarden_http_requests_total{method,path,status} counter
vaultwarden_http_request_duration_seconds{method,path,quantile} histogram
vaultwarden_http_active_connections gauge

# WebSocket
vaultwarden_websocket_connections_total gauge
vaultwarden_websocket_events_sent_total counter
vaultwarden_websocket_events_failed_total counter

# Database
vaultwarden_db_pool_size gauge
vaultwarden_db_pool_idle gauge
vaultwarden_db_query_duration_seconds{operation} histogram
vaultwarden_db_errors_total{operation} counter

# Email
vaultwarden_email_sent_total{type} counter
vaultwarden_email_failed_total{type} counter
vaultwarden_email_delivery_duration_seconds histogram

# Vault
vaultwarden_users_total gauge
vaultwarden_ciphers_total gauge
vaultwarden_organizations_total gauge
vaultwarden_attachments_size_bytes gauge

# Background Jobs
vaultwarden_job_runs_total{job} counter
vaultwarden_job_duration_seconds{job} histogram
vaultwarden_job_failures_total{job} counter

# Security Events
vaultwarden_rate_limit_triggers_total{endpoint} counter
vaultwarden_failed_logins_total{reason} counter
vaultwarden_emergency_access_requests_total counter
vaultwarden_admin_actions_total{action} counter

# System
vaultwarden_process_memory_bytes gauge
vaultwarden_process_cpu_seconds_total counter
vaultwarden_info{version,build_time,git_commit} gauge
```

### 2.2 Enhanced Health Check Endpoint

```
GET /health              # Simple: 200 OK / 503 Service Unavailable
GET /health/detailed     # Detailed status (requires auth)
GET /health/ready        # Kubernetes readiness probe
GET /health/live         # Kubernetes liveness probe
```

**Response**:
```json
{
  "status": "healthy|degraded|unhealthy",
  "version": "2.0.0",
  "uptime_seconds": 86400,
  "checks": {
    "database": {
      "status": "healthy",
      "response_time_ms": 2,
      "pool_size": 10,
      "pool_idle": 8
    },
    "redis": {
      "status": "healthy",
      "response_time_ms": 1
    },
    "email": {
      "status": "healthy",
      "last_successful_send": "2026-04-12T09:00:00Z"
    },
    "storage": {
      "status": "healthy",
      "backend": "s3",
      "available": true
    },
    "job_scheduler": {
      "status": "healthy",
      "last_run": "2026-04-12T09:30:00Z",
      "next_run": "2026-04-12T09:31:00Z"
    }
  }
}
```

### 2.3 Structured JSON Logging

Thay thế text logging bằng JSON structured logs:

```
NEW CONFIG:
LOG_FORMAT=json|text                   # Default: text (backward compat)
LOG_INCLUDE_TRACE_ID=true
LOG_INCLUDE_SPAN_ID=true
```

**JSON log format**:
```json
{
  "timestamp": "2026-04-12T10:00:00.000Z",
  "level": "INFO",
  "target": "vaultwarden::api::identity",
  "message": "User login successful",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "user_id": "user-uuid",
  "ip": "10.0.0.1",
  "device_type": "Firefox",
  "duration_ms": 145,
  "fields": {
    "method": "POST",
    "path": "/identity/connect/token",
    "status": 200
  }
}
```

**Sensitive fields always masked**:
- `password`, `token`, `secret`, `key`, `hash` → `"***"`
- Email addresses in non-audit contexts → `"u***@example.com"`

### 2.4 OpenTelemetry Distributed Tracing

```
NEW CONFIG:
OTEL_ENABLED=false
OTEL_EXPORTER=otlp|jaeger|zipkin
OTEL_ENDPOINT=http://jaeger:4317
OTEL_SERVICE_NAME=vaultwarden
OTEL_SAMPLE_RATE=0.1                   # 10% sampling in production
```

**Instrumented operations**:
- HTTP request lifecycle (Rocket middleware)
- Database queries (Diesel + custom instrumentation)
- Email sending (lettre)
- External HTTP calls (icon proxy, OIDC, push relay)
- Background job execution
- WebSocket event delivery

### 2.5 Security Alerting

```
NEW CONFIG:
SECURITY_ALERTS_ENABLED=true
SECURITY_ALERT_EMAIL=security@example.com
SECURITY_ALERT_WEBHOOK_URL=https://hooks.slack.com/...

# Thresholds
ALERT_FAILED_LOGINS_PER_MINUTE=50      # Possible brute-force
ALERT_RATE_LIMIT_PER_MINUTE=100        # Possible attack
ALERT_NEW_ADMIN_LOGIN_NOTIFY=true      # Any admin login → notify
ALERT_EMERGENCY_ACCESS_NOTIFY=true     # Emergency access event
ALERT_CONFIG_CHANGE_NOTIFY=true        # Any config change
```

**Alert types**:
- Brute-force detection (many failed logins from same IP)
- Account enumeration detection (many logins with different usernames from same IP)
- Admin panel access (any login)
- Configuration change
- Backup failure
- Certificate expiry (device certs, TLS certs)
- Database connection loss
- Job scheduler failure

### 2.6 Grafana Dashboard Template

Official Grafana dashboard (JSON) shipped with Vaultwarden:
- Available at `GET /api/admin/grafana-dashboard`
- Panels: Login rate, Active sessions, DB query time, Error rate, Memory usage, Background jobs, Security events

---

## 3. Acceptance Criteria

- [ ] `GET /metrics` returns valid Prometheus text format; Prometheus can scrape it
- [ ] `vaultwarden_login_attempts_total{result="failure"}` increments on failed login
- [ ] `GET /health/ready` returns 503 when database is unreachable
- [ ] JSON log format produces valid JSON for each log line
- [ ] OpenTelemetry spans appear in Jaeger for a full login request
- [ ] Security alert email sent when failed logins exceed threshold in 1 minute
- [ ] Grafana dashboard template imports without errors

---

## 4. Estimated Effort

| Area | Effort |
|------|--------|
| Prometheus metrics (core) | 2 sprints |
| All metric categories | 2 sprints |
| Enhanced health check | 1 sprint |
| JSON structured logging | 1 sprint |
| OpenTelemetry tracing | 2 sprints |
| Security alerting | 1 sprint |
| Grafana dashboard | 1 sprint |

---

*Status: ✅ Implemented | Author: Product Team | Date: 2026-04-12 | Cập nhật: 2026-04-17*

> **Implementation**: [SOL-010](solutions/SOL-010-observability.md) — `src/metrics.rs` (171L), `src/alerting.rs` (117L), `src/tracing.rs` (118L), `src/api/metrics.rs` (`/metrics` Prometheus endpoint)
