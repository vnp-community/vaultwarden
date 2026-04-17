/// TASK-008-013: Secrets list/get endpoints
/// TASK-008-014: Secrets export endpoint (env, dotenv, json formats)

use rocket::{serde::json::Json, Route};

use crate::{
    api::JsonResult,
    auth::ApiKeyAuth,
    db::{
        models::{Cipher, CipherId, OrganizationId},
        DbConn,
    },
    error::Error,
};

pub fn routes() -> Vec<Route> {
    routes![
        list_secrets,
        export_secrets,
        get_secret,
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-013: GET /secrets?project=<project>
// ─────────────────────────────────────────────────────────────────────────────

#[get("/secrets?<_project>")]
pub async fn list_secrets(_project: Option<&str>, auth: ApiKeyAuth, conn: DbConn) -> JsonResult {
    let org_id = OrganizationId::from(auth.org_uuid.clone());

    let secrets: Vec<serde_json::Value> = Cipher::find_by_org(&org_id, &conn).await
        .into_iter()
        .filter(|c| c.atype == 2) // SecureNote type used as secrets
        .map(|c| json!({
            "id": c.uuid,
            "organizationId": c.organization_uuid,
            "type": c.atype,
            "name": c.name,
            // data blob is E2E-encrypted; returned as-is
            "data": c.data,
            "revisionDate": c.updated_at,
            "deletedDate": c.deleted_at,
            "object": "secret"
        }))
        .collect();

    Ok(Json(json!({
        "data": secrets,
        "object": "list",
        "continuationToken": null
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-013: GET /secrets/<id>
// ─────────────────────────────────────────────────────────────────────────────

#[get("/secrets/<secret_id>")]
pub async fn get_secret(secret_id: &str, auth: ApiKeyAuth, conn: DbConn) -> JsonResult {
    let cipher_id = CipherId::from(secret_id.to_string());
    let cipher = Cipher::find_by_uuid(&cipher_id, &conn).await
        .ok_or_else(|| Error::new("NotFound", format!("Secret {secret_id} not found")))?;

    // Verify org ownership
    let cipher_org = cipher.organization_uuid.as_ref().map(|u| u.to_string());
    if cipher_org.as_deref() != Some(&auth.org_uuid) {
        return Err(Error::new("Forbidden", "Secret does not belong to your organization"));
    }

    Ok(Json(json!({
        "id": cipher.uuid,
        "organizationId": cipher.organization_uuid,
        "type": cipher.atype,
        "name": cipher.name,
        "data": cipher.data,
        "revisionDate": cipher.updated_at,
        "object": "secret"
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-008-014: GET /secrets/export?format=<fmt>&project=<project>
// ─────────────────────────────────────────────────────────────────────────────

#[get("/secrets/export?<format>&<_project>")]
pub async fn export_secrets(
    format: Option<&str>,
    _project: Option<&str>,
    auth: ApiKeyAuth,
    conn: DbConn,
) -> JsonResult {
    let org_id = OrganizationId::from(auth.org_uuid.clone());
    let ciphers: Vec<_> = Cipher::find_by_org(&org_id, &conn).await
        .into_iter()
        .filter(|c| c.atype == 2)
        .collect();

    match format.unwrap_or("json") {
        "env" | "dotenv" => {
            // KEY=ENCRYPTED_BLOB — consumer uses SDK to decrypt
            let lines: Vec<String> = ciphers.iter()
                .map(|c| {
                    let key = c.name.to_uppercase().replace([' ', '-'], "_");
                    format!("{key}={}", c.data)
                })
                .collect();

            Ok(Json(json!({
                "format": "env",
                "note": "Values are E2E-encrypted. Use the Vaultwarden SDK to decrypt.",
                "content": lines.join("\n"),
                "count": lines.len(),
                "object": "secretsExport"
            })))
        }
        _ => {
            let data: Vec<serde_json::Value> = ciphers.iter()
                .map(|c| json!({
                    "id": c.uuid,
                    "name": c.name,
                    "data": c.data,
                    "revisionDate": c.updated_at,
                }))
                .collect();

            Ok(Json(json!({
                "format": "json",
                "note": "data fields are E2E-encrypted. Use the Vaultwarden SDK to decrypt.",
                "data": data,
                "count": data.len(),
                "object": "secretsExport"
            })))
        }
    }
}
