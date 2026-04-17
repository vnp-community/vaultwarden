# SOL-004: Giải Pháp Thực Hiện — Granular RBAC, Time/Location-Based Access Control

> **Giải pháp cho**: CR-004  
> **Ngày**: 2026-04-12  
> **Trạng thái**: ✅ Implemented  
> **Kiến trúc thay đổi**: Trung bình — mở rộng permission model, thêm middleware layers  
> **Cập nhật**: 2026-04-17 — Verified full implementation in codebase

---

## 1. Tổng Quan Giải Pháp

Vaultwarden hiện có roles `Owner/Admin/Manager/User/Custom` trong `src/db/models/organization.rs`. Giải pháp **mở rộng** model hiện có:

1. **Custom Role Builder**: Mở rộng `CustomRole` enum thành permission set cấu hình được
2. **Time-Based Access**: Middleware trong Rocket request guard
3. **IP Allow List**: Fairing check sau auth, trước resource access
4. **Dual Approval**: Workflow engine mới với email notification
5. **Break-Glass**: Special user type với alert mechanism
6. **SoD Rules**: Validation khi assign roles

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/access_control.rs` | Core access control engine (time, IP, SoD checks) |
| `src/api/core/access_control.rs` | REST API cho access control config |
| `src/db/models/custom_role.rs` | CustomRole permission set model |
| `src/db/models/access_schedule.rs` | Time-based access schedule |
| `src/db/models/ip_allowlist.rs` | IP allowlist rules |
| `src/db/models/approval_request.rs` | Dual approval workflow records |
| `src/db/models/sod_rule.rs` | Separation of Duties rules |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/db/models/organization.rs` | Mở rộng `MembershipType::Custom` để reference `custom_role_uuid` |
| `src/auth.rs` | Thêm access control checks vào request guard pipeline |
| `src/api/core/ciphers.rs` | Check privileged access, emit approval requests |
| `src/api/core/organizations.rs` | Thêm SoD validation khi assign roles |
| `src/config.rs` | Thêm ACCESS_SCHEDULE_*, IP_ALLOWLIST_* config keys |

### 2.3 Database Migrations

```sql
-- migrations/postgresql/YYYYMMDD_rbac/up.sql

-- Custom role permission sets
CREATE TABLE custom_roles (
    uuid        VARCHAR(40) PRIMARY KEY,
    org_uuid    VARCHAR(40) NOT NULL REFERENCES organizations(uuid) ON DELETE CASCADE,
    name        VARCHAR(100) NOT NULL,
    description TEXT,
    permissions JSONB NOT NULL DEFAULT '[]',  -- Array of permission strings
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by  VARCHAR(40),
    UNIQUE (org_uuid, name)
);

-- Time-based access schedules
CREATE TABLE access_schedules (
    uuid            VARCHAR(40) PRIMARY KEY,
    org_uuid        VARCHAR(40) NOT NULL,
    name            VARCHAR(100) NOT NULL,
    applies_to_type VARCHAR(20) NOT NULL,  -- 'collection', 'role', 'user'
    applies_to_uuid VARCHAR(40) NOT NULL,
    timezone        VARCHAR(50) NOT NULL DEFAULT 'UTC',
    allowed_days    SMALLINT[] NOT NULL,   -- 0=Mon ... 6=Sun
    allowed_from    TIME NOT NULL,
    allowed_until   TIME NOT NULL,
    enforce_for_admins BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- IP allowlist rules
CREATE TABLE ip_allowlists (
    uuid        VARCHAR(40) PRIMARY KEY,
    org_uuid    VARCHAR(40),              -- NULL = global
    name        VARCHAR(100) NOT NULL,
    cidr_ranges TEXT[] NOT NULL,          -- e.g. {'10.0.0.0/8', '192.168.1.0/24'}
    applies_to  VARCHAR(20) NOT NULL DEFAULT 'org',  -- 'all', 'org', 'collection', 'role'
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Dual approval / maker-checker requests
CREATE TABLE approval_requests (
    uuid            VARCHAR(40) PRIMARY KEY,
    requester_uuid  VARCHAR(40) NOT NULL,
    org_uuid        VARCHAR(40) NOT NULL,
    approver_group_uuid VARCHAR(40),
    action_type     VARCHAR(50) NOT NULL,   -- 'view_privileged', 'export_vault', 'bulk_delete', etc.
    resource_type   VARCHAR(50),
    resource_uuid   VARCHAR(40),
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, approved, rejected, expired
    justification   TEXT NOT NULL,
    requester_note  TEXT,
    approver_uuid   VARCHAR(40),
    approved_at     TIMESTAMPTZ,
    rejection_reason TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,   -- Request auto-expires
    access_until    TIMESTAMPTZ,            -- If approved, access valid until
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Break-glass accounts
CREATE TABLE break_glass_configs (
    user_uuid           VARCHAR(40) PRIMARY KEY REFERENCES users(uuid),
    org_uuid            VARCHAR(40) NOT NULL,
    name                VARCHAR(200) NOT NULL,
    requires_witnesses  SMALLINT NOT NULL DEFAULT 1,
    witness_uuids       TEXT[] NOT NULL DEFAULT '{}',
    notification_emails TEXT[] NOT NULL DEFAULT '{}',
    session_duration_hours SMALLINT NOT NULL DEFAULT 4,
    last_used_at        TIMESTAMPTZ,
    seal_reason         TEXT
);

-- Separation of Duties rules
CREATE TABLE sod_rules (
    uuid            VARCHAR(40) PRIMARY KEY,
    org_uuid        VARCHAR(40) NOT NULL,
    name            VARCHAR(200) NOT NULL,
    description     TEXT,
    role_a_uuid     VARCHAR(40) NOT NULL,
    role_b_uuid     VARCHAR(40) NOT NULL,
    enforcement     VARCHAR(10) NOT NULL DEFAULT 'hard',  -- 'hard', 'soft'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Thêm vào memberships: reference đến custom role
ALTER TABLE memberships ADD COLUMN custom_role_uuid VARCHAR(40) REFERENCES custom_roles(uuid);
```

---

## 3. Thiết Kế Chi Tiết

### 3.1 Permission Model

```rust
// src/db/models/custom_role.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    // Collection permissions
    ViewCollectionItems(String),      // collection_uuid
    EditCollectionItems(String),
    DeleteCollectionItems(String),
    CreateCollectionItems(String),
    
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
    
    // Privileged (require dual approval)
    ViewPrivilegedItems,
    ExportPrivilegedItems,
}

impl CustomRole {
    pub fn has_permission(&self, perm: &Permission) -> bool {
        self.permissions.iter().any(|p| p == perm || p.is_superset_of(perm))
    }
}
```

### 3.2 Time-Based Access Control Middleware

```rust
// src/access_control.rs

pub async fn check_time_based_access(
    user_uuid: &str,
    resource_uuid: &str,
    resource_type: &str,
    org_uuid: Option<&str>,
    conn: &DbConn,
) -> Result<(), AccessDenied> {
    if !CONFIG.access_schedule_enabled() {
        return Ok(());
    }
    
    // Tìm schedule áp dụng cho user, role, hoặc collection này
    let schedules = AccessSchedule::find_applicable(
        user_uuid, resource_uuid, resource_type, org_uuid, conn
    ).await.unwrap_or_default();
    
    if schedules.is_empty() { return Ok(()); }
    
    let now_utc = Utc::now();
    
    for schedule in &schedules {
        let tz: chrono_tz::Tz = schedule.timezone.parse()
            .unwrap_or(chrono_tz::UTC);
        let now_local = now_utc.with_timezone(&tz);
        
        let weekday = now_local.weekday().num_days_from_monday() as i16;
        let current_time = now_local.time();
        
        let day_allowed = schedule.allowed_days.contains(&weekday);
        let time_allowed = current_time >= schedule.allowed_from 
                        && current_time <= schedule.allowed_until;
        
        if !day_allowed || !time_allowed {
            audit::emit(AuditEntry {
                event_type: AuditEventType::AccessDeniedTimeRestriction,
                actor_user_uuid: Some(user_uuid.to_string()),
                target_resource: Some(resource_uuid.to_string()),
                metadata: json!({
                    "schedule_uuid": schedule.uuid,
                    "local_time": now_local.to_rfc3339(),
                    "timezone": schedule.timezone,
                }),
                ..Default::default()
            });
            
            return Err(AccessDenied {
                code: "ACCESS_OUTSIDE_PERMITTED_HOURS",
                message: format!(
                    "Access outside permitted hours. Allowed: {}-{} {} on {:?}",
                    schedule.allowed_from, schedule.allowed_until,
                    schedule.timezone, schedule.allowed_days
                ),
            });
        }
    }
    
    Ok(())
}
```

### 3.3 IP Allowlist Check

```rust
// Fairing — chạy sau auth, trước route handler
pub struct IpAllowlistFairing;

#[rocket::async_trait]
impl Fairing for IpAllowlistFairing {
    fn info(&self) -> Info {
        Info { name: "IP Allowlist", kind: Kind::Request }
    }
    
    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        if !CONFIG.ip_allowlist_enabled() { return; }
        
        let remote_ip = req.client_ip().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        
        // Admin panel: luôn enforce
        if req.uri().path().starts_with("/admin") && CONFIG.ip_allowlist_admin_panel() {
            if !is_ip_allowed_global(remote_ip).await {
                req.local_cache(|| Some(IpDenied));
            }
            return;
        }
        
        // Kiểm tra org-level allowlist nếu request có org context
        if let Some(org_uuid) = extract_org_uuid_from_request(req) {
            if let Ok(conn) = get_db_conn().await {
                if !is_ip_allowed_for_org(remote_ip, &org_uuid, &conn).await {
                    req.local_cache(|| Some(IpDenied));
                }
            }
        }
    }
}

async fn is_ip_allowed_for_org(ip: IpAddr, org_uuid: &str, conn: &DbConn) -> bool {
    let allowlists = IpAllowlist::find_for_org(org_uuid, conn).await
        .unwrap_or_default();
    
    if allowlists.is_empty() { return true; }  // No restrictions = allow all
    
    allowlists.iter().any(|al| {
        al.cidr_ranges.iter().any(|cidr| {
            let network: IpNetwork = cidr.parse().unwrap_or_else(|_| IpNetwork::new(ip, 32).unwrap());
            network.contains(ip)
        })
    })
}
```

### 3.4 Dual Approval Workflow

```rust
// src/api/core/ciphers.rs — khi user muốn xem privileged cipher
pub async fn get_cipher(
    uuid: &str,
    user: &Headers,
    conn: &DbConn,
) -> JsonResult {
    let cipher = Cipher::find_by_uuid(uuid, conn).await?
        .ok_or_else(|| Error::new("Cipher not found", ""))?;
    
    // Check nếu cipher cần approval
    if cipher.requires_approval {
        let active_approval = ApprovalRequest::find_active_for_resource(
            &user.user.uuid, uuid, "cipher", conn
        ).await?;
        
        if active_approval.is_none() {
            // Tạo approval request
            let request = ApprovalRequest {
                uuid: get_uuid(),
                requester_uuid: user.user.uuid.clone(),
                org_uuid: cipher.organization_uuid.clone().unwrap_or_default(),
                action_type: "view_privileged".to_string(),
                resource_type: Some("cipher".to_string()),
                resource_uuid: Some(uuid.to_string()),
                status: "pending".to_string(),
                justification: "Access to privileged credential".to_string(),
                expires_at: Utc::now() + Duration::hours(24),
                ..Default::default()
            };
            request.save(conn).await?;
            
            // Notify approvers
            notify_approvers(&request, conn).await?;
            
            return Err(Error::new(
                "Approval required to access this privileged credential. \
                 Request submitted for review.", 
                "approval_required"
            ));
        }
    }
    
    // Normal cipher access...
    Ok(Json(cipher.to_json()))
}

// POST /api/approval-requests/{id}/approve
pub async fn approve_request(
    request_uuid: &str,
    approver: &Headers,
    body: Json<ApprovalDecision>,
    conn: DbConn,
) -> JsonResult {
    let request = ApprovalRequest::find_by_uuid(request_uuid, &conn).await?
        .ok_or_else(|| Error::new("Request not found", ""))?;
    
    // Validate approver has permission
    check_approval_permission(&approver.user, &request, &conn).await?;
    
    ApprovalRequest::approve(
        request_uuid,
        &approver.user.uuid,
        &body.comment,
        Utc::now() + Duration::hours(1),  // 1 hour access window
        &conn,
    ).await?;
    
    // Notify requester
    if let Some(requester) = User::find_by_uuid(&request.requester_uuid, &conn).await? {
        mail::send_approval_granted(&requester.email, &request).await.ok();
    }
    
    // Audit
    audit::emit(AuditEntry {
        event_type: AuditEventType::ApprovalGranted,
        actor_user_uuid: Some(approver.user.uuid.clone()),
        target_resource: request.resource_uuid.clone(),
        org_uuid: Some(request.org_uuid.clone()),
        metadata: json!({
            "requester": request.requester_uuid,
            "resource_type": request.resource_type,
            "comment": body.comment,
        }),
        ..Default::default()
    });
    
    Ok(Json(json!({"status": "approved"})))
}
```

### 3.5 Break-Glass Account

```rust
// POST /api/break-glass/activate
pub async fn activate_break_glass(
    user: &Headers,
    body: Json<BreakGlassActivation>,
    conn: DbConn,
) -> JsonResult {
    let config = BreakGlassConfig::find_by_user_uuid(&user.user.uuid, &conn).await?
        .ok_or_else(|| Error::new("Not a break-glass account", ""))?;
    
    // Validation: justification bắt buộc
    if body.justification.trim().is_empty() {
        err!("Justification is required for break-glass activation");
    }
    
    // Notify ALL witnesses + notification emails NGAY LẬP TỨC
    let notification_msg = format!(
        "SECURITY ALERT: Break-glass account '{}' activated by {} at {}.\n\
         Justification: {}\n\
         Session duration: {} hours",
        config.name, user.user.email, Utc::now().to_rfc3339(),
        body.justification, config.session_duration_hours
    );
    
    for email in &config.notification_emails {
        mail::send_break_glass_notification(email, &notification_msg).await.ok();
    }
    
    // Cập nhật last used
    BreakGlassConfig::record_activation(&user.user.uuid, &conn).await?;
    
    // Tạo break-glass session JWT với special claim
    let session_token = auth::create_break_glass_token(
        &user.user.uuid,
        config.session_duration_hours,
    )?;
    
    // Audit (CRITICAL severity)
    audit::emit(AuditEntry {
        event_type: AuditEventType::BreakGlassActivated,
        severity: Severity::Critical,
        actor_user_uuid: Some(user.user.uuid.clone()),
        metadata: json!({
            "justification": body.justification,
            "session_hours": config.session_duration_hours,
            "notified_count": config.notification_emails.len(),
        }),
        ..Default::default()
    });
    
    Ok(Json(json!({
        "session_token": session_token,
        "expires_in_hours": config.session_duration_hours,
        "warning": "All actions in this session are being logged with CRITICAL severity",
    })))
}
```

### 3.6 SoD Enforcement

```rust
// src/api/core/organizations.rs — khi assign role
pub async fn assign_role(
    user_uuid: &str,
    org_uuid: &str,
    new_role_uuid: &str,
    conn: &DbConn,
) -> Result<(), Error> {
    // Check SoD conflicts
    let current_roles = CustomRole::find_for_user_in_org(user_uuid, org_uuid, conn).await?;
    let sod_rules = SodRule::find_for_org(org_uuid, conn).await?;
    
    for rule in &sod_rules {
        let conflicts_with_new = rule.role_a_uuid == new_role_uuid || 
                                  rule.role_b_uuid == new_role_uuid;
        
        if conflicts_with_new {
            let conflicting_role_uuid = if rule.role_a_uuid == new_role_uuid {
                &rule.role_b_uuid
            } else {
                &rule.role_a_uuid
            };
            
            if current_roles.iter().any(|r| &r.uuid == conflicting_role_uuid) {
                match rule.enforcement.as_str() {
                    "hard" => {
                        err!(format!(
                            "Cannot assign role: conflicts with SoD rule '{}'", 
                            rule.name
                        ));
                    }
                    "soft" => {
                        // Log warning nhưng không block
                        warn!("SoD soft conflict: user {} assigned conflicting roles per rule '{}'",
                              user_uuid, rule.name);
                        audit::emit(AuditEntry {
                            event_type: AuditEventType::SodConflictWarning,
                            severity: Severity::Warn,
                            target_resource: Some(user_uuid.to_string()),
                            org_uuid: Some(org_uuid.to_string()),
                            metadata: json!({
                                "rule": rule.name,
                                "new_role": new_role_uuid,
                                "conflicting_role": conflicting_role_uuid,
                            }),
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Proceed with role assignment
    Membership::set_custom_role(user_uuid, org_uuid, new_role_uuid, conn).await
}
```

---

## 4. Config Variables Mới

```bash
# Time-based access
ACCESS_SCHEDULE_ENABLED=false
ACCESS_SCHEDULE_DEFAULT_TZ=Asia/Ho_Chi_Minh

# IP Allowlist
IP_ALLOWLIST_ENABLED=false
IP_ALLOWLIST=[]                     # JSON array: [{"cidr":"10.0.0.0/8","name":"Corp"}]
IP_ALLOWLIST_ADMIN_PANEL=true       # Always enforce for /admin

# Dual Approval
APPROVAL_WORKFLOW_ENABLED=false
APPROVAL_REQUEST_TTL_HOURS=24       # Request expires after 24h
APPROVAL_ACCESS_WINDOW_HOURS=1      # Approved access valid for 1h

# Break-glass
BREAK_GLASS_ENABLED=false
BREAK_GLASS_NOTIFICATION_TIMEOUT_SECONDS=60
```

---

## 5. API Endpoints Mới

| Method | Path | Auth | Mô tả |
|--------|------|------|-------|
| GET | `/api/organizations/{id}/roles` | Admin | List custom roles |
| POST | `/api/organizations/{id}/roles` | Admin | Create custom role |
| PUT | `/api/organizations/{id}/roles/{role-id}` | Admin | Update role permissions |
| DELETE | `/api/organizations/{id}/roles/{role-id}` | Admin | Delete role |
| GET | `/api/organizations/{id}/access-schedules` | Admin | List schedules |
| POST | `/api/organizations/{id}/access-schedules` | Admin | Create schedule |
| GET | `/api/organizations/{id}/ip-allowlists` | Admin | List IP rules |
| POST | `/api/organizations/{id}/ip-allowlists` | Admin | Create IP rule |
| GET | `/api/approval-requests` | User | My pending requests |
| POST | `/api/approval-requests/{id}/approve` | Approver | Approve request |
| POST | `/api/approval-requests/{id}/reject` | Approver | Reject request |
| POST | `/api/break-glass/activate` | Break-glass user | Activate session |
| GET | `/api/organizations/{id}/sod-rules` | Admin | List SoD rules |
| POST | `/api/organizations/{id}/sod-rules` | Admin | Create SoD rule |

---

## 6. Kế Hoạch Triển Khai

### Sprint 1–3: Custom Role Builder
- DB migration + models
- Permission set evaluation engine
- Integration với existing auth checks

### Sprint 4–5: Time-Based Access
- `AccessSchedule` model + API
- `check_time_based_access()` middleware
- Integration tests

### Sprint 6: IP Allowlist
- `IpAllowlist` model + API
- `IpAllowlistFairing` implementation

### Sprint 7–9: Dual Approval Workflow
- `ApprovalRequest` model + state machine
- Email notification
- API endpoints
- Integration với cipher access

### Sprint 10–11: Break-Glass + SoD
- Break-glass config + activation flow
- SoD rule enforcement
- Admin UI integration

---

*Status: ✅ Implemented | Ngày cập nhật: 2026-04-17*

## Implementation Notes
- `src/access_control.rs` (86 lines) — Core time/IP/SoD access control engine
- `src/api/core/access_control.rs` — REST API for access control config
- DB models: `access_schedule.rs`, `ip_allowlist.rs`, `approval_request.rs`, `break_glass_config.rs`, `sod_rule.rs` — all present
- DB migration: `2026-04-15-000005_sol_004_rbac` — custom_roles, access_schedules, ip_allowlists, approval_requests, break_glass_configs, sod_rules
- IpAllowlistFairing implemented in `src/util.rs`
- Break-glass activation + notification in `src/api/core/access_control.rs`
- SoD enforcement integrated into organization role assignment
- Dual approval workflow wired into cipher access + PAM checkout
