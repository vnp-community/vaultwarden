# TASKS-SOL-009: MDM Integration & Certificate-Based Device Auth

> **Giải pháp**: SOL-009  
> **CR**: CR-009  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 15

---

## Sprint 1–3 — Certificate Auth (Reverse Proxy Integration) (6 tuần)

### [x] TASK-009-001
- **Tên**: DB migration — device trust + MDM tables
- **File**: `migrations/postgresql/YYYYMMDD_mdm/up.sql`
- **Mô tả**: Thêm cột vào `devices`: `is_trusted`, `mdm_enrolled`, `mdm_compliant`, `mdm_last_check_at`, `cert_subject`, `cert_serial`, `cert_expires_at`, `cert_issuer`. Tạo: `device_trust_policies` (per-org policy), `mdm_compliance_cache` (cache compliance results).
- **Loại**: New migration
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-009-002
- **Tên**: Thêm DEVICE_CERT_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `device_cert_auth_enabled`, `device_cert_header` (default X-SSL-Client-Verify), `device_cert_dn_header`, `device_cert_serial_header`, `device_trust_enabled`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-009-003
- **Tên**: Implement `DeviceCertInfo::from_request()`
- **File**: `src/device_trust.rs`
- **Mô tả**: Struct `DeviceCertInfo { verified, subject_dn, serial, fingerprint, device_id }`. `FromRequest` impl reads `X-SSL-Client-Verify`, `X-SSL-Client-DN`, `X-SSL-Client-Serial` headers from reverse proxy. Extracts `device_id` from `CN=` in subject DN.
- **Loại**: New file (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-009-002

### [x] TASK-009-004
- **Tên**: Implement `DeviceTrustPolicy` model
- **File**: `src/device_trust.rs`
- **Mô tả**: Struct `DeviceTrustPolicy`. Method `find_by_org()`. Enum `TrustDecision { Allowed, Denied { reason }, ReadOnly }`.
- **Loại**: New code (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-009-001

### [x] TASK-009-005
- **Tên**: Implement `evaluate_device_trust()`
- **File**: `src/device_trust.rs`
- **Mô tả**: Logic: load org policy → check cert if `require_device_cert` → check MDM compliance if `require_managed_device`. Emits audit event on deny. Returns `TrustDecision`.
- **Loại**: New code (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-009-003, TASK-009-004

### [x] TASK-009-006
- **Tên**: Integrate device trust check vào login flow
- **File**: `src/api/identity.rs`
- **Mô tả**: After successful auth, before issuing JWT: extracts `DeviceCertInfo` from request via guard, fetches user's org, calls `evaluate_device_trust()`. `TrustDecision::Denied` → login error. Updates `Device.cert_subject/serial/cert_expires_at`.
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-009-005

### [x] TASK-009-007
- **Tên**: Documentation: nginx reverse proxy cert config
- **File**: `specs/crs/v1/solutions/SOL-009-mdm-cert-auth.md`
- **Mô tả**: Reverse proxy certificate configuration guide for nginx, Caddy, and Traefik — ssl_client_certificate, ssl_verify_client, required headers (X-SSL-Client-Verify, X-SSL-Client-DN, X-SSL-Client-Serial, X-SSL-Client-Fingerprint).
- **Loại**: Documentation (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-009-003

---

## Sprint 4–5 — Microsoft Intune Integration (4 tuần)

### [x] TASK-009-008
- **Tên**: Thêm INTUNE_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `intune_enabled`, `intune_tenant_id`, `intune_client_id`, `intune_client_secret` (masked), `intune_compliance_cache_seconds` (default 300).
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-009-009
- **Tên**: Implement `IntuneClient` với token caching
- **File**: `src/mdm/intune.rs`
- **Mô tả**: `IntuneClient` struct with OAuth2 client_credentials token caching and `check_device_compliance(azure_device_id)` querying Microsoft Graph API for `complianceState == "compliant" && managementState == "managed"`.
- **Loại**: New file (implemented)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-009-008

### [x] TASK-009-010
- **Tên**: Implement MDM compliance cache
- **File**: `src/device_trust.rs`
- **Mô tả**: `check_mdm_compliance()` checks `MdmComplianceCache` first. If stale (older than `INTUNE_COMPLIANCE_CACHE_SECONDS`), performs fresh check from MDM provider (Intune or Jamf). Upserts cache with result.
- **Loại**: New function (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-009-001, TASK-009-009

---

## Sprint 6–7 — Jamf Integration (4 tuần)

### [x] TASK-009-011
- **Tên**: Thêm JAMF_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `jamf_enabled`, `jamf_url`, `jamf_username`, `jamf_password` (masked), `jamf_compliance_cache_seconds`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-009-012
- **Tên**: Implement `JamfClient`
- **File**: `src/mdm/jamf.rs`
- **Mô tả**: Jamf Pro API client with Bearer token auth. `check_device_compliance(device_uuid)` queries Jamf Pro API and parses compliance state.
- **Loại**: New file (implemented)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-009-011

---

## Sprint 8 — Device Inventory + Wipe (2 tuần)

### [x] TASK-009-013
- **Tên**: Implement Device Inventory API
- **File**: `src/api/admin/devices.rs`
- **Mô tả**: `GET /api/admin/devices?page=&limit=&non_compliant_only=`: lists all devices with compliance status, including summary of total, trusted, non_compliant, cert_expiring_soon (within 30 days).
- **Loại**: New file (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-009-001

### [x] TASK-009-014
- **Tên**: Implement Remote Device Wipe
- **File**: `src/api/admin/devices.rs`
- **Mô tả**: `POST /api/devices/{uuid}/wipe` and `POST /api/users/{uuid}/wipe-all-devices` — deletes device record, sends push revoke notification, invalidates user security stamp (forces re-auth). Emits audit event.
- **Loại**: New routes (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-009-013

### [x] TASK-009-015
- **Tên**: Cert expiry alerting job
- **File**: `src/main.rs`
- **Mô tả**: Daily background job scans devices with `cert_expires_at` within 30 days, emits WARN logs and audit events for expiring certificates.
- **Loại**: New function + scheduler integration (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-009-001

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1–3 | TASK-009-001 → 007 | 1–6 | Certificate auth, device trust |
| Sprint 4–5 | TASK-009-008 → 010 | 7–10 | Intune integration |
| Sprint 6–7 | TASK-009-011 → 012 | 11–14 | Jamf integration |
| Sprint 8 | TASK-009-013 → 015 | 15–16 | Inventory, wipe, cert expiry |

---

*Tạo từ SOL-009 | Ngày: 2026-04-13*
