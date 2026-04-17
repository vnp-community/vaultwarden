# Vaultwarden Enterprise — Change Request Index v1

> **Version**: 1.0  
> **Date**: 2026-04-12  
> **Status**: Draft  
> **Objective**: Đưa Vaultwarden từ SMB/homelab tool thành enterprise-grade solution cho banking, FSI, và doanh nghiệp >10,000 nhân sự, >100M khách hàng  
> **Reference**: [product-market-analysis.md](../product-market-analysis.md)

---

## Change Request Summary

| CR | Title | Priority | Release | Issue Addressed |
|----|-------|----------|---------|----------------|
| [CR-001](CR-001-enterprise-compliance-framework.md) | Enterprise Compliance Framework (PCI DSS, SOC 2, ISO 27001, GDPR) | P1 | v2.0 | §2.1 Compliance |
| [CR-002](CR-002-system-wide-audit-log-siem.md) | System-Wide Tamper-Evident Audit Log & SIEM Integration | P1 | v2.0 | §2.1 Audit Log |
| [CR-003](CR-003-ad-ldap-scim-provisioning.md) | AD/LDAP Native Integration & SCIM 2.0 Provisioning | P1 | v2.0 | §2.2 IAM |
| [CR-004](CR-004-granular-rbac-access-control.md) | Granular RBAC, Time/Location-Based Access Control | P1 | v2.0 | §2.2 RBAC |
| [CR-005](CR-005-high-availability-clustering.md) | High Availability & Horizontal Scaling Architecture | P1 | v2.0 | §2.3 HA |
| [CR-006](CR-006-disaster-recovery-bcp.md) | Disaster Recovery & Business Continuity | P1 | v2.0 | §2.3 DR |
| [CR-007](CR-007-privileged-access-management.md) | Privileged Access Management (PAM) | P1 | v2.1 | §2.4 PAM |
| [CR-008](CR-008-enterprise-api-developer-portal.md) | Enterprise API Management & Developer Portal | P2 | v2.0 | §2.5 API |
| [CR-009](CR-009-mdm-certificate-auth.md) | MDM Integration & Certificate-Based Device Auth | P2 | v2.1 | §2.5 MDM |
| [CR-010](CR-010-observability-monitoring.md) | Enterprise Monitoring, Observability & Alerting | P2 | v2.0 | §2.6 Monitoring |
| [CR-011](CR-011-multi-tenancy-department-isolation.md) | Multi-Tenancy & Department Isolation | P2 | v2.1 | §2.6 Multi-Tenancy |

---

## Release Plan

### v2.0 — Enterprise Foundation (P1 + Core P2)

**Go/No-Go criteria**: All P1 CRs must be complete before v2.0 release.

| CR | Feature | Sprint Estimate |
|----|---------|----------------|
| CR-002 | System-wide audit log + SIEM | 10 sprints |
| CR-003 | LDAP + SCIM 2.0 | 11 sprints |
| CR-004 | Granular RBAC + Dual Approval | 13 sprints |
| CR-005 | HA Clustering + Redis | 11 sprints |
| CR-006 | DR + Backup Verification | 11 sprints |
| CR-008 | API Keys + Webhooks | 9 sprints |
| CR-010 | Prometheus + Observability | 10 sprints |
| CR-001 | Compliance Framework | 10.5 sprints |

**Total estimate**: ~12–18 months (parallel teams)

### v2.1 — Advanced Enterprise (Remaining P1 + P2)

| CR | Feature | Sprint Estimate |
|----|---------|----------------|
| CR-007 | PAM + Checkout + Rotation | 12 sprints |
| CR-009 | MDM + Cert Auth | 9 sprints |
| CR-011 | Multi-Tenancy | 12 sprints |

**Total estimate**: ~6–9 months after v2.0

---

## Architecture Impact Summary

| Layer | Change |
|-------|--------|
| **Database** | New tables: Tenants, AuditEntries (append-only), ApiKeys, Webhooks, Checkouts, DeviceTrustPolicy, AccessSchedule, ApprovalRequests | 
| **External Dependencies** | +Redis (HA mode), +LDAP server, +SCIM clients, +MDM APIs (Intune/Jamf) |
| **New Endpoints** | /metrics, /health/detailed, /scim/v2/*, /api/audit/*, /api/pam/*, /api/webhooks/*, /api/compliance/* |
| **Config Variables** | ~80 new environment variables across all CRs |
| **Breaking Changes** | None for v1 → v2.0 (additive changes only); Multi-tenancy migration required for v2.1 |

---

## Go/No-Go Matrix (Updated)

| Phân khúc | v1.x | v2.0 | v2.1 |
|-----------|------|------|------|
| Individual / Homelab | ✅ | ✅ | ✅ |
| SME không regulated | ✅ | ✅ | ✅ |
| Tech startup (<500) | ⚠️ | ✅ | ✅ |
| Fintech regulated (<1000) | ⚠️ | ✅ | ✅ |
| Large tech company (1000+) | ❌ | ✅ | ✅ |
| Banking / FSI (any size) | ❌ | ⚠️ (core compliance) | ✅ |
| Bank >100M customers | ❌ | ❌ | ✅ (with load testing) |

---

*Status: Draft | Author: Product Team | Date: 2026-04-12*
