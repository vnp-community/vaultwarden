# SOL-009: Giải Pháp Thực Hiện — MDM Integration & Certificate-Based Device Auth

> **Giải pháp cho**: CR-009  
> **Ngày**: 2026-04-12  
> **Trạng thái**: Draft  
> **Kiến trúc thay đổi**: Trung bình — thêm TLS client cert validation, MDM API integrations

---

## 1. Tổng Quan Giải Pháp

Giải pháp tận dụng:
- **Device model** hiện có (`src/db/models/device.rs`) → mở rộng với trust status
- **Login flow** hiện có (`src/api/identity.rs`) → thêm device validation step
- **Reqwest HTTP client** → MDM API queries
- **Rocket TLS config** → client certificate support

**Phạm vi v2.1**: Certificate auth + Intune + Jamf. SSH proxy là future scope.

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/device_trust.rs` | Device trust evaluation engine |
| `src/mdm/mod.rs` | MDM provider abstraction |
| `src/mdm/intune.rs` | Microsoft Intune API client |
| `src/mdm/jamf.rs` | Jamf Pro API client |
| `src/api/admin/devices.rs` | Device inventory + compliance API |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/db/models/device.rs` | Thêm trust, MDM compliance fields |
| `src/api/identity.rs` | Thêm device trust check sau authentication |
| `src/config.rs` | Thêm DEVICE_CERT_*, INTUNE_*, JAMF_* config keys |
| `src/main.rs` | Thêm TLS client cert configuration |

### 2.3 Database Migrations

```sql
-- migrations/postgresql/YYYYMMDD_mdm/up.sql

-- Mở rộng bảng devices hiện có
ALTER TABLE devices ADD COLUMN is_trusted BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE devices ADD COLUMN mdm_enrolled BOOLEAN;
ALTER TABLE devices ADD COLUMN mdm_compliant BOOLEAN;
ALTER TABLE devices ADD COLUMN mdm_last_check_at TIMESTAMPTZ;
ALTER TABLE devices ADD COLUMN cert_subject TEXT;              -- Client cert CN
ALTER TABLE devices ADD COLUMN cert_serial TEXT;              -- Certificate serial number
ALTER TABLE devices ADD COLUMN cert_expires_at DATE;
ALTER TABLE devices ADD COLUMN cert_issuer TEXT;

-- Device Trust Policy per org
CREATE TABLE device_trust_policies (
    uuid                    VARCHAR(40) PRIMARY KEY,
    org_uuid                VARCHAR(40) NOT NULL UNIQUE REFERENCES organizations(uuid),
    require_managed_device  BOOLEAN NOT NULL DEFAULT FALSE,
    require_device_cert     BOOLEAN NOT NULL DEFAULT FALSE,
    require_device_health   BOOLEAN NOT NULL DEFAULT FALSE,
    mdm_provider            VARCHAR(30),                     -- 'intune', 'jamf', 'custom'
    untrusted_action        VARCHAR(20) NOT NULL DEFAULT 'block',  -- 'block', 'readonly', 'mfa'
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- MDM compliance check cache
CREATE TABLE mdm_compliance_cache (
    device_uuid         VARCHAR(40) PRIMARY KEY REFERENCES devices(uuid),
    compliance_status   VARCHAR(20) NOT NULL,              -- 'compliant', 'non_compliant', 'unknown'
    checked_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    provider            VARCHAR(30) NOT NULL,
    provider_device_id  VARCHAR(200),
    details             JSONB
);
```

---

## 3. Thiết Kế Chi Tiết

### 3.1 Certificate-Based Device Authentication

**Approach**: Vaultwarden chạy sau reverse proxy (nginx, Caddy). Reverse proxy thực hiện TLS termination và client cert validation, sau đó pass cert thông tin qua HTTP headers.

```nginx
# nginx config (reverse proxy)
server {
    ssl_client_certificate /etc/nginx/device-ca.pem;
    ssl_verify_client optional;  # Hoặc 'on' cho strict mode
    
    location / {
        proxy_pass http://vaultwarden:8080;
        
        # Pass cert info via headers
        proxy_set_header X-SSL-Client-Verify $ssl_client_verify;
        proxy_set_header X-SSL-Client-Cert $ssl_client_escaped_cert;
        proxy_set_header X-SSL-Client-DN $ssl_client_s_dn;
        proxy_set_header X-SSL-Client-Serial $ssl_client_serial;
        proxy_set_header X-SSL-Client-Fingerprint $ssl_client_fingerprint;
    }
}
```

**Vaultwarden side** (`src/device_trust.rs`):

```rust
pub struct DeviceCertInfo {
    pub verified: bool,
    pub subject_dn: Option<String>,
    pub serial: Option<String>,
    pub fingerprint: Option<String>,
    pub device_id: Option<String>,  // Extracted from CN or SAN
}

impl DeviceCertInfo {
    pub fn from_request(req: &Request<'_>) -> Self {
        let verified = req.headers().get_one("X-SSL-Client-Verify")
            .map(|v| v == "SUCCESS")
            .unwrap_or(false);
        
        let subject_dn = req.headers().get_one("X-SSL-Client-DN")
            .map(|s| s.to_string());
        
        let serial = req.headers().get_one("X-SSL-Client-Serial")
            .map(|s| s.to_string());
        
        // Extract device_id from CN=<device_id> in subject DN
        let device_id = subject_dn.as_ref().and_then(|dn| {
            dn.split(',')
                .find(|part| part.trim().starts_with("CN="))
                .map(|cn| cn.trim_start_matches("CN=").trim().to_string())
        });
        
        Self { verified, subject_dn, serial, fingerprint: None, device_id }
    }
}

pub async fn evaluate_device_trust(
    device: &Device,
    cert_info: &DeviceCertInfo,
    org_uuid: Option<&str>,
    conn: &DbConn,
) -> Result<TrustDecision, Error> {
    // Get policy for org
    let policy = if let Some(org) = org_uuid {
        DeviceTrustPolicy::find_by_org(org, conn).await?
    } else {
        None
    };
    
    let policy = match policy {
        Some(p) => p,
        None => return Ok(TrustDecision::Allowed),  // No policy = allow all
    };
    
    // 1. Certificate check
    if policy.require_device_cert {
        if !cert_info.verified {
            audit::emit(AuditEntry {
                event_type: AuditEventType::DeviceCertMissing,
                target_resource: Some(device.uuid.clone()),
                ..Default::default()
            });
            return Ok(TrustDecision::Denied { 
                reason: "Device certificate required but not presented".to_string() 
            });
        }
    }
    
    // 2. MDM compliance check
    if policy.require_managed_device || policy.require_device_health {
        let compliance = check_mdm_compliance(device, &policy, conn).await?;
        
        if !compliance.is_compliant {
            return Ok(TrustDecision::Denied {
                reason: format!("Device not MDM compliant: {:?}", compliance.reason)
            });
        }
    }
    
    Ok(TrustDecision::Allowed)
}
```

### 3.2 Login Flow Integration

**File**: `src/api/identity.rs` — sau khi xác thực user thành công, trước khi issue token:

```rust
// Sau auth thành công, trước khi tạo JWT
if let Some(device_uuid) = &login_request.device_identifier {
    let device = Device::find_by_uuid(device_uuid, &conn).await?;
    
    if let Some(ref device) = device {
        let cert_info = DeviceCertInfo::from_request(&req);
        
        // Lấy org của user (chỉ check policy nếu user thuộc org)
        let user_org = Membership::find_primary_org(&user.uuid, &conn).await?;
        let org_uuid = user_org.as_ref().map(|m| m.org_uuid.as_str());
        
        match evaluate_device_trust(device, &cert_info, org_uuid, &conn).await? {
            TrustDecision::Allowed => {
                // Update device trust status
                Device::update_trust_info(device_uuid, &cert_info, &conn).await.ok();
            }
            TrustDecision::Denied { reason } => {
                audit::emit(AuditEntry {
                    event_type: AuditEventType::LoginFailureDeviceUntrusted,
                    actor_user_uuid: Some(user.uuid.clone()),
                    metadata: json!({
                        "device_uuid": device_uuid,
                        "reason": reason,
                        "cert_presented": cert_info.verified,
                    }),
                    ..Default::default()
                });
                err!(format!("Device access denied: {reason}"));
            }
            TrustDecision::ReadOnly => {
                // Issue token với restricted scope
                // TODO: Implement read-only scope in JWT
            }
        }
    }
}
```

### 3.3 Microsoft Intune Integration

**File**: `src/mdm/intune.rs`

```rust
pub struct IntuneClient {
    client: reqwest::Client,
    access_token: Arc<RwLock<Option<(String, Instant)>>>,
}

impl IntuneClient {
    pub async fn check_device_compliance(
        &self, 
        azure_device_id: &str,
    ) -> Result<ComplianceStatus, Error> {
        let token = self.get_access_token().await?;
        
        // Query Microsoft Graph API
        let url = format!(
            "https://graph.microsoft.com/v1.0/deviceManagement/managedDevices\
             ?$filter=azureADDeviceId eq '{azure_device_id}'\
             &$select=complianceState,managementState,deviceName,operatingSystem"
        );
        
        let resp = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| Error::new(&format!("Intune API failed: {e}"), ""))?;
        
        if !resp.status().is_success() {
            return Err(Error::new("Intune API error", ""));
        }
        
        let body: Value = resp.json().await?;
        let devices = body.get("value").and_then(|v| v.as_array())
            .ok_or_else(|| Error::new("No device found in Intune", ""))?;
        
        if devices.is_empty() {
            return Ok(ComplianceStatus { 
                is_compliant: false, 
                reason: Some("Device not enrolled in Intune".to_string()),
                provider_device_id: None,
            });
        }
        
        let device = &devices[0];
        let compliance_state = device.get("complianceState")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");
        let management_state = device.get("managementState")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");
        
        let is_compliant = compliance_state == "compliant" && management_state == "managed";
        
        Ok(ComplianceStatus {
            is_compliant,
            reason: if !is_compliant {
                Some(format!("compliance={compliance_state}, management={management_state}"))
            } else { None },
            provider_device_id: device.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }
    
    async fn get_access_token(&self) -> Result<String, Error> {
        // Check cached token
        {
            let cache = self.access_token.read().await;
            if let Some((token, expires)) = cache.as_ref() {
                if Instant::now() < *expires - Duration::from_secs(60) {
                    return Ok(token.clone());
                }
            }
        }
        
        // Refresh via client_credentials flow
        let resp = self.client
            .post(format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                CONFIG.intune_tenant_id()
            ))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", CONFIG.intune_client_id()),
                ("client_secret", CONFIG.intune_client_secret()),
                ("scope", "https://graph.microsoft.com/.default"),
            ])
            .send()
            .await?;
        
        let body: Value = resp.json().await?;
        let token = body.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::new("Failed to get Intune access token", ""))?
            .to_string();
        
        let expires_in = body.get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(3600);
        
        let mut cache = self.access_token.write().await;
        *cache = Some((token.clone(), Instant::now() + Duration::from_secs(expires_in)));
        
        Ok(token)
    }
}

// Caching compliance results
async fn check_mdm_compliance(
    device: &Device,
    policy: &DeviceTrustPolicy,
    conn: &DbConn,
) -> Result<ComplianceStatus, Error> {
    // Check cache first
    if let Some(cached) = MdmComplianceCache::find_by_device(&device.uuid, conn).await? {
        let cache_age = Utc::now() - cached.checked_at;
        let cache_seconds = CONFIG.intune_compliance_cache_seconds() as i64;
        
        if cache_age.num_seconds() < cache_seconds {
            return Ok(ComplianceStatus {
                is_compliant: cached.compliance_status == "compliant",
                reason: None,
                provider_device_id: cached.provider_device_id,
            });
        }
    }
    
    // Fresh check from MDM
    let result = match policy.mdm_provider.as_deref() {
        Some("intune") => {
            let client = INTUNE_CLIENT.get().expect("Intune client not initialized");
            client.check_device_compliance(
                device.push_uuid.as_deref().unwrap_or("")
            ).await?
        }
        Some("jamf") => {
            let client = JAMF_CLIENT.get().expect("Jamf client not initialized");
            client.check_device_compliance(&device.uuid).await?
        }
        _ => return Err(Error::new("No MDM provider configured", "")),
    };
    
    // Update cache
    MdmComplianceCache::upsert(
        &device.uuid,
        &result,
        policy.mdm_provider.as_deref().unwrap_or(""),
        conn,
    ).await.ok();
    
    Ok(result)
}
```

### 3.4 Remote Device Wipe

```rust
// POST /api/devices/{uuid}/wipe
#[post("/devices/<device_uuid>/wipe")]
async fn wipe_device(
    device_uuid: &str,
    headers: AdminHeaders,
    conn: DbConn,
) -> EmptyResult {
    let device = Device::find_by_uuid(device_uuid, &conn).await?
        .ok_or_else(|| Error::new("Device not found", ""))?;
    
    // Revoke device
    Device::delete(&device.uuid, &conn).await?;
    
    // Send push notification nếu device có push token
    if let Some(push_uuid) = &device.push_uuid {
        push::send_vault_revoke_notification(push_uuid, &device.user_uuid).await.ok();
    }
    
    // Invalidate user's security stamp để force re-auth
    User::update_security_stamp(&device.user_uuid, &conn).await?;
    
    audit::emit(AuditEntry {
        event_type: AuditEventType::DeviceWiped,
        actor_user_uuid: Some(headers.user.uuid.clone()),
        target_resource: Some(device.uuid.clone()),
        metadata: json!({
            "device_name": device.name,
            "user_uuid": device.user_uuid,
        }),
        ..Default::default()
    });
    
    Ok(())
}
```

### 3.5 Device Inventory API

```rust
// GET /api/admin/devices
#[get("/admin/devices?<page>&<limit>&<non_compliant_only>")]
async fn device_inventory(
    page: Option<i64>,
    limit: Option<i64>,
    non_compliant_only: Option<bool>,
    _admin: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    let devices = Device::find_all_with_compliance(
        page.unwrap_or(1),
        limit.unwrap_or(50),
        non_compliant_only.unwrap_or(false),
        &conn,
    ).await?;
    
    let cert_expiring_soon = devices.iter()
        .filter(|d| d.cert_expires_at.map(|e| {
            let days = (e - Utc::now().date_naive()).num_days();
            days >= 0 && days <= 30
        }).unwrap_or(false))
        .count();
    
    Ok(Json(json!({
        "devices": devices,
        "summary": {
            "total": devices.len(),
            "trusted": devices.iter().filter(|d| d.is_trusted).count(),
            "non_compliant": devices.iter().filter(|d| d.mdm_compliant == Some(false)).count(),
            "cert_expiring_soon": cert_expiring_soon,
        }
    })))
}
```

---

## 4. Config Variables Mới

```bash
# Certificate Auth
DEVICE_CERT_AUTH_ENABLED=false
DEVICE_CERT_HEADER=X-SSL-Client-Verify   # Header from reverse proxy
DEVICE_CERT_DN_HEADER=X-SSL-Client-DN
DEVICE_CERT_SERIAL_HEADER=X-SSL-Client-Serial

# Intune
INTUNE_ENABLED=false
INTUNE_TENANT_ID=""
INTUNE_CLIENT_ID=""
INTUNE_CLIENT_SECRET=""                  # Masked
INTUNE_COMPLIANCE_CACHE_SECONDS=300

# Jamf
JAMF_ENABLED=false
JAMF_URL=""
JAMF_USERNAME=""
JAMF_PASSWORD=""                         # Masked
JAMF_COMPLIANCE_CACHE_SECONDS=300

# Device Trust (global)
DEVICE_TRUST_ENABLED=false
```

---

## 5. API Endpoints Mới

| Method | Path | Auth | Mô tả |
|--------|------|------|-------|
| GET | `/api/admin/devices` | Admin | Device inventory + compliance |
| POST | `/api/devices/{id}/wipe` | Admin | Remote wipe device |
| POST | `/api/users/{id}/wipe-all-devices` | Admin | Wipe all user devices |
| GET | `/api/organizations/{id}/device-policy` | Admin | Get trust policy |
| PUT | `/api/organizations/{id}/device-policy` | Admin | Update trust policy |

---

## 6. Kế Hoạch Triển Khai

### Sprint 1–3: Certificate Auth (Reverse Proxy Integration)
- DB migration cho device trust fields
- `DeviceCertInfo` extraction từ reverse proxy headers
- Login flow integration

### Sprint 4–5: Intune Integration
- `src/mdm/intune.rs`
- Graph API client với token caching
- Compliance cache

### Sprint 6–7: Jamf Integration
- `src/mdm/jamf.rs`
- Similar to Intune but Jamf API

### Sprint 8: Device Inventory + Wipe
- Admin device listing API
- Remote wipe với push notification
- Cert expiry alerting

---

*Status: Draft | Ngày: 2026-04-12*
