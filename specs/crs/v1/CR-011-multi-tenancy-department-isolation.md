# CR-011: Multi-Tenancy & Department Isolation

> **Change Request ID**: CR-011  
> **Title**: Multi-Tenancy Architecture & Department-Level Administrative Isolation  
> **Priority**: P2 — High  
> **Target Release**: v2.1  
> **Driven By**: [specs/crs/product-market-analysis.md §2.6 Multi-Tenancy]  
> **Affects**: PRD §6.4 (F-ORG), URD §4.5, SRS §4.4

---

## 1. Problem Statement

- Một Vaultwarden instance phục vụ tất cả users mà không có tenant isolation ở data level
- Admin có thể xem thông tin tất cả users và organizations
- Không thể restrict một department admin chỉ thấy department của họ
- Treasury desk và Retail banking không thể share cùng một instance mà không có isolation
- Ngân hàng với nhiều chi nhánh/phòng ban cần hierarchical administration

---

## 2. Scope of Change

### 2.1 Tenant Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Vaultwarden Instance                │
│                                                     │
│  ┌────────────────────────────────────────────────┐ │
│  │              System Administrator              │ │
│  │           (cross-tenant visibility)            │ │
│  └────────────────────────────────────────────────┘ │
│                                                     │
│  ┌──────────────────┐  ┌──────────────────┐        │
│  │    Tenant A      │  │    Tenant B      │        │
│  │ (Treasury Dept)  │  │ (Retail Banking) │        │
│  │                  │  │                  │        │
│  │  Tenant Admin A  │  │  Tenant Admin B  │        │
│  │  Users A         │  │  Users B         │        │
│  │  Orgs A          │  │  Orgs B          │        │
│  │  Audit Logs A    │  │  Audit Logs B    │        │
│  └──────────────────┘  └──────────────────┘        │
│                                                     │
│  Data Isolation: A cannot see B's data              │
└─────────────────────────────────────────────────────┘
```

### 2.2 Tenant Data Model

```rust
Tenant {
    uuid: TenantId,
    name: String,                        // "Treasury & Markets"
    slug: String,                        // URL path segment: /t/treasury
    domain_restriction: Option<String>,  // Only @treasury.bank.com emails
    created_at: DateTime<Utc>,
    is_active: bool,
    
    // Quotas
    max_users: Option<u32>,
    max_organizations: Option<u32>,
    max_vault_items: Option<u32>,
    max_storage_bytes: Option<i64>,
    
    // Config overrides (tenant can override some global settings)
    config_overrides: TenantConfigOverrides,
    
    // Branding
    custom_logo_url: Option<String>,
    primary_color: Option<String>,
}

TenantAdmin {
    tenant_uuid: TenantId,
    user_uuid: UserId,
    can_manage_users: bool,
    can_manage_orgs: bool,
    can_view_audit_logs: bool,           // Only tenant's audit logs
    can_manage_config: bool,
}
```

### 2.3 Data Isolation Enforcement

**Database level** (preferred: Row-Level Security in PostgreSQL):

```sql
-- PostgreSQL RLS for tenant isolation
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON users
    USING (tenant_uuid = current_setting('app.current_tenant')::uuid);
```

**Application level** (fallback / MySQL compatibility):
- Every database query includes `WHERE tenant_uuid = ?` via Diesel query builder middleware
- Context-local `TenantContext` set at request start (from JWT or domain)
- Middleware validates that all returned records belong to current tenant

### 2.4 Tenant Routing

**Option A: Subdomain-based**:
- `treasury.vaultwarden.bank.com` → Tenant "Treasury"
- `retail.vaultwarden.bank.com` → Tenant "Retail"

**Option B: Path-based**:
- `/t/treasury/` → Tenant "Treasury"
- `/t/retail/` → Tenant "Retail"

**Option C: Email domain-based**:
- User with `@treasury.bank.com` → auto-assigned to Treasury tenant
- User with `@retail.bank.com` → auto-assigned to Retail tenant

```
NEW CONFIG:
MULTI_TENANCY_ENABLED=false
TENANT_ROUTING=subdomain|path|domain
TENANT_DEFAULT_UUID=<uuid>            # Fallback tenant for unmatched requests
```

### 2.5 Tenant Admin Capabilities

Tenant Admin can (within their tenant only):
- Manage all users belonging to their tenant
- Manage all organizations within their tenant
- View audit logs for their tenant
- Invite new users (subject to domain restriction)
- Configure SSO/LDAP for their tenant (within allowed parameters)
- Set tenant-level policies

Tenant Admin **cannot**:
- See users/orgs from other tenants
- Access system-level configuration
- Change global settings
- View system-wide audit logs

### 2.6 Tenant-Level Configuration Overrides

System admin sets what tenants can override:
```
TenantConfigOverrides {
    // What tenant admins can configure independently:
    allow_sso_config: bool,             // Own IdP
    allow_ldap_config: bool,            // Own AD
    allow_email_config: bool,           // Own SMTP
    allow_2fa_policy: bool,             // 2FA requirements
    allow_password_policy: bool,        // Password strength
    allow_ip_allowlist: bool,           // Network restrictions
    
    // Inherited from system (cannot override):
    // - backup config
    // - audit log retention
    // - security headers
}
```

### 2.7 Tenant Provisioning API (System Admin)

```
POST /api/system/tenants                # Create tenant
GET  /api/system/tenants                # List all tenants
PATCH /api/system/tenants/{id}          # Update tenant
GET  /api/system/tenants/{id}/stats     # Usage statistics
DELETE /api/system/tenants/{id}         # Deactivate tenant

POST /api/system/tenants/{id}/admins    # Assign tenant admin
DELETE /api/system/tenants/{id}/admins/{user_id}
```

### 2.8 Cross-Tenant Emergency Access (System Admin only)

System admin can access any tenant for emergency purposes:
- Requires break-glass procedure (CR-004 §2.5)
- All actions logged in system-level audit trail
- Tenant admin notified when system admin accesses their tenant

---

## 3. Acceptance Criteria

- [ ] Tenant A admin cannot list users from Tenant B
- [ ] Direct database query with Tenant A credentials returns only Tenant A records
- [ ] Subdomain routing correctly assigns requests to correct tenant
- [ ] Tenant quota enforcement blocks user creation above `max_users` limit
- [ ] Tenant admin can configure their own SSO but cannot change global settings
- [ ] System admin cross-tenant access triggers audit event and tenant admin notification
- [ ] PostgreSQL RLS test: direct DB connection with tenant A session cannot query tenant B rows

---

## 4. Migration Path for Existing Deployments

1. Create `DEFAULT` tenant
2. Assign all existing users, orgs, audit logs to DEFAULT tenant
3. Create additional tenants and migrate users by email domain or manual assignment
4. Enable `MULTI_TENANCY_ENABLED=true`
5. Test isolation before routing production traffic

---

## 5. Estimated Effort

| Area | Effort |
|------|--------|
| Tenant data model + migrations | 2 sprints |
| DB-level isolation (PostgreSQL RLS) | 2 sprints |
| Application-level isolation (Diesel middleware) | 2 sprints |
| Tenant routing (subdomain/path/domain) | 1 sprint |
| Tenant admin role | 2 sprints |
| Tenant provisioning API | 1 sprint |
| Multi-tenancy migration tool | 1 sprint |

---

*Status: Draft | Author: Product Team | Date: 2026-04-12*
