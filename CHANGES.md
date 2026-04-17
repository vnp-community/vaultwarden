# Changelog

All notable changes to this Vaultwarden fork are documented here.

---

## [Unreleased] — Sprint 3 Security & Architecture Hardening

### ⚠️ Breaking Changes

#### WebSocket authentication — query parameter removed (TASK-RUSTDEV-CRIT-02-A)

**Changed**: The `/notifications/hub` WebSocket endpoint no longer accepts the JWT via the
`?access_token=<token>` URL query parameter.

**Migration**: All clients must send the JWT via the standard `Authorization: Bearer <token>` HTTP
header during the WebSocket upgrade handshake. All official Bitwarden clients (web vault, browser
extension, desktop, mobile) have used header-based auth since ~2023.

**Why**: URL query parameters appear in server access logs, load balancer logs, and browser
history — a significant token exposure risk. The query-param path was deprecated via `warn!` log
(CRIT-02-B) in Sprint 2. The hard removal lands in Sprint 3 after the deprecation period.

**Impact**: Only affects very old (pre-2023) unofficial integrations or custom scripts that
explicitly pass the token in the URL. Update such scripts to include the `Authorization: Bearer`
header in the WebSocket upgrade request.

---

### Security Improvements

- **Admin token enforced**: `ADMIN_TOKEN_STRICT_MODE` defaults to `true` — only Argon2id PHC
  tokens are accepted; plaintext tokens are rejected at startup (TASK-SEC-CRIT-01).
- **Rate limiting via AppState**: Login handlers (`/identity/connect/token`) now use the injected
  `AppState.rate_limiter` instead of calling the global `check_limit_login` directly. This makes
  rate limiting testable and injectable (TASK-RUSTDEV-MED-03-C).
- **RSA key encryption**: RSA private keys can now be encrypted at rest using AES-256-GCM by
  setting `RSA_KEY_ENCRYPTION_KEY`. A startup warning is emitted when the key is unencrypted
  (TASK-RUSTDEV-MED-02).
- **Trusted proxy XFF**: Added `TRUSTED_PROXIES` config to validate `X-Forwarded-For` headers only
  from known proxies, preventing IP spoofing in rate limiting (TASK-SEC-HIGH-01-C).
- **Icon SSRF protection**: Domain blocklist implemented — icon fetcher rejects requests to known
  SSRF targets (TASK-SEC-HIGH-03).
- **Anonymous WS limit**: `WS_ANON_MAX_CONNECTIONS` (default: 100) caps concurrent anonymous
  WebSocket connections to prevent DoS (TASK-RUSTDEV-HIGH-02).

### Architecture Improvements

- **Async job scheduler**: Replaced `job_scheduler_ng` with `tokio-cron-scheduler` 0.13. All 9
  background jobs now run as native tokio tasks — no `Arc<Runtime>` or dedicated thread needed
  (TASK-RUSTDEV-LOW-04-A).
- **AppState dependency injection**: `AppState` registered with Rocket via `.manage()`. Handlers
  progressively migrate away from global statics (TASK-RUSTDEV-MED-03-A/B).
- **DashMap WS cleanup**: Background task evicts stale WebSocket entries from `WS_USERS` every 60s
  to prevent unbounded memory growth (TASK-RUSTDEV-HIGH-02).
- **ArcSwap for regex**: Replaced `Mutex<Option<Regex>>` with `ArcSwap<Option<Regex>>` in
  `http_client.rs` — eliminates lock contention on the hot path (TASK-RUSTDEV-HIGH-01).

### Documentation

- `src/config_guide.md`: Full DSL syntax reference for the `make_config!` macro.
- `CONTRIBUTING.md`: Step-by-step guide for adding config keys and database migration guidelines
  (all 3 backends: SQLite, PostgreSQL, MySQL).
- `specs/bugs/rust-dev/tasks/research-config-migration.md`: GO decision for `figment`+`serde`
  migration, deferred to Sprint 5+ post-AppState migration.
- `specs/bugs/rust-dev/tasks/research-sqlx-migration.md`: NO-GO decision for Diesel→sqlx migration
  (847 call sites, 3-backend requirement negates compile-time safety benefits).

---

*Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)*
