# CR-007: Privileged Access Management (PAM)

> **Change Request ID**: CR-007  
> **Title**: Privileged Access Management — Session Recording, Credential Checkout, JIT Elevation  
> **Priority**: P1 — Critical  
> **Target Release**: v2.1  
> **Driven By**: [specs/crs/product-market-analysis.md §2.4]  
> **Affects**: PRD §6 (New Feature F-PAM), URD §4.5–4.6, SRS §4.3–4.4

---

## 1. Problem Statement

Vaultwarden hiện tại là **password manager** thuần túy — không phải PAM solution. Nhưng banking cần PAM capabilities cho privileged accounts (database admin, system admin, trading system access):

- Không có session recording (yêu cầu PCI DSS, SOC 2)
- Không có just-in-time privilege elevation
- Không có automated password rotation
- Không có time-limited credential checkout
- Không có approval workflow cho privileged access (đã xử lý một phần ở CR-004)
- Không có integration với ITSM (ServiceNow)

---

## 2. Scope of Change

### 2.1 Privileged Vault (Restricted Cipher Type)

Thêm "Privileged" flag cho vault items:
```rust
Cipher {
    // ... existing fields ...
    is_privileged: bool,                        // NEW
    privileged_config: Option<PrivilegedConfig>, // NEW
}

PrivilegedConfig {
    requires_approval: bool,
    approval_group_uuid: Option<GroupId>,
    max_checkout_duration_minutes: u32,         // Default: 60
    auto_rotate_after_checkout: bool,
    rotation_target: Option<RotationTarget>,
    session_recording_enabled: bool,
    view_count_limit: Option<u32>,
    concurrent_access_limit: Option<u32>,
}
```

### 2.2 Credential Checkout System

**Checkout Flow**:
```
User requests privileged credential
    ↓
System checks: requires_approval?
    ├── Yes → Create ApprovalRequest (CR-004 §2.4)
    │         ↓ Approved by approver
    └── No  ──┘
    ↓
Checkout record created:
    - checkout_uuid
    - user_uuid
    - cipher_uuid
    - checked_out_at
    - expires_at (now + max_checkout_duration_minutes)
    - ip_address
    - justification (required)
    - session_recording_id (if enabled)
    ↓
Credential returned to user (one-time or TTL-based)
    ↓
On expiry OR manual check-in:
    - Session recording finalized
    - Password rotation triggered (if auto_rotate_after_checkout=true)
    - Checkout record archived
```

**API**:
```
POST /api/ciphers/{id}/checkout
{
  "justification": "Investigating incident INC-20260412-001",
  "requested_duration_minutes": 60
}

POST /api/ciphers/{id}/checkin
DELETE /api/checkouts/{checkout_id}   # Force check-in by admin

GET /api/ciphers/{id}/checkouts       # Active checkouts for this credential
GET /api/checkouts?active=true&user=  # Admin: all active checkouts
```

### 2.3 Automated Password Rotation

Sau khi credential được check in (hoặc theo schedule), tự động rotate password:

```
RotationTarget {
    type: SSH | RDPWIN | DatabaseMySQL | DatabasePostgres | APIKey | Custom,
    host: String,
    port: u16,
    username: String,
    auth_method: SSHKey | Password,
    rotation_script: Option<String>,   // Custom rotation script path
    verify_after_rotation: bool,
}

RotationPolicy {
    enabled: bool,
    trigger: AfterCheckout | Schedule | Manual,
    schedule: Option<CronExpression>,
    min_rotation_interval_hours: u32,
    notify_on_failure: Vec<String>,
}
```

**Supported rotation targets**:
- SSH (via SSH connection + passwd command)
- Windows RDP/local admin (via WinRM)
- MySQL/MariaDB database users
- PostgreSQL database users
- Custom rotation scripts (Docker-based execution)

**Config**:
```
NEW CONFIG:
PAM_ROTATION_ENABLED=false
PAM_ROTATION_WORKER_CONCURRENCY=5
PAM_ROTATION_TIMEOUT_SECONDS=60
PAM_ROTATION_SSH_KEY_PATH=data/rotation_key
```

### 2.4 Session Recording

For privileged SSH/RDP sessions proxied through Vaultwarden (advanced feature):

> **Note**: Full session recording requires Vaultwarden acting as a jump server/proxy — this is an advanced mode. Basic credential access recording (who accessed what credential, when) is logged in audit trail (CR-002) without proxying.

**Phase 1 (v2.1)**: Credential access recording (via audit log)
- Who accessed which privileged credential
- When, from which IP, with which justification
- How long they held the credential (checkout duration)

**Phase 2 (future)**: SSH proxy mode (out of scope for v2.1)

### 2.5 ITSM Integration (ServiceNow)

Approval workflow integration with ServiceNow:

```
NEW CONFIG:
ITSM_ENABLED=false
ITSM_TYPE=servicenow|jira
ITSM_SERVICENOW_INSTANCE=https://company.service-now.com
ITSM_SERVICENOW_USER=vaultwarden-integration
ITSM_SERVICENOW_PASSWORD=<secret>
ITSM_TICKET_REQUIRED=false             # Require valid ticket number for checkout
ITSM_TICKET_VALIDATION=true            # Validate ticket exists and is open
```

**Behavior with ITSM_TICKET_REQUIRED=true**:
- Checkout request must include `ticket_number`
- System validates ticket exists in ServiceNow and is in valid state
- If ticket is closed/invalid → checkout denied
- Ticket number logged in checkout record and audit trail

### 2.6 Privileged Access Dashboard

Admin view:
```
GET /api/admin/pam/dashboard
{
  "active_checkouts": 3,
  "overdue_checkouts": 0,
  "rotations_pending": 2,
  "rotations_failed_24h": 0,
  "privileged_ciphers_count": 47,
  "approval_requests_pending": 1
}
```

---

## 3. Acceptance Criteria

- [ ] Privileged credential requires justification and creates checkout record
- [ ] Checkout expires after configured duration; expired checkout triggers audit event
- [ ] Auto-rotation successfully rotates SSH password after checkout (test environment)
- [ ] ServiceNow ticket validation rejects checkout when ticket is closed
- [ ] Admin dashboard shows all active checkouts in real time
- [ ] Checkout history searchable by user, credential, date range in audit log

---

## 4. Estimated Effort

| Area | Effort |
|------|--------|
| Privileged cipher type | 1 sprint |
| Checkout system | 3 sprints |
| Auto-rotation engine (SSH + DB) | 4 sprints |
| ITSM integration | 2 sprints |
| PAM dashboard | 1 sprint |
| Phase 2 SSH proxy (future) | TBD |

---

*Status: ✅ Implemented | Author: Product Team | Date: 2026-04-12 | Cập nhật: 2026-04-17*

> **Implementation**: [SOL-007](solutions/SOL-007-pam.md) — `src/pam/` (checkout 99L, rotation 114L, itsm), `src/api/core/pam.rs`, DB migration `2026-04-15-000007_sol_007_pam` (privileged_configs, checkouts, rotation_history)
