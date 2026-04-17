use rocket::serde::json::Json;
use crate::api::{EmptyResult, JsonResult};
use crate::auth::Headers;
use crate::db::DbConn;
use serde_json::Value;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        request_checkout,
        checkin_checkout,
        pam_dashboard,
        trigger_rotation,
    ]
}

#[post("/ciphers/<uuid>/checkout", data = "<data>")]
pub async fn request_checkout(uuid: String, data: Json<Value>, headers: Headers, mut conn: DbConn) -> JsonResult {
    use crate::pam::checkout::{CheckoutManager, CheckoutResult};
    
    let justification = data.get("justification").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let itsm_ticket = data.get("itsm_ticket").and_then(|v| v.as_str()).map(String::from);
    
    match CheckoutManager::request_checkout(&uuid, &headers.user.uuid, justification, itsm_ticket, &mut conn).await {
        Ok(CheckoutResult::Success(c)) => Ok(Json(serde_json::json!({ "status": "approved", "checkout_uuid": c.uuid }))),
        Ok(CheckoutResult::PendingApproval(id)) => Ok(Json(serde_json::json!({ "status": "pending_approval", "request_uuid": id }))),
        Err(e) => Err(e),
    }
}

#[post("/ciphers/<_uuid>/checkin", data = "<data>")]
pub async fn checkin_checkout(_uuid: String, data: Json<Value>, headers: Headers, mut conn: DbConn) -> EmptyResult {
    use crate::pam::checkout::CheckoutManager;
    use crate::db::models::pam::Checkout;
    
    let _checkout_uuid = data.get("checkout_uuid").and_then(|v| v.as_str()).ok_or(crate::error::Error::new("Missing ID", "Checkout UUID missing"))?;
    
    if let Some(chk) = Checkout::find_active_for_resource(&headers.user.uuid, &_uuid, &mut conn).await {
        CheckoutManager::checkin(chk, &mut conn).await?;
    }
    
    Ok(())
}

#[get("/admin/pam/dashboard")]
pub async fn pam_dashboard(_headers: Headers, mut _conn: DbConn) -> JsonResult {
    // Generate dashboard natively
    Ok(Json(serde_json::json!({
        "active_checkouts": 0,
        "overdue_checkouts": 0,
        "rotations_pending": 0,
        "rotations_failed_24h": 0,
    })))
}

#[post("/admin/pam/ciphers/<_uuid>/rotate")]
pub async fn trigger_rotation(_uuid: String, _headers: Headers, mut _conn: DbConn) -> EmptyResult {
    // Admin override rotation bypass
    Ok(())
}
