# Vaultwarden Enterprise — Solution Index

> **Ngày**: 2026-04-12  
> **Phạm vi**: Implementation solutions cho tất cả Change Requests v1  
> **Tham chiếu**: [CR Index](../CR-000-index.md) | [TDD](../../TDD.md)

---

## Danh Sách Solutions

| Solution | CR | Tiêu đề | Kiến trúc thay đổi | Sprint |
|----------|-----|---------|-------------------|--------|
| [SOL-001](SOL-001-enterprise-compliance-framework.md) | CR-001 | Enterprise Compliance Framework | Tối thiểu | 5 |
| [SOL-002](SOL-002-audit-log-siem.md) | CR-002 | Tamper-Evident Audit Log & SIEM | Trung bình | 6 |
| [SOL-003](SOL-003-ldap-scim.md) | CR-003 | AD/LDAP & SCIM 2.0 | Trung bình | 10 |
| [SOL-004](SOL-004-granular-rbac.md) | CR-004 | Granular RBAC + Time/Location | Trung bình | 11 |
| [SOL-005](SOL-005-high-availability.md) | CR-005 | High Availability & Redis | Đáng kể | 11 |
| [SOL-006](SOL-006-disaster-recovery.md) | CR-006 | Disaster Recovery & Backup | Tối thiểu | 9 |
| [SOL-007](SOL-007-pam.md) | CR-007 | Privileged Access Management | Trung bình | 11 |
| [SOL-008](SOL-008-enterprise-api.md) | CR-008 | Enterprise API & Webhooks | Tối thiểu | 9 |
| [SOL-009](SOL-009-mdm-cert-auth.md) | CR-009 | MDM & Certificate Auth | Trung bình | 9 |
| [SOL-010](SOL-010-observability.md) | CR-010 | Observability & Monitoring | Tối thiểu | 10 |
| [SOL-011](SOL-011-multi-tenancy.md) | CR-011 | Multi-Tenancy | **Đáng kể** | 11 |

---

## Tổng Hợp Modules Mới

| Module/File | CRs | Mô tả |
|-------------|-----|-------|
| `src/audit.rs` | CR-002 | System-wide audit emitter + hash chain |
| `src/siem.rs` | CR-002 | SIEM forwarder (Splunk, Syslog, Sentinel) |
| `src/ldap.rs` | CR-003 | LDAP connector + sync |
| `src/api/scim/` | CR-003 | SCIM 2.0 endpoints |
| `src/access_control.rs` | CR-004 | Time/IP/SoD access control engine |
| `src/cache.rs` | CR-005 | Cache abstraction (InMemory / Redis) |
| `src/api/health.rs` | CR-005, CR-010 | Health check endpoints |
| `src/backup.rs` | CR-006 | Backup orchestration |
| `src/pam/` | CR-007 | PAM: checkout, rotation, ITSM |
| `src/webhook_delivery.rs` | CR-008 | Async webhook delivery |
| `src/device_trust.rs` | CR-009 | Device trust evaluation |
| `src/mdm/` | CR-009 | Intune + Jamf clients |
| `src/metrics.rs` | CR-010 | Prometheus metrics registry |
| `src/alerting.rs` | CR-010 | Security alerting engine |
| `src/tenant.rs` | CR-011 | Multi-tenancy context + routing |
| `src/compliance/` | CR-001 | Compliance evidence collector |

---

## Tổng Hợp Phụ Thuộc Mới

| Crate | Phiên bản | Feature Flag | CRs |
|-------|-----------|--------------|-----|
| `sha2` | 0.10 | default | CR-002 |
| `ldap3` | 0.11 | default | CR-003 |
| `deadpool-redis` | 0.18 | `redis` (optional) | CR-005 |
| `redis` | 0.27 | `redis` (optional) | CR-005 |
| `hmac` | 0.12 | default | CR-008 |
| `hex` | 0.4 | default | CR-008 |
| `prometheus-client` | 0.22 | default | CR-010 |
| `tracing` | 0.1 | default | CR-010 |
| `tracing-subscriber` | 0.3 | default | CR-010 |
| `opentelemetry` | 0.27 | `otel` (optional) | CR-010 |
| `csv` | 1.x | default | CR-001 |

> **Lưu ý**: `reqwest`, `tokio`, `sha2` đã có trong dependencies hiện tại. Hầu hết CRs không cần dependencies mới.

---

## Tổng Hợp Database Tables Mới

| Bảng | CRs | Append-only? |
|------|-----|-------------|
| `audit_entries` | CR-002 | ✅ (DB policy) |
| `audit_entries_archive` | CR-002 | ✅ |
| `erasure_logs` | CR-001 | ✅ |
| `data_processing_register` | CR-001 | — |
| `backup_runs` | CR-006 | — |
| `ldap_sync_state` | CR-003 | — |
| `ldap_group_mappings` | CR-003 | — |
| `access_reviews` | CR-003 | — |
| `scim_tokens` | CR-003 | — |
| `custom_roles` | CR-004 | — |
| `access_schedules` | CR-004 | — |
| `ip_allowlists` | CR-004 | — |
| `approval_requests` | CR-004, CR-007 | — |
| `break_glass_configs` | CR-004 | — |
| `sod_rules` | CR-004 | — |
| `privileged_configs` | CR-007 | — |
| `checkouts` | CR-007 | — |
| `rotation_history` | CR-007 | — |
| `api_keys_v2` | CR-008 | — |
| `api_key_usage` | CR-008 | — |
| `webhooks` | CR-008 | — |
| `webhook_deliveries` | CR-008 | — |
| `device_trust_policies` | CR-009 | — |
| `mdm_compliance_cache` | CR-009 | — |
| `tenants` | CR-011 | — |
| `tenant_admins` | CR-011 | — |

---

## Tổng Hợp Config Variables (Count)

| CR | Số biến config mới |
|----|-------------------|
| CR-001 | 8 |
| CR-002 | 9 |
| CR-003 | 16 |
| CR-004 | 6 |
| CR-005 | 9 |
| CR-006 | 12 |
| CR-007 | 7 |
| CR-008 | 5 |
| CR-009 | 11 |
| CR-010 | 14 |
| CR-011 | 3 |
| **Tổng** | **~100** |

---

## Architecture Impact Summary

### Breaking Changes: KHÔNG CÓ
Tất cả thay đổi là additive. Existing single-instance deployments không bị ảnh hưởng khi upgrade lên v2.0 nếu không bật các feature flags mới.

### Feature Flags Pattern
```bash
# Tất cả features mới đều OFF by default:
AUDIT_LOG_ENABLED=false
LDAP_ENABLED=false
SCIM_ENABLED=false
REDIS_ENABLED=false
CLUSTER_MODE=false
BACKUP_ENABLED=false
PAM_ENABLED=false
METRICS_ENABLED=false
MULTI_TENANCY_ENABLED=false
DEVICE_TRUST_ENABLED=false
```

### Backward Compatibility
- SQLite vẫn hoạt động cho development/homelab
- Không cần Redis cho single-instance
- Multi-tenancy opt-in (existing data assigned DEFAULT tenant)
- Audit log opt-in (existing org events không bị ảnh hưởng)

---

*Status: Draft | Ngày: 2026-04-12*
