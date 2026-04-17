# Vaultwarden — User Requirements Document v1 (Enterprise Edition)

> **Document Version**: 1.1  
> **Date**: 2026-04-12  
> **Status**: Draft  
> **Supersedes**: `specs/urd.md` (v1.0)  
> **References**:
> - Product Requirements Document v1: `specs/PRD_v1.md`
> - Change Requests: `specs/crs/v1/`
> - Software Requirements Specification: `specs/srs.md`

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [User Profiles & Goals](#2-user-profiles--goals)
3. [Use Cases Overview](#3-use-cases-overview)
4. [User Requirements — Original (v1.0)](#4-user-requirements--original-v10) *(unchanged)*
5. [Enterprise User Requirements (v2.x additions)](#5-enterprise-user-requirements-v2x-additions)
   - 5.1 [Enterprise: Audit & Compliance](#51-enterprise-audit--compliance-cr-002-cr-001)
   - 5.2 [Enterprise: Identity & User Lifecycle](#52-enterprise-identity--user-lifecycle-cr-003)
   - 5.3 [Enterprise: Access Governance](#53-enterprise-access-governance-cr-004)
   - 5.4 [Enterprise: High Availability & DR](#54-enterprise-high-availability--dr-cr-005-cr-006)
   - 5.5 [Enterprise: Privileged Access](#55-enterprise-privileged-access-cr-007)
   - 5.6 [Enterprise: API & Integrations](#56-enterprise-api--integrations-cr-008)
   - 5.7 [Enterprise: Device Trust & MDM](#57-enterprise-device-trust--mdm-cr-009)
   - 5.8 [Enterprise: Monitoring & Observability](#58-enterprise-monitoring--observability-cr-010)
   - 5.9 [Enterprise: Multi-Tenancy](#59-enterprise-multi-tenancy-cr-011)
6. [Cross-Cutting Enterprise Needs](#6-cross-cutting-enterprise-needs)
7. [Enterprise Constraints & Expectations](#7-enterprise-constraints--expectations)
8. [Enterprise Acceptance Criteria Summary](#8-enterprise-acceptance-criteria-summary)
9. [Glossary](#9-glossary)

---

## 1. Introduction

### 1.1 Purpose

This v1 URD extends the original URD (v1.0) with enterprise user requirements for banking, financial services institutions (FSI), and large enterprises (>10,000 employees, >100M customers).

All requirements from URD v1.0 remain in effect. This document adds the following new user classes and requirements needed to serve regulated industries.

### 1.2 New User Classes (v2.x)

| New User Class | Description |
|----------------|-------------|
| **Compliance Officer** | Reviews audit logs, generates compliance evidence, manages retention policy |
| **Tenant Administrator** | Department-level admin with visibility scoped to their tenant |
| **Privileged User** | Employee accessing privileged/shared service credentials |
| **Security Analyst** | Monitors security events, investigates alerts, reviews access reports |
| **IT/IAM Engineer** | Configures LDAP sync, SCIM, SSO, MDM integration |
| **DevOps Engineer** | Integrates vault secrets into CI/CD pipelines |
| **External Auditor** | Reads compliance evidence; cannot modify anything |

---

## 2. User Profiles & Goals

*(Profiles 2.1–2.4 from URD v1.0 are unchanged)*

### 2.5 Enterprise Compliance Officer

| Attribute | Description |
|-----------|-------------|
| **Who** | Internal compliance/risk officer or external QSA auditor |
| **Primary Goal** | Evidence that credential management meets regulatory requirements (PCI DSS, SOC 2) |
| **Key Concern** | Audit trail completeness, log integrity, retention policy |
| **Technical Level** | Low-medium — uses web vault + admin panel + export API |

### 2.6 Tenant Administrator (Department Lead)

| Attribute | Description |
|-----------|-------------|
| **Who** | Head of IT for Treasury, HR, Retail Banking, or other department |
| **Primary Goal** | Manage credentials and access for their department independently |
| **Key Concern** | Cannot see other departments' data; can manage their own users and policies |
| **Technical Level** | Medium |

### 2.7 Security Analyst (SOC)

| Attribute | Description |
|-----------|-------------|
| **Who** | Security Operations Center analyst |
| **Primary Goal** | Detect and investigate suspicious credential access activity |
| **Key Concern** | Real-time SIEM alerts, audit log search, failed login patterns |
| **Technical Level** | High — works with Splunk/SIEM |

### 2.8 DevOps / Platform Engineer

| Attribute | Description |
|-----------|-------------|
| **Who** | Developer or infrastructure engineer needing secrets for CI/CD |
| **Primary Goal** | Inject secrets into pipelines without hardcoding or manual copy-paste |
| **Key Concern** | API access, scoped tokens, no excessive permissions |
| **Technical Level** | High |

---

## 3. Use Cases Overview

*(UC-01 to UC-10 from URD v1.0 are unchanged)*

| Use Case Group | Primary Actor | Summary |
|----------------|--------------|---------|
| UC-01 to UC-10 | *(unchanged)* | *(see URD v1.0)* |
| **UC-11: Compliance Audit** | Compliance Officer | Generate evidence for PCI DSS/SOC 2 audit |
| **UC-12: User Lifecycle (SCIM/LDAP)** | IAM Engineer | Auto-provision/deprovision via SCIM or LDAP |
| **UC-13: Privileged Access** | Privileged User / Security Analyst | Checkout privileged credential with approval |
| **UC-14: Tenant Management** | System Admin / Tenant Admin | Create tenant; delegate administration |
| **UC-15: Security Monitoring** | Security Analyst | Receive SIEM alerts; investigate access events |
| **UC-16: DevOps Secret Injection** | DevOps Engineer | Use API key to inject secrets into CI/CD |

---

## 4. User Requirements — Original (v1.0)

*(All requirements from URD v1.0 §4.1–4.9 are retained without modification.)*

For full text, see `specs/urd.md` §4.

Key requirements retained:
- UR-USER-001 to UR-USER-015 (End User — vault, account, security)
- UR-SEND-001 to UR-SEND-005 (Bitwarden Send)
- UR-EMRG-001 to UR-EMRG-003 (Emergency Access)
- UR-ORG-001 to UR-ORG-007 (Organization management)
- UR-POLICY-001 to UR-POLICY-004 (Org policies)
- UR-AUDIT-001 to UR-AUDIT-003 (Audit logging — enhanced in §5.1)
- UR-ADMIN-001 to UR-ADMIN-015 (Server administrator)

---

## 5. Enterprise User Requirements (v2.x additions)

### 5.1 Enterprise: Audit & Compliance (CR-002, CR-001)

---

**UR-ENT-AUDIT-001**: As a **compliance officer**, I want to **access a complete, tamper-evident audit trail** covering all system events so that I can **provide evidence to PCI DSS, SOC 2, and ISO 27001 auditors**.

*Acceptance Criteria:*
- Audit log covers: login success/failure, admin actions, config changes, file downloads, privileged access, security events.
- Each log entry includes: actor, action, target resource, IP address, device, timestamp.
- Hash chain integrity is verifiable via `GET /api/audit/verify-chain`.
- Log cannot be deleted by any user including system administrator.

---

**UR-ENT-AUDIT-002**: As a **compliance officer**, I want to **export audit logs to our SIEM system (Splunk/Sentinel)** so that I can **correlate vault events with other security signals**.

*Acceptance Criteria:*
- SIEM forwarding configured via environment variables.
- Events delivered within 10 seconds of occurrence.
- HMAC-signed payloads prevent tampering in transit.
- Retry on delivery failure with configurable retry count.

---

**UR-ENT-AUDIT-003**: As a **compliance officer**, I want to **configure a minimum log retention period** so that logs are **retained for the 7-year regulatory requirement** without risk of premature deletion.

*Acceptance Criteria:*
- `AUDIT_RETENTION_MINIMUM_DAYS` enforced — admin cannot set below this value.
- Old entries archived (not deleted) when retention policy rotates.
- Retention period logged in compliance evidence.

---

**UR-ENT-AUDIT-004**: As a **compliance officer**, I want to **generate a compliance evidence report** for PCI DSS Requirement 10 so that I can **satisfy the QSA's documentation request within an hour**.

*Acceptance Criteria:*
- `GET /api/compliance/evidence?standard=pci_dss` returns structured JSON evidence.
- Report covers configurable date range.
- Includes: user access list, failed login summary, config change log, MFA compliance rate.
- PDF export available.

---

**UR-ENT-AUDIT-005**: As a **compliance officer**, I want to **receive a data processing register** so that I can **satisfy GDPR Article 30 documentation requirements**.

*Acceptance Criteria:*
- Register lists all PII categories stored (email, name, IP addresses).
- Includes purpose, legal basis, retention period per category.
- Exportable as PDF.

---

**UR-ENT-COMP-001**: As a **compliance officer**, I want to **configure data residency restrictions** so that I can **ensure vault data never leaves the approved region (e.g., Vietnam, Singapore)**.

*Acceptance Criteria:*
- S3 upload blocked if target bucket is in non-approved region when enforcement is enabled.
- Residency configuration logged at startup.
- Admin dashboard shows current residency configuration.

---

**UR-ENT-COMP-002**: As an **end user (GDPR)**, I want to **request deletion of all my personal data** so that my **right to erasure under GDPR Article 17 is fulfilled**.

*Acceptance Criteria:*
- Erasure request triggers automated pipeline.
- PII (email, name, IP logs) deleted or anonymized within 30 days.
- User receives erasure confirmation with audit receipt.
- Vault data (encrypted blobs) purged as part of account deletion.

---

**UR-ENT-COMP-003**: As an **end user (GDPR)**, I want to **export all my data** so that I can **exercise my right to data portability under GDPR Article 20**.

*Acceptance Criteria:*
- Export includes: vault items, folder structure, send history, org memberships, event history.
- Format: encrypted Bitwarden-compatible export (can be imported into another instance).

---

### 5.2 Enterprise: Identity & User Lifecycle (CR-003)

---

**UR-ENT-IAM-001**: As an **IAM engineer**, I want to **synchronize users and groups from our Active Directory** so that **employees automatically get vault access when they join, and lose it when they leave**.

*Acceptance Criteria:*
- LDAP connector syncs users on configurable interval (default: 15 minutes).
- New LDAP users auto-provisioned with correct collection access based on AD group.
- Disabled/removed LDAP users have vault access revoked within one sync cycle.
- Sync status and last run time visible in admin panel.

---

**UR-ENT-IAM-002**: As an **IAM engineer**, I want to **configure SCIM 2.0** so that **Azure AD / Okta automatically manages Vaultwarden users** without any manual IT intervention.

*Acceptance Criteria:*
- SCIM endpoints pass Azure AD SCIM conformance tests.
- User deactivation via SCIM (`active: false`) revokes all sessions immediately.
- Group membership changes propagate to collection access within 5 minutes.

---

**UR-ENT-IAM-003**: As an **IT administrator**, I want to **conduct periodic access reviews** so that I can **validate that all active users still require their current level of access** (SOC 2 CC6 requirement).

*Acceptance Criteria:*
- Quarterly access review email sent to org owners.
- Review UI shows: user, role, collections, last active date.
- Unreviewed accesses automatically revoked after configurable deadline.
- Review completion logged in audit trail.

---

**UR-ENT-IAM-004**: As an **organization owner**, I want to **configure Just-In-Time provisioning via SSO** that maps IdP groups to vault collections, so that **new employees' access is determined by their role in the IdP** without manual collection assignment.

*Acceptance Criteria:*
- OIDC `groups` claim mapped to Vaultwarden collections via configuration.
- First SSO login provisions user with correct collection access.
- Group removal in IdP → collection access removed on next login.

---

**UR-ENT-IAM-005**: As a **system administrator**, I want to **restrict SSO auto-provisioning to users with specific IdP group membership** so that **not all corporate directory users can access the vault**.

*Acceptance Criteria:*
- `SSO_JIT_REQUIRED_GROUP` setting: only users in this group are provisioned.
- Users outside the group receive "access denied" with instructions to request access.

---

### 5.3 Enterprise: Access Governance (CR-004)

---

**UR-ENT-RBAC-001**: As an **organization owner**, I want to **create custom roles with specific permission sets** so that I can **implement least-privilege access** for different job functions.

*Acceptance Criteria:*
- Custom role created with subset of available permissions.
- User with custom role cannot perform actions outside their permission set.
- Role definitions logged when created/modified.

---

**UR-ENT-RBAC-002**: As a **banking operations manager**, I want to **restrict credential access to business hours (Mon–Fri, 08:00–18:00 ICT)** so that **after-hours access attempts are automatically blocked and alerted**.

*Acceptance Criteria:*
- Access outside configured hours returns 403 with message.
- Blocked attempt logged in audit trail with audit event.
- Security alert sent on after-hours access attempt.
- Timezone specified per schedule; DST handled correctly.

---

**UR-ENT-RBAC-003**: As a **security officer**, I want to **restrict vault access to users connecting from approved IP ranges** so that **credentials cannot be accessed from personal devices on home networks**.

*Acceptance Criteria:*
- Request from non-approved IP for restricted collection → 403 + audit event.
- IP allowlist configurable per organization or per collection.
- Admin panel always enforced by IP allowlist regardless of collection settings.

---

**UR-ENT-RBAC-004**: As a **security officer**, I want to **require dual approval for access to sensitive credentials** so that **no single person can access critical passwords without oversight** (four-eyes principle).

*Acceptance Criteria:*
- Access request triggers notification to configured approver group.
- Access granted only after explicit approval.
- Approval has configurable time window; expired request denied.
- Both requester and approver logged in audit trail with justification.

---

**UR-ENT-RBAC-005**: As a **CISO**, I want a **break-glass account procedure** so that **authorized personnel can access any credential in an emergency** while ensuring full accountability.

*Acceptance Criteria:*
- Break-glass activation requires mandatory justification.
- All configured witnesses notified immediately via email.
- All actions during break-glass session flagged in audit log.
- Break-glass session expires after configured time (default: 4 hours).

---

**UR-ENT-RBAC-006**: As an **organization owner**, I want to **define separation of duties rules** so that **the same user cannot hold conflicting roles** (e.g., Vault Admin + Auditor).

*Acceptance Criteria:*
- SoD rule assignment attempt blocked when rule is configured as "Hard".
- SoD violation generates warning + audit event for "Soft" enforcement.
- Existing violations listed in compliance dashboard.

---

### 5.4 Enterprise: High Availability & DR (CR-005, CR-006)

---

**UR-ENT-HA-001**: As a **server administrator (banking)**, I want to **deploy Vaultwarden as a multi-instance cluster** so that **server maintenance and failures do not cause vault downtime**.

*Acceptance Criteria:*
- 3-instance cluster: killing any one instance causes no user-visible errors.
- Load balancer health check correctly routes away from unhealthy instances.
- WebSocket events delivered across instances via Redis pub/sub.

---

**UR-ENT-HA-002**: As a **server administrator**, I want **zero-downtime rolling upgrades** so that I can **update Vaultwarden without a maintenance window**.

*Acceptance Criteria:*
- New instance passes health check before receiving traffic.
- Old instance drains connections gracefully within configured timeout.
- No 5xx errors during rolling upgrade in load test.

---

**UR-ENT-HA-003**: As a **server administrator**, I want **automated database backup with daily verification** so that I can **prove to auditors that our recovery capability is tested and working**.

*Acceptance Criteria:*
- Backup runs on schedule; failure triggers alert within 5 minutes.
- Nightly verification restore completes successfully; result logged.
- `GET /api/admin/backup/status` shows last backup time, hash, and verification result.

---

**UR-ENT-DR-001**: As a **bank IT manager**, I want to **know the documented RTO and RPO** for Vaultwarden so that I can **include it in our Business Continuity Plan**.

*Acceptance Criteria:*
- RTO and RPO documented in product for each deployment mode.
- `GET /api/admin/dr-runbook` generates current deployment's DR procedure.
- DR runbook includes step-by-step restore instructions.

---

**UR-ENT-DR-002**: As a **server administrator**, I want to **restore to a specific point in time** after a data corruption incident so that I can **recover with minimal data loss**.

*Acceptance Criteria:*
- PITR available within configured retention window.
- `POST /api/admin/backup/restore` accepts target timestamp and justification.
- Restore creates audit entry with timestamp, actor, justification.

---

**UR-ENT-DR-003**: As a **compliance officer**, I want backups stored in a **geographically separate region** so that **regional disasters do not compromise recovery capability**.

*Acceptance Criteria:*
- Secondary backup destination in different region configured.
- Data replicated to secondary within 15 minutes.
- Admin dashboard shows secondary backup status.

---

**UR-ENT-DR-004**: As a **server administrator**, I want **backup files to be encrypted and integrity-verified** so that I can **trust restored data has not been tampered with**.

*Acceptance Criteria:*
- Backup files encrypted at rest with configured KMS key.
- Manifest includes SHA-256 checksum and digital signature.
- Verification rejects tampered backup files.

---

### 5.5 Enterprise: Privileged Access (CR-007)

---

**UR-ENT-PAM-001**: As a **privileged user**, I want to **check out a privileged credential with a time limit and mandatory justification** so that **my access is accountable and time-bounded**.

*Acceptance Criteria:*
- Checkout request requires justification text.
- Credential accessible only during checkout period.
- Credential access automatically expires after configured time.
- Audit log records: requester, credential, checkout time, expiry, justification.

---

**UR-ENT-PAM-002**: As a **security officer**, I want **privileged credentials to be automatically rotated after each checkout** so that **credentials are never reused by different users or compromised via session replay**.

*Acceptance Criteria:*
- Auto-rotation triggered on checkout expiry when configured.
- Rotation connects to target system (SSH, database) and changes password.
- Rotation failure triggers alert; manual rotation option available.
- New credential stored in vault; old credential invalidated.

---

**UR-ENT-PAM-003**: As a **security officer**, I want to **require a valid ITSM ticket number** before a privileged credential checkout so that **all privileged access is linked to an approved work order**.

*Acceptance Criteria:*
- ServiceNow ticket validated before checkout granted.
- Closed/invalid ticket blocks checkout.
- Ticket number recorded in checkout audit entry.

---

**UR-ENT-PAM-004**: As an **IT manager**, I want to **view all active privileged credential checkouts** in real time so that I can **monitor who has access to critical systems at any moment**.

*Acceptance Criteria:*
- Dashboard shows: credential name, current holder, checkout time, expiry, justification.
- Admin can force-expire a checkout.
- Force-expiry generates audit event.

---

**UR-ENT-PAM-005**: As a **security analyst**, I want to **search the complete checkout history** by user, credential, and date range so that I can **investigate post-incident**.

*Acceptance Criteria:*
- Checkout history searchable via audit log API (CR-002).
- Results include all checkout fields (requester, approver, duration, justification).
- Export as CSV for offline analysis.

---

### 5.6 Enterprise: API & Integrations (CR-008)

---

**UR-ENT-API-001**: As a **DevOps engineer**, I want to **create scoped API keys** for CI/CD pipelines so that **each pipeline has only the minimum permissions needed** and keys can be rotated independently.

*Acceptance Criteria:*
- API key created with specific scopes (e.g., `SecretsRead` only).
- Key with `VaultRead` scope cannot create or delete vault items.
- Key rotation generates new key without deleting old one until confirmed.

---

**UR-ENT-API-002**: As a **DevOps engineer**, I want to **retrieve secrets in environment variable format** so that I can **inject them into GitHub Actions / GitLab CI / Jenkins** without custom parsing.

*Acceptance Criteria:*
- `GET /api/secrets/export?format=env` returns `KEY=value` pairs.
- Only secrets the API key has permission to access are returned.
- Secrets access logged in audit trail.

---

**UR-ENT-API-003**: As a **security officer**, I want **webhook notifications** when sensitive events occur (member removed, config changed, privileged checkout) so that **our SOC team is notified in real time**.

*Acceptance Criteria:*
- Webhook delivers events within 10 seconds.
- HMAC-SHA256 signature validates payload authenticity.
- Delivery failure retried 3 times with exponential backoff.
- Delivery history viewable in admin panel.

---

**UR-ENT-API-004**: As a **DevOps engineer**, I want to **manage vault collections and user access via Terraform** so that **infrastructure changes are version-controlled and auditable**.

*Acceptance Criteria:*
- Terraform provider reads secrets from Vaultwarden.
- Terraform state reflects actual collection membership.
- Provider published to Terraform Registry.

---

### 5.7 Enterprise: Device Trust & MDM (CR-009)

---

**UR-ENT-MDM-001**: As a **security officer**, I want to **restrict vault access to MDM-enrolled, compliant devices** so that **employees cannot access vault from personal or unmanaged devices**.

*Acceptance Criteria:*
- Intune non-compliant device → login denied with message and audit event.
- Jamf unenrolled device → login denied.
- Compliant device → login proceeds normally.
- Compliance check result cached to avoid performance impact.

---

**UR-ENT-MDM-002**: As a **security officer**, I want to **require devices to present a corporate client certificate** so that **device identity is cryptographically verified**.

*Acceptance Criteria:*
- Login without valid client certificate → 401 Unauthorized.
- Revoked certificate (CRL/OCSP) → login denied.
- Certificate expiry within 30 days → admin alert generated.

---

**UR-ENT-MDM-003**: As an **IT administrator**, I want to **remotely revoke vault access for a specific device** so that I can **immediately respond to a lost or stolen device incident**.

*Acceptance Criteria:*
- `POST /api/devices/{id}/wipe` revokes device token immediately.
- Push notification sent to device (if device still has connectivity).
- Revocation logged in audit trail.
- Device cannot authenticate until re-enrolled.

---

### 5.8 Enterprise: Monitoring & Observability (CR-010)

---

**UR-ENT-OBS-001**: As a **DevOps engineer**, I want a **Prometheus metrics endpoint** so that I can **monitor Vaultwarden health and performance in Grafana alongside other services**.

*Acceptance Criteria:*
- `GET /metrics` returns valid Prometheus text format.
- Metrics include: login rates, active sessions, DB query time, email delivery, error rates.
- Endpoint protected by Bearer token + IP allowlist.

---

**UR-ENT-OBS-002**: As an **operations engineer**, I want **structured JSON logs** so that I can **parse and index vault logs in our centralized logging platform (ELK/Splunk)**.

*Acceptance Criteria:*
- `LOG_FORMAT=json` produces one valid JSON object per log line.
- Each entry includes: timestamp, level, message, trace_id, user_id (if applicable), IP.
- Sensitive fields (passwords, tokens) masked as `***`.

---

**UR-ENT-OBS-003**: As an **operations engineer**, I want **Kubernetes-compatible health check endpoints** so that I can **configure readiness and liveness probes** and ensure traffic is not routed to unhealthy instances.

*Acceptance Criteria:*
- `/health/ready` returns 503 when database is unreachable.
- `/health/live` returns 503 when process is deadlocked.
- Health check response includes subsystem status (DB, Redis, email, storage).

---

**UR-ENT-OBS-004**: As a **security analyst**, I want to **receive automatic alerts** when suspicious patterns are detected (many failed logins, admin access, config change) so that I can **investigate before damage occurs**.

*Acceptance Criteria:*
- Email alert sent when failed logins exceed threshold per minute.
- Admin panel login always triggers notification (configurable).
- Config change event sent to configured webhook.
- Alerts configurable without server restart.

---

### 5.9 Enterprise: Multi-Tenancy (CR-011)

---

**UR-ENT-TENANT-001**: As a **system administrator (bank)**, I want to **create isolated tenants for each department** so that **Treasury cannot view HR credentials and vice versa**, even on the same server instance.

*Acceptance Criteria:*
- Tenant A administrator cannot list or access users/orgs/ciphers from Tenant B.
- Direct database query with Tenant A credentials cannot return Tenant B rows (PostgreSQL RLS).
- Tenant boundary violation attempt generates security alert.

---

**UR-ENT-TENANT-002**: As a **department IT manager (Tenant Admin)**, I want to **manage users, organizations, and policies within my department** without requiring system administrator involvement for routine tasks.

*Acceptance Criteria:*
- Tenant Admin can invite, enable, disable, remove users within their tenant.
- Tenant Admin can create orgs and collections within their tenant.
- Tenant Admin cannot modify system-level settings.
- Tenant Admin can view audit logs scoped to their tenant only.

---

**UR-ENT-TENANT-003**: As a **system administrator**, I want to **set per-tenant resource quotas** so that one department cannot consume all server resources.

*Acceptance Criteria:*
- User creation blocked when tenant reaches `max_users` limit.
- Storage upload blocked when tenant reaches `max_storage_bytes`.
- Quota usage visible in system admin dashboard.

---

**UR-ENT-TENANT-004**: As a **system administrator**, I want to **migrate existing single-tenant data to a multi-tenant setup** without data loss or downtime.

*Acceptance Criteria:*
- Migration tool assigns all existing users/orgs/ciphers to DEFAULT tenant.
- Migration runs as a database migration step; reversible.
- Post-migration: all features work as before for users in DEFAULT tenant.
- New tenants can be created and users migrated without service interruption.

---

## 6. Cross-Cutting Enterprise Needs

### 6.1 Zero-Trust Security Model

**UR-ENT-ZT-001**: As **any enterprise user**, I want every vault access decision to be **contextual** — considering not just "who am I" but also "what device, what network, what time, what approval state" — so that **even a compromised password alone is not enough to access vault**.

*Supporting features*: F-AUTH (MFA), F-MDM (device), F-RBAC (time/location), F-RBAC (dual approval)

### 6.2 Immutable Audit Trail

**UR-ENT-ZT-002**: As **any compliance stakeholder**, I want to **trust that audit logs cannot be altered** — including by system administrators — so that I can **rely on them as evidence in regulatory audits and incident investigations**.

*Supporting features*: F-AUDIT-V2 (hash chain, append-only, SIEM offload)

### 6.3 Least Privilege

**UR-ENT-ZT-003**: As **any enterprise user**, I want **access to be granted only to what is needed, when it is needed** — not blanket access — so that **the blast radius of any compromise is minimized**.

*Supporting features*: F-RBAC (custom roles, time-based), F-PAM (checkout, time-limited), F-IAM (SCIM-managed groups)

### 6.4 Regulatory Evidence

**UR-ENT-ZT-004**: As **any regulated entity**, I want the product to **produce evidence in formats that regulators and auditors recognize** so that **compliance demonstrations are efficient and credible**.

*Supporting features*: F-COMPLIANCE (evidence API, reports), F-AUDIT-V2 (SIEM, retention)

---

## 7. Enterprise Constraints & Expectations

| # | Constraint | Impact on Enterprise Users |
|---|-----------|---------------------------|
| EC-01 | TLS termination at reverse proxy required | Same as before; additionally mTLS for device certs may require pass-through |
| EC-02 | E2E encryption: server never sees plaintext | Even with PAM access, audit logs show metadata not plaintext passwords |
| EC-03 | HA cluster mode requires Redis | Operators must provision Redis Sentinel or Redis Cluster for production HA |
| EC-04 | SCIM requires IdP SCIM app configuration | IAM engineer must configure Azure AD / Okta SCIM app pointing to Vaultwarden |
| EC-05 | LDAP sync requires service account | Active Directory service account with read-only access needed |
| EC-06 | MDM compliance check uses cached result | Compliance status cached for `INTUNE_COMPLIANCE_CACHE_SECONDS`; near-real-time, not instant |
| EC-07 | Audit log export requires admin-level auth + re-auth | Compliance officers need admin panel access or delegated audit-read role |
| EC-08 | PostgreSQL required for HA and multi-tenancy (RLS) | SQLite not supported in cluster or multi-tenant mode |
| EC-09 | Data residency enforcement is upload-time only | Existing files not retroactively moved; applies to new uploads |
| EC-10 | PAM auto-rotation Phase 1: SSH + MySQL/PostgreSQL only | Other target types (Windows RDP, Oracle DB) planned for future releases |

---

## 8. Enterprise Acceptance Criteria Summary

| User Story ID | Feature | Acceptance Signal |
|--------------|---------|------------------|
| UR-ENT-AUDIT-001 | System-wide audit log | Hash chain verified; cannot delete entries |
| UR-ENT-AUDIT-002 | SIEM forwarding | Events in Splunk within 10 seconds |
| UR-ENT-AUDIT-003 | Retention enforcement | Min retention cannot be reduced below configured floor |
| UR-ENT-AUDIT-004 | Compliance evidence | PCI DSS report generated in < 60 seconds |
| UR-ENT-COMP-001 | Data residency | Upload to non-compliant region blocked |
| UR-ENT-COMP-002 | GDPR erasure | PII deleted + receipt within 30 days |
| UR-ENT-IAM-001 | LDAP sync | Disabled AD user revoked in ≤15 min |
| UR-ENT-IAM-002 | SCIM 2.0 | Azure AD conformance test passed |
| UR-ENT-IAM-003 | Access review | Unreviewed access revoked after deadline |
| UR-ENT-RBAC-001 | Custom roles | Role limits enforced for all operations |
| UR-ENT-RBAC-002 | Time-based access | After-hours request returns 403 |
| UR-ENT-RBAC-003 | IP allowlist | Non-approved IP returns 403 + audit event |
| UR-ENT-RBAC-004 | Dual approval | Access only after approver confirms |
| UR-ENT-RBAC-005 | Break-glass | Witnesses notified within 60 seconds |
| UR-ENT-HA-001 | HA cluster | Kill one instance: no user-visible errors |
| UR-ENT-HA-002 | Zero-downtime upgrade | No 5xx errors during rolling update |
| UR-ENT-HA-003 | Backup verification | Nightly restore test passes; alerts on failure |
| UR-ENT-DR-001 | DR documentation | DR runbook generated with current topology |
| UR-ENT-DR-002 | PITR | Point-in-time restore completes successfully |
| UR-ENT-PAM-001 | Credential checkout | Access expires; audit logged with justification |
| UR-ENT-PAM-002 | Auto-rotation | Password changed on target system after checkout |
| UR-ENT-PAM-003 | ITSM validation | Closed ticket blocks checkout |
| UR-ENT-API-001 | Scoped API keys | Key scope enforced; wrong scope returns 403 |
| UR-ENT-API-002 | Secrets env export | Valid KEY=value format returned |
| UR-ENT-API-003 | Webhooks | HMAC-signed event delivered within 10 seconds |
| UR-ENT-MDM-001 | MDM compliance | Non-compliant device blocked |
| UR-ENT-MDM-002 | Certificate auth | Revoked cert blocked via OCSP |
| UR-ENT-MDM-003 | Remote wipe | Device token revoked immediately |
| UR-ENT-OBS-001 | Prometheus metrics | Valid Prometheus scrape; counters increment |
| UR-ENT-OBS-002 | JSON logging | Valid JSON per line; sensitive fields masked |
| UR-ENT-OBS-003 | K8s health checks | /health/ready returns 503 on DB failure |
| UR-ENT-OBS-004 | Security alerts | Alert sent on brute-force threshold |
| UR-ENT-TENANT-001 | Tenant isolation | Tenant A admin cannot see Tenant B data |
| UR-ENT-TENANT-002 | Tenant admin | Manages own users; cannot change global config |
| UR-ENT-TENANT-003 | Tenant quotas | User creation blocked at quota limit |
| UR-ENT-TENANT-004 | Migration | v1.x data migrated to DEFAULT tenant without loss |

---

## 9. Glossary

*(All terms from URD v1.0 §8 are retained)*

| Term | Definition |
|------|-----------|
| **ABAC** | Attribute-Based Access Control — access decisions based on user/resource/environment attributes |
| **Break-Glass** | Emergency bypass procedure allowing privileged access with mandatory accountability |
| **BCP** | Business Continuity Plan |
| **CISO** | Chief Information Security Officer |
| **CRL** | Certificate Revocation List — list of revoked digital certificates |
| **DR** | Disaster Recovery |
| **Dual Approval** | Four-eyes principle — two people must approve a sensitive action |
| **FSI** | Financial Services Industry (banks, insurance, capital markets) |
| **GDPR** | General Data Protection Regulation (EU) |
| **IAM** | Identity and Access Management |
| **ITSM** | IT Service Management (ServiceNow, Jira Service Management) |
| **JIT** | Just-In-Time provisioning — create account at first login |
| **KMS** | Key Management Service (AWS KMS, Azure Key Vault, HashiCorp Vault) |
| **LDAP** | Lightweight Directory Access Protocol — directory service protocol |
| **MAS TRM** | Monetary Authority of Singapore Technology Risk Management guidelines |
| **MDM** | Mobile Device Management (Intune, Jamf Pro) |
| **mTLS** | Mutual TLS — both client and server authenticate via certificates |
| **OCSP** | Online Certificate Status Protocol — real-time certificate revocation check |
| **PAM** | Privileged Access Management — controls access to privileged accounts |
| **PBKDF2** | Password-Based Key Derivation Function 2 |
| **PCI DSS** | Payment Card Industry Data Security Standard |
| **PITR** | Point-In-Time Recovery |
| **QSA** | Qualified Security Assessor (PCI DSS auditor) |
| **RBAC** | Role-Based Access Control |
| **RPO** | Recovery Point Objective — maximum acceptable data loss |
| **RTO** | Recovery Time Objective — maximum acceptable downtime |
| **SCIM** | System for Cross-domain Identity Management (RFC 7644) |
| **SoD** | Separation of Duties |
| **SOC 2** | Service Organization Control 2 (trust services criteria audit) |
| **SIEM** | Security Information and Event Management (Splunk, Sentinel) |
| **SLA** | Service Level Agreement |
| **WAL** | Write-Ahead Log (PostgreSQL continuous archiving mechanism) |
| **Zero-Trust** | Security model: never trust, always verify — every access requires authentication + authorization |

*(For original Vaultwarden terms, see URD v1.0 §8)*

---

*End of Document*
