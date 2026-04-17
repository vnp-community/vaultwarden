# SOL-011: Giải Pháp Thực Hiện — Multi-Tenancy & Department Isolation

> **Giải pháp cho**: CR-011  
> **Ngày**: 2026-04-12  
> **Trạng thái**: Draft  
> **Kiến trúc thay đổi**: Đáng kể — thay đổi data model cơ bản, migration strategy quan trọng

---

## 1. Tổng Quan Giải Pháp

Multi-tenancy là thay đổi kiến trúc LỚN NHẤT trong toàn bộ roadmap. Phải thêm `tenant_uuid` vào hầu hết các bảng chính. Chiến lược:

1. **Tenant model**: Bảng `tenants` + `TenantContext` trong request lifecycle
2. **Data isolation**: Application-level filtering (tất cả backends) + PostgreSQL RLS (PostgreSQL only)
3. **Tenant routing**: Subdomain/path/domain-based
4. **Migration**: Tạo DEFAULT tenant, assign tất cả existing data
5. **Feature flag**: `MULTI_TENANCY_ENABLED=false` — existing deployments không bị ảnh hưởng

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/tenant.rs` | TenantContext, tenant routing, isolation middleware |
| `src/api/system/mod.rs` | System admin API |
| `src/api/system/tenants.rs` | Tenant management endpoints |
| `src/db/models/tenant.rs` | Tenant + TenantAdmin data models |

### 2.2 Files Hiện Có Cần Sửa (Đáng Kể)

| File | Thay đổi |
|------|---------|
| `src/db/models/user.rs` | Thêm `tenant_uuid` field, queries có WHERE clause |
| `src/db/models/organization.rs` | Thêm `tenant_uuid` field |
| `src/db/models/event.rs` | Thêm `tenant_uuid` field |
| `src/auth.rs` | Thêm tenant context vào JWT claims |
| `src/config.rs` | Thêm MULTI_TENANCY_* config keys |
| `src/main.rs` | Thêm tenant routing middleware |
| Hầu hết `src/api/core/` | Scope queries theo tenant_uuid |

---

## 3. Database Design

### 3.1 Migrations

```sql
-- migrations/postgresql/YYYYMMDD_multitenancy/up.sql

-- Tenant table
CREATE TABLE tenants (
    uuid                VARCHAR(40) PRIMARY KEY,
    name                VARCHAR(200) NOT NULL,
    slug                VARCHAR(100) NOT NULL UNIQUE,   -- URL-friendly identifier
    domain_restriction  TEXT,                           -- @treasury.bank.com
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Resource quotas
    max_users           INTEGER,                        -- NULL = unlimited
    max_organizations   INTEGER,
    max_vault_items     INTEGER,
    max_storage_bytes   BIGINT,
    
    -- Config overrides
    config_overrides    JSONB NOT NULL DEFAULT '{}',
    
    -- Branding
    custom_logo_url     TEXT,
    primary_color       VARCHAR(7)
);

-- Default tenant for existing deployments
INSERT INTO tenants (uuid, name, slug) 
VALUES ('00000000-0000-0000-0000-000000000001', 'Default', 'default')
ON CONFLICT DO NOTHING;

-- Tenant admins
CREATE TABLE tenant_admins (
    tenant_uuid         VARCHAR(40) NOT NULL REFERENCES tenants(uuid) ON DELETE CASCADE,
    user_uuid           VARCHAR(40) NOT NULL REFERENCES users(uuid) ON DELETE CASCADE,
    can_manage_users    BOOLEAN NOT NULL DEFAULT TRUE,
    can_manage_orgs     BOOLEAN NOT NULL DEFAULT TRUE,
    can_view_audit_logs BOOLEAN NOT NULL DEFAULT TRUE,
    can_manage_config   BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (tenant_uuid, user_uuid)
);

-- Thêm tenant_uuid vào các bảng chính
-- NOTE: Tất cả có DEFAULT để backward compatible

ALTER TABLE users 
    ADD COLUMN tenant_uuid VARCHAR(40) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001'
        REFERENCES tenants(uuid);

ALTER TABLE organizations 
    ADD COLUMN tenant_uuid VARCHAR(40) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001'
        REFERENCES tenants(uuid);

ALTER TABLE audit_entries 
    ADD COLUMN tenant_uuid VARCHAR(40) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001'
        REFERENCES tenants(uuid);

-- Index cho tenant-scoped queries
CREATE INDEX idx_users_tenant ON users(tenant_uuid);
CREATE INDEX idx_organizations_tenant ON organizations(tenant_uuid);
CREATE INDEX idx_audit_entries_tenant ON audit_entries(tenant_uuid);
```

### 3.2 PostgreSQL Row-Level Security (Optional, PostgreSQL only)

```sql
-- RLS chỉ khi ở PostgreSQL mode VÀ multi-tenancy enabled
-- Tạo DB function để set tenant context
CREATE OR REPLACE FUNCTION set_current_tenant(tenant_uuid TEXT)
RETURNS void AS $$
BEGIN
    PERFORM set_config('app.current_tenant', tenant_uuid, true);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- RLS trên users
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_users ON users
    USING (
        tenant_uuid = current_setting('app.current_tenant', true)::VARCHAR
        OR 
        current_setting('app.current_tenant', true) = 'SYSTEM_ADMIN'  -- System admin bypass
    );

CREATE POLICY tenant_insert_users ON users FOR INSERT
    WITH CHECK (
        tenant_uuid = current_setting('app.current_tenant', true)::VARCHAR
    );

-- Tương tự cho organizations, audit_entries
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_orgs ON organizations
    USING (
        tenant_uuid = current_setting('app.current_tenant', true)::VARCHAR
        OR current_setting('app.current_tenant', true) = 'SYSTEM_ADMIN'
    );
```

---

## 4. Thiết Kế Chi Tiết

### 4.1 TenantContext — Request-Scoped Tenant Identity

**File**: `src/tenant.rs`

```rust
#[derive(Debug, Clone)]
pub enum TenantContext {
    SingleInstance,                    // Multi-tenancy disabled (v1.x compat)
    Tenant(String),                    // tenant_uuid
    SystemAdmin,                       // Cross-tenant access
}

// Rocket request guard
#[rocket::async_trait]
impl<'r> FromRequest<'r> for TenantContext {
    type Error = Error;
    
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if !CONFIG.multi_tenancy_enabled() {
            return Outcome::Success(TenantContext::SingleInstance);
        }
        
        // Lấy tenant từ request context
        let tenant_uuid = match CONFIG.tenant_routing() {
            "subdomain" => extract_tenant_from_subdomain(req),
            "path"      => extract_tenant_from_path(req),
            "domain"    => extract_tenant_from_email_domain(req),
            _           => None,
        };
        
        // Fallback: lấy từ JWT claim (nếu đã auth)
        let tenant_uuid = tenant_uuid.or_else(|| extract_tenant_from_jwt(req));
        
        // Fallback: default tenant
        let tenant_uuid = tenant_uuid
            .unwrap_or_else(|| CONFIG.tenant_default_uuid().to_string());
        
        // Validate tenant exists và active
        let conn = req.guard::<DbConn>().await.unwrap();
        match Tenant::find_by_uuid(&tenant_uuid, &conn).await {
            Ok(Some(tenant)) if tenant.is_active => {
                // Set PostgreSQL session config cho RLS
                #[cfg(feature = "postgresql")]
                set_db_tenant_context(&tenant_uuid, &conn).await.ok();
                
                Outcome::Success(TenantContext::Tenant(tenant_uuid))
            }
            _ => {
                Outcome::Error((Status::NotFound, Error::new("Tenant not found", "")))
            }
        }
    }
}

fn extract_tenant_from_subdomain(req: &Request<'_>) -> Option<String> {
    let host = req.headers().get_one("Host")?;
    // treasury.vaultwarden.bank.com → "treasury"
    let subdomain = host.split('.').next()?;
    
    // Look up tenant by slug
    // NOTE: This requires a cached tenant lookup to avoid DB call per request
    TENANT_SLUG_CACHE.get(subdomain).cloned()
}

fn extract_tenant_from_path(req: &Request<'_>) -> Option<String> {
    let path = req.uri().path().as_str();
    // /t/treasury/api/... → "treasury"
    if path.starts_with("/t/") {
        let slug = path.trim_start_matches("/t/")
            .split('/')
            .next()?;
        TENANT_SLUG_CACHE.get(slug).cloned()
    } else {
        None
    }
}
```

### 4.2 Application-Level Isolation (tất cả DB backends)

Tất cả model queries phải scope theo tenant. Approach: **thêm `tenant_uuid` parameter** vào tất cả find functions.

```rust
// src/db/models/user.rs

impl User {
    // Cũ:
    pub async fn find_by_email(email: &str, conn: &DbConn) -> Result<Option<Self>, Error> {
        conn.run(move |c| {
            users::table.filter(users::email.eq(email)).first(c).optional()
        }).await.map_err(Into::into)
    }
    
    // Mới: tenant-scoped
    pub async fn find_by_email_in_tenant(
        email: &str, 
        tenant_uuid: &str,
        conn: &DbConn,
    ) -> Result<Option<Self>, Error> {
        conn.run(move |c| {
            users::table
                .filter(users::email.eq(email))
                .filter(users::tenant_uuid.eq(tenant_uuid))
                .first(c)
                .optional()
        }).await.map_err(Into::into)
    }
    
    // Backward compat: khi multi-tenancy disabled, không filter
    pub async fn find_by_email_ctx(
        email: &str,
        ctx: &TenantContext,
        conn: &DbConn,
    ) -> Result<Option<Self>, Error> {
        match ctx {
            TenantContext::SingleInstance | TenantContext::SystemAdmin => {
                Self::find_by_email(email, conn).await
            }
            TenantContext::Tenant(tid) => {
                Self::find_by_email_in_tenant(email, tid, conn).await
            }
        }
    }
}
```

**Pattern chuẩn**: Mỗi model function có `_ctx` variant nhận `&TenantContext`. Handlers chọn variant phù hợp.

### 4.3 Tenant Routing

#### Subdomain mode

```nginx
# Reverse proxy config
server {
    server_name *.vaultwarden.bank.com;
    location / {
        proxy_set_header Host $host;
        proxy_set_header X-Tenant-Subdomain $host;
        proxy_pass http://vaultwarden:8080;
    }
}
```

Vaultwarden đọc `Host` header, extract subdomain, lookup tenant slug cache.

#### Path mode

```rust
// Rocket route prefix cho tenant path mode
// /t/{slug}/api/... → mount dưới /t/<slug>
// Rocket fairings strip /t/{slug} prefix và set TenantContext
```

### 4.4 System Admin API

```rust
// POST /api/system/tenants
#[post("/system/tenants", data = "<body>")]
async fn create_tenant(
    body: Json<CreateTenantRequest>,
    _system_admin: SystemAdminHeaders,  // Special auth for system admin
    conn: DbConn,
) -> JsonResult {
    let tenant = Tenant {
        uuid: get_uuid(),
        name: body.name.clone(),
        slug: slugify(&body.name),
        domain_restriction: body.domain_restriction.clone(),
        is_active: true,
        max_users: body.max_users,
        ..Default::default()
    };
    tenant.save(&conn).await?;
    
    // Invalidate slug cache
    TENANT_SLUG_CACHE.invalidate(&tenant.slug);
    
    audit::emit(AuditEntry {
        event_type: AuditEventType::TenantCreated,
        metadata: json!({"tenant_uuid": tenant.uuid, "name": tenant.name}),
        ..Default::default()
    });
    
    Ok(Json(json!(tenant)))
}

// GET /api/system/tenants/{id}/stats
#[get("/system/tenants/<id>/stats")]
async fn tenant_stats(
    id: &str,
    _system_admin: SystemAdminHeaders,
    conn: DbConn,
) -> JsonResult {
    let (users, orgs, ciphers) = tokio::try_join!(
        User::count_in_tenant(id, &conn),
        Organization::count_in_tenant(id, &conn),
        Cipher::count_in_tenant(id, &conn),
    )?;
    
    Ok(Json(json!({
        "tenant_uuid": id,
        "users": users,
        "organizations": orgs,
        "vault_items": ciphers,
    })))
}
```

### 4.5 Tenant Admin Role

JWT claims mở rộng:

```rust
#[derive(Serialize, Deserialize)]
pub struct LoginJwtClaims {
    // Existing fields...
    pub sub: String,
    pub exp: i64,
    
    // New tenant fields
    pub tenant_uuid: Option<String>,
    pub is_tenant_admin: bool,
    pub is_system_admin: bool,
}
```

Tenant admin có quyền:
- Quản lý users trong tenant (không thấy users của tenant khác)
- Xem audit logs của tenant
- Cấu hình SSO/LDAP riêng (nếu `allow_sso_config=true`)

---

## 5. Migration Strategy

### Step-by-Step (Zero-Downtime)

```bash
# Step 1: Apply migration (tạo tenants table, thêm tenant_uuid với DEFAULT)
# Tất cả existing data được assign DEFAULT tenant UUID automatically

# Step 2: Verify migration
SELECT COUNT(*) FROM users WHERE tenant_uuid IS NULL;  -- Should be 0

# Step 3: Deploy new code với MULTI_TENANCY_ENABLED=false (v1.x behavior)
# Test thoroughly

# Step 4: Tạo additional tenants và assign users
POST /api/system/tenants  {"name": "Treasury", "slug": "treasury"}
# Manually assign users by email domain
UPDATE users SET tenant_uuid = 'treasury-uuid' WHERE email LIKE '%@treasury.bank.com';

# Step 5: Enable multi-tenancy
MULTI_TENANCY_ENABLED=true
TENANT_ROUTING=domain
# Hoặc phân theo subdomain

# Step 6: Verify isolation
# Test cross-tenant query không trả kết quả
```

---

## 6. Config Variables Mới

```bash
# Multi-tenancy
MULTI_TENANCY_ENABLED=false
TENANT_ROUTING=subdomain              # subdomain|path|domain
TENANT_DEFAULT_UUID=00000000-0000-0000-0000-000000000001

# System admin (siêu admin, cross-tenant)
SYSTEM_ADMIN_TOKEN=""                 # Argon2id hash, separate from ADMIN_TOKEN

# PostgreSQL RLS (chỉ khi dùng PostgreSQL)
TENANT_RLS_ENABLED=false             # Enable PostgreSQL RLS (requires PostgreSQL)
```

---

## 7. API Endpoints Mới

| Method | Path | Auth | Mô tả |
|--------|------|------|-------|
| POST | `/api/system/tenants` | System Admin | Create tenant |
| GET | `/api/system/tenants` | System Admin | List all tenants |
| PATCH | `/api/system/tenants/{id}` | System Admin | Update tenant |
| GET | `/api/system/tenants/{id}/stats` | System Admin | Usage stats |
| DELETE | `/api/system/tenants/{id}` | System Admin | Deactivate tenant |
| POST | `/api/system/tenants/{id}/admins` | System Admin | Assign tenant admin |
| DELETE | `/api/system/tenants/{id}/admins/{uid}` | System Admin | Remove tenant admin |

---

## 8. Rủi Ro & Giảm Thiểu

| Rủi ro | Mức độ | Giảm thiểu |
|--------|--------|-----------|
| Query không có tenant filter → data leak | Rất cao | Code review required; PostgreSQL RLS như lớp bảo vệ thứ 2 |
| Performance degradation với tenant WHERE clause | Trung bình | Index trên `tenant_uuid`; partition tables nếu cần |
| Migration fail trên DB lớn | Cao | Online migration với DEFAULT; test trên staging trước |
| Tenant routing mismatch | Trung bình | Fallback về default tenant; log warning |

---

## 9. Kế Hoạch Triển Khai

### Sprint 1–2: Tenant Data Model + Migration
- DB migration với DEFAULT tenant
- `src/db/models/tenant.rs`
- Backward-compatible DEFAULT mode

### Sprint 3–4: Application-Level Isolation
- `TenantContext` request guard
- Convert all model functions sang `_ctx` variants
- Start với high-risk models (User, Organization)

### Sprint 5: PostgreSQL RLS
- RLS policies
- Connection-level tenant setting
- Security testing

### Sprint 6: Tenant Routing
- Subdomain/path/domain routing
- Tenant slug cache

### Sprint 7–8: Tenant Admin Role
- JWT tenant claims
- Tenant admin permission set
- System admin API

### Sprint 9–10: Testing + Migration Tool
- End-to-end isolation tests
- Cross-tenant access tests
- Migration script + runbook

---

*Status: Draft | Ngày: 2026-04-12*
