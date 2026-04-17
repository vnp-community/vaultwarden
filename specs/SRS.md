# Vaultwarden — Software Requirements Specification (SRS)

> **Document Version**: 1.0  
> **Date**: 2026-04-10  
> **Status**: Draft  
> **Reference**: Technical Design Document (`technical-design.md`)  
> **Source Project**: `dani-garcia/vaultwarden` — Rust implementation of the Bitwarden server API

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Overall Description](#2-overall-description)
3. [Stakeholders & User Classes](#3-stakeholders--user-classes)
4. [Functional Requirements](#4-functional-requirements)
   - 4.1 [Authentication & Session Management](#41-authentication--session-management)
   - 4.2 [User Account Management](#42-user-account-management)
   - 4.3 [Vault Item (Cipher) Management](#43-vault-item-cipher-management)
   - 4.4 [Organization & Team Management](#44-organization--team-management)
   - 4.5 [Bitwarden Send](#45-bitwarden-send)
   - 4.6 [File Attachments](#46-file-attachments)
   - 4.7 [Two-Factor Authentication (2FA)](#47-two-factor-authentication-2fa)
   - 4.8 [Emergency Access](#48-emergency-access)
   - 4.9 [Admin Panel](#49-admin-panel)
   - 4.10 [Real-Time Notifications](#410-real-time-notifications)
   - 4.11 [Mobile Push Notifications](#411-mobile-push-notifications)
   - 4.12 [OIDC / SSO Integration](#412-oidc--sso-integration)
   - 4.13 [Event Logging & Audit](#413-event-logging--audit)
   - 4.14 [Email Subsystem](#414-email-subsystem)
   - 4.15 [Background Scheduled Jobs](#415-background-scheduled-jobs)
   - 4.16 [Icon / Favicon Proxy](#416-icon--favicon-proxy)
5. [Non-Functional Requirements](#5-non-functional-requirements)
   - 5.1 [Security](#51-security)
   - 5.2 [Performance & Scalability](#52-performance--scalability)
   - 5.3 [Reliability & Availability](#53-reliability--availability)
   - 5.4 [Compatibility](#54-compatibility)
   - 5.5 [Maintainability & Operability](#55-maintainability--operability)
   - 5.6 [Deployment](#56-deployment)
6. [Data Requirements](#6-data-requirements)
7. [Interface Requirements](#7-interface-requirements)
8. [Constraints & Assumptions](#8-constraints--assumptions)
9. [Glossary](#9-glossary)

---

## 1. Introduction

### 1.1 Purpose

This Software Requirements Specification (SRS) defines the complete functional and non-functional requirements for **Vaultwarden**, a self-hosted, lightweight reimplementation of the Bitwarden server API written in Rust. This document serves as the authoritative reference for developers, operators, and QA engineers building, maintaining, or testing the system.

### 1.2 Scope

Vaultwarden provides:

- A fully API-compatible replacement for the official Bitwarden server.
- Support for all official Bitwarden clients: web vault, desktop applications, mobile applications, and browser extensions.
- Features including end-to-end encrypted vault management, multi-factor authentication, organization/team management, Bitwarden Send, emergency access, admin control panel, real-time sync, SSO/OIDC, and event auditing.

**Out of scope:**

- Bitwarden premium/enterprise billing and subscription management.
- The Bitwarden Directory Connector (partially supported via the public API only).
- Client-side UI implementation (delegated to official Bitwarden clients).

### 1.3 Definitions & Abbreviations

| Term | Definition |
|------|-----------|
| **Cipher** | A vault item (login, card, identity, secure note, or SSH key) |
| **KDF** | Key Derivation Function (PBKDF2 or Argon2id) |
| **JWT** | JSON Web Token |
| **OIDC** | OpenID Connect |
| **2FA / MFA** | Two-Factor / Multi-Factor Authentication |
| **Send** | Bitwarden's encrypted file/text sharing feature |
| **Organization** | A group/team sharing vault items via collections |
| **Collection** | A logical grouping of ciphers within an organization |
| **OpenDAL** | Apache OpenDAL — unified file access abstraction layer |
| **AGPL** | GNU Affero General Public License |

### 1.4 References

- Technical Design Document: `specs/technical-design.md`
- Bitwarden API Specification: https://bitwarden.com/help/
- Rocket Web Framework: https://rocket.rs
- Diesel ORM: https://diesel.rs

---

## 2. Overall Description

### 2.1 Product Perspective

Vaultwarden is a **self-hosted server** that acts as the API backend for Bitwarden clients. It is a drop-in replacement for the official Bitwarden server, targeting individuals, homelabs, and small teams who require a manageable, resource-efficient deployment.

```
[Bitwarden Clients]  ←→  [Reverse Proxy]  ←→  [Vaultwarden Server]  ←→  [Database / File Storage]
```

### 2.2 Product Functions (Summary)

- User account registration, authentication, and management
- End-to-end encrypted vault (cipher) CRUD operations
- Organization, collection, group, and membership management
- Bitwarden Send (encrypted content sharing)
- Multi-factor authentication (TOTP, WebAuthn, YubiKey, Email OTP, Duo)
- Emergency access (account recovery delegation)
- Real-time vault sync via WebSocket
- Mobile push notification relay
- OIDC-based Single Sign-On
- Admin panel for server management
- Audit/event logging
- Background maintenance jobs

### 2.3 Operating Environment

- **Server OS**: Linux (primary), macOS, Windows (via Docker)
- **Deployment**: Docker container (recommended) or native binary
- **Proxy Layer**: nginx, Caddy, or any HTTPS-terminating proxy
- **Database**: SQLite (default), PostgreSQL (production recommended), MySQL/MariaDB
- **File Storage**: Local filesystem or S3-compatible object storage

### 2.4 Assumptions & Dependencies

- Clients communicate exclusively over HTTPS.
- The operator is responsible for TLS termination at the reverse proxy layer.
- Client-side encryption is handled entirely by official Bitwarden clients.
- The server never stores or processes plaintext vault data.

---

## 3. Stakeholders & User Classes

| User Class | Description | Access Level |
|------------|-------------|--------------|
| **End User** | Individual using Bitwarden clients to manage personal vault | Standard user API |
| **Organization Owner** | Manages a Bitwarden organization (users, collections, policies) | Full org-level admin |
| **Organization Admin** | Can manage members and collections | Org admin sub-level |
| **Organization Manager** | Manages assigned collections | Collection-scoped |
| **Organization User** | Regular member of an org | Assigned collections only |
| **Server Administrator** | Manages the Vaultwarden instance (admin panel) | Admin panel + all APIs |
| **Emergency Grantee** | User delegated emergency access to another user's vault | Conditional view/takeover |

---

## 4. Functional Requirements

### 4.1 Authentication & Session Management

#### 4.1.1 Token-Based Authentication

**FR-AUTH-001**: The system **shall** authenticate users using JWT tokens signed with RS256 (2048-bit RSA key pair).

**FR-AUTH-002**: RSA key pairs **shall** be auto-generated on first launch and persisted via the configured file storage (OpenDAL).

**FR-AUTH-003**: The system **shall** issue the following token types with the specified validity windows:

| Token Type | Validity |
|------------|----------|
| Access (login) | 2 hours |
| Refresh (desktop/web) | 30 days |
| Refresh (mobile) | 90 days |
| Invite | Single-use |
| Emergency Access Invite | Single-use |
| Account Delete | Single-use |
| Email Verify | Single-use |
| Admin Panel | Session-scoped |
| Send Access | Single-use |
| Org API Key | Configurable |
| File Download | Single-use |
| Register Verify | Single-use |
| SSO | 10-minute OIDC flow |

**FR-AUTH-004**: The system **shall** support passwordless / device-based authentication via the `AuthRequest` flow.

**FR-AUTH-005**: The system **shall** enforce re-authentication for protected actions (e.g., disabling 2FA, exporting vault) using either:
- Master password hash verification, **or**
- A short-lived email OTP (single-use by default).

#### 4.1.2 Rate Limiting

**FR-AUTH-006**: The system **shall** apply per-IP rate limiting to sensitive endpoints including login, 2FA, and registration using the `governor` library.

**FR-AUTH-007**: Rate limiting parameters **shall** be configurable by the server administrator.

#### 4.1.3 Client Registration

**FR-AUTH-008**: The system **shall** register client devices upon login, storing device metadata and push tokens.

---

### 4.2 User Account Management

**FR-USER-001**: The system **shall** support user registration with configurable policies:
- Open registration (`SIGNUPS_ALLOWED=true`)
- Invitation-only registration
- Domain-restricted registration
- Email-verified registration (`SIGNUPS_VERIFY=true`)

**FR-USER-002**: The system **shall** allow users to:
- Change their email address (requires re-verification)
- Change their master password
- Change their security key (akey / symmetric key)
- View and update their profile

**FR-USER-003**: The system **shall** support KDF configuration per user:
- PBKDF2 (legacy)
- Argon2id (preferred)

**FR-USER-004**: The system **shall** support account deletion via a secure, single-use delete token sent to the user's registered email.

**FR-USER-005**: The system **shall** allow users to set equivalent domains and excluded global domains.

**FR-USER-006**: The system **shall** support user API keys for personal programmatic access.

**FR-USER-007**: The system **shall** track security stamps to invalidate existing sessions upon critical account changes.

---

### 4.3 Vault Item (Cipher) Management

**FR-CIPHER-001**: The system **shall** support the following vault item types:

| Type | ID |
|------|----|
| Login | 1 |
| Secure Note | 2 |
| Card | 3 |
| Identity | 4 |
| SSH Key | 5 |

**FR-CIPHER-002**: The system **shall** support full CRUD (Create, Read, Update, Delete) operations on vault items.

**FR-CIPHER-003**: The system **shall** support soft-delete (trash) for vault items, with a configurable auto-purge schedule.

**FR-CIPHER-004**: The system **shall** support bulk operations on multiple vault items simultaneously.

**FR-CIPHER-005**: The system **shall** support vault sync — returning all ciphers, folders, and settings in a single API response.

**FR-CIPHER-006**: The system **shall** support assigning vault items to organizations and specific collections.

**FR-CIPHER-007**: The system **shall** support per-cipher re-prompt (password re-entry required before viewing).

**FR-CIPHER-008**: The system **shall** support password history per vault item.

**FR-CIPHER-009**: The system **shall** store all vault item data in client-encrypted form (the server shall never store plaintext).

**FR-CIPHER-010**: The system **shall** support folder management (create, rename, delete) and association of ciphers to folders.

---

### 4.4 Organization & Team Management

**FR-ORG-001**: The system **shall** support creating and managing organizations.

**FR-ORG-002**: The system **shall** support the following membership roles within an organization:

| Role | atype |
|------|-------|
| Owner | 0 |
| Admin | 1 |
| User | 2 |
| Manager | 3 |
| Custom | 4 |

**FR-ORG-003**: The system **shall** track membership status:

| Status | Value |
|--------|-------|
| Revoked | -1 |
| Invited | 0 |
| Accepted | 1 |
| Confirmed | 2 |

**FR-ORG-004**: The system **shall** support collections — org-scoped groupings of vault items with granular user/group access.

**FR-ORG-005**: The system **shall** support groups — org-level grouping of users for collection access control.

**FR-ORG-006**: The system **shall** support organization-level policies:
- Master Password strength enforcement
- Single Org enforcement
- Two-Factor requirement
- Password reset / admin recovery

**FR-ORG-007**: The system **shall** support organization API keys for service account access.

**FR-ORG-008**: The system **shall** support inviting users to organizations via email.

**FR-ORG-009**: The system **shall** support resetting a member's master password (admin-initiated recovery).

**FR-ORG-010**: The system **shall** expose a public organization API compatible with the Bitwarden Directory Connector.

---

### 4.5 Bitwarden Send

**FR-SEND-001**: The system **shall** support Bitwarden Send for sharing encrypted content:
- Text sends (type 0)
- File sends (type 1)

**FR-SEND-002**: The system **shall** enforce configurable access controls per send:
- Maximum access count
- Expiration date
- Deletion date
- Password protection (Argon2id hashed)
- Sender email visibility toggle

**FR-SEND-003**: The system **shall** automatically purge expired sends on schedule.

**FR-SEND-004**: The system **shall** store send files via the OpenDAL file storage layer.

**FR-SEND-005**: The system administrator **shall** be able to disable all Send functionality via `SENDS_ALLOWED=false`.

---

### 4.6 File Attachments

**FR-ATTACH-001**: The system **shall** support file attachments for vault items.

**FR-ATTACH-002**: The system **shall** store attachment files via the OpenDAL layer (local filesystem or S3).

**FR-ATTACH-003**: File attachment downloads **shall** be authorized via single-use file download tokens.

**FR-ATTACH-004**: File upload size **shall** be enforced at a maximum of **525 MB** per upload.

---

### 4.7 Two-Factor Authentication (2FA)

**FR-2FA-001**: The system **shall** support the following 2FA methods:

| Method | atype |
|--------|-------|
| TOTP Authenticator (RFC 6238) | 0 |
| Email OTP | 1 |
| Duo Security (legacy iframe) | 2 |
| YubiKey OTP | 3 |
| FIDO2/WebAuthn | 7 |
| Recovery Code | 8 |
| Duo OIDC | 6 (org-level) |

**FR-2FA-002**: The system **shall** support TOTP generation compliant with RFC 6238.

**FR-2FA-003**: The system **shall** support FIDO2/WebAuthn registration and authentication.

**FR-2FA-004**: The system **shall** support YubiKey OTP verification.

**FR-2FA-005**: The system **shall** support Duo Security integration via both the legacy iframe flow and the modern OIDC flow.

**FR-2FA-006**: The system **shall** support organization-level Duo enforcement.

**FR-2FA-007**: The system **shall** send email alerts when 2FA login is incomplete (i.e., the user passed the password step but did not complete 2FA).

**FR-2FA-008**: The system **shall** support 2FA "remember" functionality to bypass 2FA on trusted devices.

**FR-2FA-009**: The system **shall** support multi-step protected action verification (OTP re-validation for sensitive operations).

---

### 4.8 Emergency Access

**FR-EMRG-001**: The system **shall** support emergency access delegation between users (grantor/grantee).

**FR-EMRG-002**: The system **shall** support two emergency access types:
- **View**: Grantee can view the grantor's vault after wait time.
- **Takeover**: Grantee can reset and assume the grantor's account after wait time.

**FR-EMRG-003**: The system **shall** enforce a configurable wait time before access is granted.

**FR-EMRG-004**: The system **shall** send reminder notifications to the grantor before emergency access is granted.

**FR-EMRG-005**: The system **shall** automatically approve emergency access requests after the wait period via a background job.

**FR-EMRG-006**: Emergency access invitations **shall** be delivered via email as single-use tokens.

---

### 4.9 Admin Panel

**FR-ADMIN-001**: The system **shall** provide a web-based admin panel accessible at `/admin`.

**FR-ADMIN-002**: The admin panel **shall** be protected by an Argon2id-hashed token (`ADMIN_TOKEN`), generated using the built-in `vaultwarden hash` CLI command.

**FR-ADMIN-003**: The admin panel **shall** support the following Argon2id presets:

| Preset | Memory | Iterations | Threads |
|--------|--------|------------|---------|
| bitwarden (default) | 64 MiB | 3 | 4 |
| owasp | 19 MiB | 2 | 1 |

**FR-ADMIN-004**: The admin panel **shall** allow the administrator to:
- View and manage all registered users
- Invite users
- Override configuration settings (persisted to `config.json`)
- List and manage organizations
- Trigger SQLite database backup
- View diagnostics and server info

**FR-ADMIN-005**: The system **shall** support disabling the admin token requirement (`DISABLE_ADMIN_TOKEN`) for environments using external access controls.

**FR-ADMIN-006**: Admin panel session lifetime **shall** be configurable via `ADMIN_SESSION_LIFETIME`.

---

### 4.10 Real-Time Notifications

**FR-NOTIF-001**: The system **shall** provide a WebSocket endpoint for real-time vault synchronization at `/notifications/hub`.

**FR-NOTIF-002**: The system **shall** provide an anonymous notification endpoint at `/notifications/anonymous`.

**FR-NOTIF-003**: WebSocket notifications **shall** use the MessagePack binary protocol.

**FR-NOTIF-004**: The system **shall** support concurrent multi-device sessions for a single user.

**FR-NOTIF-005**: WebSocket authentication **shall** accept Bearer tokens via query parameter (`?access_token=`) or `Authorization` header.

**FR-NOTIF-006**: The WebSocket notification system **shall** be disabled by default and enabled via `ENABLE_WEBSOCKET=true`.

**FR-NOTIF-007**: The following update event types **shall** be propagated via WebSocket:

`SyncCipherUpdate`, `SyncCipherCreate`, `SyncLoginDelete`, `SyncFolderDelete`, `SyncCiphers`, `SyncVault`, `SyncOrgKeys`, `SyncFolderCreate`, `SyncFolderUpdate`, `SyncCipherDelete`, `SyncSettings`, `LogOut`, `SyncSendCreate`, `SyncSendUpdate`, `SyncSendDelete`, `AuthRequest`, `AuthRequestResponse`

---

### 4.11 Mobile Push Notifications

**FR-PUSH-001**: The system **shall** relay push notifications to a configurable external relay server (`PUSH_RELAY_URI`) which handles APNs (Apple) and FCM (Google) delivery.

**FR-PUSH-002**: Each device **shall** be assigned a Push ID (UUID) upon login to enable targeted push delivery.

**FR-PUSH-003**: Push notifications **shall** cover the same event types as WebSocket notifications.

**FR-PUSH-004**: The push notification subsystem **shall** be independently toggleable via `PUSH_ENABLED`.

---

### 4.12 OIDC / SSO Integration

**FR-SSO-001**: The system **shall** support OpenID Connect (OIDC) based Single Sign-On when enabled via `SSO_ENABLED=true`.

**FR-SSO-002**: The SSO login flow **shall** follow this sequence:
1. Client requests SSO login → `/identity/connect/auth?sso=1`
2. Server generates PKCE code challenge + nonce, stores in `SsoNonce` DB table
3. Client is redirected to the Identity Provider (IdP)
4. IdP redirects back with authorization code
5. Server exchanges code for tokens
6. User is looked up or auto-provisioned
7. Vaultwarden JWT is issued and returned

**FR-SSO-003**: The system **shall** use PKCE (Proof Key for Code Exchange) to secure the OIDC authorization code flow.

**FR-SSO-004**: The system **shall** cache OIDC state for 10 minutes with a maximum of 1,000 concurrent states.

**FR-SSO-005**: The system **shall** purge expired SSO nonces on a configurable schedule (default: daily at 00:20).

**FR-SSO-006**: The system **shall** support the following SSO configuration parameters:
- `SSO_AUTHORITY` — IdP issuer URL
- `SSO_CLIENT_ID` — Client identifier
- `SSO_CLIENT_SECRET` — Client secret

---

### 4.13 Event Logging & Audit

**FR-EVENT-001**: The system **shall** support organization-level event logging when `ORG_EVENTS_ENABLED=true`.

**FR-EVENT-002**: Audit events **shall** capture: event type, acting user UUID, organization UUID, cipher UUID, device UUID, IP address, and timestamp.

**FR-EVENT-003**: Events **shall** be accessible via the `/events` API endpoint.

**FR-EVENT-004**: The system **shall** support periodic cleanup of old event log entries on a configurable schedule (`EVENT_CLEANUP_SCHEDULE`).

---

### 4.14 Email Subsystem

**FR-EMAIL-001**: The system **shall** support sending transactional emails via:
- SMTP (with STARTTLS or TLS support)
- Sendmail (local MTA)

**FR-EMAIL-002**: The system **shall** send emails for the following events:
- User invitation
- Email address verification
- Incomplete 2FA login alert
- Emergency access notifications (invitation, granted, rejected, reminder)
- Account deletion confirmation

**FR-EMAIL-003**: All email content **shall** be rendered from Handlebars (`.hbs`) templates.

**FR-EMAIL-004**: The following SMTP configuration parameters **shall** be supported:
`SMTP_HOST`, `SMTP_PORT`, `SMTP_FROM`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_DEBUG`

---

### 4.15 Background Scheduled Jobs

**FR-JOB-001**: The system **shall** run a background job scheduler using cron expressions to execute maintenance tasks.

**FR-JOB-002**: The following jobs **shall** be configurable via environment variables:

| Job | Config Key | Default |
|-----|-----------|---------|
| Purge expired Sends | `SEND_PURGE_SCHEDULE` | Every hour |
| Purge trashed ciphers | `TRASH_PURGE_SCHEDULE` | Daily |
| Incomplete 2FA notifications | `INCOMPLETE_2FA_SCHEDULE` | Every minute |
| Emergency access timeout grant | `EMERGENCY_REQUEST_TIMEOUT_SCHEDULE` | Every hour |
| Emergency access reminders | `EMERGENCY_NOTIFICATION_REMINDER_SCHEDULE` | Daily |
| Purge expired auth requests | `AUTH_REQUEST_PURGE_SCHEDULE` | Every minute |
| Purge Duo contexts | `DUO_CONTEXT_PURGE_SCHEDULE` | Every 15 minutes |
| Event log cleanup | `EVENT_CLEANUP_SCHEDULE` | Weekly (if enabled) |
| Purge incomplete SSO nonces | `PURGE_INCOMPLETE_SSO_NONCE` | Daily at 00:20 |

**FR-JOB-003**: The job scheduler poll interval **shall** be configurable via `JOB_POLL_INTERVAL_MS` (default: 30,000 ms).

**FR-JOB-004**: The scheduler **shall** be completely disableable by setting `JOB_POLL_INTERVAL_MS=0`.

---

### 4.16 Icon / Favicon Proxy

**FR-ICON-001**: The system **shall** provide a favicon proxy endpoint at `/icons/{domain}/icon.png`.

**FR-ICON-002**: The system **shall** cache fetched favicons to reduce external HTTP requests.

**FR-ICON-003**: The icon proxy **shall** be independently disableable by the administrator.

---

## 5. Non-Functional Requirements

### 5.1 Security

**NFR-SEC-001 (End-to-End Encryption)**: The server **shall never** store or transmit plaintext vault data. All vault items **shall** be encrypted client-side using:
- Symmetric key derived from master password via PBKDF2 or Argon2id (client-side)
- AES-256-CBC or AES-256-GCM with HMAC for symmetric encryption
- RSA-2048 for asymmetric operations (org key sharing)

**NFR-SEC-002 (No Unsafe Code)**: The Rust codebase **shall** enforce `#![forbid(unsafe_code)]` — no unsafe Rust blocks are permitted.

**NFR-SEC-003 (No Non-ASCII Identifiers)**: The codebase **shall** enforce `#![forbid(non_ascii_idents)]` to prevent homograph attacks.

**NFR-SEC-004 (Input Validation)**: The system **shall** enforce the following request body size limits:
- JSON payloads: **20 MB** maximum
- File uploads (attachments/sends): **525 MB** maximum

**NFR-SEC-005 (Admin Token)**: Admin panel tokens **shall** be stored and verified using Argon2id PHC strings only. Plain-text tokens are not supported.

**NFR-SEC-006 (Rate Limiting)**: Login, 2FA, and registration endpoints **shall** be rate-limited per client IP.

**NFR-SEC-007 (PKCE)**: SSO flows **shall** use PKCE to prevent authorization code interception attacks.

**NFR-SEC-008 (TLS)**: TLS termination is required at the reverse proxy layer. The internal server process does not handle TLS directly.

---

### 5.2 Performance & Scalability

**NFR-PERF-001**: The system **shall** use a multi-threaded async runtime (Tokio) to handle concurrent requests efficiently.

**NFR-PERF-002**: Database access **shall** use a connection pool (Diesel r2d2) with configurable pool size and connection retry.

**NFR-PERF-003**: WebSocket state **shall** use a concurrent lock-free hashmap (DashMap) to avoid contention under concurrent multi-device sessions.

**NFR-PERF-004**: In-memory caches (OIDC state) **shall** use TTL-based eviction with bounded capacity (1,000 entries, 10-minute TTL).

**NFR-PERF-005**: Alpine/musl Docker images **shall** optionally use MiMalloc as the system allocator to improve throughput on musl libc targets.

---

### 5.3 Reliability & Availability

**NFR-REL-001**: The system **shall** apply database migrations automatically on startup.

**NFR-REL-002**: The system **shall** retry database pool creation a configurable number of times (`DB_CONNECTION_RETRIES`) before failing.

**NFR-REL-003**: The system **shall** support SQLite database backup via CLI command, Unix signal (`SIGUSR1`), or cron schedule.

**NFR-REL-004**: Background jobs **shall** run in a dedicated OS thread isolated from the HTTP request handler pool.

---

### 5.4 Compatibility

**NFR-COMPAT-001**: The system **shall** be fully compatible with all official Bitwarden clients: web vault, desktop, mobile (iOS/Android), and browser extensions.

**NFR-COMPAT-002**: The system **shall** expose the Bitwarden REST API surface exactly as expected by official clients.

**NFR-COMPAT-003**: The system **shall** be compatible with the Bitwarden Directory Connector public API.

**NFR-COMPAT-004**: The minimum supported Rust edition is **2021** with MSRV **1.89.0**.

---

### 5.5 Maintainability & Operability

**NFR-OPS-001**: All configuration **shall** be manageable via environment variables and/or a `config.json` file editable through the admin panel, with environment variables taking priority.

**NFR-OPS-002**: The system **shall** support structured logging with configurable:
- Log level (`LOG_LEVEL`)
- Log file output (`LOG_FILE`)
- Extended logging (`EXTENDED_LOGGING`)
- Timestamp format (`LOG_TIMESTAMP_FORMAT`)
- SQL query logging (`DB_QUERY_LOGGER`)

**NFR-OPS-003**: Sensitive configuration values (passwords, API keys) **shall** be masked (`***`) in all log output and API responses.

**NFR-OPS-004**: The system **shall** expose Clippy lint checks at workspace level to enforce code quality.

---

### 5.6 Deployment

**NFR-DEPLOY-001**: The system **shall** be deployable as a Docker container using the official `vaultwarden/server:latest` image.

**NFR-DEPLOY-002**: Container images **shall** be published to `ghcr.io`, `docker.io`, and `quay.io`.

**NFR-DEPLOY-003**: The system **shall** support the following build profiles:

| Profile | Use Case |
|---------|---------|
| `release` | Standard production (fat LTO, 1 CGU) |
| `release-micro` | Minimal binary size (opt-level z, no debug) |
| `release-low` | Low-resource build machines (thin LTO) |
| `dbg` | Profiling (full debug symbols + release opts) |
| `ci` | Fast CI builds (no debug assertions) |

**NFR-DEPLOY-004**: The system **shall** bind by default to port **80** inside the container (exposed as `127.0.0.1:8000` in the example above).

---

## 6. Data Requirements

### 6.1 Supported Databases

| Backend | Feature Flag | Notes |
|---------|-------------|-------|
| SQLite | `sqlite` | Default; recommended for small/personal deployments |
| PostgreSQL | `postgresql` | Recommended for production multi-user deployments |
| MySQL/MariaDB | `mysql` | Pinned to Diesel 2.3.3 for compatibility |

### 6.2 Core Data Entities

| Entity | Description |
|--------|-------------|
| `User` | Account, credentials, KDF config, keys |
| `Cipher` | Encrypted vault item (login, card, note, identity, SSH key) |
| `Organization` | Multi-user org with shared vault |
| `Membership` | User ↔ Org relationship with role and status |
| `Collection` | Org-scoped grouping of ciphers |
| `Group` | Org-scoped grouping of users |
| `Folder` | Personal cipher grouping |
| `Attachment` | File metadata for cipher attachments |
| `Send` | Encrypted sharing item (text or file) |
| `Device` | Client device registration with push token |
| `TwoFactor` | Per-user 2FA method configuration |
| `EmergencyAccess` | Delegation relationship between two users |
| `Event` | Org-level audit log entry |
| `OrgPolicy` | Organization policy rules |
| `AuthRequest` | Passwordless auth initiation record |
| `SsoNonce` | OIDC PKCE/nonce state for SSO flow |

### 6.3 File Storage Paths

| Purpose | Default Path | S3 Supported |
|---------|-------------|--------------|
| Data root | `data/` | Yes (`s3://…`) |
| Attachments | `data/attachments/` | Inherits |
| Sends | `data/sends/` | Inherits |
| RSA key | `data/rsa_key.pem` | Inherits |

### 6.4 Data Retention

- Deleted (soft-delete) ciphers: purged on configurable schedule (`TRASH_PURGE_SCHEDULE`)
- Expired Sends: purged on schedule (`SEND_PURGE_SCHEDULE`)
- Expired auth requests: purged on schedule (`AUTH_REQUEST_PURGE_SCHEDULE`)
- Incomplete SSO nonces: purged on schedule (`PURGE_INCOMPLETE_SSO_NONCE`)
- Organization event logs: purged on schedule (`EVENT_CLEANUP_SCHEDULE`, if enabled)

---

## 7. Interface Requirements

### 7.1 External API Interfaces

| Mount Point | Module | Purpose |
|-------------|--------|---------|
| `/` | `api::web` | Static web vault assets and error catchers |
| `/api` | `api::core` | Main Bitwarden REST API |
| `/events` | `api::core` (events) | Organization event log API |
| `/identity` | `api::identity` | Authentication: login, token refresh, registration |
| `/icons` | `api::icons` | Website favicon proxy |
| `/notifications` | `api::notifications` | WebSocket real-time sync hub |
| `/admin` | `api::admin` | Vaultwarden admin panel |

### 7.2 WebSocket Interface

- **Endpoint**: `/notifications/hub` (authenticated), `/notifications/anonymous`
- **Protocol**: MessagePack over WebSocket
- **Auth**: Bearer token in `?access_token=` query parameter or `Authorization` header

### 7.3 Email Interface

- **Outbound**: SMTP or Sendmail
- **Templates**: Handlebars `.hbs` files in `src/static/templates/`

### 7.4 Push Notification Interface

- **Outbound relay**: Configurable external push relay via `PUSH_RELAY_URI`
- **Platforms**: APNs (iOS), FCM (Android) — handled by the relay

### 7.5 File Storage Interface

- **Abstraction**: Apache OpenDAL
- **Backends**: Local filesystem (default), S3-compatible object storage

### 7.6 SSO/OIDC Interface

- **Protocol**: OpenID Connect (OIDC) with PKCE
- **Library**: `openidconnect` crate v4.0.1
- **Config**: `SSO_AUTHORITY`, `SSO_CLIENT_ID`, `SSO_CLIENT_SECRET`

---

## 8. Constraints & Assumptions

| # | Constraint / Assumption |
|---|------------------------|
| C-01 | TLS termination **must** be handled by the reverse proxy; Vaultwarden does not terminate TLS. |
| C-02 | All vault data encryption/decryption is performed client-side; the server is an encrypted data store. |
| C-03 | The system is licensed under **AGPL-3.0-only**. Any deployment that exposes the service over a network requires source availability. |
| C-04 | PostgreSQL is recommended for any deployment with more than a single user or requiring high availability. |
| C-05 | S3 file storage requires the `s3` Cargo feature flag to be enabled at compile time. |
| C-06 | Push notifications require integration with an external push relay server (not included in Vaultwarden itself). |
| C-07 | WebSocket notifications are disabled by default and must be explicitly enabled. |
| C-08 | The admin panel is the only interface for server-side configuration changes at runtime (persisted to `config.json`). |
| C-09 | Mobile push relay integration requires a valid `PUSH_RELAY_URI` and `PUSH_IDENTITY_URI`. |
| C-10 | SSO/OIDC requires an external Identity Provider (IdP) accessible by the server. |

---

## 9. Glossary

| Term | Definition |
|------|-----------|
| **AES-256-CBC** | Advanced Encryption Standard, 256-bit key, Cipher Block Chaining mode |
| **AES-256-GCM** | Advanced Encryption Standard, 256-bit key, Galois/Counter Mode |
| **APNs** | Apple Push Notification service |
| **Argon2id** | Memory-hard password hashing algorithm (hybrid of Argon2i and Argon2d) |
| **Cipher** | A vault item in the Bitwarden data model |
| **Collection** | An org-level grouping of vault items |
| **DashMap** | Lock-free concurrent hashmap library for Rust |
| **Diesel** | Rust ORM and query builder |
| **FCM** | Firebase Cloud Messaging (Google's push service) |
| **FIDO2** | Fast IDentity Online v2 — standard for hardware security keys |
| **JWT** | JSON Web Token — compact, self-contained authentication token |
| **KDF** | Key Derivation Function — algorithm to derive cryptographic keys from passwords |
| **MiMalloc** | High-performance memory allocator from Microsoft |
| **MSRV** | Minimum Supported Rust Version |
| **OIDC** | OpenID Connect — identity layer on top of OAuth 2.0 |
| **OpenDAL** | Apache OpenDAL — unified data access layer |
| **PBKDF2** | Password-Based Key Derivation Function 2 |
| **PKCE** | Proof Key for Code Exchange — OAuth 2.0 extension for public clients |
| **r2d2** | Rust database connection pool library |
| **Rocket** | Async web framework for Rust |
| **RS256** | RSA Signature with SHA-256 (JWT signing algorithm) |
| **RSA-2048** | 2048-bit RSA asymmetric key pair |
| **Send** | Bitwarden feature for encrypted, ephemeral content sharing |
| **SSO** | Single Sign-On |
| **TOTP** | Time-based One-Time Password (RFC 6238) |
| **WebAuthn** | Web Authentication API (FIDO2 standard for browsers) |
| **YubiKey** | Hardware security key supporting OTP and FIDO2 |

---

*End of Document*
