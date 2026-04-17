# CR-009: MDM Integration & Certificate-Based Device Authentication

> **Change Request ID**: CR-009  
> **Title**: Mobile Device Management Integration & Certificate-Based Device Authentication  
> **Priority**: P2 — High  
> **Target Release**: v2.1  
> **Driven By**: [specs/crs/product-market-analysis.md §2.5 MDM]  
> **Affects**: PRD §6.2 (F-AUTH), URD §4.2, SRS §4.1

---

## 1. Problem Statement

- Không có MDM policy enforcement — mọi thiết bị đều có thể đăng nhập
- Không có certificate-based device authentication
- Không có remote wipe cho device-specific vault access
- Không có integration với Microsoft Intune, Jamf Pro
- Banking yêu cầu: chỉ corporate-managed devices được phép truy cập vault

---

## 2. Scope of Change

### 2.1 Device Trust Framework

```rust
DeviceTrustPolicy {
    org_uuid: OrganizationId,
    
    // Trust requirements
    require_managed_device: bool,        // Must be MDM-enrolled
    require_device_certificate: bool,    // Must present client certificate
    require_device_health: bool,         // MDM compliance check
    
    // MDM Integration
    mdm_provider: MdmProvider,
    mdm_config: MdmConfig,
    
    // Untrusted device behavior
    untrusted_action: Block | ReadOnlyAccess | MfaRequired,
}

enum MdmProvider {
    MicrosoftIntune,
    JamfPro,
    VMwareWorkspaceOne,
    MobileIron,
    Custom(WebhookUrl),
}
```

### 2.2 Certificate-Based Device Authentication

**Mutual TLS (mTLS) for device authentication**:

```
NEW CONFIG:
DEVICE_CERT_AUTH_ENABLED=false
DEVICE_CERT_CA_PATH=data/device-ca.pem    # CA that signed device certs
DEVICE_CERT_REQUIRED_ORG_UUID=<org-uuid>  # Only require for this org
DEVICE_CERT_CN_CLAIM=serial_number        # Field to extract device ID
```

**Flow**:
1. Organization's MDM issues client certificate to device (signed by corporate CA)
2. Vaultwarden configured with corporate CA certificate
3. At login, device presents client certificate in TLS handshake
4. Vaultwarden validates: cert signed by configured CA, cert not revoked (CRL/OCSP), device ID extracted from CN/SAN
5. Device ID matched against MDM enrollment database
6. If device not enrolled/compliant → login blocked

**CRL/OCSP Support**:
```
NEW CONFIG:
DEVICE_CERT_CRL_URL=https://pki.example.com/crl.pem
DEVICE_CERT_OCSP_URL=https://ocsp.example.com
DEVICE_CERT_OCSP_STAPLING=true
DEVICE_CERT_REVOCATION_CACHE_SECONDS=300
```

### 2.3 Microsoft Intune Integration

```
INTUNE_ENABLED=false
INTUNE_TENANT_ID=<azure-tenant-id>
INTUNE_CLIENT_ID=<app-registration-id>
INTUNE_CLIENT_SECRET=<secret>

# Compliance check: device must pass these Intune compliance policies
INTUNE_REQUIRED_COMPLIANCE_POLICIES=baseline-policy,encryption-policy
INTUNE_COMPLIANCE_CACHE_SECONDS=300     # Cache compliance check results
```

**Integration flow**:
1. User logs in with username/password/2FA
2. Device UUID sent with login request (existing `device_identifier` field)
3. Vaultwarden queries Intune: is this device enrolled? is it compliant?
4. If not compliant → login denied with message "Your device does not meet compliance requirements"
5. Compliance status cached for `INTUNE_COMPLIANCE_CACHE_SECONDS`

**Intune Graph API queries**:
- `GET /deviceManagement/managedDevices?$filter=azureADDeviceId eq '{device_id}'`
- Check: `complianceState == compliant`, `managementState == managed`

### 2.4 Jamf Pro Integration

```
JAMF_ENABLED=false
JAMF_URL=https://company.jamfcloud.com
JAMF_USERNAME=vaultwarden-integration
JAMF_PASSWORD=<secret>
JAMF_REQUIRED_GROUPS=["Vaultwarden Users","Engineering"]
JAMF_COMPLIANCE_CHECK=true
```

### 2.5 Remote Device Wipe (Vault Access Revocation)

"Remote wipe" trong context Vaultwarden = revoking device's vault access, không phải wipe thiết bị vật lý (đó là MDM's job):

```
DELETE /api/devices/{device_uuid}           # Revoke device (existing)
POST   /api/devices/{device_uuid}/wipe      # NEW: Revoke + clear vault cache hint

POST   /api/users/{user_uuid}/wipe-all-devices   # Revoke all devices for user
```

**Wipe notification**:
- Push notification sent to device: "Your vault access has been remotely revoked"
- Next sync attempt returns 401 → client shows "Access revoked by administrator"

### 2.6 Device Inventory & Compliance Dashboard

```
GET /api/admin/devices
{
  "devices": [
    {
      "uuid": "...",
      "user_email": "user@example.com",
      "device_name": "iPhone 15 Pro",
      "device_type": "iOS",
      "last_seen": "2026-04-12T09:00:00Z",
      "ip_address": "10.0.0.5",
      "is_trusted": true,
      "mdm_enrolled": true,
      "mdm_compliant": true,
      "cert_present": true,
      "cert_expires": "2027-04-12"
    }
  ],
  "summary": {
    "total": 1247,
    "trusted": 1198,
    "non_compliant": 49,
    "cert_expiring_soon": 12
  }
}
```

---

## 3. Acceptance Criteria

- [ ] Device with expired/revoked client certificate blocked from login
- [ ] Intune non-compliant device blocked from login; compliant device allowed
- [ ] `POST /api/devices/{id}/wipe` revokes device token and triggers push notification
- [ ] Device inventory API shows MDM compliance status for all registered devices
- [ ] CRL check blocks login when device certificate is revoked
- [ ] Admin dashboard shows devices with expiring certificates (< 30 days)

---

## 4. Estimated Effort

| Area | Effort |
|------|--------|
| Certificate-based device auth | 3 sprints |
| CRL/OCSP validation | 1 sprint |
| Intune integration | 2 sprints |
| Jamf Pro integration | 2 sprints |
| Device inventory dashboard | 1 sprint |
| Remote wipe API | 0.5 sprint |

---

*Status: Draft | Author: Product Team | Date: 2026-04-12*
