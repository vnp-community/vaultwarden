# SOL-003: Giải Pháp Thực Hiện — AD/LDAP Native Integration & SCIM 2.0 Provisioning

> **Giải pháp cho**: CR-003  
> **Ngày**: 2026-04-12  
> **Trạng thái**: ✅ Implemented  
> **Kiến trúc thay đổi**: Trung bình — thêm LDAP connector + SCIM API module mới  
> **Cập nhật**: 2026-04-17 — Verified full implementation in codebase

---

## 1. Tổng Quan Giải Pháp

Giải pháp tận dụng các thành phần sẵn có:
- **SSO JIT provisioning** hiện có trong `src/sso.rs` → mở rộng cho LDAP/SCIM
- **Job scheduler** hiện có → thêm LDAP sync job
- **Organization/Membership models** hiện có → mapping từ LDAP groups
- **Reqwest HTTP client** hiện có → SCIM outbound notifications

**Chiến lược**:
1. LDAP Connector: module mới `src/ldap.rs` + background sync job
2. SCIM 2.0: module mới `src/api/scim/` với Rocket routes
3. JIT Enhancement: mở rộng `src/sso.rs`
4. Access Review: background job + email workflow

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/ldap.rs` | LDAP connector, sync logic, user/group mapping |
| `src/api/scim/mod.rs` | SCIM 2.0 route registration |
| `src/api/scim/users.rs` | SCIM User resource endpoints |
| `src/api/scim/groups.rs` | SCIM Group resource endpoints |
| `src/api/scim/schema.rs` | SCIM ServiceProviderConfig, Schemas, ResourceTypes |
| `src/db/models/ldap_sync.rs` | Tracking LDAP sync state |
| `src/db/models/access_review.rs` | Access review workflow state |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/sso.rs` | Mở rộng JIT provisioning với group claim mapping |
| `src/config.rs` | Thêm LDAP_* và SCIM_* config keys |
| `src/main.rs` | Mount SCIM routes, thêm LDAP sync job |
| `src/db/models/user.rs` | Thêm `external_id`, `provisioning_source` fields |
| `src/db/models/organization.rs` | Thêm `ldap_group_dn` mapping field |

### 2.3 Database Migrations

```sql
-- migrations/postgresql/YYYYMMDD_ldap_scim/up.sql

-- LDAP sync tracking
CREATE TABLE ldap_sync_state (
    id              SERIAL PRIMARY KEY,
    last_sync_at    TIMESTAMPTZ,
    last_sync_status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- ok, error, running
    users_synced    INTEGER NOT NULL DEFAULT 0,
    users_created   INTEGER NOT NULL DEFAULT 0,
    users_disabled  INTEGER NOT NULL DEFAULT 0,
    error_message   TEXT,
    next_sync_at    TIMESTAMPTZ
);

-- LDAP group → Collection mapping
CREATE TABLE ldap_group_mappings (
    id              SERIAL PRIMARY KEY,
    ldap_group_dn   TEXT NOT NULL UNIQUE,
    org_uuid        VARCHAR(40) NOT NULL REFERENCES organizations(uuid),
    collection_uuid VARCHAR(40) REFERENCES collections(uuid),
    access_level    VARCHAR(20) NOT NULL DEFAULT 'read',  -- read, write, admin
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Access review workflow
CREATE TABLE access_reviews (
    uuid            VARCHAR(40) PRIMARY KEY,
    org_uuid        VARCHAR(40) NOT NULL,
    initiated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    due_at          TIMESTAMPTZ NOT NULL,               -- 14 days default
    status          VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, completed, overdue
    reviewed_by     VARCHAR(40),
    completed_at    TIMESTAMPTZ
);

CREATE TABLE access_review_items (
    id              BIGSERIAL PRIMARY KEY,
    review_uuid     VARCHAR(40) NOT NULL REFERENCES access_reviews(uuid),
    user_uuid       VARCHAR(40) NOT NULL,
    collection_uuid VARCHAR(40),
    access_level    VARCHAR(20),
    decision        VARCHAR(20),                        -- approved, revoked, null=pending
    decided_by      VARCHAR(40),
    decided_at      TIMESTAMPTZ
);

-- Thêm vào users
ALTER TABLE users ADD COLUMN provisioning_source VARCHAR(20) DEFAULT 'manual';  -- manual, ldap, scim, sso
ALTER TABLE users ADD COLUMN provisioning_external_id TEXT;
ALTER TABLE users ADD COLUMN suspension_scheduled_at TIMESTAMPTZ;  -- For graceful deprovisioning

-- SCIM token table
CREATE TABLE scim_tokens (
    uuid        VARCHAR(40) PRIMARY KEY,
    org_uuid    VARCHAR(40) NOT NULL REFERENCES organizations(uuid),
    token_hash  VARCHAR(64) NOT NULL,   -- SHA-256 của token (không lưu plaintext)
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE
);
```

---

## 3. Thiết Kế Chi Tiết

### 3.1 LDAP Connector

**File**: `src/ldap.rs`

**Phụ thuộc mới**: `ldap3` crate (pure Rust LDAP client)

```rust
use ldap3::{LdapConnAsync, Scope, SearchEntry};

pub struct LdapConnector {
    config: LdapConfig,
}

impl LdapConnector {
    pub async fn sync(&self, conn: &DbConn) -> Result<SyncStats, Error> {
        let (ldap_conn, mut ldap) = LdapConnAsync::with_settings(
            ConnectionSettings::new().set_starttls(self.config.use_tls),
            &format!("ldaps://{}:{}", self.config.host, self.config.port),
        ).await?;
        
        drive!(ldap_conn);
        
        // Bind với service account
        ldap.simple_bind(&self.config.bind_dn, &self.config.bind_password).await?
            .success()?;
        
        // Tìm tất cả users trong LDAP
        let (entries, _) = ldap.search(
            &self.config.base_dn,
            Scope::Subtree,
            &self.config.user_filter,
            vec![
                &self.config.attr_email,
                &self.config.attr_name,
                &self.config.attr_uuid,
                "memberOf",
            ],
        ).await?.success()?;
        
        let mut stats = SyncStats::default();
        
        for entry in SearchEntry::construct(entries) {
            let email = entry.attrs.get(&self.config.attr_email)
                .and_then(|v| v.first())
                .ok_or_else(|| Error::new("LDAP entry missing email", ""))?;
            
            let external_id = entry.attrs.get(&self.config.attr_uuid)
                .and_then(|v| v.first())
                .unwrap_or(email);
            
            match User::find_by_email(email, conn).await? {
                Some(user) if user.provisioning_source == "ldap" => {
                    // User đã tồn tại từ LDAP — cập nhật nếu cần
                    stats.updated += self.update_user_if_changed(&user, &entry, conn).await?;
                }
                None => {
                    // User mới trong LDAP — auto-provision
                    self.provision_user(email, &entry, conn).await?;
                    stats.created += 1;
                }
                _ => {} // User tồn tại nhưng không phải từ LDAP — bỏ qua
            }
        }
        
        // Revoke users đã bị remove khỏi LDAP
        let ldap_emails: HashSet<&str> = entries_emails.iter().map(|s| s.as_str()).collect();
        let vw_ldap_users = User::find_all_by_source("ldap", conn).await?;
        
        for user in vw_ldap_users {
            if !ldap_emails.contains(user.email.as_str()) {
                self.deprovision_user(&user, conn).await?;
                stats.disabled += 1;
            }
        }
        
        // Sync group → collection mappings
        self.sync_group_memberships(conn).await?;
        
        Ok(stats)
    }
    
    async fn provision_user(
        &self, 
        email: &str, 
        entry: &SearchEntry,
        conn: &DbConn,
    ) -> Result<(), Error> {
        let name = entry.attrs.get(&self.config.attr_name)
            .and_then(|v| v.first())
            .unwrap_or(email);
        
        // Tạo user với random secure password (họ sẽ đăng nhập qua SSO/LDAP)
        let user = User {
            uuid: crate::util::get_uuid(),
            email: email.to_string(),
            name: name.to_string(),
            enabled: true,
            provisioning_source: "ldap".to_string(),
            ..User::new_empty()
        };
        user.save(conn).await?;
        
        // Thêm vào org được cấu hình
        if let Some(org_uuid) = &self.config.sync_org_uuid {
            Membership::invite_user(&user.uuid, org_uuid, MembershipType::User, conn).await?;
        }
        
        // Gửi email mời
        mail::send_invite(&user.email, &user.name).await.ok();
        
        Ok(())
    }
    
    async fn deprovision_user(&self, user: &User, conn: &DbConn) -> Result<(), Error> {
        // Revoke tất cả sessions ngay lập tức
        Device::delete_all_by_user(&user.uuid, conn).await?;
        
        // Schedule suspension sau 90 ngày (vault data preserved)
        User::schedule_suspension(&user.uuid, 
            Utc::now() + Duration::days(90), conn).await?;
        
        // Emit audit event
        audit::emit(AuditEntry {
            event_type: AuditEventType::UserDeprovisioned { source: "ldap".to_string() },
            actor_user_uuid: None,
            target_resource: Some(user.uuid.clone()),
            metadata: json!({"email": user.email, "reason": "removed_from_ldap"}),
            ..Default::default()
        });
        
        Ok(())
    }
}

// Background sync job — thêm vào job scheduler
pub async fn ldap_sync_job() {
    if !CONFIG.ldap_enabled() { return; }
    
    let connector = LdapConnector::from_config();
    let pool = DB_POOL.get().expect("DB pool");
    let conn = pool.get().expect("DB connection");
    
    match connector.sync(&conn).await {
        Ok(stats) => {
            info!("LDAP sync completed: {} created, {} updated, {} disabled", 
                  stats.created, stats.updated, stats.disabled);
            LdapSyncState::record_success(stats, &conn).await.ok();
        }
        Err(e) => {
            error!("LDAP sync failed: {e}");
            LdapSyncState::record_error(e.to_string(), &conn).await.ok();
        }
    }
}
```

### 3.2 SCIM 2.0 Endpoints

**File**: `src/api/scim/users.rs`

```rust
// SCIM Bearer token middleware
struct ScimAuth(ScimToken);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ScimAuth {
    type Error = Error;
    
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req.headers().get_one("Authorization")
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| Error::new("Missing Authorization header", ""))?;
        
        let conn = req.guard::<DbConn>().await.unwrap();
        match ScimToken::verify(token, &conn).await {
            Ok(t) => Outcome::Success(ScimAuth(t)),
            Err(_) => Outcome::Error((Status::Unauthorized, Error::new("Invalid SCIM token", ""))),
        }
    }
}

// GET /scim/v2/Users
#[get("/scim/v2/Users?<filter>&<startIndex>&<count>")]
async fn list_users(
    filter: Option<&str>,
    start_index: Option<i64>,
    count: Option<i64>,
    _auth: ScimAuth,
    conn: DbConn,
) -> JsonResult {
    let (users, total) = User::list_for_scim(filter, start_index, count, &conn).await?;
    
    Ok(Json(json!({
        "schemas": ["urn:ietf:params:scim:api:messages:2.0:ListResponse"],
        "totalResults": total,
        "startIndex": start_index.unwrap_or(1),
        "itemsPerPage": count.unwrap_or(100),
        "Resources": users.into_iter().map(|u| user_to_scim(u)).collect::<Vec<_>>()
    })))
}

// POST /scim/v2/Users — Tạo user mới
#[post("/scim/v2/Users", data = "<scim_user>")]
async fn create_user(
    scim_user: Json<ScimUser>,
    auth: ScimAuth,
    conn: DbConn,
) -> Result<(Status, Json<Value>), Error> {
    let email = scim_user.user_name.to_lowercase();
    
    if User::find_by_email(&email, &conn).await?.is_some() {
        return Err(Error::new("User already exists", "uniqueness"));
    }
    
    let user = User {
        uuid: get_uuid(),
        email: email.clone(),
        name: scim_user.display_name.clone().unwrap_or_else(|| email.clone()),
        enabled: scim_user.active.unwrap_or(true),
        provisioning_source: "scim".to_string(),
        provisioning_external_id: scim_user.external_id.clone(),
        ..User::new_empty()
    };
    user.save(&conn).await?;
    
    // Thêm vào org nếu auth token belongs to an org
    if let Some(org_uuid) = &auth.0.org_uuid {
        Membership::invite_user(&user.uuid, org_uuid, MembershipType::User, &conn).await?;
    }
    
    // Sync groups → collections
    if let Some(groups) = &scim_user.groups {
        sync_user_collections(&user.uuid, groups, &conn).await?;
    }
    
    // Audit log
    audit::emit(AuditEntry {
        event_type: AuditEventType::UserProvisioned { source: "scim".to_string() },
        target_resource: Some(user.uuid.clone()),
        metadata: json!({"email": email, "external_id": scim_user.external_id}),
        ..Default::default()
    });
    
    Ok((Status::Created, Json(user_to_scim(user))))
}

// PATCH /scim/v2/Users/{id} — Partial update (active: false → revoke)
#[patch("/scim/v2/Users/<id>", data = "<patch>")]
async fn patch_user(
    id: &str,
    patch: Json<ScimPatch>,
    _auth: ScimAuth,
    conn: DbConn,
) -> JsonResult {
    let user = User::find_by_scim_external_id(id, &conn).await?
        .ok_or_else(|| Error::new("User not found", ""))?;
    
    for op in &patch.operations {
        match (op.op.as_str(), op.path.as_deref()) {
            ("Replace", Some("active")) => {
                let active = op.value.as_bool().unwrap_or(true);
                if !active {
                    // Revoke tất cả sessions ngay lập tức
                    Device::delete_all_by_user(&user.uuid, &conn).await?;
                    User::disable(&user.uuid, &conn).await?;
                    
                    audit::emit(AuditEntry {
                        event_type: AuditEventType::UserRevoked { reason: "scim_deactivate".to_string() },
                        target_resource: Some(user.uuid.clone()),
                        ..Default::default()
                    });
                }
            }
            ("Add", Some("groups")) | ("Replace", Some("groups")) => {
                let group_ids: Vec<String> = op.value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|g| g.get("value").and_then(|v| v.as_str()).map(|s| s.to_string()))
                        .collect())
                    .unwrap_or_default();
                sync_user_collections(&user.uuid, &group_ids, &conn).await?;
            }
            _ => {}
        }
    }
    
    Ok(Json(user_to_scim(user)))
}
```

### 3.3 JIT Provisioning Enhancement

Mở rộng `src/sso.rs`:

```rust
pub async fn jit_provision_from_claims(
    claims: &IdTokenClaims,
    conn: &DbConn,
) -> Result<User, Error> {
    let email = claims.email().ok_or_else(|| Error::new("Missing email claim", ""))?;
    
    let user = match User::find_by_email(email, conn).await? {
        Some(u) => u,
        None => {
            // Tạo user mới từ SSO claims
            let user = User {
                uuid: get_uuid(),
                email: email.to_lowercase(),
                name: claims.name().map(|n| n.to_string()).unwrap_or_else(|| email.to_string()),
                enabled: true,
                provisioning_source: "sso".to_string(),
                provisioning_external_id: Some(claims.subject().to_string()),
                ..User::new_empty()
            };
            user.save(conn).await?;
            user
        }
    };
    
    // Map OIDC groups → Collections
    if CONFIG.sso_jit_provision_enabled() {
        if let Some(groups_claim) = claims.additional_claims()
            .get(CONFIG.sso_jit_group_claim()) 
        {
            let groups: Vec<String> = serde_json::from_value(groups_claim.clone())
                .unwrap_or_default();
            
            let group_map: HashMap<String, String> = 
                serde_json::from_str(CONFIG.sso_jit_group_collection_map())
                .unwrap_or_default();
            
            for group in &groups {
                if let Some(collection_uuid) = group_map.get(group) {
                    ensure_collection_membership(
                        &user.uuid, 
                        collection_uuid,
                        CONFIG.sso_jit_org_uuid(),
                        conn
                    ).await.ok();
                }
            }
        }
    }
    
    Ok(user)
}
```

### 3.4 Access Review Workflow

Background job (quarterly):

```rust
pub async fn access_review_job(conn: &DbConn) {
    // Lấy tất cả orgs cần review
    let orgs = Organization::find_requiring_access_review(conn).await
        .unwrap_or_default();
    
    for org in orgs {
        let review = AccessReview::create(&org.uuid, 
            Utc::now() + Duration::days(14), conn).await.unwrap();
        
        // Tạo review items cho tất cả memberships
        let memberships = Membership::find_all_by_org(&org.uuid, conn).await.unwrap();
        for m in &memberships {
            AccessReviewItem::create(&review.uuid, &m.user_uuid, conn).await.ok();
        }
        
        // Gửi email cho org owners
        let owners = Membership::find_owners(&org.uuid, conn).await.unwrap();
        for owner in owners {
            if let Some(user) = User::find_by_uuid(&owner.user_uuid, conn).await.ok().flatten() {
                mail::send_access_review_required(
                    &user.email, 
                    &org.name, 
                    &review.uuid,
                    review.due_at
                ).await.ok();
            }
        }
    }
}

// Background job: auto-revoke unreviewed access after deadline
pub async fn access_review_deadline_job(conn: &DbConn) {
    let overdue = AccessReview::find_overdue(conn).await.unwrap_or_default();
    
    for review in overdue {
        let unreviewed = AccessReviewItem::find_pending(&review.uuid, conn).await.unwrap();
        for item in unreviewed {
            // Auto-revoke unreviewed access
            if let Some(col_uuid) = &item.collection_uuid {
                CollectionUser::delete(&item.user_uuid, col_uuid, conn).await.ok();
            }
            AccessReviewItem::mark_auto_revoked(&item.id, conn).await.ok();
            
            audit::emit(AuditEntry {
                event_type: AuditEventType::AccessAutoRevoked { reason: "review_deadline".to_string() },
                target_resource: Some(item.user_uuid.clone()),
                org_uuid: Some(review.org_uuid.clone()),
                ..Default::default()
            });
        }
        AccessReview::mark_completed(&review.uuid, conn).await.ok();
    }
}
```

---

## 4. Config Variables Mới

```bash
# LDAP
LDAP_ENABLED=false
LDAP_HOST=ldap.example.com
LDAP_PORT=636
LDAP_USE_TLS=true
LDAP_BIND_DN=""
LDAP_BIND_PASSWORD=""               # Masked
LDAP_BASE_DN=""
LDAP_USER_FILTER=(objectClass=person)
LDAP_USER_ATTR_EMAIL=mail
LDAP_USER_ATTR_NAME=displayName
LDAP_USER_ATTR_UUID=objectGUID
LDAP_GROUP_BASE_DN=""
LDAP_GROUP_FILTER=(objectClass=group)
LDAP_GROUP_ATTR_MEMBER=member
LDAP_SYNC_INTERVAL_MINUTES=15
LDAP_SYNC_ORG_UUID=""
LDAP_GROUPS_TO_COLLECTIONS=true
LDAP_DEPROVISION_GRACE_DAYS=90      # Days to keep vault before deletion

# SCIM 2.0
SCIM_ENABLED=false

# SSO JIT Enhancement
SSO_JIT_PROVISION_ENABLED=false
SSO_JIT_ORG_UUID=""
SSO_JIT_GROUP_CLAIM=groups
SSO_JIT_GROUP_COLLECTION_MAP={}
SSO_JIT_DEFAULT_ROLE=user

# Access Review
ACCESS_REVIEW_ENABLED=false
ACCESS_REVIEW_INTERVAL_DAYS=90      # Quarterly
ACCESS_REVIEW_DEADLINE_DAYS=14
```

---

## 5. Phụ Thuộc Mới

| Crate | Phiên bản | Lý do |
|-------|-----------|-------|
| `ldap3` | 0.11 | Pure Rust async LDAP client |

> `reqwest` đã có sẵn cho SCIM outbound operations.

---

## 6. Mount Points Mới

```rust
// src/main.rs
rocket.mount("/scim", scim::routes())
```

**SCIM routes** không mount dưới `/api` để tuân thủ SCIM 2.0 spec (path phải là `/scim/v2/`).

---

## 7. Kế Hoạch Triển Khai

### Sprint 1–3: LDAP Connector
- `ldap3` integration
- `src/ldap.rs` — sync logic, user provision/deprovision
- Background job trong scheduler

### Sprint 4–7: SCIM 2.0
- `src/api/scim/` module với all endpoints
- SCIM Bearer token management
- Azure AD + Okta integration tests

### Sprint 8: JIT Enhancement
- Group claim mapping trong `src/sso.rs`
- ABAC via OIDC claims

### Sprint 9–10: Access Review
- DB models, background jobs
- Email templates
- Auto-revoke deadline logic

---

*Status: ✅ Implemented | Ngày cập nhật: 2026-04-17*

## Implementation Notes
- `src/ldap.rs` (319 lines) — LDAP connector, sync logic, user/group mapping fully implemented
- `src/api/scim/mod.rs` (427 lines) — Full SCIM 2.0 endpoints (Users, Groups, ServiceProviderConfig)
- `src/db/models/ldap_sync.rs`, `src/db/models/access_review.rs` — tracking models
- DB migration: `2026-04-15-000004_sol_003_ldap` — ldap_sync_state, ldap_group_mappings, access_reviews, scim_tokens
- JIT provisioning enhancement in `src/sso.rs`
- Access review workflow with background jobs
- SCIM bearer token middleware
