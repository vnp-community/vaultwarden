# CR-003: AD/LDAP Native Integration & SCIM 2.0 Provisioning

> **Change Request ID**: CR-003  
> **Title**: Active Directory / LDAP Native Integration & SCIM 2.0 Automated Provisioning  
> **Priority**: P1 — Critical  
> **Target Release**: v2.0  
> **Driven By**: [specs/crs/product-market-analysis.md §2.2, §3.3]  
> **Affects**: PRD §6.8, URD §4.5, SRS §4.12

---

## 1. Problem Statement

- Vaultwarden chỉ hỗ trợ OIDC/SSO — không tích hợp gốc với Microsoft Active Directory hoặc LDAP
- Bitwarden Directory Connector chỉ là workaround bên thứ ba, không được maintain chính thức bởi Vaultwarden
- Không có SCIM 2.0 — không thể tích hợp với Azure AD, Okta, OneLogin để tự động provision/deprovision
- Ngân hàng 10,000–100,000 nhân viên **không thể** quản lý user thủ công

---

## 2. Scope of Change

### 2.1 LDAP Native Connector

```
NEW CONFIG:
LDAP_ENABLED=true
LDAP_HOST=ldap.example.com
LDAP_PORT=636
LDAP_USE_TLS=true
LDAP_BIND_DN=cn=vaultwarden-service,ou=service-accounts,dc=example,dc=com
LDAP_BIND_PASSWORD=<secret>
LDAP_BASE_DN=ou=users,dc=example,dc=com
LDAP_USER_FILTER=(objectClass=person)
LDAP_USER_ATTR_EMAIL=mail
LDAP_USER_ATTR_NAME=displayName
LDAP_USER_ATTR_UUID=objectGUID
LDAP_GROUP_BASE_DN=ou=groups,dc=example,dc=com
LDAP_GROUP_FILTER=(objectClass=group)
LDAP_GROUP_ATTR_MEMBER=member
LDAP_SYNC_INTERVAL_MINUTES=15
LDAP_SYNC_ORG_UUID=<org-uuid>          # Org to sync into
LDAP_GROUPS_TO_COLLECTIONS=true        # Map LDAP groups → VW Collections
```

**Sync behavior**:
- User tồn tại trong LDAP nhưng chưa có trong Vaultwarden → auto-provision
- User bị disabled/removed trong LDAP → tài khoản Vaultwarden bị revoke tự động trong vòng 1 sync cycle
- LDAP group changes → collection membership updated

### 2.2 SCIM 2.0 Endpoint

SCIM 2.0 (System for Cross-domain Identity Management) — tiêu chuẩn công nghiệp để Azure AD, Okta, Google Workspace, OneLogin tự động quản lý users/groups.

**Endpoints implemented**:

```
# Users
GET    /scim/v2/Users                    # List users (with filter)
GET    /scim/v2/Users/{id}               # Get user by ID
POST   /scim/v2/Users                    # Create user
PUT    /scim/v2/Users/{id}               # Replace user
PATCH  /scim/v2/Users/{id}              # Partial update (activate/deactivate)
DELETE /scim/v2/Users/{id}              # Delete/deactivate user

# Groups  
GET    /scim/v2/Groups                   # List groups (orgs/collections)
GET    /scim/v2/Groups/{id}              # Get group
POST   /scim/v2/Groups                   # Create group
PATCH  /scim/v2/Groups/{id}             # Add/remove members
DELETE /scim/v2/Groups/{id}             # Delete group

# Service Provider Config
GET    /scim/v2/ServiceProviderConfig    # SCIM capabilities
GET    /scim/v2/Schemas                  # Schema definitions
GET    /scim/v2/ResourceTypes            # Resource type definitions
```

**Authentication**: SCIM endpoint requires Bearer token (`SCIM_TOKEN`) — separate from user JWT, long-lived admin credential.

**SCIM → Vaultwarden Mapping**:

| SCIM Attribute | Vaultwarden Field | Behavior |
|----------------|------------------|---------|
| `userName` | `email` | Unique identifier |
| `displayName` | `name` | Display name |
| `active` | User enabled/revoked | false → revoke all sessions immediately |
| `groups` | Org membership + collection | Sync on every change |
| `externalId` | `external_id` | IdP's internal ID for reference |

### 2.3 Just-In-Time (JIT) Provisioning Enhancement

Mở rộng SSO JIT provisioning hiện tại:
- Map OIDC claims → Vaultwarden Org + Collection membership
- Map OIDC groups claim → Collection access levels
- Support attribute-based access control (ABAC) via OIDC claims

```
NEW CONFIG:
SSO_JIT_PROVISION_ENABLED=true
SSO_JIT_ORG_UUID=<org-uuid>
SSO_JIT_GROUP_CLAIM=groups              # OIDC claim containing groups
SSO_JIT_GROUP_COLLECTION_MAP='{"IT Admins":"col-uuid-1","Finance":"col-uuid-2"}'
SSO_JIT_DEFAULT_ROLE=user
```

### 2.4 User Lifecycle Management

**Automated deprovisioning**:
- User leaves LDAP/SCIM → sessions revoked within 1 sync cycle (max 15 minutes)
- Vault data không bị xóa — chuyển vào "suspended" state cho 90 ngày (configurable)
- Admin có thể trigger manual deprovision ngay lập tức

**Access Review Workflow**:
```
Periodic Access Review (configurable interval, default: quarterly)
    ↓
System generates list: users + their collection access
    ↓
Email sent to Org Owners: "Access review required"
    ↓
Owner approves/revokes each access within 14 days
    ↓
Unreviewed access automatically revoked after deadline
    ↓
Review audit entry logged
```

---

## 3. Acceptance Criteria

- [ ] LDAP sync provisions a new user within one sync cycle after LDAP user creation
- [ ] LDAP user disabled → Vaultwarden account revoked within 15 minutes
- [ ] SCIM `PATCH /Users/{id}` with `active: false` revokes all user sessions immediately
- [ ] Azure AD SCIM integration test: create/update/delete user flow passes
- [ ] Okta SCIM integration test: group-to-collection mapping works
- [ ] SCIM authentication rejects requests without valid Bearer token
- [ ] JIT provisioning maps OIDC groups claim to collections correctly
- [ ] Access review workflow generates email and logs completion

---

## 4. Security Considerations

- LDAP bind password encrypted at rest; masked in all logs
- SCIM token is separate long-lived credential; should be rotatable without downtime
- LDAP/SCIM sync operations logged in audit trail (CR-002)
- LDAP connection must use LDAPS (TLS) — reject non-TLS unless explicitly overridden

---

## 5. Estimated Effort

| Area | Effort |
|------|--------|
| LDAP connector | 3 sprints |
| SCIM 2.0 endpoints | 4 sprints |
| JIT provisioning enhancement | 1 sprint |
| Access review workflow | 2 sprints |
| Azure AD integration test | 1 sprint |

---

*Status: ✅ Implemented | Author: Product Team | Date: 2026-04-12 | Cập nhật: 2026-04-17*

> **Implementation**: [SOL-003](solutions/SOL-003-ldap-scim.md) — `src/ldap.rs` (319L), `src/api/scim/mod.rs` (427L), DB migration `2026-04-15-000004_sol_003_ldap`
