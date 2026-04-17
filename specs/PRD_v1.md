# Vaultwarden — Product Requirements Document v1 (Enterprise Edition)

> **Document Version**: 1.1  
> **Date**: 2026-04-12  
> **Status**: Draft  
> **Supersedes**: `specs/prd.md` (v1.0)  
> **Author**: Product Team  
> **Change Drivers**: [specs/crs/product-market-analysis.md], [specs/crs/v1/CR-000-index.md]  
> **References**:
> - User Requirements Document v1: `specs/URD_v1.md`
> - Software Requirements Specification: `specs/srs.md`
> - Technical Design Document: `specs/technical-design.md`
> - Change Requests: `specs/crs/v1/`

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Product Vision & Strategy](#2-product-vision--strategy)
3. [Target Users & Market](#3-target-users--market)
4. [Problem Statement](#4-problem-statement)
5. [Product Goals & Success Metrics](#5-product-goals--success-metrics)
6. [Feature Catalog](#6-feature-catalog)
   - 6.1 [Core Vault Management](#61-core-vault-management) *(unchanged)*
   - 6.2 [Authentication & Security](#62-authentication--security) *(enhanced)*
   - 6.3 [Multi-Factor Authentication](#63-multi-factor-authentication) *(unchanged)*
   - 6.4 [Organization & Team Collaboration](#64-organization--team-collaboration) *(enhanced)*
   - 6.5 [Bitwarden Send](#65-secure-sharing--bitwarden-send) *(unchanged)*
   - 6.6 [Emergency Access](#66-emergency-access) *(enhanced)*
   - 6.7 [Real-Time Sync & Notifications](#67-real-time-sync--notifications) *(enhanced — HA)*
   - 6.8 [Single Sign-On (SSO / OIDC)](#68-single-sign-on-sso--oidc) *(enhanced)*
   - 6.9 [Admin Panel & Server Management](#69-admin-panel--server-management) *(enhanced)*
   - 6.10 [Audit & Event Logging](#610-audit--event-logging-cr-002) 🆕 **CR-002**
   - 6.11 [Email Notifications](#611-email-notifications) *(unchanged)*
   - 6.12 [File & Attachment Storage](#612-file--attachment-storage) *(unchanged)*
   - 6.13 [Enterprise Compliance Framework](#613-enterprise-compliance-framework-cr-001) 🆕 **CR-001**
   - 6.14 [Identity & Access Management (IAM)](#614-identity--access-management-cr-003) 🆕 **CR-003**
   - 6.15 [Granular RBAC & Access Policies](#615-granular-rbac--access-policies-cr-004) 🆕 **CR-004**
   - 6.16 [High Availability & Clustering](#616-high-availability--clustering-cr-005) 🆕 **CR-005**
   - 6.17 [Disaster Recovery & BCP](#617-disaster-recovery--bcp-cr-006) 🆕 **CR-006**
   - 6.18 [Privileged Access Management (PAM)](#618-privileged-access-management-cr-007) 🆕 **CR-007**
   - 6.19 [Enterprise API & Developer Portal](#619-enterprise-api--developer-portal-cr-008) 🆕 **CR-008**
   - 6.20 [MDM & Device Trust](#620-mdm--device-trust-cr-009) 🆕 **CR-009**
   - 6.21 [Monitoring & Observability](#621-monitoring--observability-cr-010) 🆕 **CR-010**
   - 6.22 [Multi-Tenancy & Department Isolation](#622-multi-tenancy--department-isolation-cr-011) 🆕 **CR-011**
7. [Feature Prioritization (MoSCoW)](#7-feature-prioritization-moscow)
8. [User Flows](#8-user-flows)
9. [Non-Functional Product Requirements](#9-non-functional-product-requirements)
10. [Release Strategy & Milestones](#10-release-strategy--milestones)
11. [Risks & Mitigations](#11-risks--mitigations)
12. [Open Questions & Decisions](#12-open-questions--decisions)
13. [Appendix: Traceability Matrix](#13-appendix-traceability-matrix)

---

## 1. Executive Summary

**Vaultwarden Enterprise** is an open-source, self-hosted credential and secrets management platform — fully compatible with the Bitwarden client ecosystem — purpose-built for banking, financial services institutions (FSI), and large enterprises with >10,000 employees and >100M end customers.

Building on the proven Vaultwarden v1.x foundation (written in Rust, AGPL-3.0), v2.x adds a comprehensive enterprise layer addressing:

- **Regulatory Compliance**: PCI DSS v4.0, SOC 2 Type II, ISO 27001:2022, GDPR/PDPA, MAS TRM, Basel III, Circular 09/2020/TT-NHNN
- **Enterprise Identity**: LDAP/AD native integration, SCIM 2.0 automated provisioning, granular RBAC, dual approval workflows
- **Operational Excellence**: High availability clustering, 99.99% uptime SLA support, automated DR with verified backup
- **Privileged Access**: Credential checkout, automated password rotation, session accountability
- **Observability**: Prometheus metrics, OpenTelemetry tracing, SIEM integration
- **Scale**: Multi-tenancy for department isolation, horizontal scaling via Redis-backed clustering

---

## 2. Product Vision & Strategy

### 2.1 Vision Statement

> **Own your credentials. Comply with confidence. Scale without limits.**
>
> Vaultwarden Enterprise enables banks, financial institutions, and large enterprises to manage credentials and secrets with the security rigor required by regulators — on infrastructure they fully control, at a fraction of the cost of commercial PAM solutions.

### 2.2 Strategic Positioning (v2.x)

| Dimension | Vaultwarden v1.x | Vaultwarden v2.x | CyberArk / Thales | Official Bitwarden Enterprise |
|-----------|-----------------|-----------------|------------------|-----------------------------|
| **Hosting** | Self-hosted | Self-hosted | Cloud + On-prem | Cloud + Self-hosted |
| **Cost** | Free (AGPL) | Free (AGPL) | $$$$ | $$$ |
| **Target** | Homelab, SMB | Banking, FSI, Enterprise | Banking, FSI | SMB to Enterprise |
| **Compliance** | None certified | PCI DSS, SOC 2, ISO 27001 | SOC 2, ISO 27001, PCI DSS | SOC 2 |
| **PAM** | None | Checkout, Rotation | Full PAM | None |
| **HA** | Single-instance | Multi-instance cluster | Built-in | Limited |
| **Multi-tenancy** | None | Full isolation | Yes | Limited |
| **SCIM** | None | SCIM 2.0 | Yes | Yes |
| **Max Scale** | ~100 users | >100,000 users | Unlimited | Large enterprise |

### 2.3 Design Principles (v2.x additions)

1. **Backward Compatibility** — All v1.x features and APIs remain unchanged. Enterprise features are additive.
2. **Compliance by Design** — Every new feature includes audit logging, access control, and retention policy by default.
3. **Zero-Trust Architecture** — Every access decision is contextual (user + device + location + time + approval state).
4. **Operator Sovereignty** — No data ever leaves the operator's infrastructure. No phone-home, no telemetry.
5. **Scale Gracefully** — Single-instance for development; cluster mode for production. Same binary, different config.

---

## 3. Target Users & Market

### 3.1 Primary Personas (v2.x additions)

#### Persona 5 — Nguyen Van Thanh, CISO at a Vietnamese Commercial Bank
- **Organization**: 15,000 employees, 8M customers, 200 branches
- **Need**: Replace spreadsheet-based password sharing with auditable, compliant credential management
- **Key Requirements**: PCI DSS audit trail, AD integration, maker-checker workflow, SLA 99.99%
- **Key Features**: F-AUDIT, F-IAM, F-RBAC, F-HA, F-COMPLIANCE

#### Persona 6 — Sarah Chen, Head of DevSecOps at a Singapore Fintech (MAS-licensed)
- **Organization**: 800 employees, 2M customers, MAS payment institution license
- **Need**: Centralized secrets management for 120+ microservices + developer self-service
- **Key Requirements**: SCIM/Okta integration, CI/CD webhook, Terraform provider, MAS TRM compliance
- **Key Features**: F-IAM, F-API, F-AUDIT, F-HA, F-COMPLIANCE

#### Persona 7 — Trần Minh Khoa, IT Infrastructure Lead at a State-Owned Enterprise (50,000 employees)
- **Organization**: Multiple departments with different security classification levels
- **Need**: Department isolation — Treasury cannot see HR credentials
- **Key Requirements**: Multi-tenancy, LDAP sync, department-level admin delegation
- **Key Features**: F-TENANT, F-IAM, F-RBAC, F-HA

### 3.2 Market Opportunity (v2.x)

- **Addressable market**: Banking & FSI in ASEAN alone represents 500+ institutions needing compliant credential management
- **Cost disruption**: CyberArk Enterprise PAM costs $200K–$2M+ annually; Vaultwarden Enterprise = infrastructure cost only
- **Regulatory tailwind**: SBV Circular 09/2020 (Vietnam), MAS TRM (Singapore), BOT IT regulations (Thailand) create compliance urgency
- **Open-source moat**: AGPL license + community + Bitwarden client compatibility = deep switching cost

---

## 4. Problem Statement

### 4.1 Enterprise Problems Solved (v2.x additions)

| Problem | Current Pain | Vaultwarden v2.x Solution |
|---------|-------------|--------------------------|
| **Regulatory compliance** | No evidence for PCI DSS/SOC 2 auditors | Compliance Evidence API, tamper-evident audit log |
| **Single point of failure** | One server down = all passwords inaccessible | HA cluster with Redis-backed state |
| **Manual user provisioning** | IT manually creates/removes accounts | SCIM 2.0 + LDAP auto-sync |
| **No privileged access control** | Admins share privileged passwords informally | PAM checkout, dual approval, auto-rotation |
| **Invisible to monitoring** | Ops team has no metrics | Prometheus endpoint, JSON logs, SIEM |
| **Department data mixing** | All admins can see all data | Multi-tenancy with PostgreSQL RLS |
| **No maker-checker** | Single person can access any credential | Dual approval workflow |
| **Device trust gaps** | Any device can access the vault | MDM integration + certificate-based device auth |

---

## 5. Product Goals & Success Metrics

### 5.1 Product Goals (v2.x additions)

| Goal ID | Goal | Category |
|---------|------|---------|
| G-01 | 100% compatibility with all official Bitwarden clients | Compatibility |
| G-02 | Deployable in under 10 minutes via a single Docker command (single-instance) | Operator Experience |
| G-03 | Vault data is provably inaccessible to the server (E2EE) | Security |
| G-04 | **Support organizations of up to 100,000 users without performance degradation** | **Performance** |
| G-05 | Zero critical security vulnerabilities in core auth and encryption paths | Security |
| G-06 | All configurable via environment variables; no code changes required | Operability |
| **G-07** | **Pass PCI DSS Req 10 audit evidence generation** | **Compliance** |
| **G-08** | **Support 99.99% uptime SLA in cluster deployment** | **Availability** |
| **G-09** | **SCIM provisioning round-trip < 15 minutes for user lifecycle changes** | **IAM** |
| **G-10** | **All privileged credential accesses logged with requester, approver, justification** | **PAM** |

### 5.2 Key Success Metrics (v2.x additions)

| Metric | Target | Measurement |
|--------|--------|------------|
| **Availability (cluster mode)** | 99.99% (≤52 min/year) | External health check monitoring |
| **SCIM sync latency** | < 15 minutes | SCIM provisioning test |
| **Audit log write latency** | < 100ms p99 | APM tracing |
| **Backup verification success rate** | 100% or alert | Automated test |
| **RTO (HA cluster)** | < 15 minutes | DR drill |
| **RPO (WAL archiving)** | < 5 minutes | DR drill |
| **Compliance report generation** | < 60 seconds | Benchmark |
| **Prometheus scrape** | < 5ms per scrape | Prometheus metrics |
| **Cluster scale** | 100,000 users, 5M vault items | Load test |

---

## 6. Feature Catalog

*(Features 6.1–6.12 are unchanged from PRD v1.0. Key enhancements noted.)*

### 6.1 Core Vault Management
*(Unchanged from PRD v1.0)*  
**Feature ID**: F-VAULT | **Priority**: 🔴 Must Have

---

### 6.2 Authentication & Security *(Enhanced)*
**Feature ID**: F-AUTH | **Priority**: 🔴 Must Have  
**Changes from v1.0**:
- JWT in URL query parameter removed (security fix — see specs/security-analysis.md SEC-HIGH-01)
- Account lockout policy added (configurable failed attempts before lockout)
- Per-account rate limiting added alongside per-IP rate limiting
- KDF iterations server-side minimum enforcement added

---

### 6.3 Multi-Factor Authentication
*(Unchanged from PRD v1.0)*  
**Feature ID**: F-MFA | **Priority**: 🔴 Must Have

---

### 6.4 Organization & Team Collaboration *(Enhanced)*
**Feature ID**: F-ORG | **Priority**: 🔴 Must Have  
**Changes from v1.0**:
- Custom Role Builder added (see CR-004)
- Separation of Duties (SoD) rules added
- Org-level IP allowlist added
- Access review workflow added (see CR-003)

---

### 6.5 Secure Sharing — Bitwarden Send
*(Unchanged from PRD v1.0)*  
**Feature ID**: F-SEND | **Priority**: 🟠 Should Have

---

### 6.6 Emergency Access *(Enhanced)*
**Feature ID**: F-EMERGENCY | **Priority**: 🟠 Should Have  
**Changes from v1.0**:
- Multi-channel notification added (WebSocket in-app + email) — reduces single email delivery dependency
- Emergency access event triggers SIEM alert (see CR-002)
- Break-glass workflow formalized (see CR-004 §2.5)

---

### 6.7 Real-Time Sync & Notifications *(Enhanced)*
**Feature ID**: F-SYNC | **Priority**: 🟠 Should Have  
**Changes from v1.0**:
- JWT **no longer accepted** via URL query parameter — Authorization header only
- Redis pub/sub backend for WebSocket in cluster mode (see CR-005)

---

### 6.8 Single Sign-On (SSO / OIDC) *(Enhanced)*
**Feature ID**: F-SSO | **Priority**: 🟠 Should Have (🔴 Must for enterprise)  
**Changes from v1.0**:
- OIDC groups claim → collection mapping added (see CR-003 §2.3)
- SSO bypasses SIGNUPS_ALLOWED restriction removed — group claim required for provisioning
- Tenant-specific SSO config supported (see CR-011)

---

### 6.9 Admin Panel & Server Management *(Enhanced)*
**Feature ID**: F-ADMIN | **Priority**: 🔴 Must Have  
**Changes from v1.0**:
- Admin panel access now logged in system-wide audit log
- IP allowlist enforced for admin panel by default
- Config changes require re-authentication (protected action)

---

### 6.10 Audit & Event Logging — CR-002
**Feature ID**: F-AUDIT-V2 | **Priority**: 🔴 Must Have (was 🟠 Should Have)  
**Reference**: [CR-002](crs/v1/CR-002-system-wide-audit-log-siem.md)

#### What It Does
System-wide, tamper-evident audit log capturing all security-relevant events with hash chain integrity verification and SIEM forwarding.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Scope** | System-wide (not just org-level) |
| **Tamper evidence** | Hash chain: each entry includes SHA-256 of previous entry |
| **Append-only** | No DELETE permission on audit table |
| **SIEM export** | Splunk HEC, Syslog RFC 5424, JSON Lines, Microsoft Sentinel |
| **Retention** | Configurable; minimum enforced by admin (default: 7 years for banking) |
| **Chain verification** | `GET /api/audit/verify-chain` validates integrity |
| **Extended events** | Failed logins, admin actions, config changes, file downloads, security events |

#### Acceptance Criteria
- [ ] Failed admin login generates audit entry with IP, timestamp
- [ ] Deleting an audit entry breaks subsequent hash chain verification
- [ ] SIEM events delivered to Splunk HEC within 10 seconds
- [ ] Retention policy rejects configuration below minimum

---

### 6.11 Email Notifications
*(Unchanged from PRD v1.0)*  
**Feature ID**: F-EMAIL | **Priority**: 🔴 Must Have

---

### 6.12 File & Attachment Storage
*(Unchanged from PRD v1.0)*  
**Feature ID**: F-STORAGE | **Priority**: 🔴 Must Have

---

### 6.13 Enterprise Compliance Framework — CR-001
**Feature ID**: F-COMPLIANCE | **Priority**: 🔴 Must Have (enterprise)  
**Reference**: [CR-001](crs/v1/CR-001-enterprise-compliance-framework.md)

#### What It Does
Provides compliance evidence generation, data residency controls, GDPR right-to-erasure pipeline, and security header enforcement to enable deployment in regulated industries.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Compliance Evidence API** | Structured evidence export per standard (PCI DSS, SOC 2, ISO 27001) |
| **Data residency** | Restrict S3 storage to configured regions; enforce at upload time |
| **GDPR erasure** | 30-day automated PII deletion pipeline with audit receipt |
| **Data portability** | User can export all their data (GDPR Art 20) |
| **Security headers** | CSP, HSTS, X-Frame-Options, X-Content-Type-Options enforced at application layer |
| **PII register** | Automated data processing register for GDPR compliance |

#### Acceptance Criteria
- [ ] `GET /api/compliance/evidence?standard=pci_dss` returns structured JSON evidence
- [ ] GDPR erasure completes within 30 days; audit receipt generated
- [ ] All HTTP responses include HSTS and CSP headers
- [ ] Data residency enforcement blocks upload to non-compliant region

---

### 6.14 Identity & Access Management — CR-003
**Feature ID**: F-IAM | **Priority**: 🔴 Must Have (enterprise)  
**Reference**: [CR-003](crs/v1/CR-003-ad-ldap-scim-provisioning.md)

#### What It Does
Native LDAP/AD integration and SCIM 2.0 endpoint for automated user lifecycle management, eliminating manual provisioning for large organizations.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **LDAP sync** | Configurable sync interval (default: 15 min); auto-provision and auto-revoke |
| **SCIM 2.0** | Full RFC 7644 compliance; tested with Azure AD, Okta, OneLogin |
| **JIT provisioning** | OIDC groups claim → collection membership |
| **Access review** | Quarterly access review workflow with auto-revocation |
| **Deprovisioning** | User removed from LDAP → sessions revoked within 15 minutes |

#### Acceptance Criteria
- [ ] LDAP user disabled → Vaultwarden account revoked in ≤15 min
- [ ] Azure AD SCIM test: create/update/deactivate user flow passes
- [ ] Access review email sent; unreviewed access revoked after deadline

---

### 6.15 Granular RBAC & Access Policies — CR-004
**Feature ID**: F-RBAC | **Priority**: 🔴 Must Have (enterprise)  
**Reference**: [CR-004](crs/v1/CR-004-granular-rbac-access-control.md)

#### What It Does
Replaces the hardcoded 4-role model with a configurable permission system, adds time/location-based access restrictions, dual approval workflows, break-glass accounts, and SoD enforcement.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Custom roles** | Per-org custom permission sets |
| **Time-based access** | Credentials accessible only within configured hours/days per timezone |
| **IP allowlist** | Per-org or per-collection network restrictions |
| **Dual approval** | Maker-checker workflow for privileged access |
| **Break-glass** | Formalized emergency access with mandatory witness notification |
| **SoD rules** | Prevent same user holding conflicting roles |

#### Acceptance Criteria
- [ ] Request outside allowed hours returns 403 with audit event
- [ ] Dual approval flow grants access only after approver confirms
- [ ] Break-glass activation notifies all witnesses within 60 seconds
- [ ] SoD rule blocks conflicting role assignment (hard enforcement)

---

### 6.16 High Availability & Clustering — CR-005
**Feature ID**: F-HA | **Priority**: 🔴 Must Have (enterprise)  
**Reference**: [CR-005](crs/v1/CR-005-high-availability-clustering.md)

#### What It Does
Enables horizontal scaling via Redis-backed shared state, allowing multiple Vaultwarden instances behind a load balancer — eliminating the single point of failure.

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Cluster mode** | Enabled via `CLUSTER_MODE=true` + `REDIS_URL` |
| **Shared state** | Rate limiters, OIDC cache, WebSocket events via Redis |
| **WebSocket HA** | Redis pub/sub ensures events reach all connected clients regardless of instance |
| **Health check** | `GET /health/ready` + `GET /health/live` for Kubernetes |
| **Zero-downtime** | Graceful shutdown with configurable drain timeout |
| **DB read replica** | Optional read replica for vault sync queries |

#### Acceptance Criteria
- [ ] Kill one instance in 3-node cluster: no user-visible errors
- [ ] WebSocket event on Instance A delivered to user on Instance B
- [ ] Rolling update: zero downtime during instance restart

---

### 6.17 Disaster Recovery & BCP — CR-006
**Feature ID**: F-DR | **Priority**: 🔴 Must Have (enterprise)  
**Reference**: [CR-006](crs/v1/CR-006-disaster-recovery-bcp.md)

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **PostgreSQL WAL** | Continuous WAL archiving to S3; RPO < 5 minutes |
| **Backup verification** | Automated nightly restore test; alert on failure |
| **PITR** | Point-in-time recovery within retention window |
| **DR runbook** | Auto-generated PDF with current topology |
| **Multi-region backup** | Secondary S3 bucket in different region |
| **RTO/RPO SLA** | Documented: RTO < 15 min (HA), RPO < 5 min (WAL) |

---

### 6.18 Privileged Access Management — CR-007
**Feature ID**: F-PAM | **Priority**: 🔴 Must Have (enterprise)  
**Reference**: [CR-007](crs/v1/CR-007-privileged-access-management.md)

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Privileged ciphers** | Flag credentials as privileged; require checkout |
| **Credential checkout** | Time-limited access with mandatory justification |
| **Auto-rotation** | Automatic password change after checkout (SSH, DB) |
| **ITSM integration** | ServiceNow ticket validation before checkout |
| **Audit** | Every checkout logged: who, when, duration, justification |

---

### 6.19 Enterprise API & Developer Portal — CR-008
**Feature ID**: F-API | **Priority**: 🟠 Should Have  
**Reference**: [CR-008](crs/v1/CR-008-enterprise-api-developer-portal.md)

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Scoped API keys** | Fine-grained permissions per key (VaultRead, SecretsWrite, etc.) |
| **Rate limiting** | Per API key; configurable limits |
| **Webhooks** | HMAC-signed event delivery to ITSM/SIEM/Slack |
| **Secrets API** | CI/CD-friendly secrets endpoint; env var export format |
| **Terraform provider** | Official provider published to Terraform Registry |
| **Usage analytics** | Per-key request counts, error rates, last used |

---

### 6.20 MDM & Device Trust — CR-009
**Feature ID**: F-MDM | **Priority**: 🟠 Should Have  
**Reference**: [CR-009](crs/v1/CR-009-mdm-certificate-auth.md)

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Certificate auth** | mTLS client certificate validation |
| **Intune integration** | Compliance check before vault access |
| **Jamf Pro integration** | Group membership validation |
| **CRL/OCSP** | Certificate revocation checking |
| **Device inventory** | Admin dashboard: enrollment, compliance, cert expiry |
| **Remote wipe** | Revoke device vault access + push notification |

---

### 6.21 Monitoring & Observability — CR-010
**Feature ID**: F-OBS | **Priority**: 🔴 Must Have (enterprise)  
**Reference**: [CR-010](crs/v1/CR-010-observability-monitoring.md)

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Prometheus metrics** | `GET /metrics` — 30+ metrics across all subsystems |
| **Health checks** | `/health`, `/health/ready`, `/health/live` |
| **JSON logging** | Structured JSON with trace IDs; sensitive fields masked |
| **OpenTelemetry** | Distributed tracing to Jaeger/Zipkin/OTLP |
| **Security alerting** | Email/webhook on brute force, admin access, backup failure |
| **Grafana dashboard** | Official template shipped with product |

---

### 6.22 Multi-Tenancy & Department Isolation — CR-011
**Feature ID**: F-TENANT | **Priority**: 🟠 Should Have  
**Reference**: [CR-011](crs/v1/CR-011-multi-tenancy-department-isolation.md)

#### Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Tenant isolation** | PostgreSQL Row-Level Security; A cannot see B's data |
| **Tenant routing** | Subdomain, path, or email domain based |
| **Tenant admin** | Department-level admin with scoped visibility |
| **Quotas** | Per-tenant user/item/storage limits |
| **Config delegation** | Tenant can configure own SSO/LDAP within system constraints |
| **Migration** | Existing v1.x data migrated to DEFAULT tenant |

---

## 7. Feature Prioritization (MoSCoW)

### v2.0 Features

| Feature | ID | Priority | Justification |
|---------|----|---------:|--------------|
| Core Vault (CRUD, sync) | F-VAULT | 🔴 Must | Primary value |
| Authentication | F-AUTH | 🔴 Must | Gateway |
| MFA (all types) | F-MFA | 🔴 Must | Security baseline |
| Organizations | F-ORG | 🔴 Must | Team use cases |
| Admin Panel | F-ADMIN | 🔴 Must | Operator interface |
| Email Notifications | F-EMAIL | 🔴 Must | Critical flows |
| File Storage | F-STORAGE | 🔴 Must | Premium parity |
| **System Audit Log + SIEM** | **F-AUDIT-V2** | **🔴 Must** | **PCI DSS Req 10** |
| **Compliance Framework** | **F-COMPLIANCE** | **🔴 Must** | **Regulatory** |
| **LDAP + SCIM** | **F-IAM** | **🔴 Must** | **User lifecycle** |
| **Granular RBAC** | **F-RBAC** | **🔴 Must** | **Banking control** |
| **HA Clustering** | **F-HA** | **🔴 Must** | **SLA 99.99%** |
| **Disaster Recovery** | **F-DR** | **🔴 Must** | **Banking DR req** |
| **Prometheus + Observability** | **F-OBS** | **🔴 Must** | **Ops visibility** |
| SSO / OIDC | F-SSO | 🔴 Must | Enterprise IAM |
| Bitwarden Send | F-SEND | 🟠 Should | Common use case |
| Emergency Access | F-EMERGENCY | 🟠 Should | Account resilience |
| Real-Time Sync | F-SYNC | 🟠 Should | UX |
| **Enterprise API + Webhooks** | **F-API** | **🟠 Should** | **CI/CD + ITSM** |
| **MDM + Device Trust** | **F-MDM** | **🟠 Should** | **Device control** |

### v2.1 Features

| Feature | ID | Priority | Justification |
|---------|----|---------:|--------------|
| **PAM (Checkout + Rotation)** | **F-PAM** | **🔴 Must** | **Privileged access** |
| **Multi-Tenancy** | **F-TENANT** | **🔴 Must** | **Dept isolation** |
| S3 File Storage | F-STORAGE | 🟡 Could | Advanced operator |
| Duo Security | F-MFA | 🟡 Could | Enterprise specific |
| Directory Connector API | F-ORG | 🟡 Could | Advanced enterprise |

---

## 8. User Flows

*(8.1–8.4 from PRD v1.0 are unchanged)*

### 8.5 Enterprise User Provisioning via SCIM

```
HR System creates employee in Azure AD
    ↓
Azure AD SCIM sync → POST /scim/v2/Users
    ↓
Vaultwarden creates user + assigns to org based on group claim
    ↓
User receives welcome email + link to set master password
    ↓
User logs in with SSO (no separate VW password required)
    ↓
Access to collections determined by Azure AD group membership
```

### 8.6 Privileged Credential Access Flow

```
Engineer needs production database password
    ↓
Requests checkout: POST /api/ciphers/{id}/checkout
   {"justification": "INC-20260412 — prod DB performance", "ticket": "INC-20260412"}
    ↓
System validates ServiceNow ticket is open
    ↓
If requires_approval=true:
    → Approval request sent to Security team
    → Security team approves via web vault
    ↓
Credential returned (time-limited: 60 minutes)
    ↓
[60 minutes later] Checkout expires
    ↓
Password rotated automatically (if auto_rotate=true)
    ↓
Audit entry: requester, approver, credential, duration, justification
```

### 8.7 Banking Compliance Audit Flow

```
External PCI DSS QSA auditor requests evidence
    ↓
IT Admin → GET /api/compliance/evidence?standard=pci_dss&from=2025-04-01&to=2026-04-01
    ↓
System generates evidence package:
    - Access log extract (who accessed what, when)
    - User access review records
    - Configuration change log
    - Failed authentication report
    - MFA compliance report
    ↓
Export as PDF or JSON for auditor
```

---

## 9. Non-Functional Product Requirements

### 9.1 Security Requirements (v2.x additions)

| Requirement | Product Rationale |
|-------------|------------------|
| End-to-end encryption (AES-256-GCM/CBC) | Core trust promise |
| Argon2id for admin token | GPU-resistant |
| Rate limiting: per-IP + per-account | Defense against credential stuffing |
| `#![forbid(unsafe_code)]` | Memory safety |
| PKCE for SSO | Authorization code protection |
| HTTPS-only | No credentials in transit plaintext |
| **mTLS device certificates** | **Zero-trust device authentication** |
| **Tamper-evident audit log** | **PCI DSS / SOC 2 compliance** |
| **JWT only via Authorization header** | **Prevent token in server logs** |
| **KDF minimum iterations enforced** | **Prevent weak client-side KDF** |
| **CSP + HSTS + security headers** | **XSS protection at app layer** |

### 9.2 Performance Requirements (v2.x additions)

| Scenario | Target |
|---------|--------|
| Login (`/identity/connect/token`) | < 300ms p95 |
| Vault sync (500 items) | < 500ms p95 |
| Vault sync (10,000 items) | < 2s p95 |
| WebSocket event delivery | < 2 seconds |
| **SCIM user provisioning** | **< 15 minutes end-to-end** |
| **Prometheus scrape** | **< 5ms** |
| **Audit log write** | **< 100ms p99** |
| **Cluster scale** | **100,000 users, 5M vault items** |
| Server memory (100 users) | < 150 MB |
| **Server memory (cluster, 1000 concurrent)** | **< 2 GB per instance** |

### 9.3 Reliability Requirements (v2.x additions)

| Requirement | Target |
|-------------|--------|
| Database migration | Auto-applied on startup |
| **Availability (cluster mode)** | **99.99% (≤52 min downtime/year)** |
| **RTO** | **< 15 minutes (HA cluster)** |
| **RPO** | **< 5 minutes (WAL archiving)** |
| **Backup verification** | **Automated daily; alert on failure** |
| Background jobs | Isolated; automatic restart on failure |

### 9.4 Compliance Requirements (New)

| Standard | Requirement | Status |
|----------|-------------|--------|
| PCI DSS v4.0 Req 7 | Access control per least privilege | F-RBAC |
| PCI DSS v4.0 Req 8 | MFA enforcement | F-MFA (existing) |
| PCI DSS v4.0 Req 10 | Audit logging, tamper-evident | F-AUDIT-V2 |
| SOC 2 CC6 | Logical access management | F-IAM + F-RBAC |
| SOC 2 CC7 | System operations monitoring | F-OBS + F-AUDIT-V2 |
| ISO 27001 A.5.15 | Access control policy | F-RBAC |
| GDPR Art 17 | Right to erasure | F-COMPLIANCE |
| GDPR Art 32 | Security of processing | All security features |

---

## 10. Release Strategy & Milestones

### 10.1 v2.0 — Enterprise Foundation

**Target**: Banking and large enterprise production readiness  
**Duration**: ~12–18 months (from v1.x baseline)

| Feature | CR | Status |
|---------|----|-|
| System-wide audit log + SIEM | CR-002 | 📋 Planned |
| LDAP + SCIM 2.0 | CR-003 | 📋 Planned |
| Granular RBAC + Dual Approval | CR-004 | 📋 Planned |
| HA Clustering (Redis) | CR-005 | 📋 Planned |
| Disaster Recovery | CR-006 | 📋 Planned |
| Prometheus + JSON logs + OTel | CR-010 | 📋 Planned |
| Enterprise Compliance Framework | CR-001 | 📋 Planned |
| Enhanced API Keys + Webhooks | CR-008 | 📋 Planned |
| MDM + Device Trust | CR-009 | 📋 Planned |

### 10.2 v2.1 — Advanced Enterprise

**Target**: Full PAM + Multi-tenancy for the largest deployments  
**Duration**: ~6–9 months after v2.0

| Feature | CR | Status |
|---------|----|-|
| Privileged Access Management | CR-007 | 📋 Planned |
| Multi-Tenancy | CR-011 | 📋 Planned |
| Secrets API (Bitwarden SM compatible) | CR-008 §2.3 | 📋 Planned |
| Terraform Provider | CR-008 §2.4 | 📋 Planned |

---

## 11. Risks & Mitigations (v2.x additions)

| Risk ID | Risk | Likelihood | Impact | Mitigation |
|---------|------|:---------:|:------:|-----------|
| R-01 | Bitwarden client API changes break compatibility | Medium | High | Monitor API changelogs; integration test suite |
| R-02 | Security vulnerability in auth/crypto | Low | Critical | Established libraries; `forbid(unsafe_code)`; audits |
| R-03 | SQLite under concurrent writes | Medium | High | WAL mode; PostgreSQL for production |
| R-04 | SMTP misconfiguration | High | Medium | SMTP test endpoint; clear errors |
| R-05 | SSO IdP downtime | Medium | High | Password login coexistence |
| R-06 | S3 credentials exposure | Low | High | Log masking; least-privilege IAM |
| R-07 | Admin token brute-force | Low | Critical | Argon2id; rate limiting |
| R-08 | Dependency supply chain | Low | High | `cargo audit`; RUSTSEC |
| R-09 | No TLS (operator misconfiguration) | Medium | High | Documentation; header enforcement |
| **R-11** | **Redis cluster failure in HA mode** | **Medium** | **High** | **Graceful degradation to single-instance; Redis Sentinel/Cluster** |
| **R-12** | **LDAP bind account compromise** | **Low** | **High** | **Dedicated service account; LDAPS only; regular rotation** |
| **R-13** | **Audit log tamper by insider** | **Low** | **Critical** | **Append-only table; separate DB credentials; SIEM offload** |
| **R-14** | **SCIM token exposure** | **Low** | **High** | **Separate token; rotatable; masked in logs** |
| **R-15** | **Multi-tenancy data leak via query bug** | **Low** | **Critical** | **PostgreSQL RLS as defense-in-depth; test suite** |

---

## 12. Open Questions & Decisions

| # | Question | Owner | Status | Decision |
|---|---------|-------|--------|---------|
| OQ-01 | Secrets manager API scope? | Product | 🔴 Open | CR-008 §2.3 covers basic; full Bitwarden SM TBD |
| OQ-02 | Prometheus endpoint — auth method? | Engineering | ✅ Decided | Bearer token + IP allowlist |
| OQ-03 | Rate limit on admin panel? | Security | ✅ Decided | Yes — existing + account lockout |
| OQ-04 | WebSocket enabled by default? | Product | ✅ Decided | Disabled by default |
| OQ-05 | Email verification mandatory? | Product | ✅ Decided | Optional (`SIGNUPS_VERIFY`) |
| OQ-06 | SQLite max users? | Engineering | ✅ Decided | <100 SQLite; PostgreSQL for more |
| **OQ-07** | **Should PAM include SSH proxy/session recording?** | **Product** | **🔴 Open** | **v2.1 Phase 1: audit only; Phase 2: proxy TBD** |
| **OQ-08** | **Multi-tenancy: Schema-per-tenant vs RLS?** | **Engineering** | **🟡 Discussing** | **RLS preferred (PostgreSQL); schema-per-tenant as option** |
| **OQ-09** | **Open-source vs paid enterprise tier for CR features?** | **Product/Legal** | **🔴 Open** | **AGPL v2.x; enterprise features in same binary** |
| **OQ-10** | **FIPS 140-3 compliance for cryptographic modules?** | **Security** | **🔴 Open** | **US federal market requirement; evaluate ring/aws-lc** |

---

## 13. Appendix: Traceability Matrix

| PRD Feature | URD Reference | CR Reference | SRS Reference | TDD Section |
|-------------|--------------|-------------|--------------|------------|
| F-VAULT | UR-USER-003–008 | — | FR-CIPHER-001–010 | §6.2 |
| F-AUTH | UR-USER-001–002, 010, 013 | — | FR-AUTH-001–008 | §5 |
| F-MFA | UR-USER-012, UR-MFA-001–003 | — | FR-2FA-001–009 | §6.4 |
| F-ORG | UR-ORG-001–007 | CR-004 (RBAC) | FR-ORG-001–010 | §6.3 |
| F-SEND | UR-SEND-001–005 | — | FR-SEND-001–005 | §6.5 |
| F-EMERGENCY | UR-EMRG-001–003 | — | FR-EMRG-001–006 | §6.6 |
| F-SYNC | UR-SYNC-001–002 | CR-005 (Redis WS) | FR-NOTIF-001–007 | §9 |
| F-SSO | UR-ADMIN-011 | CR-003 (JIT) | FR-SSO-001–006 | §10 |
| F-ADMIN | UR-ADMIN-001–008 | — | FR-ADMIN-001–006 | §5.2 |
| **F-AUDIT-V2** | **UR-ENT-AUDIT-001–005** | **CR-002** | TBD | TBD |
| **F-COMPLIANCE** | **UR-ENT-COMP-001–003** | **CR-001** | TBD | TBD |
| **F-IAM** | **UR-ENT-IAM-001–005** | **CR-003** | TBD | TBD |
| **F-RBAC** | **UR-ENT-RBAC-001–006** | **CR-004** | TBD | TBD |
| **F-HA** | **UR-ENT-HA-001–003** | **CR-005** | TBD | TBD |
| **F-DR** | **UR-ENT-DR-001–004** | **CR-006** | TBD | TBD |
| **F-PAM** | **UR-ENT-PAM-001–005** | **CR-007** | TBD | TBD |
| **F-API** | **UR-ENT-API-001–004** | **CR-008** | TBD | TBD |
| **F-MDM** | **UR-ENT-MDM-001–003** | **CR-009** | TBD | TBD |
| **F-OBS** | **UR-ENT-OBS-001–004** | **CR-010** | TBD | TBD |
| **F-TENANT** | **UR-ENT-TENANT-001–004** | **CR-011** | TBD | TBD |
| F-EMAIL | UR-ADMIN-010 | — | FR-EMAIL-001–004 | §13 |
| F-STORAGE | UR-USER-007, UR-ADMIN-008 | — | FR-ATTACH-001–004 | §8 |

---

*End of Document*
