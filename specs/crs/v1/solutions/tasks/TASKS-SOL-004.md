# TASKS-SOL-004: Granular RBAC, Time/Location-Based Access Control

> **Giải pháp**: SOL-004  
> **CR**: CR-004  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 22

---

## Sprint 1–3 — Custom Role Builder (6 tuần)

### [x] TASK-004-001
- **Tên**: DB migration — RBAC tables
- **File**: `migrations/postgresql/YYYYMMDD_rbac/up.sql`
- **Mô tả**: Tạo: `custom_roles` (uuid, org_uuid, name, permissions JSONB), `access_schedules`, `ip_allowlists`, `approval_requests`, `break_glass_configs`, `sod_rules`. Thêm cột `custom_role_uuid` vào `memberships`.
- **Loại**: New migration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

### [x] TASK-004-002
- **Tên**: Implement `Permission` enum và `CustomRole` model
- **File**: `src/db/models/custom_role.rs` (mới)
- **Mô tả**: Enum `Permission` (ViewCollectionItems, EditCollectionItems, InviteMembers, ManageOrgSettings, ViewPrivilegedItems, v.v.). Struct `CustomRole` với `permissions: Vec<Permission>`. Method `has_permission()` với superset check.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-001

### [x] TASK-004-003
- **Tên**: Mở rộng `MembershipType::Custom` trong organization model
- **File**: `src/db/models/organization.rs`
- **Mô tả**: Thêm field `custom_role_uuid` vào `Membership` struct. Update Diesel schema. Thêm method `set_custom_role()`, `find_for_user_in_org()`.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-001

### [x] TASK-004-004
- **Tên**: Integrate custom role checks vào auth pipeline
- **File**: `src/auth.rs`
- **Mô tả**: Mở rộng request guards để check custom role permissions. Nếu user có `custom_role_uuid`, load role và kiểm tra permission thay vì dùng fixed role hierarchy.
- **Loại**: Modify existing
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-004-002, TASK-004-003

### [x] TASK-004-005
- **Tên**: Implement Custom Role CRUD API
- **File**: `src/api/core/access_control.rs` (mới)
- **Mô tả**: Routes: `GET/POST /api/organizations/{id}/roles`, `PUT/DELETE /api/organizations/{id}/roles/{role-id}`. Admin auth required. Validate permissions array.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-002

---

## Sprint 4–5 — Time-Based Access Control (4 tuần)

### [x] TASK-004-006
- **Tên**: Implement `AccessSchedule` model
- **File**: `src/db/models/access_schedule.rs` (mới)
- **Mô tả**: Struct `AccessSchedule` với fields từ migration. Methods: `find_applicable(user_uuid, resource_uuid, resource_type, org_uuid)`, CRUD operations.
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-004-001

### [x] TASK-004-007
- **Tên**: Implement `check_time_based_access()` function
- **File**: `src/access_control.rs` (mới)
- **Mô tả**: Lấy schedules áp dụng, parse timezone với `chrono_tz`, check ngày trong tuần và giờ hiện tại so với `allowed_days` + `allowed_from/until`. Emit audit event khi deny. Return `AccessDenied` với error message.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-006
- **Dependency mới**: `chrono-tz = "0.x"`

### [x] TASK-004-008
- **Tên**: Thêm ACCESS_SCHEDULE config + API routes
- **File**: `src/config.rs`, `src/api/core/access_control.rs`
- **Mô tả**: Config: `access_schedule_enabled`, `access_schedule_default_tz`. Routes: `GET/POST /api/organizations/{id}/access-schedules`.
- **Loại**: Modify + new routes
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-004-006

---

## Sprint 6 — IP Allowlist (2 tuần)

### [x] TASK-004-009
- **Tên**: Implement `IpAllowlist` model
- **File**: `src/db/models/ip_allowlist.rs` (mới)
- **Mô tả**: Struct `IpAllowlist` với `cidr_ranges: Vec<String>`. Methods: `find_for_org()`, CRUD. Parse CIDR với `ipnetwork` crate.
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-004-001

### [x] TASK-004-010
- **Tên**: Implement `IpAllowlistFairing`
- **File**: `src/access_control.rs`
- **Mô tả**: Rocket Fairing (Kind::Request): extract remote IP, check global allowlist cho `/admin`, check org-level allowlist nếu request có org context. Set `req.local_cache(IpDenied)` nếu denied.
- **Loại**: New code
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-009

### [x] TASK-004-011
- **Tên**: Thêm IP_ALLOWLIST config + API routes
- **File**: `src/config.rs`, `src/api/core/access_control.rs`
- **Mô tả**: Config: `ip_allowlist_enabled`, `ip_allowlist`, `ip_allowlist_admin_panel`. Routes: `GET/POST /api/organizations/{id}/ip-allowlists`.
- **Loại**: Modify + new routes
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-004-009

---

## Sprint 7–9 — Dual Approval Workflow (6 tuần)

### [x] TASK-004-012
- **Tên**: Implement `ApprovalRequest` model
- **File**: `src/db/models/approval_request.rs` (mới)
- **Mô tả**: Struct `ApprovalRequest` với state machine: pending → approved/rejected/expired. Methods: `find_active_for_resource()`, `approve()`, `reject()`, `find_pending_for_approver()`.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-001

### [x] TASK-004-013
- **Tên**: Integrate approval check vào cipher access
- **File**: `src/api/core/ciphers.rs`
- **Mô tả**: Trong `get_cipher()`: check `cipher.requires_approval`. Nếu có và không có active approval: tạo `ApprovalRequest`, notify approvers, trả 403 với `"approval_required"`. Track `access_count` khi approved.
- **Loại**: Modify existing
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-004-012

### [x] TASK-004-014
- **Tên**: Implement approval API endpoints
- **File**: `src/api/core/access_control.rs`
- **Mô tả**: Routes: `GET /api/approval-requests` (my pending), `POST /api/approval-requests/{id}/approve`, `POST /api/approval-requests/{id}/reject`. Validate approver permission. Gửi email khi approve/reject.
- **Loại**: New routes
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-012

### [x] TASK-004-015
- **Tên**: Implement approver notification
- **File**: `src/api/core/access_control.rs`
- **Mô tả**: `notify_approvers()`: tìm approver group members, gửi email notification với link approve/reject. Email template cho approval request.
- **Loại**: New function
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-014

### [x] TASK-004-016
- **Tên**: Thêm APPROVAL_WORKFLOW config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `approval_workflow_enabled`, `approval_request_ttl_hours` (default 24), `approval_access_window_hours` (default 1).
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

---

## Sprint 10–11 — Break-Glass + SoD (4 tuần)

### [x] TASK-004-017
- **Tên**: Implement `BreakGlassConfig` model
- **File**: `src/db/models/approval_request.rs` (hoặc file riêng)
- **Mô tả**: Struct `BreakGlassConfig`. Methods: `find_by_user_uuid()`, `record_activation()`. Field `witness_uuids`, `notification_emails`, `session_duration_hours`.
- **Loại**: New code
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-004-001

### [x] TASK-004-018
- **Tên**: Implement break-glass activation endpoint
- **File**: `src/api/core/access_control.rs`
- **Mô tả**: `POST /api/break-glass/activate`: validate justification, gửi SECURITY ALERT email đến tất cả notification_emails ngay lập tức, tạo break-glass JWT token với special claim, emit CRITICAL audit event.
- **Loại**: New route
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-017

### [x] TASK-004-019
- **Tên**: Implement `SodRule` model
- **File**: `src/db/models/sod_rule.rs` (mới)
- **Mô tả**: Struct `SodRule` với `role_a_uuid`, `role_b_uuid`, `enforcement` (hard/soft). Methods: `find_for_org()`, CRUD.
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-004-001

### [x] TASK-004-020
- **Tên**: Implement SoD enforcement trong role assignment
- **File**: `src/api/core/organizations.rs`
- **Mô tả**: Trong `assign_role()`: load SoD rules cho org, check conflicts với new role vs current roles. Hard enforcement → err!(), soft → warn + audit event.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-004-019, TASK-004-003

### [x] TASK-004-021
- **Tên**: SoD Rules CRUD API
- **File**: `src/api/core/access_control.rs`
- **Mô tả**: Routes: `GET/POST /api/organizations/{id}/sod-rules`.
- **Loại**: New routes
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-004-019

### [x] TASK-004-022
- **Tên**: Thêm BREAK_GLASS config keys và mount routes
- **File**: `src/config.rs`, `src/api/core/mod.rs`
- **Mô tả**: Config: `break_glass_enabled`, `break_glass_notification_timeout_seconds`. Mount tất cả access_control routes.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-004-005, TASK-004-008, TASK-004-011, TASK-004-014, TASK-004-018, TASK-004-021

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1–3 | TASK-004-001 → 005 | 1–6 | Custom Role Builder |
| Sprint 4–5 | TASK-004-006 → 008 | 7–10 | Time-Based Access |
| Sprint 6 | TASK-004-009 → 011 | 11–12 | IP Allowlist |
| Sprint 7–9 | TASK-004-012 → 016 | 13–18 | Dual Approval Workflow |
| Sprint 10–11 | TASK-004-017 → 022 | 19–22 | Break-Glass + SoD |

---

*Tạo từ SOL-004 | Ngày: 2026-04-13*
