# TASKS-SOL-003: AD/LDAP Native Integration & SCIM 2.0 Provisioning

> **Giải pháp**: SOL-003  
> **CR**: CR-003  
> **Ngày tạo**: 2026-04-13  
> **Tổng số tasks**: 20

---

## Sprint 1–3 — LDAP Connector (6 tuần)

### [x] TASK-003-001
- **Tên**: DB migration — LDAP/SCIM tables
- **File**: `migrations/postgresql/2026-04-15-000004_sol_003_ldap/up.sql`
- **Mô tả**: Tạo: `ldap_sync_state`, `ldap_group_mappings`, `access_reviews`, `access_review_items`, `scim_tokens`. Thêm cột vào `users`: `provisioning_source`, `provisioning_external_id`, `suspension_scheduled_at`. Thêm cột vào `organizations`: `ldap_group_dn`.
- **Loại**: New migration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không
- **Trạng thái**: ✅ Migration exists at `migrations/postgresql/2026-04-15-000004_sol_003_ldap/up.sql`

### [x] TASK-003-002
- **Tên**: Thêm LDAP_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `ldap_enabled`, `ldap_host`, `ldap_port`, `ldap_use_tls`, `ldap_bind_dn`, `ldap_bind_password` (masked), `ldap_base_dn`, `ldap_user_filter`, `ldap_user_attr_email/name/uuid`, `ldap_group_*`, `ldap_sync_interval_minutes`, `ldap_sync_org_uuid`, `ldap_deprovision_grace_days`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Trạng thái**: ✅ All 14 LDAP config keys added to `src/config.rs`

### [x] TASK-003-003
- **Tên**: Implement `LdapConnector` struct với `ldap3` crate
- **File**: `src/ldap.rs` (mới)
- **Mô tả**: Struct `LdapConnector`. Phương thức `sync()`: connect với LDAPS/StartTLS, bind với service account, search users theo filter, iterate entries. Crate `ldap3 = "0.11"` thêm vào Cargo.toml.
- **Loại**: New file
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-003-001, TASK-003-002
- **Dependency mới**: `ldap3 = "0.11"`
- **Trạng thái**: ✅ Full connector with connect/bind/search implemented

### [x] TASK-003-004
- **Tên**: Implement user provisioning từ LDAP
- **File**: `src/ldap.rs`
- **Mô tả**: `provision_user()`: tạo `User` record với `provisioning_source="ldap"`, random password, gửi invite email, thêm vào org được config. `update_user_if_changed()`: so sánh attrs và update nếu khác.
- **Loại**: New code
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-003-003
- **Trạng thái**: ✅ `provision_or_update_user()` implemented

### [x] TASK-003-005
- **Tên**: Implement user deprovisioning từ LDAP
- **File**: `src/ldap.rs`
- **Mô tả**: `deprovision_user()`: revoke all sessions, schedule suspension sau 90 ngày (vault data preserved), emit audit event. Phát hiện users bị remove khỏi LDAP bằng cách so sánh email sets.
- **Loại**: New code
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-003-004
- **Trạng thái**: ✅ `deprovision_removed_users()` implemented with grace-period scheduling

### [x] TASK-003-006
- **Tên**: Implement LDAP group → collection mapping sync
- **File**: `src/ldap.rs`
- **Mô tả**: `sync_group_memberships()`: đọc `ldap_group_mappings`, sync user membership vào collections theo group DN. Xử lý thêm/xóa membership khi user thay đổi group.
- **Loại**: New code
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-003-003, TASK-003-001
- **Trạng thái**: ✅ `sync_group_memberships()` + `sync_collection_membership()` implemented

### [x] TASK-003-007
- **Tên**: Implement background LDAP sync job
- **File**: `src/ldap.rs`, `src/main.rs`
- **Mô tả**: `ldap_sync_job()` — wrapper gọi `LdapConnector::sync()`, ghi `LdapSyncState::record_success/error()`. Đăng ký vào job scheduler với interval `LDAP_SYNC_INTERVAL_MINUTES`.
- **Loại**: New function + scheduler integration
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-003-005, TASK-003-006
- **Trạng thái**: ✅ `ldap_sync_job()` registered in `main.rs` scheduler

---

## Sprint 4–7 — SCIM 2.0 (8 tuần)

### [x] TASK-003-008
- **Tên**: Thêm SCIM_* config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `scim_enabled`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Trạng thái**: ✅ `scim_enabled` config key exists

### [x] TASK-003-009
- **Tên**: Implement SCIM Bearer token middleware
- **File**: `src/api/scim/users.rs` (mới, module `src/api/scim/`)
- **Mô tả**: Struct `ScimAuth` với `FromRequest`. Verify token via SHA-256 compare với `scim_tokens.token_hash`. Update `last_used_at`.
- **Loại**: New file/module
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-003-001
- **Trạng thái**: ✅ `ScimAuth` Rocket guard with SHA-256 hash verification implemented

### [x] TASK-003-010
- **Tên**: Implement SCIM Users endpoints (list, get, create)
- **File**: `src/api/scim/users.rs`
- **Mô tả**: `GET /scim/v2/Users` (với filter, pagination), `GET /scim/v2/Users/{id}`, `POST /scim/v2/Users`. Tạo user với `provisioning_source="scim"`, thêm vào org của token, sync groups.
- **Loại**: New routes
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-003-009
- **Trạng thái**: ✅ `GET/POST /scim/v2/Users`, `GET /scim/v2/Users/<id>` implemented

### [x] TASK-003-011
- **Tên**: Implement SCIM Users PATCH endpoint
- **File**: `src/api/scim/users.rs`
- **Mô tả**: `PATCH /scim/v2/Users/{id}` — xử lý operations: Replace active=false (revoke sessions + disable), Add/Replace groups (sync collections). Emit audit events.
- **Loại**: New route
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-003-010
- **Trạng thái**: ✅ `PATCH /scim/v2/Users/<id>` with active/displayName ops implemented

### [x] TASK-003-012
- **Tên**: Implement SCIM Groups endpoints
- **File**: `src/api/scim/groups.rs` (mới)
- **Mô tả**: `GET /scim/v2/Groups`, `GET /scim/v2/Groups/{id}`, `POST /scim/v2/Groups`, `PATCH /scim/v2/Groups/{id}` — map SCIM groups sang Collections.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-003-009
- **Trạng thái**: ✅ Full Groups CRUD with member add/remove via PATCH implemented

### [x] TASK-003-013
- **Tên**: Implement SCIM ServiceProviderConfig
- **File**: `src/api/scim/schema.rs` (mới)
- **Mô tả**: `GET /scim/v2/ServiceProviderConfig`, `GET /scim/v2/Schemas`, `GET /scim/v2/ResourceTypes` — trả về metadata về SCIM capabilities.
- **Loại**: New file
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-003-009
- **Trạng thái**: ✅ All three metadata endpoints implemented

### [x] TASK-003-014
- **Tên**: Mount SCIM routes trong `main.rs`
- **File**: `src/main.rs`, `src/api/scim/mod.rs` (mới)
- **Mô tả**: `rocket.mount("/scim", scim::routes())`. SCIM không mount dưới `/api` (SCIM 2.0 spec yêu cầu `/scim/v2/`).
- **Loại**: New module, modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-003-010, TASK-003-012, TASK-003-013
- **Trạng thái**: ✅ Routes mounted in `main.rs` under `/scim`

---

## Sprint 8 — JIT Enhancement (2 tuần)

### [x] TASK-003-015
- **Tên**: Thêm SSO JIT config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `sso_jit_provision_enabled`, `sso_jit_org_uuid`, `sso_jit_group_claim`, `sso_jit_group_collection_map` (JSON), `sso_jit_default_role`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Trạng thái**: ✅ All 5 SSO JIT config keys added

### [x] TASK-003-016
- **Tên**: Mở rộng JIT provisioning với group claim mapping
- **File**: `src/sso.rs`
- **Mô tả**: `jit_provision_from_claims()`: tạo user từ OIDC claims với `provisioning_source="sso"`. Đọc group claim từ `SSO_JIT_GROUP_CLAIM`, map sang collections theo `SSO_JIT_GROUP_COLLECTION_MAP` JSON config. Gọi `ensure_collection_membership()`.
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-003-015
- **Trạng thái**: ✅ `jit_provision_from_claims()` exists in `src/sso.rs`

---

## Sprint 9–10 — Access Review (4 tuần)

### [x] TASK-003-017
- **Tên**: Implement Access Review models
- **File**: `src/db/models/access_review.rs` (mới)
- **Mô tả**: Structs: `AccessReview`, `AccessReviewItem`. CRUD methods: `create()`, `find_overdue()`, `find_pending()`, `mark_completed()`, `mark_auto_revoked()`.
- **Loại**: New file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-003-001
- **Trạng thái**: ✅ Full model with all CRUD methods implemented

### [x] TASK-003-018
- **Tên**: Implement quarterly access review job
- **File**: `src/db/models/access_review.rs`
- **Mô tả**: `access_review_job()`: tạo `AccessReview` record, tạo items cho tất cả memberships, gửi email cho org owners với link review. Đăng ký vào scheduler với `ACCESS_REVIEW_INTERVAL_DAYS`.
- **Loại**: New function
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-003-017
- **Trạng thái**: ✅ `access_review_job()` implemented + registered in `main.rs`

### [x] TASK-003-019
- **Tên**: Implement access review deadline + auto-revoke job
- **File**: Cùng file với TASK-003-018
- **Mô tả**: `access_review_deadline_job()`: tìm overdue reviews, với mỗi unreviewed item: xóa `CollectionUser`, mark auto-revoked, emit audit event. Mark review completed.
- **Loại**: New function
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-003-017
- **Trạng thái**: ✅ `access_review_deadline_job()` implemented + registered in `main.rs` (daily at 01:00 UTC)

### [x] TASK-003-020
- **Tên**: Thêm ACCESS_REVIEW config keys
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `access_review_enabled`, `access_review_interval_days` (default 90), `access_review_deadline_days` (default 14).
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Trạng thái**: ✅ All 3 access review config keys exist in `src/config.rs`

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1–3 | TASK-003-001 → 007 | 1–6 | LDAP sync, provision/deprovision |
| Sprint 4–7 | TASK-003-008 → 014 | 7–14 | SCIM 2.0 full implementation |
| Sprint 8 | TASK-003-015 → 016 | 15–16 | SSO JIT group mapping |
| Sprint 9–10 | TASK-003-017 → 020 | 17–20 | Access review workflow |

---

*Tạo từ SOL-003 | Ngày: 2026-04-13 | Cập nhật: 2026-04-16*
