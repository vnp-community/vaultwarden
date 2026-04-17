// SOL-011: Multi-Tenancy — TenantContext, routing, quotas, RLS

use std::sync::LazyLock;
use dashmap::DashMap;
use rocket::request::{FromRequest, Outcome, Request};
use crate::db::DbConn;

/// The resolved tenant scope for a given request
#[derive(Debug, Clone)]
pub enum TenantContext {
    SingleInstance,
    Tenant(String), // tenant UUID
    SystemAdmin,
}

impl TenantContext {
    pub fn tenant_uuid(&self) -> Option<&str> {
        match self {
            TenantContext::Tenant(u) => Some(u.as_str()),
            _ => None,
        }
    }

    pub fn is_system_admin(&self) -> bool {
        matches!(self, TenantContext::SystemAdmin)
    }
}

/// TASK-011-006: slug → uuid cache populated at startup
pub static TENANT_SLUG_CACHE: LazyLock<DashMap<String, String>> = LazyLock::new(DashMap::new);

/// Populate slug cache from DB — call at startup
pub async fn populate_tenant_slug_cache(conn: &mut DbConn) {
    use crate::db::models::Tenant;
    for tenant in Tenant::get_all(conn).await {
        TENANT_SLUG_CACHE.insert(tenant.slug.clone(), tenant.uuid.clone());
    }
    info!("TenantSlugCache: loaded {} tenants", TENANT_SLUG_CACHE.len());
}

/// Invalidate one entry (call after create/update tenant)
pub fn invalidate_tenant_cache(slug: &str) {
    TENANT_SLUG_CACHE.remove(slug);
}

// ──────────────────────────────────────────────────────────────────────────────
// TASK-011-013: Subdomain routing
// ──────────────────────────────────────────────────────────────────────────────

/// Extract tenant UUID from Host header subdomain (e.g. `acme.vault.example.com`)
pub fn extract_tenant_from_subdomain(request: &Request<'_>) -> Option<String> {
    let host = request.headers().get_one("Host")?;
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let slug = parts[0];
    TENANT_SLUG_CACHE.get(slug).map(|r| r.clone())
}

// ──────────────────────────────────────────────────────────────────────────────
// TASK-011-014: Path-based routing
// ──────────────────────────────────────────────────────────────────────────────

/// Extract tenant UUID from request path (e.g. `/t/acme/api/...`)
pub fn extract_tenant_from_path(request: &Request<'_>) -> Option<String> {
    // url_decode_lossy() returns Cow<str>; bind to a named variable so the
    // temporary lives for the entire scope of this function.
    let uri_path = request.uri().path();
    let decoded = uri_path.url_decode_lossy();
    let path: &str = decoded.as_ref();
    // Expect pattern `/t/{slug}/...`
    let rest = path.strip_prefix("/t/")?;
    let slug = rest.split('/').next()?;
    TENANT_SLUG_CACHE.get(slug).map(|r| r.clone())
}

// ──────────────────────────────────────────────────────────────────────────────
// TASK-011-015: Domain-based routing (email domain matching)
// ──────────────────────────────────────────────────────────────────────────────

/// Match a user's email domain against tenants with domain_restriction set.
/// Returns tenant UUID if found.
pub async fn extract_tenant_from_email_domain(email: &str, conn: &mut DbConn) -> Option<String> {
    use crate::db::models::Tenant;
    let domain = email.split('@').nth(1)?;
    for tenant in Tenant::get_all(conn).await {
        if let Some(ref restriction) = tenant.domain_restriction {
            if domain.ends_with(restriction.trim_start_matches('*').trim_start_matches('.')) {
                return Some(tenant.uuid);
            }
        }
    }
    None
}

// ──────────────────────────────────────────────────────────────────────────────
// TASK-011-005: TenantContext Rocket FromRequest guard
// ──────────────────────────────────────────────────────────────────────────────

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TenantContext {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if !crate::CONFIG.multi_tenancy_enabled() {
            return Outcome::Success(TenantContext::SingleInstance);
        }

        // Check for X-System-Admin-Token header for system admin context
        if let Some(token) = request.headers().get_one("X-System-Admin-Token") {
            use ring::digest;
            use data_encoding::HEXLOWER;
            let stored = crate::CONFIG.system_admin_token();
            if !stored.is_empty() {
                // Constant-time compare via hash
                let provided_hash = HEXLOWER.encode(digest::digest(&digest::SHA256, token.as_bytes()).as_ref());
                let stored_hash = HEXLOWER.encode(digest::digest(&digest::SHA256, stored.as_bytes()).as_ref());
                if provided_hash == stored_hash {
                    return Outcome::Success(TenantContext::SystemAdmin);
                }
            }
        }

        let routing = crate::CONFIG.tenant_routing();

        // Try X-Tenant-Id header first (internal/proxy override)
        if let Some(tenant_uuid) = request.headers().get_one("X-Tenant-Id") {
            return Outcome::Success(TenantContext::Tenant(tenant_uuid.to_string()));
        }

        // Routing modes
        let maybe_uuid = match routing.as_str() {
            "subdomain" => extract_tenant_from_subdomain(request),
            "path" => extract_tenant_from_path(request),
            _ => extract_tenant_from_subdomain(request)
                .or_else(|| extract_tenant_from_path(request)),
        };

        if let Some(uuid) = maybe_uuid {
            return Outcome::Success(TenantContext::Tenant(uuid));
        }

        // Fallback: use default tenant
        let default = crate::CONFIG.tenant_default_uuid();
        if !default.is_empty() {
            Outcome::Success(TenantContext::Tenant(default))
        } else {
            Outcome::Success(TenantContext::SingleInstance)
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// TASK-011-012: PostgreSQL RLS context setter
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "postgresql")]
pub async fn set_db_tenant_context(conn: &mut DbConn, tenant_uuid: &str) -> Result<(), String> {
    if !crate::CONFIG.tenant_rls_enabled() {
        return Ok(());
    }
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::Text;

    db_run! { conn: {
        sql_query("SELECT set_current_tenant($1)")
            .bind::<Text, _>(tenant_uuid)
            .execute(conn)
            .map(|_| ())
            .map_err(|e| format!("RLS set_current_tenant failed: {e}"))
    }}
}

#[cfg(not(feature = "postgresql"))]
pub async fn set_db_tenant_context(_conn: &mut DbConn, _tenant_uuid: &str) -> Result<(), String> {
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// TASK-011-020: Resource quota enforcement helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Check if a tenant has capacity to add another user. Returns Err with message if quota exceeded.
pub async fn check_user_quota(tenant_uuid: &str, conn: &mut DbConn) -> Result<(), String> {
    use crate::db::models::Tenant;
    let tenant = Tenant::find_by_uuid(tenant_uuid, conn).await
        .ok_or_else(|| "Tenant not found".to_string())?;
    if let Some(max) = tenant.max_users {
        let current = Tenant::count_users_in_tenant(tenant_uuid, conn).await;
        if current >= max as i64 {
            return Err(format!("Tenant user quota exceeded ({current}/{max})"));
        }
    }
    Ok(())
}

/// Check if a tenant has capacity to add another organization.
pub async fn check_org_quota(tenant_uuid: &str, conn: &mut DbConn) -> Result<(), String> {
    use crate::db::models::Tenant;
    let tenant = Tenant::find_by_uuid(tenant_uuid, conn).await
        .ok_or_else(|| "Tenant not found".to_string())?;
    if let Some(max) = tenant.max_organizations {
        let current = Tenant::count_orgs_in_tenant(tenant_uuid, conn).await;
        if current >= max as i64 {
            return Err(format!("Tenant organization quota exceeded ({current}/{max})"));
        }
    }
    Ok(())
}

/// Check if a tenant has capacity to add another vault item.
pub async fn check_vault_item_quota(tenant_uuid: &str, conn: &mut DbConn) -> Result<(), String> {
    use crate::db::models::Tenant;
    let tenant = Tenant::find_by_uuid(tenant_uuid, conn).await
        .ok_or_else(|| "Tenant not found".to_string())?;
    if let Some(max) = tenant.max_vault_items {
        let current = Tenant::count_ciphers_in_tenant(tenant_uuid, conn).await;
        if current >= max as i64 {
            return Err(format!("Tenant vault item quota exceeded ({current}/{max})"));
        }
    }
    Ok(())
}
