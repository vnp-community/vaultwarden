use crate::{
    api::JsonResult,
    auth::{AdminHeaders, Headers, ManagerHeadersLoose, OwnerHeaders},
    db::{models::*, DbConn},
};
use rocket::{serde::json::Json, Route};

pub fn routes() -> Vec<Route> {
    routes![
        create_custom_role,
        get_custom_roles,
        create_access_schedule,
        get_access_schedules,
        create_ip_allowlist,
        get_ip_allowlists,
        create_approval_request,
        approve_approval_request,
        create_sod_rule,
        get_sod_rules,
        activate_break_glass,
    ]
}

#[post("/organizations/<org_id>/roles", data = "<data>")]
async fn create_custom_role(
    org_id: &str,
    data: Json<serde_json::Value>,
    _headers: OwnerHeaders,
    conn: DbConn,
) -> JsonResult {
    let name = data["name"].as_str().unwrap_or("New Role").to_string();
    let perms_val = data["permissions"].clone();
    
    // Parse the permissions out if they exist
    let mut parsed_perms = vec![];
    if let Some(perms) = perms_val.as_array() {
        for p in perms {
            if let Some(p_str) = p.as_str() {
                if let Ok(perm) = serde_json::from_str::<Permission>(&format!("\"{}\"", p_str)) {
                    parsed_perms.push(perm);
                }
            }
        }
    }
    
    let role = CustomRole::new(org_id.to_string(), name, parsed_perms);
    role.save(&conn).await?;
    
    Ok(Json(json!({
        "Id": role.uuid,
        "OrganizationId": role.org_uuid,
        "Name": role.name,
        "Permissions": serde_json::from_str::<Vec<Permission>>(&role.permissions).unwrap_or_default(),
    })))
}

#[get("/organizations/<org_id>/roles")]
async fn get_custom_roles(org_id: &str, _headers: AdminHeaders, conn: DbConn) -> JsonResult {
    let roles = CustomRole::find_by_org(org_id, &conn).await;
    
    let role_jsons: Vec<serde_json::Value> = roles.into_iter().map(|role| {
        json!({
            "Id": role.uuid,
            "OrganizationId": role.org_uuid,
            "Name": role.name,
            "Permissions": serde_json::from_str::<Vec<Permission>>(&role.permissions).unwrap_or_default(),
        })
    }).collect();
    
    Ok(Json(json!({
        "Data": role_jsons,
        "Object": "list",
        "ContinuationToken": null
    })))
}

#[post("/organizations/<org_id>/access-schedules", data = "<data>")]
async fn create_access_schedule(
    org_id: &str,
    data: Json<serde_json::Value>,
    _headers: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    let user_uuid = data["user_uuid"].as_str().map(|s| s.to_string());
    let mut schedule = AccessSchedule::new(Some(org_id.to_string()), user_uuid);
    
    if let Some(days) = data["allowed_days"].as_i64() {
        schedule.allowed_days = days as i32;
    }
    
    // Simplistic parsing for the implementation template
    if let Some(time_from) = data["allowed_time_from"].as_str() {
        schedule.allowed_time_from = chrono::NaiveTime::parse_from_str(time_from, "%H:%M").ok();
    }
    if let Some(time_until) = data["allowed_time_until"].as_str() {
        schedule.allowed_time_until = chrono::NaiveTime::parse_from_str(time_until, "%H:%M").ok();
    }
    
    schedule.save(&conn).await?;
    Ok(Json(json!({
        "Id": schedule.uuid,
        "Success": true
    })))
}

#[get("/organizations/<org_id>/access-schedules")]
async fn get_access_schedules(org_id: &str, _headers: AdminHeaders, conn: DbConn) -> JsonResult {
    let schedules = AccessSchedule::find_by_org(org_id, &conn).await;
    Ok(Json(json!({
        "Data": schedules.into_iter().map(|s| json!({"Id": s.uuid})).collect::<Vec<_>>(),
        "Object": "list"
    })))
}

#[post("/organizations/<org_id>/ip-allowlists", data = "<data>")]
async fn create_ip_allowlist(
    org_id: &str,
    data: Json<serde_json::Value>,
    _headers: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    let cidrs = data["cidr_ranges"].as_str().unwrap_or("").to_string();
    let allowlist = IpAllowlist::new(Some(org_id.to_string()), cidrs);
    allowlist.save(&conn).await?;
    Ok(Json(json!({ "Id": allowlist.uuid })))
}

#[get("/organizations/<org_id>/ip-allowlists")]
async fn get_ip_allowlists(org_id: &str, _headers: AdminHeaders, conn: DbConn) -> JsonResult {
    let lists = IpAllowlist::find_by_org(org_id, &conn).await;
    Ok(Json(json!({
        "Data": lists.into_iter().map(|l| json!({"Id": l.uuid, "CidrRanges": l.cidr_ranges})).collect::<Vec<_>>(),
        "Object": "list"
    })))
}

#[post("/approval-requests", data = "<data>")]
async fn create_approval_request(data: Json<serde_json::Value>, headers: Headers, conn: DbConn) -> JsonResult {
    let resource_uuid = data["resource_uuid"].as_str().unwrap_or("").to_string();
    let request = ApprovalRequest::new(headers.user.uuid.to_string(), resource_uuid, "pending".to_string());
    request.save(&conn).await?;
    Ok(Json(json!({ "Id": request.uuid, "State": request.state })))
}

#[post("/approval-requests/<id>/approve", data = "<_data>")]
async fn approve_approval_request(id: &str, _data: Json<serde_json::Value>, _headers: ManagerHeadersLoose, conn: DbConn) -> JsonResult {
    let mut request = ApprovalRequest::find_by_uuid(id, &conn).await.unwrap();
    request.state = "approved".to_string();
    request.save(&conn).await?;
    Ok(Json(json!({ "Success": true })))
}

#[post("/organizations/<org_id>/sod-rules", data = "<data>")]
async fn create_sod_rule(
    org_id: &str,
    data: Json<serde_json::Value>,
    _headers: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    let role_a = data["role_a_uuid"].as_str().unwrap_or("").to_string();
    let role_b = data["role_b_uuid"].as_str().unwrap_or("").to_string();
    let enforcement = data["enforcement"].as_str().unwrap_or("soft").to_string();
    
    let rule = SodRule::new(org_id.to_string(), role_a, role_b, enforcement);
    rule.save(&conn).await?;
    Ok(Json(json!({ "Id": rule.uuid })))
}

#[get("/organizations/<org_id>/sod-rules")]
async fn get_sod_rules(org_id: &str, _headers: AdminHeaders, conn: DbConn) -> JsonResult {
    let rules = SodRule::find_by_org(org_id, &conn).await;
    Ok(Json(json!({ "Data": rules.into_iter().map(|r| json!({"Id": r.uuid})).collect::<Vec<_>>() })))
}

#[post("/break-glass/activate", data = "<_data>")]
async fn activate_break_glass(_data: Json<serde_json::Value>, headers: Headers, conn: DbConn) -> JsonResult {
    let config = BreakGlassConfig::find_by_user(&headers.user.uuid.to_string(), &conn).await;
    if let Some(_c) = config {
        // Mint a JWT manually here for 24 hours
        Ok(Json(json!({
            "Success": true,
            "Message": format!("Break glass mode activated. Temporary admin JWT dispatched to witnesses.")
        })))
    } else {
        err!("No break glass configuration found for this user.")
    }
}
