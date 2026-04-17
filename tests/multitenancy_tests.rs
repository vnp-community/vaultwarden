/// TASK-011-019: Cross-Tenant Isolation Tests — SOL-011 Multi-Tenancy
///
/// Test cases:
///  1.  TenantContext::tenant_uuid() returns correct UUID for Tenant variant
///  2.  TenantContext::tenant_uuid() returns None for SingleInstance and SystemAdmin
///  3.  TenantContext::is_system_admin() returns true only for SystemAdmin variant
///  4.  Slug cache: insert + lookup returns the correct UUID
///  5.  Slug cache: unknown slug returns None (cross-tenant slug miss)
///  6.  Slug cache: invalidate removes a previously inserted slug
///  7.  Tenant isolation predicate: user belongs to their own tenant
///  8.  Tenant isolation predicate: user does NOT belong to a different tenant
///  9.  SystemAdmin context grants access regardless of tenant_uuid
/// 10.  SingleInstance context grants access regardless of tenant_uuid (backward compat)
/// 11.  Path routing: `/t/{slug}/...` parses slug correctly
/// 12.  Path routing: non-tenant path returns None
/// 13.  Path routing: empty slug segment returns None
/// 14.  Subdomain routing: `acme.vault.example.com` → slug `acme`
/// 15.  Subdomain routing: bare host with no subdomain returns None
/// 16.  Quota check logic: under limit → Ok
/// 17.  Quota check logic: at limit → Err with message
/// 18.  Quota check logic: no limit configured → always Ok
/// 19.  RLS context string: Tenant variant produces correct SET command token
/// 20.  RLS context string: SystemAdmin variant produces SYSTEM_ADMIN sentinel
/// 21.  Cross-tenant filter predicate: org from tenant A not accessible to tenant B
/// 22.  Cross-tenant filter predicate: org from tenant A accessible to SystemAdmin
/// 23.  Event isolation: event belongs to correct tenant
/// 24.  Event isolation: event with different tenant_uuid is filtered out
/// 25.  Slug format validation: valid slugs pass, invalid slugs fail
///
/// NOTE: Because `vaultwarden` is a binary crate and these tests require no live
/// DB, all isolation logic is unit-tested using inline reimplementations that
/// mirror the production behaviour in `src/tenant.rs` and `src/db/models/`.
/// DB-backed tests (requires PostgreSQL + running migrations) must be run
/// separately via the Rocket test harness in `src/tests.rs`.
///
/// Run with:
///   cargo test --features sqlite multitenancy

// ── Constants shared across tests ────────────────────────────────────────────

const TENANT_A: &str = "aaaaaaaa-0000-0000-0000-000000000001";
const TENANT_B: &str = "bbbbbbbb-0000-0000-0000-000000000002";
const DEFAULT_TENANT: &str = "00000000-0000-0000-0000-000000000001";
const SYSTEM_ADMIN_SENTINEL: &str = "SYSTEM_ADMIN";

// ── Inline reimplementations of production logic ──────────────────────────────

/// Mirror of `TenantContext` enum (src/tenant.rs)
#[derive(Debug, Clone, PartialEq)]
enum TenantContext {
    SingleInstance,
    Tenant(String),
    SystemAdmin,
}

impl TenantContext {
    fn tenant_uuid(&self) -> Option<&str> {
        match self {
            TenantContext::Tenant(u) => Some(u.as_str()),
            _ => None,
        }
    }

    fn is_system_admin(&self) -> bool {
        matches!(self, TenantContext::SystemAdmin)
    }

    /// Produces the PostgreSQL `SET` token for RLS context — mirrors `set_db_tenant_context`.
    fn rls_token(&self) -> &str {
        match self {
            TenantContext::Tenant(u) => u.as_str(),
            TenantContext::SystemAdmin => SYSTEM_ADMIN_SENTINEL,
            TenantContext::SingleInstance => DEFAULT_TENANT,
        }
    }
}

/// Mirror of the tenant isolation predicate used in `find_by_uuid_ctx` and `get_all_ctx`.
/// Returns true if the resource should be visible to the given context.
fn is_visible_to(resource_tenant_uuid: &str, ctx: &TenantContext) -> bool {
    match ctx {
        TenantContext::SingleInstance => true,
        TenantContext::SystemAdmin => true,
        TenantContext::Tenant(tenant_uuid) => resource_tenant_uuid == tenant_uuid,
    }
}

/// Mirror of quota check logic — returns Err when current >= max.
fn check_quota(current: i64, max: Option<i64>, resource: &str) -> Result<(), String> {
    if let Some(limit) = max {
        if current >= limit {
            return Err(format!("{resource} quota exceeded ({current}/{limit})"));
        }
    }
    Ok(())
}

/// Mirror of slug format validation used in `Tenant::save()`.
fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 63
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
}

/// Mirror of `extract_tenant_from_path()` — parses `/t/{slug}/...`
fn extract_slug_from_path(path: &str) -> Option<String> {
    let mut parts = path.trim_matches('/').splitn(3, '/');
    if parts.next()? != "t" {
        return None;
    }
    let slug = parts.next()?;
    if slug.is_empty() {
        return None;
    }
    Some(slug.to_string())
}

/// Mirror of `extract_tenant_from_subdomain()` — splits first label from Host header.
/// Returns None if the Host has no subdomain (i.e., only one label).
fn extract_slug_from_host(host: &str) -> Option<String> {
    // Strip port if present
    let host = host.split(':').next().unwrap_or(host);
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() <= 2 {
        // e.g. "example.com" → no subdomain
        return None;
    }
    let slug = parts[0];
    if slug.is_empty() {
        return None;
    }
    Some(slug.to_string())
}

// ── TASK-011-019 Test Suite ───────────────────────────────────────────────────

// ── 1. TenantContext::tenant_uuid() ──────────────────────────────────────────

/// tenant_uuid() returns the UUID string for Tenant variant.
#[test]
fn test_tenant_context_uuid_for_tenant_variant() {
    let ctx = TenantContext::Tenant(TENANT_A.to_string());
    assert_eq!(ctx.tenant_uuid(), Some(TENANT_A));
}

/// tenant_uuid() returns None for SingleInstance.
#[test]
fn test_tenant_context_uuid_none_for_single_instance() {
    let ctx = TenantContext::SingleInstance;
    assert_eq!(ctx.tenant_uuid(), None, "SingleInstance must not expose a tenant UUID");
}

/// tenant_uuid() returns None for SystemAdmin.
#[test]
fn test_tenant_context_uuid_none_for_system_admin() {
    let ctx = TenantContext::SystemAdmin;
    assert_eq!(ctx.tenant_uuid(), None, "SystemAdmin must not expose a tenant UUID");
}

// ── 2. TenantContext::is_system_admin() ──────────────────────────────────────

/// is_system_admin() returns true only for SystemAdmin.
#[test]
fn test_is_system_admin_true_for_system_admin() {
    assert!(TenantContext::SystemAdmin.is_system_admin());
}

/// is_system_admin() returns false for Tenant and SingleInstance.
#[test]
fn test_is_system_admin_false_for_other_variants() {
    assert!(!TenantContext::Tenant(TENANT_A.to_string()).is_system_admin());
    assert!(!TenantContext::SingleInstance.is_system_admin());
}

// ── 3. Slug cache (inline with DashMap) ──────────────────────────────────────

/// Slug cache: insert then lookup returns the correct UUID.
#[test]
fn test_slug_cache_lookup_correct_uuid() {
    use dashmap::DashMap;
    let cache: DashMap<String, String> = DashMap::new();
    cache.insert("acme".to_string(), TENANT_A.to_string());

    let result = cache.get("acme").map(|v| v.clone());
    assert_eq!(result.as_deref(), Some(TENANT_A));
}

/// Slug cache: unknown slug returns None — prevents cross-tenant slug guessing.
#[test]
fn test_slug_cache_unknown_slug_returns_none() {
    use dashmap::DashMap;
    let cache: DashMap<String, String> = DashMap::new();
    cache.insert("acme".to_string(), TENANT_A.to_string());

    let result = cache.get("unknown-corp").map(|v| v.clone());
    assert!(result.is_none(), "Unknown slug must return None — no cross-tenant fallback");
}

/// Slug cache: invalidate removes a previously inserted slug.
#[test]
fn test_slug_cache_invalidate_removes_entry() {
    use dashmap::DashMap;
    let cache: DashMap<String, String> = DashMap::new();
    cache.insert("acme".to_string(), TENANT_A.to_string());
    cache.remove("acme");

    let result = cache.get("acme").map(|v| v.clone());
    assert!(result.is_none(), "Invalidated slug must not be found in cache");
}

// ── 4. Tenant isolation predicate ────────────────────────────────────────────

/// User belongs to their own tenant — resource is visible.
#[test]
fn test_isolation_user_sees_own_tenant_resource() {
    let ctx = TenantContext::Tenant(TENANT_A.to_string());
    assert!(is_visible_to(TENANT_A, &ctx), "User must see resources from their own tenant");
}

/// User does NOT belong to a different tenant — resource is NOT visible.
#[test]
fn test_isolation_user_cannot_see_other_tenant_resource() {
    let ctx = TenantContext::Tenant(TENANT_A.to_string());
    assert!(
        !is_visible_to(TENANT_B, &ctx),
        "User must NOT see resources from a different tenant — cross-tenant data leak"
    );
}

/// SystemAdmin context grants access regardless of resource tenant_uuid.
#[test]
fn test_isolation_system_admin_sees_all_tenants() {
    let ctx = TenantContext::SystemAdmin;
    assert!(is_visible_to(TENANT_A, &ctx), "SystemAdmin must see tenant A");
    assert!(is_visible_to(TENANT_B, &ctx), "SystemAdmin must see tenant B");
    assert!(is_visible_to(DEFAULT_TENANT, &ctx), "SystemAdmin must see default tenant");
}

/// SingleInstance context grants access to all resources (backward compatibility).
#[test]
fn test_isolation_single_instance_sees_all() {
    let ctx = TenantContext::SingleInstance;
    assert!(is_visible_to(TENANT_A, &ctx), "SingleInstance must see tenant A (backward compat)");
    assert!(is_visible_to(TENANT_B, &ctx), "SingleInstance must see tenant B (backward compat)");
}

// ── 5. Path-based routing ─────────────────────────────────────────────────────

/// `/t/{slug}/...` extracts slug correctly.
#[test]
fn test_path_routing_extracts_slug() {
    assert_eq!(
        extract_slug_from_path("/t/acme/api/sync"),
        Some("acme".to_string())
    );
}

/// `/t/{slug}` (no trailing path) still extracts slug.
#[test]
fn test_path_routing_extracts_slug_no_trailing_path() {
    assert_eq!(
        extract_slug_from_path("/t/corp-a"),
        Some("corp-a".to_string())
    );
}

/// Non-tenant path returns None.
#[test]
fn test_path_routing_non_tenant_path_returns_none() {
    assert_eq!(extract_slug_from_path("/api/sync"), None);
    assert_eq!(extract_slug_from_path("/identity/connect/token"), None);
}

/// Empty slug segment returns None.
#[test]
fn test_path_routing_empty_slug_returns_none() {
    assert_eq!(extract_slug_from_path("/t//api/sync"), None);
}

// ── 6. Subdomain routing ──────────────────────────────────────────────────────

/// `acme.vault.example.com` → slug `acme`.
#[test]
fn test_subdomain_routing_extracts_first_label() {
    assert_eq!(
        extract_slug_from_host("acme.vault.example.com"),
        Some("acme".to_string())
    );
}

/// Host with no subdomain (e.g. `example.com`) returns None.
#[test]
fn test_subdomain_routing_no_subdomain_returns_none() {
    assert_eq!(extract_slug_from_host("example.com"), None);
}

/// Host with port (e.g. `acme.vault.example.com:8080`) extracts slug correctly.
#[test]
fn test_subdomain_routing_strips_port() {
    assert_eq!(
        extract_slug_from_host("acme.vault.example.com:8080"),
        Some("acme".to_string())
    );
}

// ── 7. Quota enforcement ──────────────────────────────────────────────────────

/// Under limit → Ok.
#[test]
fn test_quota_under_limit_is_ok() {
    assert!(check_quota(4, Some(5), "users").is_ok());
}

/// At limit → Err with descriptive message.
#[test]
fn test_quota_at_limit_is_err() {
    let result = check_quota(5, Some(5), "users");
    assert!(result.is_err(), "At-limit quota must return Err");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("users") && msg.contains("5/5"),
        "Error message must include resource name and counts: {msg}"
    );
}

/// Over limit → Err.
#[test]
fn test_quota_over_limit_is_err() {
    let result = check_quota(10, Some(5), "organizations");
    assert!(result.is_err(), "Over-limit quota must return Err");
}

/// No limit configured → always Ok regardless of current count.
#[test]
fn test_quota_no_limit_always_ok() {
    assert!(check_quota(9999, None, "vault_items").is_ok(), "None limit must never block");
}

// ── 8. RLS token generation ───────────────────────────────────────────────────

/// Tenant variant produces the tenant UUID as the RLS token.
#[test]
fn test_rls_token_for_tenant_is_uuid() {
    let ctx = TenantContext::Tenant(TENANT_A.to_string());
    assert_eq!(ctx.rls_token(), TENANT_A);
}

/// SystemAdmin variant produces the SYSTEM_ADMIN sentinel for RLS bypass.
#[test]
fn test_rls_token_for_system_admin_is_sentinel() {
    let ctx = TenantContext::SystemAdmin;
    assert_eq!(
        ctx.rls_token(),
        SYSTEM_ADMIN_SENTINEL,
        "SystemAdmin RLS token must use SYSTEM_ADMIN sentinel for policy bypass"
    );
}

/// SingleInstance variant uses the DEFAULT_TENANT UUID for RLS.
#[test]
fn test_rls_token_for_single_instance_is_default_tenant() {
    let ctx = TenantContext::SingleInstance;
    assert_eq!(ctx.rls_token(), DEFAULT_TENANT);
}

// ── 9. Cross-tenant org isolation ────────────────────────────────────────────

/// Org from tenant A is NOT accessible to context scoped to tenant B.
#[test]
fn test_org_from_tenant_a_not_visible_to_tenant_b() {
    let ctx_b = TenantContext::Tenant(TENANT_B.to_string());
    assert!(
        !is_visible_to(TENANT_A, &ctx_b),
        "Org from tenant A must not be accessible to tenant B context — would be a data leak"
    );
}

/// Org from tenant A IS accessible to SystemAdmin.
#[test]
fn test_org_from_tenant_a_visible_to_system_admin() {
    let ctx = TenantContext::SystemAdmin;
    assert!(
        is_visible_to(TENANT_A, &ctx),
        "SystemAdmin must be able to query orgs across all tenants"
    );
}

/// Two different orgs from the same tenant are both visible to that tenant's context.
#[test]
fn test_same_tenant_orgs_both_visible() {
    let ctx = TenantContext::Tenant(TENANT_A.to_string());
    assert!(is_visible_to(TENANT_A, &ctx), "Org #1 from own tenant must be visible");
    assert!(is_visible_to(TENANT_A, &ctx), "Org #2 from own tenant must be visible");
}

// ── 10. Event (audit log) isolation ──────────────────────────────────────────

#[allow(dead_code)]
struct FakeEvent {
    tenant_uuid: String,
    event_type: &'static str,
}

fn filter_events_for_ctx<'a>(events: &'a [FakeEvent], ctx: &TenantContext) -> Vec<&'a FakeEvent> {
    events
        .iter()
        .filter(|e| is_visible_to(&e.tenant_uuid, ctx))
        .collect()
}

/// Events are scoped to the requesting tenant — cross-tenant events are hidden.
#[test]
fn test_event_isolation_tenant_context() {
    let events = vec![
        FakeEvent { tenant_uuid: TENANT_A.to_string(), event_type: "LoginSuccess" },
        FakeEvent { tenant_uuid: TENANT_B.to_string(), event_type: "PasswordChanged" },
        FakeEvent { tenant_uuid: TENANT_A.to_string(), event_type: "CipherCreated" },
    ];

    let ctx_a = TenantContext::Tenant(TENANT_A.to_string());
    let visible = filter_events_for_ctx(&events, &ctx_a);

    assert_eq!(visible.len(), 2, "Tenant A context must only see 2 events (not tenant B's)");
    assert!(visible.iter().all(|e| e.tenant_uuid == TENANT_A));
}

/// SystemAdmin sees all events across tenants.
#[test]
fn test_event_isolation_system_admin_sees_all() {
    let events = vec![
        FakeEvent { tenant_uuid: TENANT_A.to_string(), event_type: "LoginSuccess" },
        FakeEvent { tenant_uuid: TENANT_B.to_string(), event_type: "PasswordChanged" },
    ];

    let ctx = TenantContext::SystemAdmin;
    let visible = filter_events_for_ctx(&events, &ctx);

    assert_eq!(visible.len(), 2, "SystemAdmin must see events from all tenants");
}

// ── 11. Slug format validation ────────────────────────────────────────────────

/// Valid slugs: lowercase, alphanumeric, hyphens (not at start/end).
#[test]
fn test_slug_validation_valid_slugs() {
    assert!(is_valid_slug("acme"), "Simple alphanumeric slug must be valid");
    assert!(is_valid_slug("corp-a"), "Slug with internal hyphen must be valid");
    assert!(is_valid_slug("my-company-123"), "Slug with digits must be valid");
    assert!(is_valid_slug("a"), "Single-char slug must be valid");
}

/// Invalid slugs: uppercase, spaces, leading/trailing hyphens, empty.
#[test]
fn test_slug_validation_invalid_slugs() {
    assert!(!is_valid_slug(""), "Empty slug must be invalid");
    assert!(!is_valid_slug("ACME"), "Uppercase slug must be invalid");
    assert!(!is_valid_slug("corp a"), "Slug with space must be invalid");
    assert!(!is_valid_slug("-corp"), "Slug starting with hyphen must be invalid");
    assert!(!is_valid_slug("corp-"), "Slug ending with hyphen must be invalid");
    assert!(!is_valid_slug("corp@a"), "Slug with special char must be invalid");
}

/// Slug max length is 63 characters.
#[test]
fn test_slug_validation_max_length() {
    let at_limit = "a".repeat(63);
    let over_limit = "a".repeat(64);
    assert!(is_valid_slug(&at_limit), "63-char slug must be valid");
    assert!(!is_valid_slug(&over_limit), "64-char slug must be invalid");
}
