# Vaultwarden — Technical Design Document

> Generated: 2026-04-09  
> Source: `dani-garcia/vaultwarden` — Rust implementation of the Bitwarden server API

---

## 1. Overview

Vaultwarden is a lightweight, self-hosted reimplementation of the **Bitwarden server API**, written in Rust. It is fully compatible with the official Bitwarden clients (web, desktop, mobile, CLI). It is designed for personal or small-team deployments where the official resource-intensive service is not practical.

- **License**: AGPL-3.0-only
- **Rust edition**: 2021, MSRV 1.89.0
- **Web framework**: [Rocket](https://rocket.rs) v0.5.1
- **ORM**: [Diesel](https://diesel.rs) v2.3.3
- **Async runtime**: Tokio (multi-thread)

---

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Bitwarden Clients                        │
│           (Web Vault, Desktop, Mobile, Browser Extension)        │
└──────────────────────┬──────────────────────────────────────────┘
                       │  HTTPS / WSS
┌──────────────────────▼──────────────────────────────────────────┐
│                    Reverse Proxy (nginx, Caddy…)                 │
└──────────────────────┬──────────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────────┐
│                        Vaultwarden Process                       │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐   │
│  │ Web/API  │  │ Identity │  │  Admin   │  │Notifications │   │
│  │ Routes   │  │  Routes  │  │  Routes  │  │  (WebSocket) │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────┬───────┘   │
│       └──────────────┴──────────────┴───────────────┘           │
│                          Auth Layer (JWT/RS256)                  │
│                          Rate Limiter                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                       Database Layer                      │   │
│  │  Diesel ORM — SQLite | PostgreSQL | MySQL/MariaDB         │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌───────────────────┐  ┌──────────────────────────────────┐   │
│  │  File Storage     │  │       Background Jobs            │   │
│  │  OpenDAL (FS/S3)  │  │       (job_scheduler_ng)         │   │
│  └───────────────────┘  └──────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Module Structure

```
src/
├── main.rs              — Entry point, Rocket launcher, job scheduler
├── config.rs            — Macro-generated configuration system (env + config.json)
├── auth.rs              — JWT encoding/decoding, key management, token types
├── crypto.rs            — Random byte generation, key derivation helpers
├── error.rs             — Unified Error type and MapResult trait
├── mail.rs              — Email dispatch via SMTP or sendmail (lettre)
├── ratelimit.rs         — Governor-based rate limiting
├── sso.rs               — OIDC SSO flow, token exchange, OIDC cache
├── sso_client.rs        — OpenID Connect client wrapper
├── http_client.rs       — Shared reqwest HTTP client configuration
├── util.rs              — Helpers: UUIDs, dates, headers, CORS, logging fairings
├── api/
│   ├── mod.rs           — Route re-exports, common request/response types
│   ├── admin.rs         — Admin panel API (protected by Argon2id token)
│   ├── identity.rs      — /identity: login, token refresh, register
│   ├── icons.rs         — /icons: website favicon fetching & caching
│   ├── notifications.rs — WebSocket hub for real-time sync
│   ├── push.rs          — Mobile push notification relay (relay-based)
│   ├── web.rs           — Static file serving (web vault + embedded resources)
│   └── core/
│       ├── mod.rs
│       ├── accounts.rs  — User profile, email change, password change, keys
│       ├── ciphers.rs   — Vault item CRUD, bulk operations, sync
│       ├── folders.rs   — Folder management
│       ├── organizations.rs — Org management, members, collections, groups
│       ├── sends.rs     — Bitwarden Send (encrypted file/text sharing)
│       ├── emergency_access.rs
│       ├── events.rs    — Organisation event log
│       ├── public.rs    — Public organisation API (directory connector)
│       └── two_factor/
│           ├── authenticator.rs  — TOTP (RFC 6238)
│           ├── email.rs          — Email OTP
│           ├── duo.rs / duo_oidc.rs — Duo Security (legacy iframe + OIDC)
│           ├── webauthn.rs       — FIDO2/WebAuthn (webauthn-rs)
│           ├── yubikey.rs        — YubiKey OTP (yubico_ng)
│           └── protected_actions.rs — Per-action OTP verification
└── db/
    ├── mod.rs           — Pool creation, migrations, backup helpers
    ├── query_logger.rs  — Configurable SQL query logger
    ├── schema.rs        — Diesel table! macros (auto-generated from migrations)
    └── models/
        ├── user.rs          — User, Invitation, SsoUser
        ├── cipher.rs        — Cipher (vault items)
        ├── organization.rs  — Organization, Membership, OrganizationApiKey
        ├── collection.rs    — Collection, CollectionCipher, CollectionUser
        ├── group.rs         — Group, GroupUser, CollectionGroup
        ├── folder.rs        — Folder, FolderCipher
        ├── attachment.rs    — File attachment metadata
        ├── send.rs          — Send item (text/file)
        ├── device.rs        — Registered client devices, push tokens
        ├── two_factor.rs    — TwoFactor records per user
        ├── two_factor_incomplete.rs
        ├── two_factor_duo_context.rs
        ├── emergency_access.rs
        ├── event.rs         — Audit/event log entries
        ├── org_policy.rs    — Organisation policy enforcement
        ├── auth_request.rs  — Passwordless auth request flow
        ├── favorite.rs      — Cipher–User favorite mapping
        └── sso_nonce.rs     — OIDC PKCE/nonce state
```

---

## 4. HTTP Route Mapping

| Mount Point       | Module                  | Purpose                                      |
|-------------------|-------------------------|----------------------------------------------|
| `/`               | `api::web`              | Static web vault assets, error catchers      |
| `/api`            | `api::core`             | Main Bitwarden REST API                      |
| `/events`         | `api::core` (events)    | Organisation event log API                   |
| `/identity`       | `api::identity`         | Authentication (login, token refresh, register) |
| `/icons`          | `api::icons`            | Website favicon proxy                        |
| `/notifications`  | `api::notifications`    | WebSocket real-time notification hub         |
| `/admin`          | `api::admin`            | Vaultwarden admin panel                      |

Request body limits:
- JSON: 20 MB (supports large vault imports)
- File uploads (Send/Attachments): 525 MB

---

## 5. Authentication & Authorization

### 5.1 JWT Tokens (RS256)

All tokens are signed with a 2048-bit RSA key pair (auto-generated on first launch, stored via OpenDAL).

| Token Type          | Issuer Suffix              | Validity        |
|---------------------|----------------------------|-----------------|
| Login (access)      | `|login`                   | 2 hours         |
| Refresh             | `|login`                   | 30d (desktop/web), 90d (mobile) |
| Invite              | `|invite`                  | Single-use      |
| Emergency Access    | `|emergencyaccessinvite`   | Single-use      |
| Account Delete      | `|delete`                  | Single-use      |
| Email Verify        | `|verifyemail`             | Single-use      |
| Admin Panel         | `|admin`                   | Session-scoped  |
| Send Access         | `|send`                    | Single-use      |
| Org API Key         | `|api.organization`        | Configured      |
| File Download       | `|file_download`           | Single-use      |
| Register Verify     | `|register_verify`         | Single-use      |
| SSO                 | `|sso`                     | 10-minute OIDC flow |

### 5.2 Admin Authentication

The admin panel is protected by an **Argon2id PHC string** stored in `ADMIN_TOKEN`. The built-in `vaultwarden hash` CLI command generates these tokens using two presets:

| Preset           | Memory  | Iterations | Threads |
|------------------|---------|------------|---------|
| bitwarden (default) | 64 MiB | 3       | 4       |
| owasp            | 19 MiB  | 2          | 1       |

### 5.3 Protected Actions

Sensitive operations (e.g., disabling 2FA, exporting vault) require re-validation via either:
1. Master password hash, or
2. A short-lived email OTP (single-use by default)

### 5.4 Rate Limiting

Uses the [governor](https://crates.io/crates/governor) crate. Applied per-IP to sensitive endpoints (login, 2FA, etc.).

---

## 6. Data Models

### 6.1 User

```
User {
  uuid, enabled, created_at, updated_at,
  verified_at, last_verifying_at, login_verify_count,
  email, email_new, email_new_token, name,
  password_hash, salt, password_iterations, password_hint,
  akey,              -- encrypted symmetric key (client-encrypted)
  private_key,       -- encrypted RSA private key (client-encrypted)
  public_key,        -- RSA public key (plaintext)
  totp_recover,
  security_stamp, stamp_exception,
  equivalent_domains, excluded_globals,
  client_kdf_type, client_kdf_iter, client_kdf_memory, client_kdf_parallelism,
  api_key, avatar_color, external_id
}
```

KDF types supported: PBKDF2 and Argon2id (client-side key derivation).

### 6.2 Cipher (Vault Item)

```
Cipher {
  uuid, created_at, updated_at,
  user_uuid OR organization_uuid,
  key,      -- individual cipher key (for key rotation)
  atype: Login=1 | SecureNote=2 | Card=3 | Identity=4 | SshKey=5,
  name,     -- encrypted
  notes,    -- encrypted
  fields,   -- encrypted custom fields JSON
  data,     -- encrypted type-specific data JSON
  password_history,  -- encrypted
  deleted_at,        -- soft-delete / trash
  reprompt: None=0 | Password=1
}
```

### 6.3 Organization & Membership

```
Organization { uuid, name, billing_email, private_key, public_key }

Membership {
  uuid, user_uuid, org_uuid,
  invited_by_email,
  access_all, akey,
  status: Revoked=-1 | Invited=0 | Accepted=1 | Confirmed=2,
  atype: Owner=0 | Admin=1 | User=2 | Manager=3 | Custom=4,
  reset_password_key, external_id
}
```

### 6.4 Two-Factor Authentication

```
TwoFactor {
  uuid, user_uuid,
  atype: Authenticator=0 | Email=1 | Duo=2 | YubiKey=3 | U2f=4 |
         Remember=5 | OrganizationDuo=6 | Webauthn=7 | RecoveryCode=8,
  enabled, data, last_used
}
```

Internal challenge types (atype ≥ 1000) are used during registration/login flow only.

### 6.5 Send

```
Send {
  uuid, user_uuid?, organization_uuid?,
  name, notes, akey,
  type: Text=0 | File=1,
  data,            -- encrypted
  file_id,         -- OpenDAL path for File sends
  max_access_count, access_count,
  expiration_date, deletion_date,
  password_hash, password_salt, password_iter,
  disabled, hide_email
}
```

### 6.6 Other Models

| Model                   | Purpose                                                  |
|-------------------------|----------------------------------------------------------|
| `Device`                | Registered client devices; stores push token (PushId)   |
| `Attachment`            | File attachment metadata; file stored via OpenDAL        |
| `Collection`            | Org-level grouping of ciphers                            |
| `Group`                 | Org-level user group for collection access control       |
| `OrgPolicy`             | Policy enforcement (MasterPassword, SingleOrg, etc.)     |
| `EmergencyAccess`       | Grantor/grantee relationship, status, type, wait time    |
| `Event`                 | Audit log entry with type, user, org, cipher, device     |
| `AuthRequest`           | Passwordless/device authentication request               |
| `SsoNonce`              | OIDC PKCE state tracking (nonce, code_challenge, state)  |
| `TwoFactorDuoContext`   | Temporary storage for Duo OIDC auth contexts             |
| `TwoFactorIncomplete`   | Tracks logins that completed password but not 2FA        |

---

## 7. Database Layer

### 7.1 Supported Backends

| Backend        | Feature Flag  | Notes                                              |
|----------------|---------------|----------------------------------------------------|
| SQLite         | `sqlite`      | Default for small deployments; bundled via libsqlite3-sys |
| PostgreSQL     | `postgresql`  | Recommended for production multi-user deployments  |
| MySQL/MariaDB  | `mysql`       | Pinned to diesel 2.3.3 for compatibility           |

### 7.2 Connection Pool

Uses Diesel `r2d2` connection pool. Retry logic wraps pool creation with configurable `DB_CONNECTION_RETRIES`.

### 7.3 Migrations

Managed by `diesel_migrations`, applied on startup. Migration directories:
- `migrations/sqlite/`
- `migrations/postgresql/`
- `migrations/mysql/`

Migration history spans 2018–2025, covering all features incrementally (collections, orgs, groups, events, SSO, Duo OIDC, etc.).

### 7.4 SQLite Backup

SQLite-only feature: triggered via:
- CLI: `vaultwarden backup`
- Unix signal: `SIGUSR1`
- Cron schedule: `SEND_PURGE_SCHEDULE` (configurable)

---

## 8. File Storage (OpenDAL)

Files (attachments, Send files, RSA key) are accessed through [Apache OpenDAL](https://opendal.apache.org/):

| Path Type   | Default Location         | S3 Support |
|-------------|--------------------------|------------|
| Data folder | `data/`                  | Yes (`s3://…`) |
| Attachments | `data/attachments/`      | Inherits   |
| Sends       | `data/sends/`            | Inherits   |
| RSA Key     | `data/rsa_key.pem`       | Inherits   |

S3 support is enabled via the `s3` feature flag, which pulls in `opendal/services-s3`, `aws-config`, `aws-credential-types`, and `reqsign`.

---

## 9. Notification System

### 9.1 WebSocket (Real-Time Sync)

- Endpoint: `/notifications/hub` and `/notifications/anonymous`
- Protocol: MessagePack over WebSocket (via `rmpv`)
- State: `DashMap<UserId, Vec<(uuid, Sender<Message>)>>` — concurrent multi-device per user
- Authentication: Bearer token in `?access_token=` query param or `Authorization` header
- Disabled by default; enabled via `ENABLE_WEBSOCKET=true`

Update types propagated:
`SyncCipherUpdate`, `SyncCipherCreate`, `SyncLoginDelete`, `SyncFolderDelete`, `SyncCiphers`, `SyncVault`, `SyncOrgKeys`, `SyncFolderCreate`, `SyncFolderUpdate`, `SyncCipherDelete`, `SyncSettings`, `LogOut`, `SyncSendCreate`, `SyncSendUpdate`, `SyncSendDelete`, `AuthRequest`, `AuthRequestResponse`

### 9.2 Mobile Push Notifications

Delegated to a configurable push relay server (`PUSH_RELAY_URI`). The relay handles APNs/FCM delivery. Each device registers a `PushId` (UUID) at login. Push events are fired for the same update types as WebSocket.

---

## 10. OIDC / SSO Integration

Enabled via `SSO_ENABLED=true`. Uses the `openidconnect` crate.

### Flow

1. Client requests SSO login → `/identity/connect/auth?sso=1`
2. Server generates PKCE code challenge + nonce, stores in `SsoNonce` DB table
3. Client redirected to IdP
4. IdP redirects back with authorization code
5. Server exchanges code for tokens via `sso_client.rs`
6. Authenticated user looked up or auto-provisioned
7. Vaultwarden JWT issued and returned to client

### Cache

`AC_CACHE: Cache<OIDCState, AuthenticatedUser>` (mini-moka) — TTL 10 minutes, capacity 1000 entries — bridges the OIDC callback to the Bitwarden client token request.

### Nonce Cleanup

Expired SSO nonces are purged on schedule (default: daily at 00:20, `PURGE_INCOMPLETE_SSO_NONCE`).

---

## 11. Background Job Scheduler

Runs in a dedicated OS thread (`job-scheduler`). Uses `job_scheduler_ng` with cron expressions.

| Job                              | Config Key                              | Default Schedule     |
|----------------------------------|-----------------------------------------|----------------------|
| Purge expired Sends              | `SEND_PURGE_SCHEDULE`                   | Every hour           |
| Purge trashed ciphers            | `TRASH_PURGE_SCHEDULE`                  | Daily                |
| Incomplete 2FA notifications     | `INCOMPLETE_2FA_SCHEDULE`               | Every minute         |
| Emergency access timeout grant   | `EMERGENCY_REQUEST_TIMEOUT_SCHEDULE`    | Every hour           |
| Emergency access reminders       | `EMERGENCY_NOTIFICATION_REMINDER_SCHEDULE` | Daily             |
| Purge expired auth requests      | `AUTH_REQUEST_PURGE_SCHEDULE`           | Every minute         |
| Purge Duo contexts               | `DUO_CONTEXT_PURGE_SCHEDULE`            | Every 15 minutes     |
| Event log cleanup                | `EVENT_CLEANUP_SCHEDULE`               | Weekly (if enabled)  |
| Purge incomplete SSO nonces      | `PURGE_INCOMPLETE_SSO_NONCE`            | Daily at 00:20       |

Poll interval: `JOB_POLL_INTERVAL_MS` (default: 30,000 ms). Set to `0` to disable the scheduler entirely.

---

## 12. Configuration System

Configuration is loaded from (in priority order):
1. Environment variables
2. `{DATA_FOLDER}/config.json` (editable via admin panel)

A macro (`make_config!`) auto-generates the `Config` struct, `ConfigBuilder`, deserializer, and display logic from a declarative DSL. Passwords are masked (`***`) in display/API output.

Key configuration groups:
- **Domain**: `DOMAIN`, `DOMAIN_ORIGIN`, `DOMAIN_PATH`
- **Database**: `DATABASE_URL`, `DB_CONNECTION_RETRIES`, `DATABASE_TIMEOUT`
- **Features**: `WEB_VAULT_ENABLED`, `SENDS_ALLOWED`, `ORG_EVENTS_ENABLED`, etc.
- **Email (SMTP)**: `SMTP_HOST`, `SMTP_PORT`, `SMTP_FROM`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_DEBUG`
- **Authentication**: `SIGNUPS_ALLOWED`, `SIGNUPS_VERIFY`, `INVITATION_ORG_NAME`
- **Push**: `PUSH_RELAY_URI`, `PUSH_IDENTITY_URI`, `PUSH_ENABLED`
- **SSO/OIDC**: `SSO_ENABLED`, `SSO_AUTHORITY`, `SSO_CLIENT_ID`, `SSO_CLIENT_SECRET`
- **Admin**: `ADMIN_TOKEN`, `DISABLE_ADMIN_TOKEN`, `ADMIN_SESSION_LIFETIME`
- **Storage**: `DATA_FOLDER`, `ATTACHMENTS_FOLDER`, `SENDS_FOLDER`
- **Logging**: `LOG_LEVEL`, `LOG_FILE`, `EXTENDED_LOGGING`, `LOG_TIMESTAMP_FORMAT`
- **Scheduling**: See §11

---

## 13. Email Subsystem

Uses [lettre](https://lettre.rs) with:
- Transports: SMTP (with STARTTLS/TLS), Sendmail
- TLS: `rustls` with native root certificates
- Templates: Handlebars (`.hbs` files in `src/static/templates/`)
- Emails: invitation, verification, 2FA incomplete alert, emergency access, etc.

---

## 14. Security Design

### End-to-End Encryption

All vault data is encrypted **client-side** before reaching the server. The server stores only encrypted blobs. The server never sees plaintext vault items, passwords, or personal data.

- **Symmetric key**: derived from master password using PBKDF2 or Argon2id (client-side)
- **Vault encryption**: AES-256-CBC or AES-256-GCM with HMAC (client-chosen)
- **Asymmetric key pair**: RSA-2048, private key encrypted with symmetric key, public key stored plaintext for org key sharing

### Code Safety

- `#![forbid(unsafe_code)]` — no unsafe Rust in the codebase
- `#![forbid(non_ascii_idents)]` — prevents homograph attacks
- Comprehensive Clippy deny-list enforced at workspace level

### Input Limits

- JSON payload: 20 MB
- File uploads: 525 MB
- URL-encoded forms: standard Rocket limits

---

## 15. Deployment

### Container (Recommended)

```bash
docker run --detach --name vaultwarden \
  --env DOMAIN="https://vw.domain.tld" \
  --volume /vw-data/:/data/ \
  --restart unless-stopped \
  --publish 127.0.0.1:8000:80 \
  vaultwarden/server:latest
```

Images published to: `ghcr.io`, `docker.io`, `quay.io`.

### Build Profiles

| Profile         | Use Case                                   |
|-----------------|--------------------------------------------|
| `release`       | Standard production (fat LTO, 1 CGU)       |
| `release-micro` | Minimal binary size (opt-level z, no debug)|
| `release-low`   | Low-resource build machines (thin LTO)     |
| `dbg`           | Profiling (full debug symbols + release opts)|
| `ci`            | Fast CI builds (no debug assertions)       |

### Memory Allocator

Alpine/musl builds can enable MiMalloc (`enable_mimalloc` feature) to work around the slow default musl malloc.

---

## 16. Key Dependencies Summary

| Crate                  | Purpose                                      |
|------------------------|----------------------------------------------|
| `rocket` 0.5.1         | Async web framework                          |
| `diesel` 2.3.3         | ORM + query builder                          |
| `tokio` 1.48           | Async runtime (multi-thread)                 |
| `jsonwebtoken` 10.2    | JWT encoding/decoding (RS256)                |
| `openssl` 0.10         | RSA key generation                           |
| `ring` 0.17            | Cryptographic primitives                     |
| `argon2` 0.5.3         | Argon2id password hashing (admin token)      |
| `webauthn-rs` 0.5.3    | FIDO2/WebAuthn registration and authentication |
| `openidconnect` 4.0.1  | OIDC/SSO client                              |
| `lettre` 0.11          | Email sending                                |
| `opendal` 0.55         | Unified file storage (local + S3)            |
| `reqwest` 0.12         | HTTP client (favicons, HIBP, Duo, push)      |
| `hickory-resolver`     | DNS resolution (with optional IPv6 preference) |
| `handlebars` 6.3       | Email and HTML templates                     |
| `dashmap` 6.1          | Concurrent hashmap for WebSocket sessions    |
| `governor` 0.10        | Rate limiting                                |
| `mini-moka` 0.10       | In-memory cache (OIDC state)                 |
| `job_scheduler_ng` 2.4 | Cron-based background jobs                   |
| `totp-lite` 2.0        | TOTP/HOTP generation                         |
| `yubico_ng` 0.14       | YubiKey OTP verification                     |
| `rmpv` 1.3             | MessagePack serialization (WebSocket)        |
| `fern` 0.7             | Logging dispatcher                           |
