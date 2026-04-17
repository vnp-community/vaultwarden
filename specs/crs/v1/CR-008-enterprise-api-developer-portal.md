# CR-008: Enterprise API Management & Developer Portal

> **Change Request ID**: CR-008  
> **Title**: Enterprise API Management, Webhook Support & Developer Portal  
> **Priority**: P2 — High  
> **Target Release**: v2.0  
> **Driven By**: [specs/crs/product-market-analysis.md §2.5 API Management]  
> **Affects**: PRD §6.4 (F-ORG), URD §4.9, SRS §4.7

---

## 1. Problem Statement

- Org API Key hiện tại không có rate limiting per-key, không có usage analytics
- Không có webhook integration với ITSM hoặc SIEM
- Không có Terraform provider official
- DevOps team cần secret injection vào CI/CD pipelines — tích hợp hiện tại yêu cầu custom scripting
- Không có developer portal để quản lý API keys

---

## 2. Scope of Change

### 2.1 Enhanced API Key Management

```
APIKey {
    uuid: ApiKeyId,
    name: String,
    description: String,
    org_uuid: OrganizationId,
    created_by: UserId,
    
    // Access Control
    scopes: Vec<ApiScope>,               // Fine-grained permissions
    allowed_ips: Vec<IpNetwork>,         // IP whitelist
    allowed_collections: Vec<CollectionId>, // Collection-scoped access
    
    // Rate Limiting
    rate_limit_per_minute: u32,          // Default: 60
    rate_limit_per_hour: u32,            // Default: 1000
    
    // Lifecycle
    expires_at: Option<DateTime<Utc>>,
    last_used_at: Option<DateTime<Utc>>,
    is_active: bool,
    
    // Rotation
    rotate_reminder_days: Option<u32>,   // Email reminder before expiry
}

enum ApiScope {
    VaultRead,           // Read ciphers in assigned collections
    VaultWrite,          // Create/update ciphers
    VaultDelete,         // Delete ciphers
    OrgRead,             // Read org structure
    MembersRead,         // Read member list
    EventsRead,          // Read audit events
    SecretsRead,         // Read secrets (restricted subset of vault)
    SecretsWrite,        // Write secrets
}
```

**API**:
```
POST /api/organizations/{id}/api-keys
GET  /api/organizations/{id}/api-keys
PATCH /api/organizations/{id}/api-keys/{key-id}
POST /api/organizations/{id}/api-keys/{key-id}/rotate
DELETE /api/organizations/{id}/api-keys/{key-id}

GET /api/organizations/{id}/api-keys/{key-id}/usage   # Usage analytics
```

### 2.2 Webhook System

```
Webhook {
    uuid: WebhookId,
    org_uuid: OrganizationId,
    name: String,
    url: String,                         // HTTPS only
    secret: String,                      // HMAC-SHA256 signing secret
    events: Vec<WebhookEvent>,
    is_active: bool,
    retry_count: u8,                     // Default: 3
    timeout_seconds: u32,               // Default: 30
    last_delivery_at: Option<DateTime<Utc>>,
    last_delivery_status: Option<u16>,
}

enum WebhookEvent {
    // Vault events
    CipherCreated, CipherUpdated, CipherDeleted,
    // Member events  
    MemberInvited, MemberJoined, MemberRevoked, MemberRemoved,
    // Security events
    LoginFailed, LoginSucceeded, TwoFactorFailed,
    RateLimitTriggered, SuspiciousActivity,
    // Admin events
    ConfigChanged, BackupCompleted, BackupFailed,
    // Compliance events
    AccessReviewRequired, EmergencyAccessGranted,
    PrivilegedCheckout, PasswordRotationFailed,
}
```

**Webhook payload format**:
```json
{
  "id": "evt_01HX...",
  "type": "cipher.updated",
  "timestamp": "2026-04-12T10:00:00Z",
  "organization_id": "org-uuid",
  "actor": {
    "user_id": "user-uuid",
    "email": "user@example.com",
    "ip": "10.0.0.1"
  },
  "data": { ... event-specific data ... },
  "signature": "sha256=a1b2c3..."  // HMAC-SHA256 of payload
}
```

**API**:
```
POST /api/organizations/{id}/webhooks
GET  /api/organizations/{id}/webhooks
PATCH /api/organizations/{id}/webhooks/{wh-id}
POST /api/organizations/{id}/webhooks/{wh-id}/test
GET  /api/organizations/{id}/webhooks/{wh-id}/deliveries
DELETE /api/organizations/{id}/webhooks/{wh-id}
```

### 2.3 Secrets Management API (Bitwarden Secrets Manager Compatible)

Thin secrets API layer for CI/CD injection:

```
# Secrets (subset of vault items tagged as secrets)
GET  /api/secrets                              # List secrets in org
GET  /api/secrets/{id}                         # Get secret value
POST /api/secrets                              # Create secret
PUT  /api/secrets/{id}                         # Update secret
DELETE /api/secrets/{id}

# Secret injection for CI/CD
GET  /api/secrets/export?format=env|json|dotenv   # Export as env vars
GET  /api/secrets/{project}                    # Get all secrets for a project
```

**Example CI/CD integration** (GitHub Actions):
```yaml
- uses: vaultwarden/secrets-action@v1
  with:
    access-token: ${{ secrets.VW_TOKEN }}
    secrets: |
      DATABASE_URL: "Production DB/connection-string"
      API_KEY: "Payment Gateway/api-key"
```

### 2.4 Terraform Provider Support

Define official data sources and resources for Vaultwarden Terraform provider:

```hcl
# terraform-provider-vaultwarden

data "vaultwarden_secret" "db_password" {
  organization_id = var.org_id
  collection_name = "Production"
  name           = "Database/password"
}

resource "vaultwarden_collection" "engineering" {
  organization_id = var.org_id
  name           = "Engineering"
}

resource "vaultwarden_collection_access" "engineer_group" {
  collection_id = vaultwarden_collection.engineering.id
  group_id      = var.engineer_group_id
  read_only     = false
}
```

Terraform provider published to Terraform Registry.

### 2.5 API Usage Analytics

```
GET /api/admin/api-analytics?period=7d|30d|90d
{
  "api_keys": [
    {
      "key_id": "...",
      "name": "CI/CD Pipeline",
      "requests_total": 12453,
      "requests_by_day": [...],
      "top_endpoints": [...],
      "error_rate": 0.002,
      "last_used": "2026-04-12T09:45:00Z"
    }
  ]
}
```

---

## 3. Acceptance Criteria

- [ ] API key with `VaultRead` scope cannot write ciphers (returns 403)
- [ ] Rate limit per API key enforced independently from user rate limits
- [ ] Webhook delivery includes valid HMAC-SHA256 signature
- [ ] Webhook retry delivers event within 3 attempts on transient failure
- [ ] Secrets export as `.env` format returns correct key=value pairs
- [ ] Terraform provider reads secret value from Vaultwarden
- [ ] API usage dashboard shows per-key request counts

---

## 4. Estimated Effort

| Area | Effort |
|------|--------|
| Enhanced API key management | 2 sprints |
| Webhook system | 3 sprints |
| Secrets API | 2 sprints |
| Terraform provider (external) | 3 sprints |
| Usage analytics | 2 sprints |

---

*Status: Draft | Author: Product Team | Date: 2026-04-12*
