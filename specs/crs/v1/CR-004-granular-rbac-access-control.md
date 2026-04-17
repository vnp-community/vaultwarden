# CR-004: Granular RBAC, Time-Based & Location-Based Access Control

> **Change Request ID**: CR-004  
> **Title**: Granular RBAC, Attribute-Based Access Control, Time/Location Restrictions  
> **Priority**: P1 — Critical  
> **Target Release**: v2.0  
> **Driven By**: [specs/crs/product-market-analysis.md §2.2 RBAC]  
> **Affects**: PRD §6.4, URD §4.5–4.6, SRS §4.4

---

## 1. Problem Statement

Role model hiện tại (Owner/Admin/Manager/User/Custom) quá đơn giản cho banking:
- Không có time-based access (credential chỉ accessible trong giờ làm việc)
- Không có location-based access (chỉ từ mạng nội bộ)
- Không có maker-checker / dual-approval workflow
- Không có break-glass account workflow được formalize
- Không có separation of duties (SoD) enforcement

---

## 2. Scope of Change

### 2.1 Custom Role Builder

Thay thế hardcoded roles bằng configurable permission sets:

```
CustomRole {
    uuid: RoleId,
    org_uuid: OrganizationId,
    name: String,
    permissions: Vec<Permission>,
}

enum Permission {
    // Collection permissions
    ViewCollectionItems(CollectionId),
    EditCollectionItems(CollectionId),
    DeleteCollectionItems(CollectionId),
    CreateCollectionItems(CollectionId),
    
    // Member management
    InviteMembers,
    RemoveMembers,
    ChangeRoles,
    
    // Admin
    ManageCollections,
    ManageGroups,
    ViewEventLogs,
    ExportOrgVault,
    ManageOrgSettings,
    
    // Privileged
    ViewPrivilegedItems,          // Requires dual approval
    ExportPrivilegedItems,        // Requires dual approval + audit
}
```

### 2.2 Time-Based Access Control

```
AccessSchedule {
    role_uuid OR collection_uuid,
    timezone: String,                     // IANA timezone
    allowed_days: Vec<Weekday>,           // Mon-Fri
    allowed_from: Time,                   // 08:00
    allowed_until: Time,                  // 18:00
    enforce_for_admins: bool,
}
```

**Behavior**:
- Requests outside allowed schedule → 403 Forbidden với message "Access outside permitted hours"
- Sessions active during allowed hours remain valid; next request outside hours → session auto-expired
- Configurable per collection, per role, or per user

**Config**:
```
NEW CONFIG:
ACCESS_SCHEDULE_ENABLED=true
ACCESS_SCHEDULE_DEFAULT_TZ=Asia/Ho_Chi_Minh
```

### 2.3 Location-Based Access Control (Network Restriction)

```
IpAllowList {
    name: String,
    cidr_ranges: Vec<IpNetwork>,     // e.g. "10.0.0.0/8", "192.168.1.0/24"
    applies_to: AppliesTo,           // All | Org | Collection | Role
    org_uuid: Option<OrganizationId>,
}
```

**Config**:
```
NEW CONFIG:
IP_ALLOWLIST_ENABLED=false            # Global default: allow all
IP_ALLOWLIST='[{"cidr":"10.0.0.0/8","name":"Corporate"},{"cidr":"203.0.113.0/24","name":"VPN"}]'
IP_ALLOWLIST_ADMIN_PANEL=true         # Always enforce for admin panel
IP_ALLOWLIST_BYPASS_FOR_WEBAUTHN=false
```

**Behavior**:
- Request from non-allowed IP for restricted resource → 403 with audit event logged
- IP check happens after auth, before resource access
- Emergency bypass: break-glass account can override (logged heavily)

### 2.4 Dual Approval / Maker-Checker Workflow

Required for banking's "four-eyes principle":

```
ApprovalRequest {
    uuid: ApprovalRequestId,
    requester_uuid: UserId,
    approver_group: GroupId,        // Who can approve
    action: ApprovalAction,
    resource: Resource,
    expires_at: DateTime<Utc>,      // Request expires if not actioned
    status: Pending | Approved | Rejected | Expired,
    approver_uuid: Option<UserId>,
    approved_at: Option<DateTime<Utc>>,
    justification: String,          // Required reason
    requester_note: String,
}

enum ApprovalAction {
    ViewPrivilegedCipher(CipherId),
    ExportVault,
    AccessOutsideHours,
    AccessFromUnknownLocation,
    BulkDelete,
    ResetMemberPassword,
}
```

**Flow**:
```
User requests access to privileged resource
    ↓
System creates ApprovalRequest
    ↓
Email + in-app notification sent to approvers
    ↓
Approver approves/rejects (with mandatory comment)
    ↓
If approved: access granted for single use + time window (e.g., 1 hour)
    ↓
Audit entry: requester, approver, resource, justification, timestamp
```

### 2.5 Break-Glass Account

```
BreakGlassAccount {
    uuid: UserId,
    name: String,                    // e.g., "Emergency CISO Access"
    requires_approvers: u8,          // How many people must witness activation
    witness_uuids: Vec<UserId>,
    seal_reason: String,             // Encrypted justification
    last_used: Option<DateTime<Utc>>,
    usage_notification_emails: Vec<String>,  // Notify these when used
}
```

**Behavior**:
- Break-glass activation sends email to ALL witness emails + CISO immediately
- Activation logged with mandatory justification field
- Break-glass sessions expire after configurable time (default: 4 hours)
- All actions during break-glass session flagged in audit log

### 2.6 Separation of Duties (SoD) Rules

```
SoDRule {
    name: String,
    conflict_role_a: RoleId,
    conflict_role_b: RoleId,
    enforcement: Hard | Soft,        // Hard = block, Soft = warn + log
}
```

Example SoD rules:
- "Vault Admin" + "Auditor" → conflict (cannot audit yourself)
- "Key Custodian" + "Transaction Approver" → conflict

---

## 3. Acceptance Criteria

- [ ] Custom role with specific permissions limits access to only those permissions
- [ ] Request outside allowed hours returns 403; access within hours succeeds
- [ ] Request from blocked IP returns 403 and generates audit event
- [ ] Dual approval flow: requester → approver → resource access within time window
- [ ] Break-glass activation sends notifications to all configured witnesses within 60 seconds
- [ ] SoD rule blocks assigning conflicting roles to same user (Hard enforcement)
- [ ] All access control decisions (allow/deny) logged in audit trail (CR-002)

---

## 4. Estimated Effort

| Area | Effort |
|------|--------|
| Custom role builder | 3 sprints |
| Time-based access | 2 sprints |
| IP allowlist | 1 sprint |
| Dual approval workflow | 3 sprints |
| Break-glass account | 2 sprints |
| SoD rules engine | 2 sprints |

---

*Status: Draft | Author: Product Team | Date: 2026-04-12*
