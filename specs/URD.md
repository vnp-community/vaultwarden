# Vaultwarden — User Requirements Document (URD)

> **Document Version**: 1.0  
> **Date**: 2026-04-10  
> **Status**: Draft  
> **References**:  
> - Software Requirements Specification: `specs/srs.md`  
> - Technical Design Document: `specs/technical-design.md`  
> **Source Project**: `dani-garcia/vaultwarden` — Self-hosted Bitwarden-compatible password manager server

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [User Profiles & Goals](#2-user-profiles--goals)
3. [Use Cases Overview](#3-use-cases-overview)
4. [User Requirements by Role](#4-user-requirements-by-role)
   - 4.1 [End User — Personal Vault Management](#41-end-user--personal-vault-management)
   - 4.2 [End User — Account & Security Settings](#42-end-user--account--security-settings)
   - 4.3 [End User — Secure Sharing (Send)](#43-end-user--secure-sharing-send)
   - 4.4 [End User — Emergency Access](#44-end-user--emergency-access)
   - 4.5 [Organization Owner / Admin — Team Management](#45-organization-owner--admin--team-management)
   - 4.6 [Organization Owner / Admin — Access Control & Policies](#46-organization-owner--admin--access-control--policies)
   - 4.7 [Organization Owner / Admin — Audit & Compliance](#47-organization-owner--admin--audit--compliance)
   - 4.8 [Server Administrator — Instance Management](#48-server-administrator--instance-management)
   - 4.9 [Server Administrator — Configuration & Integration](#49-server-administrator--configuration--integration)
5. [Cross-Cutting User Needs](#5-cross-cutting-user-needs)
   - 5.1 [Security & Privacy](#51-security--privacy)
   - 5.2 [Multi-Device & Real-Time Sync](#52-multi-device--real-time-sync)
   - 5.3 [Multi-Factor Authentication](#53-multi-factor-authentication)
   - 5.4 [Usability & Client Compatibility](#54-usability--client-compatibility)
6. [User Constraints & Expectations](#6-user-constraints--expectations)
7. [Acceptance Criteria Summary](#7-acceptance-criteria-summary)
8. [Glossary](#8-glossary)

---

## 1. Introduction

### 1.1 Purpose

This User Requirements Document (URD) describes the needs, goals, and expectations of all users and stakeholders of **Vaultwarden** — a self-hosted, open-source password manager server fully compatible with the Bitwarden client ecosystem.

Unlike the SRS (which defines *what the system shall do* in technical terms), this URD describes *what users want to be able to do* and *why*, using the language of user goals, scenarios, and acceptance criteria.

### 1.2 Project Overview

Vaultwarden enables individuals, families, and small teams to self-host a Bitwarden-compatible server — giving them full control over their vault data without depending on a third-party cloud service. Users interact with Vaultwarden exclusively through official Bitwarden clients (web, desktop, mobile, browser extension); the server is invisible to end users.

### 1.3 Scope

This document covers user requirements for:

- **End users** managing personal vaults
- **Organization owners and admins** managing shared team vaults
- **Server administrators** deploying and operating the Vaultwarden instance

**Not in scope:**
- UI design of Bitwarden clients (delegated to Bitwarden)
- Billing or subscription management
- Enterprise features exclusive to the official Bitwarden cloud (e.g., Secrets Manager)

### 1.4 Document Conventions

User requirements are written in the format:

> **UR-[ROLE]-[NNN]**: As a **[role]**, I want to **[action]** so that **[goal/benefit]**.

Acceptance criteria follow each requirement where needed.

---

## 2. User Profiles & Goals

### 2.1 End User (Personal)

| Attribute | Description |
|-----------|-------------|
| **Who** | An individual who stores passwords, credit cards, identities, and secure notes |
| **Primary Goal** | Securely store and access credentials across all their devices |
| **Key Concern** | Privacy — no third-party cloud service should ever see their data |
| **Technical Level** | Low to medium — uses Bitwarden clients; unaware of server internals |
| **Devices Used** | Web browser, desktop app, mobile (iOS/Android), browser extension |

### 2.2 Organization Owner / Admin

| Attribute | Description |
|-----------|-------------|
| **Who** | A team lead, IT admin, or business owner managing shared credentials |
| **Primary Goal** | Securely share credentials with team members and enforce access policies |
| **Key Concern** | Granular access control, auditability, and member management |
| **Technical Level** | Medium — understands roles, permissions, and collections |
| **Devices Used** | Web vault primarily |

### 2.3 Server Administrator

| Attribute | Description |
|-----------|-------------|
| **Who** | A DevOps engineer, sysadmin, or technical user who deploys and maintains the server |
| **Primary Goal** | Run a stable, secure Vaultwarden instance with minimal overhead |
| **Key Concern** | Reliability, upgradability, configuration flexibility, and data backup |
| **Technical Level** | High — comfortable with Docker, environment variables, and databases |
| **Devices Used** | CLI, admin web panel, SSH |

### 2.4 Emergency Grantee

| Attribute | Description |
|-----------|-------------|
| **Who** | A trusted person (friend, family member, colleague) designated by a vault owner |
| **Primary Goal** | Access a trusted person's vault in an emergency situation |
| **Key Concern** | Clear, time-based process with consent and safeguards |
| **Technical Level** | Low — uses a standard Bitwarden client |

---

## 3. Use Cases Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Vaultwarden System                           │
│                                                                     │
│  ┌─────────────────┐   ┌──────────────────┐   ┌─────────────────┐  │
│  │  Personal Vault │   │  Team / Org Vault│   │  Admin Panel    │  │
│  │                 │   │                  │   │                 │  │
│  │ • Manage items  │   │ • Share passwords│   │ • Manage users  │  │
│  │ • Use 2FA       │   │ • Control access │   │ • Configure app │  │
│  │ • Share via Send│   │ • Audit events   │   │ • Backup data   │  │
│  │ • Emergency     │   │ • Enforce policy │   │ • Monitor health│  │
│  │   access        │   │                  │   │                 │  │
│  └────────┬────────┘   └────────┬─────────┘   └────────┬────────┘  │
│           │                     │                       │           │
│     [End User]           [Org Owner/Admin]       [Server Admin]     │
└─────────────────────────────────────────────────────────────────────┘
```

| Use Case Group | Primary Actor | Summary |
|----------------|--------------|---------|
| UC-01: Account Setup | End User | Register, verify email, configure 2FA |
| UC-02: Daily Vault Use | End User | Add, view, copy, autofill credentials |
| UC-03: Secure Sharing | End User | Create a Send link for text or file |
| UC-04: Emergency Access | End User / Grantee | Delegate or invoke emergency vault access |
| UC-05: Team Onboarding | Org Owner | Create org, invite members, assign collections |
| UC-06: Access Management | Org Admin | Manage member roles and collection permissions |
| UC-07: Policy Enforcement | Org Owner | Set 2FA requirements, password policies |
| UC-08: Audit Review | Org Admin | Review event logs for compliance |
| UC-09: Server Setup | Server Admin | Deploy, configure, and maintain the server |
| UC-10: SSO Integration | Server Admin | Connect an enterprise Identity Provider |

---

## 4. User Requirements by Role

### 4.1 End User — Personal Vault Management

---

**UR-USER-001**: As an **end user**, I want to **create an account** so that I can **start storing my credentials securely**.

*Acceptance Criteria:*
- I can register using my email address and a master password.
- I receive a verification email before my account is activated (if enabled by the admin).
- I can also be invited by an organization before receiving an account.

---

**UR-USER-002**: As an **end user**, I want to **log in from any official Bitwarden client** (web, desktop, mobile, browser extension) so that I can **access my vault on any device**.

*Acceptance Criteria:*
- Login works on all official Bitwarden clients without any additional configuration.
- My session stays active for a reasonable time (desktop/web: 30 days, mobile: 90 days).
- I am automatically logged out when my access token expires.

---

**UR-USER-003**: As an **end user**, I want to **add, edit, and delete vault items** so that I can **keep my credentials organized and up-to-date**.

*Acceptance Criteria:*
- I can create items of the following types: Login, Secure Note, Credit Card, Identity, SSH Key.
- I can add custom fields, notes, and URLs to each item.
- I can move items to the trash and restore or permanently delete them.
- I can view the password history of any item.

---

**UR-USER-004**: As an **end user**, I want to **organize my vault items into folders** so that I can **find them easily**.

*Acceptance Criteria:*
- I can create, rename, and delete folders.
- I can assign vault items to one or more folders.
- Folder structure is personal and not visible to organization members unless the item is shared.

---

**UR-USER-005**: As an **end user**, I want to **mark vault items as favorites** so that I can **quickly access the ones I use most**.

*Acceptance Criteria:*
- I can toggle favorite status on any vault item.
- Favorites appear at the top or in a dedicated section in the client.

---

**UR-USER-006**: As an **end user**, I want **my vault to sync automatically across all my devices** so that **any change I make is immediately reflected everywhere**.

*Acceptance Criteria:*
- When I add, update, or delete an item on one device, my other signed-in devices receive the change in real time (or on next sync).
- I do not need to manually trigger a sync.

---

**UR-USER-007**: As an **end user**, I want to **attach files to vault items** so that I can **store related documents alongside my credentials**.

*Acceptance Criteria:*
- I can upload a file (up to 500 MB) and attach it to a vault item.
- I can download the attachment from any device.
- Attached files are encrypted client-side before being sent to the server.

---

**UR-USER-008**: As an **end user**, I want to **set a re-prompt on sensitive vault items** so that **anyone with access to my unlocked client must re-enter my master password before viewing those items**.

*Acceptance Criteria:*
- I can enable "Master Password Re-prompt" on any individual vault item.
- The client prompts for my master password when I try to view, copy, or use that item.

---

### 4.2 End User — Account & Security Settings

---

**UR-USER-010**: As an **end user**, I want to **change my master password** so that I can **maintain strong account security over time**.

*Acceptance Criteria:*
- I can change my master password from the web vault account settings.
- Changing my password automatically invalidates all other active sessions.
- My vault data remains fully accessible after the password change.

---

**UR-USER-011**: As an **end user**, I want to **change my registered email address** so that I can **keep my account information current**.

*Acceptance Criteria:*
- I must verify ownership of the new email address before the change takes effect.
- I receive a confirmation notification at both the old and new email addresses.

---

**UR-USER-012**: As an **end user**, I want to **enable two-factor authentication (2FA)** so that my account **remains protected even if my master password is compromised**.

*Acceptance Criteria:*
- I can set up at least one 2FA method: authenticator app (TOTP), email OTP, hardware key (YubiKey or FIDO2/WebAuthn), or Duo.
- I am provided with recovery codes in case I lose access to my 2FA device.
- I can mark trusted devices to skip 2FA for convenience.

---

**UR-USER-013**: As an **end user**, I want to **log in without a password using device approval** so that I can **authenticate securely from a new device**.

*Acceptance Criteria:*
- I can initiate a login on a new device and approve it from a trusted device.
- The request expires if not approved within a reasonable time.

---

**UR-USER-014**: As an **end user**, I want to **delete my account** so that I can **permanently remove all my data from the server**.

*Acceptance Criteria:*
- Account deletion requires confirmation via a link sent to my registered email.
- After deletion, all my vault data, attachments, and profile information are permanently removed.

---

**UR-USER-015**: As an **end user**, I want to **generate a personal API key** so that I can **use the Bitwarden CLI or automations to access my vault programmatically**.

*Acceptance Criteria:*
- I can generate and revoke my personal API key from account settings.
- The API key grants the same access as my user session.

---

### 4.3 End User — Secure Sharing (Send)

---

**UR-SEND-001**: As an **end user**, I want to **share a piece of text or a file with anyone** using a secure, time-limited link so that **I don't need to use insecure channels like email or chat**.

*Acceptance Criteria:*
- I can create a Send containing text or a file (up to 500 MB).
- The recipient can access the Send via a unique URL without needing a Vaultwarden account.
- Content is decrypted in the recipient's browser using a key embedded in the URL fragment (never sent to the server).

---

**UR-SEND-002**: As an **end user**, I want to **add a password to my Send** so that **only intended recipients can open it**.

*Acceptance Criteria:*
- I can optionally require a password before a Send can be accessed.
- The password is verified on the server without storing the plaintext password.

---

**UR-SEND-003**: As an **end user**, I want to **set an expiration date and access limit on my Send** so that it **automatically becomes unavailable after a set time or number of views**.

*Acceptance Criteria:*
- I can set maximum access count, expiration date, and deletion date independently.
- The Send is automatically deactivated when any of these limits are reached.

---

**UR-SEND-004**: As an **end user**, I want to **hide my email address from Send recipients** so that I can **share content anonymously**.

*Acceptance Criteria:*
- I can toggle "Hide my email" when creating or editing a Send.
- If hidden, my email is not shown on the Send access page.

---

**UR-SEND-005**: As an **end user**, I want to **manually deactivate or delete a Send at any time** so that I **can revoke access immediately if needed**.

*Acceptance Criteria:*
- I can disable a Send (making the link inaccessible) without deleting it.
- I can permanently delete a Send from the client.

---

### 4.4 End User — Emergency Access

---

**UR-EMRG-001**: As an **end user**, I want to **designate a trusted contact as my emergency access grantee** so that **they can access my vault if I am incapacitated or unavailable**.

*Acceptance Criteria:*
- I can invite any Vaultwarden user as my emergency contact via email.
- I set whether the grantee can only **view** my vault or fully **take over** my account.
- I define the wait time (e.g., 7 days) before the grantee's request is automatically approved.

---

**UR-EMRG-002**: As an **end user (grantor)**, I want to **review and reject an emergency access request** within the wait period so that I can **prevent unauthorized access if I am able to respond**.

*Acceptance Criteria:*
- I receive an email notification when my grantee initiates an emergency access request.
- I have the full wait period to approve or reject the request directly.
- If I reject the request, the grantee's access is denied.

---

**UR-EMRG-003**: As an **emergency grantee**, I want to **initiate an emergency access request** so that I can **access my trusted contact's vault when needed**.

*Acceptance Criteria:*
- I can submit a request from my Bitwarden client.
- I receive a notification when the wait period ends and access is granted.
- If access type is "View", I can read vault items but cannot make changes.
- If access type is "Takeover", I can reset the account and gain full ownership.

---

### 4.5 Organization Owner / Admin — Team Management

---

**UR-ORG-001**: As an **organization owner**, I want to **create an organization** so that my team can **share credentials and work collaboratively**.

*Acceptance Criteria:*
- I can create a new organization with a name and billing email.
- The organization has its own shared vault, separate from personal vaults.
- I am automatically assigned the Owner role.

---

**UR-ORG-002**: As an **organization owner or admin**, I want to **invite team members by email** so that they can **join the shared vault**.

*Acceptance Criteria:*
- I can send invitations to one or more email addresses.
- Invitees receive an email with a link to accept the invitation.
- Invitations expire if not accepted within a configurable time.

---

**UR-ORG-003**: As an **organization owner**, I want to **assign roles to members** so that I can **control what each person can manage**.

*Acceptance Criteria:*
- Available roles: Owner, Admin, Manager, User, Custom.
- Owners and Admins can manage all collections and members.
- Managers can manage their assigned collections.
- Users can only access items they are granted access to.

---

**UR-ORG-004**: As an **organization owner or admin**, I want to **create collections** so that I can **logically group shared vault items by project, department, or access level**.

*Acceptance Criteria:*
- I can create, rename, and delete collections.
- I can assign specific users or groups to each collection with read-only or full access.
- A vault item can belong to one or more collections.

---

**UR-ORG-005**: As an **organization owner or admin**, I want to **create user groups** so that I can **manage collection access for multiple members at once**.

*Acceptance Criteria:*
- I can create groups and add/remove members.
- I can assign collection access to a group rather than individual members.
- Member changes to a group are automatically reflected in collection access.

---

**UR-ORG-006**: As an **organization owner or admin**, I want to **revoke a member's access** so that **departing employees or contractors immediately lose access**.

*Acceptance Criteria:*
- Revoking a member's status immediately prevents them from accessing org collections.
- A revoked member's status can be restored without losing their role/collection assignments.

---

**UR-ORG-007**: As an **organization owner or admin**, I want to **recover a member's account** (password reset) so that **employees who forget their master password are not permanently locked out**.

*Acceptance Criteria:*
- I can initiate an admin-recovery password reset on a member's account if they have consented.
- The member must re-confirm their new password on next login.

---

### 4.6 Organization Owner / Admin — Access Control & Policies

---

**UR-POLICY-001**: As an **organization owner**, I want to **require all members to use two-factor authentication** so that **the organization's shared vault meets security compliance requirements**.

*Acceptance Criteria:*
- I can enable a "Require 2FA for all members" policy.
- Members without 2FA configured receive a warning and may be restricted until compliant.

---

**UR-POLICY-002**: As an **organization owner**, I want to **enforce a minimum master password strength** so that **members use passwords that meet our security standards**.

*Acceptance Criteria:*
- I can set a minimum password complexity score.
- Members are prompted to update their password if it does not meet the requirement.

---

**UR-POLICY-003**: As an **organization owner**, I want to **restrict members to being part of only this organization** so that **sensitive credentials are not mixed with other organizations' data**.

*Acceptance Criteria:*
- I can enable a "Single Organization" policy.
- Members who belong to multiple organizations are required to leave others before joining.

---

**UR-POLICY-004**: As an **organization owner**, I want to **create an organization API key** so that **automated processes (CI/CD pipelines, deployment scripts) can securely access shared vault items**.

*Acceptance Criteria:*
- I can generate and revoke an organization API key from the admin settings.
- The API key provides access scoped to the organization's vault.

---

### 4.7 Organization Owner / Admin — Audit & Compliance

---

**UR-AUDIT-001**: As an **organization owner or admin**, I want to **view a log of all actions taken within the organization** so that I can **monitor activity for security and compliance purposes**.

*Acceptance Criteria:*
- The event log captures: who performed an action, what action was taken, what item was affected, when it occurred, and from which IP/device.
- Events include: member login, vault item creation/update/deletion, collection changes, member invitations, role changes, and policy modifications.

---

**UR-AUDIT-002**: As an **organization owner**, I want to **retain audit logs for a configurable period** so that I can **meet regulatory or internal compliance requirements**.

*Acceptance Criteria:*
- The administrator can configure how long event logs are retained before automatic cleanup.

---

**UR-AUDIT-003**: As an **organization owner or admin**, I want to **export the event log** so that I can **archive or analyze events in external tools**.

*Acceptance Criteria:*
- Event data is accessible via the organization's events API endpoint.
- Exported data includes all captured fields (user, action, timestamp, IP).

---

### 4.8 Server Administrator — Instance Management

---

**UR-ADMIN-001**: As a **server administrator**, I want to **deploy Vaultwarden as a Docker container** so that I can **quickly set up a secure, isolated instance without complex installations**.

*Acceptance Criteria:*
- A single `docker run` command with the domain and data volume configured is sufficient to start the server.
- The server is accessible from official Bitwarden clients immediately after startup.
- Database migrations are applied automatically on first run.

---

**UR-ADMIN-002**: As a **server administrator**, I want to **access a web-based admin panel** so that I can **manage users, view server status, and change settings without touching the command line**.

*Acceptance Criteria:*
- The admin panel is accessible at `/admin`.
- Access requires a secure admin token (Argon2id hashed).
- I can view all registered users, invite users, and see server diagnostics.

---

**UR-ADMIN-003**: As a **server administrator**, I want to **configure the server entirely through environment variables** so that I can **integrate Vaultwarden into my existing infrastructure automation (Docker Compose, Kubernetes, Ansible)**.

*Acceptance Criteria:*
- All configuration options (database URL, SMTP, domain, features) are available as environment variables.
- Configuration changes made via the admin panel are persisted in `config.json`.
- Environment variables always take priority over `config.json` values.

---

**UR-ADMIN-004**: As a **server administrator**, I want to **back up the database** so that I can **recover data in the event of server failure**.

*Acceptance Criteria:*
- I can trigger a SQLite backup via CLI command (`vaultwarden backup`), Unix signal (`SIGUSR1`), or automated cron schedule.
- The backup creates a consistent snapshot of the database.
- *(PostgreSQL/MySQL deployments rely on external DB backup tools.)*

---

**UR-ADMIN-005**: As a **server administrator**, I want to **manage user registrations** so that I can **control who can create accounts on my server**.

*Acceptance Criteria:*
- I can restrict signups to invitation-only mode.
- I can restrict signups to specific email domains.
- I can require email verification before accounts become active.
- I can manually invite users from the admin panel.

---

**UR-ADMIN-006**: As a **server administrator**, I want to **disable user accounts** so that I can **immediately prevent access for suspended users without deleting their data**.

*Acceptance Criteria:*
- I can enable or disable any user account from the admin panel.
- A disabled user cannot log in until the account is re-enabled.

---

**UR-ADMIN-007**: As a **server administrator**, I want to **choose between SQLite, PostgreSQL, and MySQL/MariaDB** so that I can **use the database that best fits my infrastructure**.

*Acceptance Criteria:*
- I can configure the database via the `DATABASE_URL` environment variable.
- SQLite is the default for simple single-user or homelab deployments.
- PostgreSQL is available for multi-user or high-availability production deployments.

---

**UR-ADMIN-008**: As a **server administrator**, I want to **store attachments and Send files on S3-compatible object storage** so that I can **scale file storage independently of the server**.

*Acceptance Criteria:*
- I can configure an S3-compatible backend via environment variables.
- File read/write operations are transparently routed to the configured storage backend.
- Local filesystem storage is used by default if S3 is not configured.

---

### 4.9 Server Administrator — Configuration & Integration

---

**UR-ADMIN-010**: As a **server administrator**, I want to **configure SMTP email delivery** so that my users can **receive verification emails, 2FA alerts, and notifications**.

*Acceptance Criteria:*
- I can set SMTP host, port, credentials, and sender address via environment variables.
- The server sends emails for: account invitations, email verification, 2FA alerts, emergency access, and account deletion.
- STARTTLS and TLS are both supported.

---

**UR-ADMIN-011**: As a **server administrator**, I want to **enable SSO via an external Identity Provider** so that my users can **log in using our corporate credentials (e.g., Okta, Azure AD, Google Workspace)**.

*Acceptance Criteria:*
- I can configure an OIDC provider via `SSO_AUTHORITY`, `SSO_CLIENT_ID`, and `SSO_CLIENT_SECRET`.
- SSO login follows the standard OIDC authorization code flow with PKCE.
- New users can be auto-provisioned on first SSO login.
- SSO can coexist with standard username/password login.

---

**UR-ADMIN-012**: As a **server administrator**, I want to **enable mobile push notifications** so that my users' mobile apps **sync in real time without manual refresh**.

*Acceptance Criteria:*
- I can configure a push relay URI that handles APNs and FCM delivery.
- Push events are triggered for the same vault changes as WebSocket notifications.
- Push can be disabled if not needed.

---

**UR-ADMIN-013**: As a **server administrator**, I want to **enable WebSocket real-time sync** so that **users' Bitwarden clients receive vault updates instantly across all open sessions**.

*Acceptance Criteria:*
- I can enable WebSocket support via `ENABLE_WEBSOCKET=true`.
- WebSocket is disabled by default and must be explicitly enabled.
- Multiple devices per user are supported concurrently.

---

**UR-ADMIN-014**: As a **server administrator**, I want to **configure structured logging** so that I can **troubleshoot issues and integrate with log aggregation systems**.

*Acceptance Criteria:*
- I can set the log level, log file path, timestamp format, and enable extended logging.
- Sensitive values (passwords, tokens) are masked in all log output.
- SQL query logging can be enabled for debugging database interactions.

---

**UR-ADMIN-015**: As a **server administrator**, I want to **disable features I don't need** (e.g., Send, web vault, WebSocket) so that I can **reduce the attack surface of my deployment**.

*Acceptance Criteria:*
- I can disable Bitwarden Send via `SENDS_ALLOWED=false`.
- I can disable the web vault via `WEB_VAULT_ENABLED=false`.
- I can disable the admin panel token check via `DISABLE_ADMIN_TOKEN` (for environments using external authentication).
- I can disable the favicon proxy independently.

---

## 5. Cross-Cutting User Needs

### 5.1 Security & Privacy

**UR-SEC-001**: As **any user**, I want my vault data to be **encrypted end-to-end** so that **the server operator can never read my passwords or personal information**.

> *The server stores only encrypted blobs. Encryption and decryption occur exclusively on the client using a key derived from the master password. The server has no access to plaintext data at any time.*

**UR-SEC-002**: As **any user**, I want **sensitive operations to require re-confirmation** so that **an unattended logged-in session cannot be used to export my data or disable my 2FA**.

> *Re-authentication is required (master password or email OTP) before: disabling 2FA, exporting the vault, or performing other high-risk operations.*

**UR-SEC-003**: As **any user**, I want to be **protected against brute-force attacks** so that **automated guessing of my master password is not feasible**.

> *Login, 2FA, and registration endpoints are rate-limited per IP address.*

**UR-SEC-004**: As **any user**, I want to trust that **the server software has no intentionally unsafe code paths** so that I can **deploy it with confidence**.

> *The Rust codebase enforces `#![forbid(unsafe_code)]` at the compiler level.*

---

### 5.2 Multi-Device & Real-Time Sync

**UR-SYNC-001**: As **any user**, I want **vault changes to appear on all my devices automatically** so that I **never work with stale data**.

> *Real-time sync is provided via WebSocket (when enabled) and mobile push relay. All Bitwarden clients support automatic background sync.*

**UR-SYNC-002**: As **any user**, I want to **use multiple devices simultaneously** without sync conflicts so that my **vault remains consistent**.

> *Each device maintains its own authenticated session. The server applies changes sequentially and propagates updates to all connected sessions.*

---

### 5.3 Multi-Factor Authentication

**UR-MFA-001**: As **any user**, I want **multiple 2FA options** so that I can **choose the method that best fits my security posture and available hardware**.

| Method | When to Use |
|--------|------------|
| TOTP (Authenticator App) | Best balance of security and convenience |
| Email OTP | Fallback when no authenticator app is available |
| FIDO2 / WebAuthn | Highest security (phishing-resistant hardware key) |
| YubiKey OTP | Hardware key with OTP support |
| Duo Security | Enterprise 2FA with push approval |
| Recovery Code | Emergency fallback if primary 2FA is lost |

**UR-MFA-002**: As **any user**, I want to **remember a trusted device** so that I **don't have to enter 2FA every time I log in on devices I own**.

**UR-MFA-003**: As **any user**, I want **recovery codes** so that I can **regain access to my account if I lose my 2FA device**.

---

### 5.4 Usability & Client Compatibility

**UR-COMPAT-001**: As **any user**, I want to **use the standard Bitwarden client apps without any special configuration** so that I can **benefit from Vaultwarden with minimal setup**.

> *Vaultwarden is fully API-compatible with official Bitwarden clients. Users only need to point the client to the Vaultwarden server URL.*

**UR-COMPAT-002**: As **any user**, I want the server to **remain compatible with future Bitwarden client updates** so that I **don't experience breakage when my client app auto-updates**.

---

## 6. User Constraints & Expectations

| # | Constraint | Impact on Users |
|---|-----------|----------------|
| UC-01 | The server must be placed behind a reverse proxy (nginx/Caddy) that handles HTTPS. | Users must access Vaultwarden over HTTPS only; plain HTTP is not supported. |
| UC-02 | All encryption is client-side; the server is a blind encrypted store. | If a user forgets their master password and has no recovery mechanism, vault data cannot be recovered. |
| UC-03 | File upload size is limited to 525 MB per upload. | Very large attachments need to be compressed or split before uploading. |
| UC-04 | WebSocket notifications must be explicitly enabled by the server administrator. | If not enabled, clients will sync on a polling interval rather than in real time. |
| UC-05 | Push notifications require an external relay server. | The server admin must configure the relay; mobile sync may be delayed without it. |
| UC-06 | SSO requires an external Identity Provider configured by the server admin. | SSO is not self-service for end users; it must be set up at the infrastructure level. |
| UC-07 | S3 file storage requires a compile-time feature flag. | The Docker image must include S3 support if object storage is desired. |
| UC-08 | The server is licensed under AGPL-3.0. | Organizations that modify and deploy Vaultwarden must make their modifications available. |

---

## 7. Acceptance Criteria Summary

The following table summarizes the high-level acceptance criteria for validating user requirements:

| User Story ID | Feature | Acceptance Signal |
|--------------|---------|------------------|
| UR-USER-001 | Registration | New account activated; email verified if required |
| UR-USER-002 | Multi-client login | All official Bitwarden clients authenticate successfully |
| UR-USER-003 | Vault item CRUD | Items created/edited/deleted and reflected in sync |
| UR-USER-012 | 2FA setup | 2FA enabled; login requires second factor |
| UR-USER-013 | Passwordless login | Device approval flow completes; session established |
| UR-SEND-001 | Send creation | Recipient accesses Send via URL without an account |
| UR-SEND-002 | Password-protected Send | Access denied without correct password |
| UR-EMRG-001 | Emergency access delegation | Grantee can request and receive access after wait period |
| UR-ORG-001 | Organization creation | Org created; owner can invite members |
| UR-ORG-004 | Collections | Items assigned to collections; access enforced by role |
| UR-POLICY-001 | 2FA requirement policy | Members without 2FA warned/restricted |
| UR-AUDIT-001 | Event log | All org actions recorded with correct metadata |
| UR-ADMIN-001 | Docker deployment | Server starts; clients connect; migrations applied |
| UR-ADMIN-002 | Admin panel | Admin can manage users and view diagnostics |
| UR-ADMIN-010 | SMTP email | Verification/notification emails delivered |
| UR-ADMIN-011 | SSO/OIDC | Users can log in via configured IdP |

---

## 8. Glossary

| Term | Plain-Language Definition |
|------|--------------------------|
| **2FA / MFA** | A second verification step required at login beyond your password |
| **Admin Panel** | A web interface only the server administrator can access, at `/admin` |
| **AES-256** | Industry-standard symmetric encryption algorithm used to protect vault data |
| **Argon2id** | A secure password hashing algorithm used to protect the admin token |
| **Cipher** | A single item in your vault (login, card, note, identity, or SSH key) |
| **Collection** | A folder that belongs to an organization and can be shared with multiple members |
| **Emergency Access** | A feature that lets you designate a trusted person to access your vault in an emergency |
| **End-to-End Encryption (E2EE)** | Encryption where only the user (not the server) can decrypt the data |
| **FIDO2 / WebAuthn** | A standard for phishing-resistant hardware security keys (e.g., YubiKey) |
| **Grantee** | The person designated to receive emergency access to a vault |
| **Grantor** | The vault owner who grants emergency access to a grantee |
| **Group** | A named set of organization members used to assign collection access in bulk |
| **Identity Provider (IdP)** | An external service that authenticates users for SSO (e.g., Okta, Azure AD) |
| **Master Password** | The primary password used to derive the encryption key — never sent to or stored by the server |
| **OIDC** | OpenID Connect — the protocol used for Single Sign-On integration |
| **OpenDAL** | The file storage abstraction layer used by Vaultwarden (supports local disk and S3) |
| **Organization** | A shared workspace on Vaultwarden where a team can collaborate on vault items |
| **PKCE** | A security extension for OAuth/OIDC that prevents code interception |
| **Push Notification** | A real-time notification sent to a mobile app to trigger a vault sync |
| **Rate Limiting** | Automatic restriction of failed login attempts to prevent brute-force attacks |
| **Recovery Code** | A one-time-use backup code used to access an account if the primary 2FA method is unavailable |
| **Reverse Proxy** | A server (nginx, Caddy) that sits in front of Vaultwarden and handles HTTPS |
| **Role** | A defined level of permission within an organization (Owner, Admin, Manager, User, Custom) |
| **S3** | Amazon S3 or any S3-compatible object storage service (e.g., MinIO) |
| **Send** | A feature for sharing encrypted text or files via a one-time secure link |
| **Session** | An authenticated login on a device; sessions expire after a set time |
| **Single Sign-On (SSO)** | Logging in using a corporate identity account rather than a direct Vaultwarden password |
| **SQLite / PostgreSQL / MySQL** | Database backends supported by Vaultwarden |
| **TOTP** | Time-Based One-Time Password — the 6-digit code generated by authenticator apps |
| **Vault** | Your encrypted collection of passwords and other sensitive items |
| **WebSocket** | A technology enabling real-time, two-way communication between the server and client for instant sync |
| **YubiKey** | A physical hardware security token used for 2FA authentication |

---

*End of Document*
