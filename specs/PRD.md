# Vaultwarden — Product Requirements Document (PRD)

> **Document Version**: 1.0  
> **Date**: 2026-04-10  
> **Status**: Draft  
> **Author**: Product Team  
> **References**:
> - User Requirements Document: `specs/urd.md`
> - Software Requirements Specification: `specs/srs.md`
> - Technical Design Document: `specs/technical-design.md`

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Product Vision & Strategy](#2-product-vision--strategy)
3. [Target Users & Market](#3-target-users--market)
4. [Problem Statement](#4-problem-statement)
5. [Product Goals & Success Metrics](#5-product-goals--success-metrics)
6. [Feature Catalog](#6-feature-catalog)
   - 6.1 [Core Vault Management](#61-core-vault-management)
   - 6.2 [Authentication & Security](#62-authentication--security)
   - 6.3 [Multi-Factor Authentication](#63-multi-factor-authentication)
   - 6.4 [Organization & Team Collaboration](#64-organization--team-collaboration)
   - 6.5 [Secure Sharing — Bitwarden Send](#65-secure-sharing--bitwarden-send)
   - 6.6 [Emergency Access](#66-emergency-access)
   - 6.7 [Real-Time Sync & Notifications](#67-real-time-sync--notifications)
   - 6.8 [Single Sign-On (SSO / OIDC)](#68-single-sign-on-sso--oidc)
   - 6.9 [Admin Panel & Server Management](#69-admin-panel--server-management)
   - 6.10 [Audit & Event Logging](#610-audit--event-logging)
   - 6.11 [Email Notifications](#611-email-notifications)
   - 6.12 [File & Attachment Storage](#612-file--attachment-storage)
7. [Feature Prioritization (MoSCoW)](#7-feature-prioritization-moscow)
8. [User Flows](#8-user-flows)
9. [Non-Functional Product Requirements](#9-non-functional-product-requirements)
10. [Release Strategy & Milestones](#10-release-strategy--milestones)
11. [Risks & Mitigations](#11-risks--mitigations)
12. [Open Questions & Decisions](#12-open-questions--decisions)
13. [Appendix: Traceability Matrix](#13-appendix-traceability-matrix)

---

## 1. Executive Summary

**Vaultwarden** is an open-source, self-hosted password manager server that is fully compatible with the official Bitwarden client ecosystem. It enables individuals, homelab enthusiasts, and small-to-medium teams to run a Bitwarden-compatible server on their own infrastructure — eliminating dependency on the official Bitwarden cloud service while retaining full functionality.

Written in Rust and licensed under AGPL-3.0, Vaultwarden is designed to be:
- **Resource-efficient**: runs on low-power hardware (Raspberry Pi, VPS with 256 MB RAM)
- **Fully compatible**: works with all official Bitwarden clients (web, desktop, mobile, browser extension) without any client modification
- **Feature-complete**: implements all Bitwarden free and most premium features
- **Operator-friendly**: simple Docker deployment, minimal dependencies, environment-variable-driven configuration

This PRD defines the complete product scope, feature set, prioritization, success metrics, and release plan for Vaultwarden.

---

## 2. Product Vision & Strategy

### 2.1 Vision Statement

> **Enable anyone to own their passwords.**
>
> Vaultwarden gives every individual and team the ability to run a secure, fully-featured password manager on their own infrastructure — with no subscription fees, no vendor lock-in, and no compromise on privacy.

### 2.2 Strategic Positioning

| Dimension | Vaultwarden | Official Bitwarden | 1Password / LastPass |
|-----------|------------|--------------------|--------------------|
| **Hosting** | Self-hosted | Cloud + Self-hosted | Cloud only |
| **Cost** | Free (AGPL) | Free tier + paid plans | Subscription required |
| **Data Control** | Full (operator owns data) | Bitwarden controls cloud | Provider controls data |
| **Resource Usage** | Very low (~50 MB RAM) | High (Java stack) | N/A |
| **Client Compatibility** | All Bitwarden clients | All Bitwarden clients | 1Password clients only |
| **Target Audience** | Privacy-conscious, homelabbers, SMBs | Individuals to enterprise | Individuals to enterprise |

### 2.3 Design Principles

1. **Privacy by Architecture** — The server must never be able to read user vault data. End-to-end encryption is non-negotiable.
2. **Client Compatibility First** — No changes to client behavior. Vaultwarden must work transparently with official Bitwarden clients.
3. **Operator Simplicity** — Deployment and configuration must be achievable in under 10 minutes via Docker.
4. **Security Without Compromise** — Memory-safe Rust, no unsafe code, Argon2id for secrets, rate limiting on all sensitive endpoints.
5. **Feature Parity** — Maintain compatibility with all Bitwarden features available to free and premium individual accounts.

---

## 3. Target Users & Market

### 3.1 Primary Personas

#### Persona 1 — Alex, the Privacy-Conscious Individual
- **Background**: Software developer, 28 years old, runs a personal homelab.
- **Need**: Wants a Bitwarden-compatible server they fully control. Does not trust third-party cloud storage of passwords.
- **Technical Level**: High — comfortable with Docker and Linux.
- **Key Features**: Vault management, 2FA, self-hosted control.

#### Persona 2 — Maya, the SMB IT Administrator
- **Background**: IT admin at a 20-person company. Team uses shared passwords for services and infrastructure.
- **Need**: Share credentials securely across the team with role-based access control and audit trails.
- **Technical Level**: Medium-high.
- **Key Features**: Organizations, collections, groups, audit logs, SSO, admin panel.

#### Persona 3 — Jordan, the Homelab Family Admin
- **Background**: Tech-enthusiast running services for family members (4–6 users).
- **Need**: Simple self-hosted vault that family can use on their phones and laptops without friction.
- **Technical Level**: Medium.
- **Key Features**: Easy client setup, email notifications, 2FA, emergency access.

#### Persona 4 — Sam, the Security-Focused Team Lead
- **Background**: Leads a 10-person engineering team. Requires 2FA enforcement and organizational security policies.
- **Need**: Enforce MFA across the team, review access events, and integrate with company SSO.
- **Technical Level**: High.
- **Key Features**: Org policies, event log, SSO, Duo integration.

### 3.2 Market Opportunity

- Growing demand for self-hosted tools driven by privacy regulations (GDPR, CCPA) and high-profile cloud breaches.
- Official Bitwarden self-hosted requires significant resources (Java + SQL Server or PostgreSQL), making Vaultwarden the only lightweight alternative.
- Active open-source community with tens of thousands of production deployments.

---

## 4. Problem Statement

### 4.1 Core Problems Solved

| Problem | Current Pain | Vaultwarden Solution |
|---------|-------------|---------------------|
| **Cloud trust** | Trusting a third-party service with all passwords | Full self-hosting; operator owns the data |
| **Cost of official self-hosting** | Official Bitwarden server requires ~2 GB RAM, Java, SQL Server | Vaultwarden runs in ~50–100 MB RAM with SQLite |
| **Subscription friction** | Some Bitwarden premium features require a paid plan | Vaultwarden provides premium-equivalent features for free |
| **Team password sharing** | No secure, auditable way to share credentials without a service subscription | Organizations, collections, event logs |
| **Insecure sharing** | Teams use email/chat to share one-off credentials | Bitwarden Send provides encrypted ephemeral sharing |
| **Account recovery** | Lost master password = permanent lockout | Emergency access delegation |

### 4.2 Problem Boundaries

Vaultwarden does **not** aim to solve:
- Secrets management for machine/service (that is Bitwarden Secrets Manager's domain).
- Enterprise billing, provisioning, and subscription management.
- Client UI customization.

---

## 5. Product Goals & Success Metrics

### 5.1 Product Goals

| Goal ID | Goal | Category |
|---------|------|---------|
| G-01 | 100% compatibility with all official Bitwarden clients | Compatibility |
| G-02 | Deployable in under 10 minutes via a single Docker command | Operator Experience |
| G-03 | Vault data is provably inaccessible to the server (E2EE) | Security |
| G-04 | Support organizations of up to 100 users without performance degradation | Performance |
| G-05 | Zero critical security vulnerabilities in core auth and encryption paths | Security |
| G-06 | All configurable via environment variables; no code changes required | Operability |

### 5.2 Key Success Metrics (KPIs)

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| **Client compatibility** | 100% of Bitwarden client API endpoints supported | Client integration test suite |
| **Deployment time** | < 10 minutes from zero to working server | Operator onboarding benchmark |
| **Memory footprint** | < 150 MB RAM under normal load (10–50 users) | Load test with memory profiling |
| **Login latency** | < 300 ms p95 for `/identity/connect/token` | API latency monitoring |
| **Vault sync latency** | < 500 ms p95 for full `/api/sync` response | API latency monitoring |
| **WebSocket delivery** | < 2 seconds for change propagation to connected clients | E2E sync latency test |
| **Uptime** | 99.9% availability (single-node deployment) | Health check monitoring |
| **Build size** | < 20 MB for `release-micro` binary | Binary size CI check |

---

## 6. Feature Catalog

Each feature is described with: **What it does**, **Why it matters**, **Key behavior**, and **Owner/Actor**.

---

### 6.1 Core Vault Management

**Feature ID**: F-VAULT  
**Priority**: 🔴 Must Have  
**Actors**: End User

#### What It Does
Provides the core CRUD operations for managing encrypted vault items (called *ciphers*) across five types: Login, Secure Note, Credit Card, Identity, and SSH Key.

#### Why It Matters
This is the primary value proposition of the product. Without a functional vault, all other features are moot.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Item types** | Login (1), Secure Note (2), Card (3), Identity (4), SSH Key (5) |
| **CRUD** | Create, Read, Update, Delete (soft-delete to trash, permanent purge on schedule) |
| **Bulk operations** | Move, delete, share multiple items simultaneously |
| **Vault sync** | `/api/sync` returns all ciphers, folders, collections, and settings |
| **Password history** | Per-item password history stored encrypted |
| **Re-prompt** | Per-item "require master password to view" flag |
| **Folders** | Personal, private grouping of items |
| **Favorites** | User-specific favorite flag per item |
| **Encryption** | All item data encrypted client-side before leaving the device |

#### Acceptance Criteria
- [ ] All 5 item types can be created, edited, and deleted from any Bitwarden client.
- [ ] Deleted items appear in trash and are purged after the configured schedule.
- [ ] Vault sync returns consistent state across all connected devices.
- [ ] Server stores zero plaintext vault data (verifiable via database inspection).

---

### 6.2 Authentication & Security

**Feature ID**: F-AUTH  
**Priority**: 🔴 Must Have  
**Actors**: End User, Server Administrator

#### What It Does
Manages the complete authentication lifecycle: registration, login, token issuance and refresh, device registration, rate limiting, and re-authentication for protected actions.

#### Why It Matters
Authentication is the gateway to the vault. Any weakness here directly compromises user data security.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Token signing** | RS256 with auto-generated 2048-bit RSA key pair |
| **Access token TTL** | 2 hours |
| **Refresh token TTL** | 30 days (desktop/web), 90 days (mobile) |
| **Rate limiting** | Per-IP, on login, 2FA, and registration endpoints |
| **Protected actions** | Re-auth required (master password or email OTP) for: disabling 2FA, vault export, key changes |
| **Passwordless login** | Device-to-device auth request approval (`AuthRequest` flow) |
| **Device registration** | Every client device registered with UUID and push token |
| **Registration policies** | Open / invite-only / domain-restricted / email-verified |
| **Security stamp** | Changed on sensitive account updates; invalidates all sessions |

#### Acceptance Criteria
- [ ] Login from any official Bitwarden client succeeds.
- [ ] Login fails after exceeding rate limit threshold and succeeds after cooldown.
- [ ] Changing master password invalidates all other active sessions.
- [ ] Protected actions cannot be completed without re-authentication.
- [ ] Passwordless device approval flow completes the login successfully.

---

### 6.3 Multi-Factor Authentication

**Feature ID**: F-MFA  
**Priority**: 🔴 Must Have  
**Actors**: End User, Organization Owner/Admin

#### What It Does
Provides optional (or policy-enforced) second-factor verification at login, supporting six distinct methods.

#### Why It Matters
2FA is the single most impactful security upgrade for user accounts. It must be frictionless to set up and use.

#### Supported Methods

| Method | Use Case | Security Level |
|--------|---------|---------------|
| **TOTP** (RFC 6238) | Authenticator apps (Google Authenticator, Authy) | ⭐⭐⭐ |
| **Email OTP** | Fallback; no hardware required | ⭐⭐ |
| **FIDO2 / WebAuthn** | Hardware keys (YubiKey 5, Passkeys) | ⭐⭐⭐⭐⭐ |
| **YubiKey OTP** | YubiKey in OTP mode | ⭐⭐⭐⭐ |
| **Duo Security** | Enterprise push approval; integration with Duo admin | ⭐⭐⭐⭐ |
| **Recovery Code** | Emergency access when primary 2FA is unavailable | N/A (recovery) |

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Trusted devices** | 2FA can be skipped on devices the user marks as trusted |
| **Org enforcement** | Organization policy can mandate 2FA for all members |
| **Incomplete 2FA alert** | Scheduled job detects and emails users who passed password but not 2FA |
| **Duo OIDC** | Modern Duo integration at org level via OIDC |

#### Acceptance Criteria
- [ ] User can enroll in and use each of the six 2FA methods.
- [ ] Login is blocked without the second factor when 2FA is enabled.
- [ ] Recovery codes can be used when primary 2FA is unavailable.
- [ ] Org 2FA policy prevents access for non-compliant members.

---

### 6.4 Organization & Team Collaboration

**Feature ID**: F-ORG  
**Priority**: 🔴 Must Have  
**Actors**: Organization Owner, Admin, Manager, User

#### What It Does
Enables teams to share encrypted vault items through a hierarchical system of organizations, collections, groups, roles, and membership policies.

#### Why It Matters
For SMB and team users, shared vault management is the primary reason to self-host. Without robust org features, Vaultwarden is only useful for individuals.

#### Key Behaviors

**Organization Structure:**
```
Organization
  └── Collections (logical item groups)
        ├── assigned to Users (directly)
        └── assigned to Groups → Users
```

**Roles & Permissions:**

| Role | Manage Members | Manage All Collections | Manage Assigned Collections | Access Items |
|------|:---:|:---:|:---:|:---:|
| Owner | ✅ | ✅ | ✅ | ✅ |
| Admin | ✅ | ✅ | ✅ | ✅ |
| Manager | ❌ | ❌ | ✅ | ✅ |
| User | ❌ | ❌ | ❌ | ✅ (assigned only) |

**Membership Lifecycle:**

```
Invited → Accepted → Confirmed → [Active Member]
                                       ↓
                                   Revoked (access suspended; data retained)
```

**Additional Behaviors:**

| Behavior | Detail |
|----------|--------|
| **Groups** | Assign collection access to a group; add/remove users from group |
| **Admin recovery** | Owner/Admin can reset a member's master password (with user consent) |
| **Org API key** | Machine-account access for automation pipelines |
| **Public API** | Directory Connector compatibility |

#### Acceptance Criteria
- [ ] Owner can create an org, invite members, assign roles, and create collections.
- [ ] Users can only access vault items in collections they are assigned to.
- [ ] Revoking a member immediately prevents their access.
- [ ] Groups propagate collection access changes to all group members.
- [ ] Admin master password recovery allows an owner to regain access on behalf of a locked-out member.

---

### 6.5 Secure Sharing — Bitwarden Send

**Feature ID**: F-SEND  
**Priority**: 🟠 Should Have  
**Actors**: End User

#### What It Does
Provides a secure, encrypted, temporary file or text sharing mechanism. Recipients access the shared content without needing a Vaultwarden account.

#### Why It Matters
Eliminates the need to share passwords via email or chat. The encryption key is embedded in the URL fragment and never sent to the server, providing true end-to-end security for shared content.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Types** | Text (type 0), File (type 1, up to 500 MB) |
| **Access controls** | Max access count, expiration date, deletion date |
| **Password protection** | Optional; verified server-side via Argon2id |
| **Email privacy** | Sender can hide their email address from recipients |
| **Key security** | Decryption key in URL fragment — never reaches the server |
| **Auto-cleanup** | Expired sends purged by background scheduler |
| **Admin opt-out** | Admin can disable all Sends via `SENDS_ALLOWED=false` |

#### Acceptance Criteria
- [ ] A Send can be created and accessed by a recipient without a Vaultwarden account.
- [ ] Password-protected Sends reject incorrect passwords.
- [ ] A Send automatically becomes inaccessible after its expiration date or max access count.
- [ ] File Sends up to 500 MB upload and download successfully.
- [ ] The server database contains no recoverable plaintext of the Send content.

---

### 6.6 Emergency Access

**Feature ID**: F-EMERGENCY  
**Priority**: 🟠 Should Have  
**Actors**: End User (Grantor), Emergency Grantee

#### What It Does
Allows a user to designate a trusted contact (grantee) who can request access to their vault in an emergency, subject to a configurable wait period and consent mechanism.

#### Why It Matters
Prevents permanent data loss when a vault owner dies or is incapacitated. Provides peace of mind for families and small teams.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Access types** | View (read-only), Takeover (full account reset) |
| **Wait period** | Configurable per grant (e.g., 7, 14, 30 days) |
| **Consent window** | Grantor can approve or reject within wait period |
| **Auto-approval** | Background job grants access after wait period |
| **Reminder emails** | Grantor notified before access is granted |
| **Invitation** | Grantee invited via email; single-use token |

#### Emergency Access Flow

```
Grantee initiates request
    ↓
Wait period begins
    ↓
Grantor receives email notification (can approve/reject at any time)
    ↓
[If no action taken] → Auto-approve after wait period
    ↓
Grantee receives notification of granted access
    ↓
Grantee views vault (View mode) or resets account (Takeover mode)
```

#### Acceptance Criteria
- [ ] A grantor can invite a grantee and set wait time and access type.
- [ ] The grantee can initiate a request and gain access after the wait period.
- [ ] The grantor can reject the request before automatic approval.
- [ ] A "View" grantee can read vault items but not modify them.
- [ ] A "Takeover" grantee can set a new master password and assume full control.

---

### 6.7 Real-Time Sync & Notifications

**Feature ID**: F-SYNC  
**Priority**: 🟠 Should Have  
**Actors**: End User

#### What It Does
Pushes vault change events to all connected Bitwarden clients in real time, eliminating the need for manual sync or polling.

#### Why It Matters
Real-time sync dramatically improves the user experience for multi-device users. Without it, changes made on one device may not appear on another device for minutes.

#### Key Behaviors

| Channel | Technology | Default |
|---------|-----------|---------|
| **WebSocket** | MessagePack over WSS at `/notifications/hub` | Disabled (requires `ENABLE_WEBSOCKET=true`) |
| **Mobile Push** | External relay → APNs / FCM | Disabled (requires relay config) |

**Event Types Propagated:**
`SyncCipherCreate`, `SyncCipherUpdate`, `SyncCipherDelete`, `SyncFolderCreate`, `SyncFolderUpdate`, `SyncFolderDelete`, `SyncVault`, `SyncOrgKeys`, `SyncSendCreate`, `SyncSendUpdate`, `SyncSendDelete`, `SyncSettings`, `LogOut`, `AuthRequest`, `AuthRequestResponse`

| Behavior | Detail |
|----------|--------|
| **Multi-device** | One user → many concurrent sessions, all notified |
| **Auth** | Bearer token in query param (`?access_token=`) or header |
| **Concurrent sessions** | DashMap (lock-free) for O(1) per-user lookup |

#### Acceptance Criteria
- [ ] When WebSocket is enabled, a vault change on device A appears on device B within 2 seconds.
- [ ] Multiple devices logged in as the same user all receive the same event.
- [ ] Mobile devices receive a push notification when a vault change occurs.

---

### 6.8 Single Sign-On (SSO / OIDC)

**Feature ID**: F-SSO  
**Priority**: 🟡 Nice to Have (mandatory for enterprise deployments)  
**Actors**: Server Administrator, End User

#### What It Does
Integrates with any OpenID Connect-compatible Identity Provider (Okta, Azure AD, Google Workspace, Keycloak, etc.) to enable corporate SSO login.

#### Why It Matters
For organizations already using an IdP, SSO provides seamless onboarding, centralized access revocation, and eliminates per-user password management on the Vaultwarden side.

#### SSO Login Flow

```
1. User clicks "Login with SSO" in Bitwarden client
2. Client hits /identity/connect/auth?sso=1
3. Server generates PKCE code_challenge + nonce → stores in DB
4. User redirected to IdP login page
5. IdP authenticates user → redirects back with auth code
6. Server exchanges code for tokens (via sso_client.rs)
7. User looked up or auto-provisioned in Vaultwarden
8. Vaultwarden JWT issued → returned to client
```

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Protocol** | OIDC Authorization Code + PKCE |
| **Auto-provision** | New users created on first SSO login |
| **State cache** | 10-minute TTL, max 1,000 concurrent states |
| **Config** | `SSO_AUTHORITY`, `SSO_CLIENT_ID`, `SSO_CLIENT_SECRET` |
| **Coexistence** | Username/password login still available alongside SSO |
| **Nonce cleanup** | Expired nonces purged daily |

#### Acceptance Criteria
- [ ] A user can log in via a configured OIDC provider without a Vaultwarden password.
- [ ] A new user is auto-provisioned on first SSO login.
- [ ] SSO login fails gracefully if the IdP is unreachable.
- [ ] Disabling SSO does not affect standard username/password login.

---

### 6.9 Admin Panel & Server Management

**Feature ID**: F-ADMIN  
**Priority**: 🔴 Must Have  
**Actors**: Server Administrator

#### What It Does
Provides a web-based admin interface at `/admin` for managing users, organizations, configuration, and server health without requiring direct database or CLI access.

#### Why It Matters
Most server administrators are not comfortable editing databases directly. The admin panel is the primary operational interface for the people who run Vaultwarden.

#### Key Behaviors

| Capability | Detail |
|-----------|--------|
| **Access control** | Argon2id-hashed token (`ADMIN_TOKEN`); generated via `vaultwarden hash` CLI |
| **User management** | List, invite, enable, disable, delete users |
| **Org management** | List organizations and their members |
| **Configuration** | Edit all settings; persisted to `config.json` |
| **Diagnostics** | Server info, version, DB status |
| **SQLite backup** | Trigger backup from the panel |
| **Session control** | Configurable session lifetime (`ADMIN_SESSION_LIFETIME`) |
| **Token-less mode** | `DISABLE_ADMIN_TOKEN` for environments with external auth |

**Argon2id Presets for Token Generation:**

| Preset | Memory | Iterations | Threads | Recommended For |
|--------|--------|------------|---------|-----------------|
| `bitwarden` (default) | 64 MiB | 3 | 4 | Standard deployments |
| `owasp` | 19 MiB | 2 | 1 | Low-resource hosts |

#### Acceptance Criteria
- [ ] Admin panel accessible at `/admin` with a valid Argon2id token.
- [ ] Admin can invite a user who successfully receives an invitation email.
- [ ] Admin can change configuration settings; changes survive a server restart.
- [ ] Plain-text or bcrypt admin tokens are rejected.

---

### 6.10 Audit & Event Logging

**Feature ID**: F-AUDIT  
**Priority**: 🟠 Should Have  
**Actors**: Organization Owner, Admin

#### What It Does
Records all significant actions taken within an organization into a time-stamped, immutable audit log accessible via API.

#### Why It Matters
Audit logs are often a compliance requirement (SOC 2, ISO 27001, GDPR) and critical for incident investigation ("who changed this password and when?").

#### Captured Event Fields

| Field | Example |
|-------|---------|
| Event type | `CipherUpdated`, `MemberRemoved` |
| Acting user UUID | `uuid` |
| Target cipher UUID | `uuid` |
| Organization UUID | `uuid` |
| Device UUID | `uuid` |
| IP address | `192.168.1.10` |
| Timestamp | `2026-04-10T09:00:00Z` |

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Enablement** | Requires `ORG_EVENTS_ENABLED=true` |
| **API access** | `/events` endpoint |
| **Retention** | Configurable cleanup schedule (`EVENT_CLEANUP_SCHEDULE`) |
| **Auto-cleanup** | Background job purges old entries |

#### Acceptance Criteria
- [ ] Every org-level action generates a corresponding event log entry.
- [ ] Events include all required fields (actor, type, target, IP, timestamp).
- [ ] Logs persist across server restarts.
- [ ] Old events are cleaned up per the configured schedule.

---

### 6.11 Email Notifications

**Feature ID**: F-EMAIL  
**Priority**: 🔴 Must Have  
**Actors**: All users (recipients), Server Administrator (configuration)

#### What It Does
Sends transactional emails for account lifecycle events, security alerts, and invitation workflows.

#### Why It Matters
Email is the primary out-of-band communication channel for account actions and alerts. Without email, many critical flows (email verification, invitations, emergency access) are blocked.

#### Triggered Emails

| Event | Recipient |
|-------|----------|
| Account invitation | Invitee |
| Email address verification | New user |
| Incomplete 2FA login alert | Account owner |
| Organization invitation | Invitee |
| Emergency access invitation | Grantee |
| Emergency access request initiated | Grantor |
| Emergency access granted | Grantee |
| Emergency access rejected | Grantee |
| Emergency access reminder | Grantor |
| Account deletion confirmation | Account owner |

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Transport** | SMTP (STARTTLS / TLS) or Sendmail |
| **Templates** | Handlebars `.hbs` files; fully customizable |
| **TLS** | `rustls` with native root certificates |
| **Debug mode** | `SMTP_DEBUG=true` for verbose SMTP logging |

#### Acceptance Criteria
- [ ] All triggered emails are delivered when SMTP is correctly configured.
- [ ] Emails render correctly in major email clients.
- [ ] Undeliverable emails are logged with an informative error.

---

### 6.12 File & Attachment Storage

**Feature ID**: F-STORAGE  
**Priority**: 🔴 Must Have  
**Actors**: End User, Server Administrator

#### What It Does
Manages file storage for vault item attachments and Bitwarden Send files through a unified abstraction layer (OpenDAL) that supports both local filesystem and S3-compatible object storage.

#### Why It Matters
File attachments are a key premium feature. S3 support allows operators to scale storage independently of the server process.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Storage abstraction** | Apache OpenDAL — same API for local and S3 |
| **Default paths** | `data/attachments/`, `data/sends/`, `data/rsa_key.pem` |
| **S3 support** | Enabled via `s3` Cargo feature; configured via env vars |
| **Max upload** | 525 MB per file |
| **Auth** | File downloads require a single-use JWT file download token |
| **RSA key** | Server's RSA signing key stored via OpenDAL |

**Storage Path Layout:**
```
data/
├── attachments/    ← Cipher file attachments
├── sends/          ← Bitwarden Send files
├── rsa_key.pem     ← JWT signing key
└── config.json     ← Runtime configuration
```

#### Acceptance Criteria
- [ ] File attachments up to 525 MB can be uploaded and downloaded.
- [ ] Files are stored in the correct directory structure.
- [ ] S3-configured deployments store and retrieve files from the S3 bucket.
- [ ] File download links expire after first use.

---

## 7. Feature Prioritization (MoSCoW)

### 7.1 Priority Definitions

| Priority | Label | Definition |
|----------|-------|-----------|
| 🔴 | **Must Have** | Core functionality; product is not viable without this |
| 🟠 | **Should Have** | Important for most users; should be included in v1 |
| 🟡 | **Could Have** | Valuable for specific segments; include if time permits |
| ⚪ | **Won't Have (now)** | Explicitly deferred to a future version |

### 7.2 Feature Priority Table

| Feature | ID | Priority | Justification |
|---------|----|---------:|--------------|
| Core Vault (CRUD, sync, folders) | F-VAULT | 🔴 Must | Primary product value |
| Authentication & JWT Management | F-AUTH | 🔴 Must | Gateway to all features |
| Email Notifications (SMTP) | F-EMAIL | 🔴 Must | Required for invitations, verification |
| File & Attachment Storage | F-STORAGE | 🔴 Must | Bitwarden premium parity |
| Admin Panel | F-ADMIN | 🔴 Must | Primary operator interface |
| Multi-Factor Authentication (all types) | F-MFA | 🔴 Must | Core security requirement |
| Organization & Team Management | F-ORG | 🔴 Must | Key for small teams |
| Bitwarden Send | F-SEND | 🟠 Should | Common use case; privacy-first sharing |
| Emergency Access | F-EMERGENCY | 🟠 Should | Critical for trust and account resilience |
| Real-Time Sync (WebSocket) | F-SYNC | 🟠 Should | Significantly improves UX |
| Mobile Push Notifications | F-SYNC | 🟠 Should | Required for mobile UX |
| Audit & Event Logging | F-AUDIT | 🟠 Should | Required for compliance-conscious orgs |
| Single Sign-On (OIDC) | F-SSO | 🟡 Could | Needed for enterprise; complex to configure |
| S3 File Storage | F-STORAGE | 🟡 Could | Advanced operator feature |
| Duo Security Integration | F-MFA | 🟡 Could | Enterprise-specific requirement |
| Directory Connector API | F-ORG | 🟡 Could | Advanced enterprise feature |
| MiMalloc Allocator | (NFR) | ⚪ Won't (now) | Optimization; low priority |

---

## 8. User Flows

### 8.1 New User Registration & First Login

```
User opens Bitwarden client
    → Sets server URL to Vaultwarden instance
    → Clicks "Create Account"
    → Enters email + master password
    → Client derives encryption key locally (PBKDF2/Argon2id)
    → POST /identity/accounts/register
        ← Server creates user account (stores password hash, encrypted key)
    → [If SIGNUPS_VERIFY=true]
        Server sends verification email
        User clicks link → GET /api/accounts/verify-email?token=…
    → User logs in:
        POST /identity/connect/token
        ← Server validates credentials, returns access + refresh tokens
    → Vault unlocked → user can add items
```

### 8.2 Organization Member Onboarding

```
Org Owner opens web vault
    → Navigates to Organization → Members
    → Clicks "Invite" → enters member email
    → Server creates Invitation JWT → sends email

Member receives email → clicks "Accept Invitation"
    → Signs up or logs in (if account exists)
    → Status: Invited → Accepted

Org Owner returns to Members
    → Confirms the member (assigns akey)
    → Status: Accepted → Confirmed

Member can now access collections they've been assigned to
```

### 8.3 Bitwarden Send — Secure Text Sharing

```
User opens Bitwarden client
    → Goes to Send → Create New Send
    → Types message (or uploads file)
    → Optionally: sets expiration, max views, password
    → [Client-side] generates random AES-256 key
    → Encrypts content with key
    → POST /api/sends {encrypted_data, access_controls}
        ← Server stores encrypted blob + returns send URL

User shares URL with recipient (URL contains #key_fragment)

Recipient opens URL in any browser
    → Client JS extracts key from URL fragment (never sent to server)
    → GET /api/sends/{id}/access
        ← Server returns encrypted blob
    → Client decrypts with key from URL → displays content
```

### 8.4 SSO Login Flow

```
User clicks "Log in with SSO"
    → Client hits GET /identity/connect/auth?sso=1
    → Server generates PKCE challenge + nonce → stores in SsoNonce
        ← Returns redirect_uri to IdP

User browser redirected to IdP login page
    → User authenticates with corporate credentials
    → IdP redirects back: /identity/connect/oidc-signin?code=…

Server receives callback
    → Validates PKCE + state
    → Exchanges code for tokens with IdP
    → Looks up or auto-creates Vaultwarden user
    → Issues Vaultwarden JWT
        ← Returns access token to client

User is now logged in to their Vaultwarden vault via SSO
```

---

## 9. Non-Functional Product Requirements

### 9.1 Security Requirements

| Requirement | Product Rationale |
|-------------|------------------|
| End-to-end encryption (AES-256-GCM/CBC) | Core trust promise: the server is a blind store |
| No plaintext storage of secrets | Regulatory and trust requirement |
| Argon2id for admin token | Resistant to GPU-based cracking |
| Rate limiting on auth endpoints | Defense against credential stuffing |
| `#![forbid(unsafe_code)]` | Memory safety guarantee from Rust |
| PKCE for SSO flows | Prevents authorization code interception |
| HTTPS-only via reverse proxy | No credentials in plaintext over the network |

### 9.2 Performance Requirements

| Scenario | Target |
|---------|--------|
| Login (`/identity/connect/token`) | < 300ms p95 |
| Vault sync (`/api/sync`) | < 500ms p95 for vaults up to 500 items |
| WebSocket event delivery | < 2 seconds end-to-end |
| File upload (10 MB) | < 5 seconds |
| Server memory (idle, 10 users) | < 50 MB RAM |
| Server memory (active, 50 concurrent users) | < 150 MB RAM |

### 9.3 Reliability Requirements

| Requirement | Target |
|-------------|--------|
| Database migration | Auto-applied on startup; zero manual steps |
| DB connection resilience | Retry `DB_CONNECTION_RETRIES` times before failing |
| SQLite backup | On-demand and scheduled; consistent snapshots |
| Background jobs | Isolated OS thread; do not block HTTP handlers |

### 9.4 Compatibility Requirements

| Requirement | Target |
|-------------|--------|
| Bitwarden client compatibility | 100% — all endpoints functional |
| Database support | SQLite (default), PostgreSQL, MySQL/MariaDB |
| Container support | Docker (amd64, arm64), Podman |
| Build targets | Linux (glibc, musl), macOS |
| Rust MSRV | 1.89.0 |

### 9.5 Operability Requirements

| Requirement | Target |
|-------------|--------|
| Configuration | 100% via environment variables |
| Deployment | Single `docker run` command |
| Logs | Structured; configurable level; sensitive values masked |
| Admin UX | No CLI required for day-to-day operations |

---

## 10. Release Strategy & Milestones

### 10.1 Versioning Approach

Vaultwarden follows a continuous delivery model with **calendar-versioned releases** aligned to Bitwarden API compatibility milestones.

### 10.2 Milestone Plan

#### Milestone 1 — Core Vault (v1.0 Baseline)
**Goal**: Minimum viable self-hosted vault.

| Feature | Status |
|---------|--------|
| User registration & login | ✅ Implemented |
| Vault item CRUD (all 5 types) | ✅ Implemented |
| Folders & favorites | ✅ Implemented |
| File attachments (local storage) | ✅ Implemented |
| SMTP email | ✅ Implemented |
| Admin panel | ✅ Implemented |
| SQLite database | ✅ Implemented |
| Docker container deployment | ✅ Implemented |

#### Milestone 2 — Security & MFA (v1.1)
**Goal**: Production-grade security posture.

| Feature | Status |
|---------|--------|
| TOTP 2FA | ✅ Implemented |
| Email OTP | ✅ Implemented |
| WebAuthn / FIDO2 | ✅ Implemented |
| YubiKey OTP | ✅ Implemented |
| Duo Security | ✅ Implemented |
| Rate limiting | ✅ Implemented |
| Protected actions re-auth | ✅ Implemented |
| Passwordless (AuthRequest) | ✅ Implemented |

#### Milestone 3 — Teams & Collaboration (v1.2)
**Goal**: Enable team use cases.

| Feature | Status |
|---------|--------|
| Organizations & memberships | ✅ Implemented |
| Collections | ✅ Implemented |
| Groups | ✅ Implemented |
| Organization policies | ✅ Implemented |
| Event / audit log | ✅ Implemented |
| Admin password recovery | ✅ Implemented |

#### Milestone 4 — Advanced Features (v1.3)
**Goal**: Premium parity and advanced integrations.

| Feature | Status |
|---------|--------|
| Bitwarden Send | ✅ Implemented |
| Emergency Access | ✅ Implemented |
| WebSocket real-time sync | ✅ Implemented |
| Mobile push notifications | ✅ Implemented |
| SSO / OIDC | ✅ Implemented |
| S3 object storage | ✅ Implemented |

#### Milestone 5 — Hardening & Operations (v1.4+)
**Goal**: Production hardening, observability, and operational tooling.

| Feature | Status |
|---------|--------|
| PostgreSQL support | ✅ Implemented |
| MySQL/MariaDB support | ✅ Implemented |
| Duo OIDC (modern flow) | ✅ Implemented |
| SQLite backup (`SIGUSR1`) | ✅ Implemented |
| Configurable background jobs | ✅ Implemented |
| MiMalloc allocator (musl builds) | ✅ Implemented |
| Extended logging & query logger | ✅ Implemented |

---

## 11. Risks & Mitigations

| Risk ID | Risk | Likelihood | Impact | Mitigation |
|---------|------|:---------:|:------:|-----------|
| R-01 | Bitwarden client API changes break compatibility | Medium | High | Monitor Bitwarden API changelogs; maintain integration test suite against latest clients |
| R-02 | Security vulnerability in core auth or encryption | Low | Critical | Use established libraries (ring, jsonwebtoken); forbid unsafe code; regular dependency audits |
| R-03 | SQLite data corruption under concurrent writes | Medium | High | Use WAL mode; recommend PostgreSQL for multi-user deployments; provide backup tooling |
| R-04 | SMTP misconfiguration blocks verification emails | High | Medium | Provide SMTP test endpoint in admin panel; clear error messages; debug mode |
| R-05 | SSO IdP downtime prevents all logins | Medium | High | Allow username/password login to coexist with SSO; operator documentation |
| R-06 | S3 credentials exposure via misconfiguration | Low | High | Mask all credentials in logs and API output; document least-privilege IAM policies |
| R-07 | Admin token brute-force | Low | Critical | Enforce Argon2id; rate-limit admin login; reject plain-text tokens |
| R-08 | Dependency supply chain attack | Low | High | Use `cargo audit`; pin dependency versions; review RUSTSEC advisories |
| R-09 | Operator uses plain HTTP (no TLS reverse proxy) | Medium | High | Documentation warning; HTTPS enforcement recommendations |
| R-10 | AGPL compliance violation by operator | Medium | Medium | Clear licensing documentation; community awareness |

---

## 12. Open Questions & Decisions

| # | Question | Owner | Status | Decision |
|---|---------|-------|--------|---------|
| OQ-01 | Should Vaultwarden natively support secrets manager API? | Product | 🔴 Open | Likely out of scope — separate project |
| OQ-02 | Should an official Prometheus metrics endpoint be added? | Engineering | 🟡 Discussing | Use log-based metrics for now |
| OQ-03 | Should there be a rate limit on admin panel login attempts? | Security | 🟡 Discussing | TBD — Argon2id provides time-cost by default |
| OQ-04 | Should WebSocket be enabled by default? | Product | ✅ Decided | Disabled by default; operator must opt in |
| OQ-05 | Should email verification be mandatory by default? | Product | ✅ Decided | Optional (configurable): `SIGNUPS_VERIFY` |
| OQ-06 | Maximum supported users for a single SQLite deployment? | Engineering | 🟡 Discussing | Recommended: < 100 users for SQLite; PostgreSQL for larger |

---

## 13. Appendix: Traceability Matrix

This matrix links Product Requirements (PRD) to User Requirements (URD), Software Requirements (SRS), and the Technical Design (TDD).

| PRD Feature | URD Reference | SRS Reference | TDD Section |
|-------------|--------------|--------------|------------|
| F-VAULT (Core vault CRUD) | UR-USER-003, UR-USER-004, UR-USER-005, UR-USER-007, UR-USER-008 | FR-CIPHER-001 to FR-CIPHER-010 | §6.2 Cipher Model, §4 HTTP Routes |
| F-AUTH (Authentication) | UR-USER-001, UR-USER-002, UR-USER-010, UR-USER-013 | FR-AUTH-001 to FR-AUTH-008 | §5 Authentication & Authorization |
| F-MFA (Two-factor auth) | UR-USER-012, UR-MFA-001 to UR-MFA-003, UR-POLICY-001 | FR-2FA-001 to FR-2FA-009 | §6.4 TwoFactor Model |
| F-ORG (Organizations) | UR-ORG-001 to UR-ORG-007, UR-POLICY-001 to UR-POLICY-004 | FR-ORG-001 to FR-ORG-010 | §6.3 Org & Membership Model |
| F-SEND (Bitwarden Send) | UR-SEND-001 to UR-SEND-005 | FR-SEND-001 to FR-SEND-005 | §6.5 Send Model |
| F-EMERGENCY (Emergency access) | UR-EMRG-001 to UR-EMRG-003 | FR-EMRG-001 to FR-EMRG-006 | §6.6 Other Models |
| F-SYNC (Real-time sync) | UR-SYNC-001, UR-SYNC-002, UR-ADMIN-012, UR-ADMIN-013 | FR-NOTIF-001 to FR-PUSH-004 | §9 Notification System |
| F-SSO (Single sign-on) | UR-ADMIN-011 | FR-SSO-001 to FR-SSO-006 | §10 OIDC/SSO Integration |
| F-ADMIN (Admin panel) | UR-ADMIN-001 to UR-ADMIN-008, UR-ADMIN-015 | FR-ADMIN-001 to FR-ADMIN-006 | §5.2 Admin Auth, §12 Config |
| F-AUDIT (Event logging) | UR-AUDIT-001 to UR-AUDIT-003 | FR-EVENT-001 to FR-EVENT-004 | §6.6 Event Model |
| F-EMAIL (Email subsystem) | UR-ADMIN-010 | FR-EMAIL-001 to FR-EMAIL-004 | §13 Email Subsystem |
| F-STORAGE (File storage) | UR-USER-007, UR-ADMIN-008 | FR-ATTACH-001 to FR-ATTACH-004 | §8 File Storage (OpenDAL) |

---

*End of Document*
