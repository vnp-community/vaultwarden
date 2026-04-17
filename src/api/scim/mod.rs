/// TASK-003-009: SCIM Bearer token authentication guard
/// TASK-003-010: SCIM Users endpoints (list, get, create)
/// TASK-003-011: SCIM Users PATCH endpoint
/// TASK-003-012: SCIM Groups endpoints
/// TASK-003-013: SCIM ServiceProviderConfig / Schemas / ResourceTypes
/// TASK-003-014: Route mounting is done in main.rs

use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    serde::json::Json,
    Route,
};
use sha2::{Digest, Sha256};

use crate::{
    api::JsonResult,
    db::{
        models::{Collection, CollectionId, CollectionUser, Membership, MembershipStatus, MembershipType, Organization, OrganizationId, User, UserId},
        schema::scim_tokens,
        DbConn,
    },
    db_run,
    error::Error,
    CONFIG,
};

// ─────────────────────────────────────────────────────────────────────────────
// TASK-003-009: SCIM Bearer token guard
// ─────────────────────────────────────────────────────────────────────────────

/// Validated SCIM auth token — carries the org UUID the token belongs to.
pub struct ScimAuth {
    pub org_uuid: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ScimAuth {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if !CONFIG.scim_enabled() {
            return Outcome::Error((Status::ServiceUnavailable, "SCIM is disabled"));
        }

        let auth_header = match req.headers().get_one("Authorization") {
            Some(h) => h,
            None => return Outcome::Error((Status::Unauthorized, "Missing Authorization header")),
        };

        let token = match auth_header.strip_prefix("Bearer ") {
            Some(t) => t,
            None => return Outcome::Error((Status::Unauthorized, "Expected Bearer token")),
        };

        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = hasher.finalize().to_vec();

        let conn = match req.guard::<DbConn>().await.succeeded() {
            Some(c) => c,
            None => return Outcome::Error((Status::ServiceUnavailable, "DB unavailable")),
        };

        match find_scim_token_org(&token_hash, &conn).await {
            Some(org_uuid) => Outcome::Success(ScimAuth { org_uuid }),
            None => Outcome::Error((Status::Unauthorized, "Invalid SCIM token")),
        }
    }
}

async fn find_scim_token_org(token_hash: &[u8], conn: &DbConn) -> Option<String> {
    use diesel::prelude::*;

    db_run! { conn: {
        scim_tokens::table
            .filter(scim_tokens::token_hash.eq(token_hash))
            .select(scim_tokens::org_uuid)
            .first::<String>(conn)
            .ok()
    }}
}

// ─────────────────────────────────────────────────────────────────────────────
// SCIM response helpers
// ─────────────────────────────────────────────────────────────────────────────

fn user_to_scim(user: &User) -> serde_json::Value {
    let base_url = CONFIG.domain();
    json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
        "id": user.uuid,
        "externalId": user.uuid,
        "userName": user.email,
        "displayName": user.name,
        "name": { "formatted": user.name },
        "emails": [{ "value": user.email, "primary": true, "type": "work" }],
        "active": user.enabled,
        "meta": {
            "resourceType": "User",
            "location": format!("{base_url}/scim/v2/Users/{}", user.uuid)
        }
    })
}

fn coll_to_scim(coll: &Collection) -> serde_json::Value {
    let base_url = CONFIG.domain();
    json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Group"],
        "id": coll.uuid,
        "displayName": coll.name,
        "meta": {
            "resourceType": "Group",
            "location": format!("{base_url}/scim/v2/Groups/{}", coll.uuid)
        }
    })
}

fn list_response(resources: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "schemas": ["urn:ietf:params:scim:api:messages:2.0:ListResponse"],
        "totalResults": resources.len(),
        "startIndex": 1,
        "itemsPerPage": resources.len(),
        "Resources": resources
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-003-010: SCIM Users — list / get / create
// ─────────────────────────────────────────────────────────────────────────────

#[get("/v2/Users")]
pub async fn get_users(auth: ScimAuth, conn: DbConn) -> JsonResult {
    let org_id = OrganizationId::from(auth.org_uuid);
    let memberships = Membership::find_by_org(&org_id, &conn).await;

    let mut users: Vec<serde_json::Value> = Vec::new();
    for m in memberships {
        if let Some(user) = User::find_by_uuid(&m.user_uuid, &conn).await {
            users.push(user_to_scim(&user));
        }
    }

    Ok(Json(list_response(users)))
}

#[get("/v2/Users/<user_id>")]
pub async fn get_user(user_id: &str, _auth: ScimAuth, conn: DbConn) -> JsonResult {
    let uuid = UserId::from(user_id.to_string());
    let user = User::find_by_uuid(&uuid, &conn).await
        .ok_or_else(|| Error::new("NotFound", format!("User {user_id} not found")))?;
    Ok(Json(user_to_scim(&user)))
}

#[post("/v2/Users", data = "<data>")]
pub async fn create_user(data: Json<serde_json::Value>, auth: ScimAuth, conn: DbConn) -> JsonResult {
    // Extract email from userName or emails[0].value
    let email = data["userName"].as_str()
        .or_else(|| {
            data["emails"].as_array()
                .and_then(|arr| arr.first())
                .and_then(|e| e["value"].as_str())
        })
        .map(|s| s.to_lowercase())
        .ok_or_else(|| Error::new("BadRequest", "userName or emails.value required"))?;

    // Idempotent: return existing user if already exists
    if let Some(existing) = User::find_by_mail(&email, &conn).await {
        return Ok(Json(user_to_scim(&existing)));
    }

    let name = data["displayName"].as_str()
        .or_else(|| data["name"]["formatted"].as_str())
        .unwrap_or(&email)
        .to_string();

    let mut user = User::new(&email, Some(name));
    user.set_password(&crate::util::get_uuid(), None, true, None);
    user.provisioning_source = Some("scim".to_string());
    user.save(&conn).await?;

    // Add to the org associated with this SCIM token
    let org_id = OrganizationId::from(auth.org_uuid);
    if Organization::find_by_uuid(&org_id, &conn).await.is_some() {
        let user_id = UserId::from(user.uuid.to_string());
        let mut membership = Membership::new(user_id, org_id, None);
        membership.atype = MembershipType::User as i32;
        membership.status = MembershipStatus::Invited as i32;
        let _r = membership.save(&conn).await;
    }

    info!("[SCIM] Provisioned user: {}", user.email);
    Ok(Json(user_to_scim(&user)))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-003-011: SCIM Users PATCH
// ─────────────────────────────────────────────────────────────────────────────

#[patch("/v2/Users/<user_id>", data = "<data>")]
pub async fn patch_user(user_id: &str, data: Json<serde_json::Value>, _auth: ScimAuth, conn: DbConn) -> JsonResult {
    let uuid = UserId::from(user_id.to_string());
    let mut user = User::find_by_uuid(&uuid, &conn).await
        .ok_or_else(|| Error::new("NotFound", format!("User {user_id} not found")))?;

    let ops = data["Operations"].as_array()
        .ok_or_else(|| Error::new("BadRequest", "Operations array required for PATCH"))?
        .clone();

    for op in ops {
        let op_type = op["op"].as_str().unwrap_or("").to_lowercase();
        let path = op["path"].as_str().unwrap_or("").to_lowercase();
        let value = &op["value"];

        match (op_type.as_str(), path.as_str()) {
            // Deactivate/activate user
            ("replace", "active") | ("replace", "") => {
                let active_flag = value["active"].as_bool().or_else(|| value.as_bool());
                if let Some(active) = active_flag {
                    if !active && user.enabled {
                        info!("[SCIM] Deactivating user {}: revoking sessions", user.email);
                        if let Err(e) = crate::db::models::Device::delete_all_by_user(&user.uuid, &conn).await {
                            warn!("[SCIM] Failed to revoke sessions: {e:?}");
                        }
                        user.enabled = false;
                        user.save(&conn).await?;
                    } else if active && !user.enabled {
                        user.enabled = true;
                        user.save(&conn).await?;
                    }
                }
            }
            ("replace", "displayname") | ("replace", "name.formatted") => {
                if let Some(name) = value.as_str() {
                    user.name = name.to_string();
                    user.save(&conn).await?;
                }
            }
            _ => {
                debug!("[SCIM] PATCH op ignored: op={op_type} path={path}");
            }
        }
    }

    Ok(Json(user_to_scim(&user)))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-003-012: SCIM Groups
// ─────────────────────────────────────────────────────────────────────────────

#[get("/v2/Groups")]
pub async fn get_groups(auth: ScimAuth, conn: DbConn) -> JsonResult {
    let org_id = OrganizationId::from(auth.org_uuid);
    let collections = Collection::find_by_organization(&org_id, &conn).await;
    let resources: Vec<serde_json::Value> = collections.iter().map(coll_to_scim).collect();
    Ok(Json(list_response(resources)))
}

#[get("/v2/Groups/<group_id>")]
pub async fn get_group(group_id: &str, _auth: ScimAuth, conn: DbConn) -> JsonResult {
    let coll_id = CollectionId::from(group_id.to_string());
    let coll = Collection::find_by_uuid(&coll_id, &conn).await
        .ok_or_else(|| Error::new("NotFound", format!("Group {group_id} not found")))?;
    Ok(Json(coll_to_scim(&coll)))
}

#[post("/v2/Groups", data = "<data>")]
pub async fn create_group(data: Json<serde_json::Value>, auth: ScimAuth, conn: DbConn) -> JsonResult {
    let name = data["displayName"].as_str()
        .ok_or_else(|| Error::new("BadRequest", "displayName required"))?
        .to_string();
    let external_id = data["externalId"].as_str().map(|s| s.to_string());
    let org_id = OrganizationId::from(auth.org_uuid);

    let collection = Collection::new(org_id, name, external_id);
    collection.save(&conn).await?;

    info!("[SCIM] Created collection: {}", collection.name);
    Ok(Json(coll_to_scim(&collection)))
}

#[patch("/v2/Groups/<group_id>", data = "<data>")]
pub async fn patch_group(group_id: &str, data: Json<serde_json::Value>, _auth: ScimAuth, conn: DbConn) -> JsonResult {
    let coll_id = CollectionId::from(group_id.to_string());
    let mut coll = Collection::find_by_uuid(&coll_id, &conn).await
        .ok_or_else(|| Error::new("NotFound", format!("Group {group_id} not found")))?;

    let ops = data["Operations"].as_array()
        .ok_or_else(|| Error::new("BadRequest", "Operations array required"))?
        .clone();

    for op in ops {
        let op_type = op["op"].as_str().unwrap_or("").to_lowercase();
        let path = op["path"].as_str().unwrap_or("").to_lowercase();
        let value = &op["value"];

        match (op_type.as_str(), path.as_str()) {
            ("replace", "displayname") => {
                if let Some(new_name) = value.as_str() {
                    coll.name = new_name.to_string();
                    coll.save(&conn).await?;
                }
            }
            ("add", "members") | ("replace", "members") => {
                if let Some(members) = value.as_array() {
                    for member in members {
                        let user_id_str = member["value"].as_str().unwrap_or_default();
                        let user_id = UserId::from(user_id_str.to_string());
                        if User::find_by_uuid(&user_id, &conn).await.is_some() {
                            let existing = CollectionUser::find_by_collection_and_user(&coll.uuid, &user_id, &conn).await;
                            if existing.is_none() {
                                let _r = CollectionUser::save(&user_id, &coll.uuid, false, false, false, &conn).await;
                            }
                        }
                    }
                }
            }
            ("remove", "members") => {
                if let Some(members) = value.as_array() {
                    for member in members {
                        let user_id_str = member["value"].as_str().unwrap_or_default();
                        let user_id = UserId::from(user_id_str.to_string());
                        if let Some(cu) = CollectionUser::find_by_collection_and_user(&coll.uuid, &user_id, &conn).await {
                            let _r = cu.delete(&conn).await;
                        }
                    }
                }
            }
            _ => debug!("[SCIM] Group PATCH op ignored: {op_type} path={path}"),
        }
    }

    Ok(Json(coll_to_scim(&coll)))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-003-013: SCIM ServiceProviderConfig / Schemas / ResourceTypes
// ─────────────────────────────────────────────────────────────────────────────

#[get("/v2/ServiceProviderConfig")]
pub async fn service_provider_config(_auth: ScimAuth) -> JsonResult {
    Ok(Json(json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig"],
        "patch": { "supported": true },
        "bulk": { "supported": false, "maxOperations": 0, "maxPayloadSize": 0 },
        "filter": { "supported": true, "maxResults": 200 },
        "changePassword": { "supported": false },
        "sort": { "supported": false },
        "etag": { "supported": false },
        "authenticationSchemes": [{
            "name": "OAuth Bearer Token",
            "description": "Authentication scheme using OAuth Bearer Token Standard (RFC 6750)",
            "specUri": "http://www.rfc-editor.org/info/rfc6750",
            "type": "oauthbearertoken",
            "primary": true
        }],
        "meta": {
            "resourceType": "ServiceProviderConfig",
            "location": format!("{}/scim/v2/ServiceProviderConfig", CONFIG.domain())
        }
    })))
}

#[get("/v2/Schemas")]
pub async fn get_schemas(_auth: ScimAuth) -> JsonResult {
    Ok(Json(json!({
        "schemas": ["urn:ietf:params:scim:api:messages:2.0:ListResponse"],
        "totalResults": 2,
        "Resources": [
            {
                "id": "urn:ietf:params:scim:schemas:core:2.0:User",
                "name": "User",
                "description": "User Account",
                "attributes": [
                    { "name": "userName", "type": "string", "required": true },
                    { "name": "displayName", "type": "string" },
                    { "name": "emails", "type": "complex", "multiValued": true },
                    { "name": "active", "type": "boolean" }
                ]
            },
            {
                "id": "urn:ietf:params:scim:schemas:core:2.0:Group",
                "name": "Group",
                "description": "Group / Collection",
                "attributes": [
                    { "name": "displayName", "type": "string", "required": true },
                    { "name": "members", "type": "complex", "multiValued": true }
                ]
            }
        ]
    })))
}

#[get("/v2/ResourceTypes")]
pub async fn get_resource_types(_auth: ScimAuth) -> JsonResult {
    let base = CONFIG.domain();
    Ok(Json(json!({
        "schemas": ["urn:ietf:params:scim:api:messages:2.0:ListResponse"],
        "totalResults": 2,
        "Resources": [
            {
                "id": "User", "name": "User", "endpoint": "/v2/Users",
                "schema": "urn:ietf:params:scim:schemas:core:2.0:User",
                "meta": { "resourceType": "ResourceType", "location": format!("{base}/scim/v2/ResourceTypes/User") }
            },
            {
                "id": "Group", "name": "Group", "endpoint": "/v2/Groups",
                "schema": "urn:ietf:params:scim:schemas:core:2.0:Group",
                "meta": { "resourceType": "ResourceType", "location": format!("{base}/scim/v2/ResourceTypes/Group") }
            }
        ]
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Route registration
// ─────────────────────────────────────────────────────────────────────────────

pub fn routes() -> Vec<Route> {
    routes![
        get_users, get_user, create_user, patch_user,
        get_groups, get_group, create_group, patch_group,
        service_provider_config, get_schemas, get_resource_types,
    ]
}
