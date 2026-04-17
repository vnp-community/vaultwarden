# TASKS-SOL-011: Multi-Tenancy & Department Isolation

> **Giải pháp**: SOL-011  
> **CR**: CR-011  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 21

> **Cảnh báo**: Đây là thay đổi kiến trúc LỚN NHẤT trong toàn bộ roadmap. Yêu cầu code review đặc biệt cẩn thận cho mọi DB query để tránh data leak cross-tenant.

---

## Sprint 1–2 — Tenant Data Model + Migration (4 tuần)

### [x] TASK-011-001
- **Tên**: DB migration — tenant tables và thêm `tenant_uuid` vào bảng chính
- **File**: `migrations/postgresql/YYYYMMDD_multitenancy/up.sql`
- **Mô tả**: Tạo bảng `tenants` (uuid, name, slug UNIQUE, domain_restriction, is_active, max_users/orgs/vault_items/storage_bytes, config_overrides JSONB, branding). Insert DEFAULT tenant `00000000-0000-0000-0000-000000000001`. Tạo `tenant_admins`. Thêm cột `tenant_uuid NOT NULL DEFAULT 'default-uuid'` vào `users`, `organizations`, `audit_entries`. Tạo indexes trên tenant_uuid.
- **Loại**: New migration
- **Độ phức tạp**: Cao
- **Phụ thuộc**: Không
- **Ghi chú**: Migration phải được test trên production-scale data trước khi deploy

### [x] TASK-011-002
- **Tên**: Implement `Tenant` và `TenantAdmin` models
- **File**: `src/db/models/tenant.rs`
- **Mô tả**: Structs `Tenant`, `TenantAdmin`. Methods: `find_by_uuid()`, `find_by_slug()`, `save()`, `delete()`, `count_users/orgs/ciphers_in_tenant()`. Validate slug format (lowercase, alphanumeric, hyphens). `TenantAdmin::save()`, `delete()`, `find_for_user()`.
- **Loại**: New file (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-001

### [x] TASK-011-003
- **Tên**: Thêm MULTI_TENANCY_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `multi_tenancy_enabled` (default false), `tenant_routing` (subdomain|path|domain), `tenant_default_uuid`, `system_admin_token` (masked), `tenant_rls_enabled`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### [x] TASK-011-004
- **Tên**: Implement backward-compatible DEFAULT mode
- **File**: `src/tenant.rs`
- **Mô tả**: Enum `TenantContext { SingleInstance, Tenant(String), SystemAdmin }`. Khi `MULTI_TENANCY_ENABLED=false`, tất cả functions dùng `TenantContext::SingleInstance`.
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-011-003

---

## Sprint 3–4 — Application-Level Isolation (4 tuần)

### [x] TASK-011-005
- **Tên**: Implement `TenantContext` Rocket request guard
- **File**: `src/tenant.rs`
- **Mô tả**: Full `FromRequest<'r>` impl: SingleInstance (disabled), SystemAdmin (X-System-Admin-Token header), X-Tenant-Id override, subdomain/path routing, fallback to default tenant.
- **Loại**: New code (implemented)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-011-002, TASK-011-004

### [x] TASK-011-006
- **Tên**: Implement tenant slug cache
- **File**: `src/tenant.rs`
- **Mô tả**: `TENANT_SLUG_CACHE: LazyLock<DashMap<String, String>>` — map slug → uuid. `populate_tenant_slug_cache()` at startup. `invalidate_tenant_cache()` after create/update.
- **Loại**: New code (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-011-002

### [x] TASK-011-007
- **Tên**: Convert `User` model sang tenant-aware
- **File**: `src/db/models/user.rs`
- **Mô tả**: Added `find_by_mail_ctx()`, `count_all_ctx()`, `get_all_ctx()` with full `TenantContext` match — SingleInstance/SystemAdmin = no filter, Tenant = tenant_uuid WHERE clause.
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-011-004
- **Ghi chú**: Code review bắt buộc. Mọi query thiếu tenant filter là security bug

### [x] TASK-011-008
- **Tên**: Convert `Organization` model sang tenant-aware
- **File**: `src/db/models/organization.rs`
- **Mô tả**: Added `get_all_ctx()` (filter by tenant_uuid) and `find_by_uuid_ctx()` (validates org belongs to the requesting tenant — returns None for cross-tenant access, preventing data leakage).
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-011-004

### [x] TASK-011-009
- **Tên**: Convert `Event` (audit) model sang tenant-aware
- **File**: `src/db/models/event.rs`
- **Mô tả**: Added `find_by_organization_uuid_ctx()` (tenant-scoped event log query) and `find_by_tenant()` (full tenant audit export for system admin API).
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-004

### [x] TASK-011-010
- **Tên**: Update handlers trong `src/api/core/` để dùng `TenantContext`
- **File**: `src/api/core/organizations.rs`
- **Mô tả**: `create_organization`: added `TenantContext` guard, org-quota check via `check_org_quota()`, assigns `org.tenant_uuid` from ctx. `get_organization`: uses `find_by_uuid_ctx()` to prevent cross-tenant org access. `TenantContext` import added to module.
- **Loại**: Modify existing (implemented — critical handlers wired)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-011-007, TASK-011-008

---

## Sprint 5 — PostgreSQL RLS (2 tuần)

### [x] TASK-011-011
- **Tên**: Implement PostgreSQL RLS policies
- **File**: `migrations/postgresql/2026-04-15-000011_sol_011_rls/up.sql`
- **Mô tả**: `set_current_tenant(tenant_uuid TEXT)` function + `current_tenant()` helper. Enable RLS + FORCE ROW LEVEL SECURITY trên `users`, `organizations`, `audit_entries`. PERMISSIVE policies allowing tenant_uuid match OR SYSTEM_ADMIN sentinel. Rollback in down.sql.
- **Loại**: New migration (implemented)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-011-001
- **Ghi chú**: Chỉ áp dụng khi `TENANT_RLS_ENABLED=true` và PostgreSQL backend

### [x] TASK-011-012
- **Tên**: Implement `set_db_tenant_context()` cho RLS
- **File**: `src/tenant.rs`
- **Mô tả**: `#[cfg(feature = "postgresql")]` function gọi `SELECT set_current_tenant($1)` trên connection. `#[cfg(not)]` no-op stub. Guards controlled by `CONFIG.tenant_rls_enabled()`.
- **Loại**: New code (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-011

---

## Sprint 6 — Tenant Routing (2 tuần)

### [x] TASK-011-013
- **Tên**: Implement subdomain routing
- **File**: `src/tenant.rs`
- **Mô tả**: `extract_tenant_from_subdomain()`: lấy `Host` header, split subdomain, lookup `TENANT_SLUG_CACHE`.
- **Loại**: New function (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-011-006

### [x] TASK-011-014
- **Tên**: Implement path-based routing
- **File**: `src/tenant.rs`
- **Mô tả**: `extract_tenant_from_path()`: parse `/t/{slug}/...`, lookup slug cache. Borrow-safe temporary lifetime fix applied.
- **Loại**: New function (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-006

### [x] TASK-011-015
- **Tên**: Implement domain-based routing
- **File**: `src/tenant.rs`
- **Mô tả**: `extract_tenant_from_email_domain()`: check email domain vs `Tenant.domain_restriction` pattern matching.
- **Loại**: New function (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-002

---

## Sprint 7–8 — Tenant Admin Role (4 tuần)

### [x] TASK-011-016
- **Tên**: Thêm tenant fields vào JWT claims
- **File**: `src/auth.rs`
- **Mô tả**: Mở rộng `LoginJwtClaims` struct: `tenant_uuid: Option<String>`, `is_tenant_admin: bool`, `is_system_admin: bool`. Initialized to None/false in constructor.
- **Loại**: Modify existing (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-002

### [x] TASK-011-017
- **Tên**: Implement `SystemAdminHeaders` request guard
- **File**: `src/auth.rs`
- **Mô tả**: Guard validates `X-System-Admin-Token` header via constant-time SHA-256 comparison. Returns `Outcome::Success(SystemAdminHeaders)` on match, `Outcome::Error(401)` otherwise.
- **Loại**: New code (implemented)
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-011-003

### [x] TASK-011-018
- **Tên**: Implement System Admin API
- **File**: `src/api/system/tenants.rs`, `src/api/system/mod.rs`
- **Mô tả**: Routes: `POST /api/system/tenants`, `GET /api/system/tenants`, `GET /api/system/tenants/{id}`, `PATCH /api/system/tenants/{id}`, `DELETE /api/system/tenants/{id}`, `GET /api/system/tenants/{id}/stats`, `POST/DELETE /api/system/tenants/{id}/admins`. Mounted at `/api/system`.
- **Loại**: New file + module (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-002, TASK-011-017

---

## Sprint 9–10 — Testing + Migration Tool (4 tuần)

### [x] TASK-011-019
- **Tên**: Cross-tenant isolation tests
- **File**: `tests/multitenancy_tests.rs` (mới)
- **Mô tả**: 34 unit tests covering: `TenantContext` variant behaviour (uuid/is_system_admin), slug cache insert/lookup/invalidate, tenant isolation predicate (user sees own tenant, blocked from others, SystemAdmin sees all, SingleInstance backward compat), path-based routing (`/t/{slug}/...`), subdomain routing, quota enforcement (under/at/over limit, no limit), RLS token generation (tenant UUID, SYSTEM_ADMIN sentinel, default tenant), cross-tenant org isolation, event (audit log) tenant filtering, slug format validation (valid/invalid/max-length). All tests are standalone — no live DB required.
- **Loại**: New test file (implemented — 34/34 passing)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-011-007 → TASK-011-012

### [x] TASK-011-020
- **Tên**: Implement resource quota enforcement
- **File**: `src/tenant.rs`
- **Mô tả**: `check_user_quota()`, `check_org_quota()`, `check_vault_item_quota()`: check max_users/orgs/vault_items before create operations. Returns `Err(String)` dengan message khi quota exceeded.
- **Loại**: New code (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-002

### [x] TASK-011-021
- **Tên**: Migration script: assign existing users sang tenants theo email domain
- **File**: `scripts/migrate_to_tenants.sql`
- **Mô tả**: SQL script: 1) Verify existing data DEFAULT tenant. 2) Create tenants per unique email domain. 3) UPDATE users.tenant_uuid. 4) Verification SELECT. Rollback procedure documented in script.
- **Loại**: Migration script (implemented)
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-011-001, TASK-011-002

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1–2 | TASK-011-001 → 004 | 1–4 | Tenant model + migration |
| Sprint 3–4 | TASK-011-005 → 010 | 5–8 | Application-level isolation |
| Sprint 5 | TASK-011-011 → 012 | 9–10 | PostgreSQL RLS |
| Sprint 6 | TASK-011-013 → 015 | 11–12 | Tenant routing |
| Sprint 7–8 | TASK-011-016 → 018 | 13–16 | Tenant admin role + system API |
| Sprint 9–10 | TASK-011-019 → 021 | 17–20 | Testing + migration tool |

---

## Rủi Ro Quan Trọng

| Rủi ro | Xử lý |
|--------|-------|
| Query không có tenant filter → data leak | Bắt buộc code review; PostgreSQL RLS như lớp 2 |
| Performance với tenant WHERE | Index trên `tenant_uuid`; monitor query plans |
| Migration fail | Test trên staging; có rollback script |
| JWT tenant mismatch | Validate tenant trong guard; không trust client-provided tenant |

---

*Tạo từ SOL-011 | Ngày: 2026-04-13*
