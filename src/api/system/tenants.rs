use rocket::{serde::json::Json, Route};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    api::{EmptyResult, JsonResult},
    auth::SystemAdminHeaders,
    db::{models::Tenant, DbConn},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTenantData {
    name: String,
    slug: String,
    domain_restriction: Option<String>,
    max_users: Option<i32>,
    max_organizations: Option<i32>,
    max_vault_items: Option<i32>,
    max_storage_bytes: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTenantData {
    name: Option<String>,
    domain_restriction: Option<String>,
    is_active: Option<bool>,
    max_users: Option<i32>,
    max_organizations: Option<i32>,
    max_vault_items: Option<i32>,
    max_storage_bytes: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminData {
    user_uuid: String,
}

/// POST /api/system/tenants — Create a new tenant
#[post("/tenants", data = "<data>")]
pub async fn create_tenant(
    _auth: SystemAdminHeaders,
    data: Json<CreateTenantData>,
    mut conn: DbConn,
) -> JsonResult {
    let data = data.into_inner();

    if !Tenant::validate_slug(&data.slug) {
        err!("Invalid slug format: must be lowercase alphanumeric with hyphens");
    }

    // Check slug uniqueness
    if Tenant::find_by_slug(&data.slug, &mut conn).await.is_some() {
        err!("Tenant with this slug already exists");
    }

    let mut tenant = Tenant::new(data.name, data.slug);
    tenant.domain_restriction = data.domain_restriction;
    tenant.max_users = data.max_users;
    tenant.max_organizations = data.max_organizations;
    tenant.max_vault_items = data.max_vault_items;
    tenant.max_storage_bytes = data.max_storage_bytes;

    tenant.save(&mut conn).await?;

    // Invalidate slug cache so next lookup repopulates
    crate::tenant::invalidate_tenant_cache(&tenant.slug);

    Ok(Json(tenant.to_json()))
}

/// GET /api/system/tenants — List all tenants
#[get("/tenants")]
pub async fn list_tenants(_auth: SystemAdminHeaders, mut conn: DbConn) -> JsonResult {
    let tenants = Tenant::get_all(&mut conn).await;
    Ok(Json(serde_json::json!({
        "Data": tenants.iter().map(|t| t.to_json()).collect::<Vec<Value>>(),
        "Object": "list"
    })))
}

/// GET /api/system/tenants/<uuid> — Get single tenant
#[get("/tenants/<uuid>")]
pub async fn get_tenant(_auth: SystemAdminHeaders, uuid: &str, mut conn: DbConn) -> JsonResult {
    match Tenant::find_by_uuid(uuid, &mut conn).await {
        Some(t) => Ok(Json(t.to_json())),
        None => err_not_found!("Tenant not found"),
    }
}

/// PATCH /api/system/tenants/<uuid> — Update tenant fields
#[patch("/tenants/<uuid>", data = "<data>")]
pub async fn update_tenant(
    _auth: SystemAdminHeaders,
    uuid: &str,
    data: Json<UpdateTenantData>,
    mut conn: DbConn,
) -> JsonResult {
    let data = data.into_inner();
    let mut tenant = match Tenant::find_by_uuid(uuid, &mut conn).await {
        Some(t) => t,
        None => err_not_found!("Tenant not found"),
    };

    if let Some(v) = data.name { tenant.name = v; }
    if let Some(v) = data.domain_restriction { tenant.domain_restriction = Some(v); }
    if let Some(v) = data.is_active { tenant.is_active = v; }
    if let Some(v) = data.max_users { tenant.max_users = Some(v); }
    if let Some(v) = data.max_organizations { tenant.max_organizations = Some(v); }
    if let Some(v) = data.max_vault_items { tenant.max_vault_items = Some(v); }
    if let Some(v) = data.max_storage_bytes { tenant.max_storage_bytes = Some(v); }

    tenant.save(&mut conn).await?;
    Ok(Json(tenant.to_json()))
}

/// DELETE /api/system/tenants/<uuid> — Deactivate a tenant
#[delete("/tenants/<uuid>")]
pub async fn deactivate_tenant(
    _auth: SystemAdminHeaders,
    uuid: &str,
    mut conn: DbConn,
) -> EmptyResult {
    let mut tenant = match Tenant::find_by_uuid(uuid, &mut conn).await {
        Some(t) => t,
        None => err_not_found!("Tenant not found"),
    };
    tenant.is_active = false;
    tenant.save(&mut conn).await
}

/// GET /api/system/tenants/<uuid>/stats — Get user/org/cipher counts
#[get("/tenants/<uuid>/stats")]
pub async fn tenant_stats(_auth: SystemAdminHeaders, uuid: &str, mut conn: DbConn) -> JsonResult {
    if Tenant::find_by_uuid(uuid, &mut conn).await.is_none() {
        err_not_found!("Tenant not found");
    }
    let users = Tenant::count_users_in_tenant(uuid, &mut conn).await;
    let orgs = Tenant::count_orgs_in_tenant(uuid, &mut conn).await;
    let ciphers = Tenant::count_ciphers_in_tenant(uuid, &mut conn).await;

    Ok(Json(serde_json::json!({
        "TenantId": uuid,
        "UserCount": users,
        "OrganizationCount": orgs,
        "CipherCount": ciphers,
        "Object": "tenantStats"
    })))
}

/// POST /api/system/tenants/<uuid>/admins — Add a tenant admin
#[post("/tenants/<uuid>/admins", data = "<data>")]
pub async fn add_tenant_admin(
    _auth: SystemAdminHeaders,
    uuid: &str,
    data: Json<AdminData>,
    mut conn: DbConn,
) -> EmptyResult {
    if Tenant::find_by_uuid(uuid, &mut conn).await.is_none() {
        err_not_found!("Tenant not found");
    }
    crate::db::models::TenantAdmin::save(uuid.to_string(), data.user_uuid.clone(), &mut conn).await
}

/// DELETE /api/system/tenants/<uuid>/admins/<user_uuid> — Remove a tenant admin
#[delete("/tenants/<uuid>/admins/<user_uuid>")]
pub async fn remove_tenant_admin(
    _auth: SystemAdminHeaders,
    uuid: &str,
    user_uuid: &str,
    mut conn: DbConn,
) -> EmptyResult {
    crate::db::models::TenantAdmin::delete(uuid, user_uuid, &mut conn).await
}

pub fn routes() -> Vec<Route> {
    routes![
        create_tenant,
        list_tenants,
        get_tenant,
        update_tenant,
        deactivate_tenant,
        tenant_stats,
        add_tenant_admin,
        remove_tenant_admin,
    ]
}
