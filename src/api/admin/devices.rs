use rocket::{get, post, serde::json::Json, Route};
use crate::{
    api::{admin::AdminToken, JsonResult, EmptyResult},
    db::{DbConn, models::{Device, DeviceId, User, UserId}},
    error::Error,
};
use serde_json::Value;

pub fn routes() -> Vec<Route> {
    routes![
        get_devices,
        wipe_device,
        wipe_all_user_devices,
    ]
}

#[get("/devices")]
async fn get_devices(_token: AdminToken, conn: DbConn) -> JsonResult {
    // We implement a custom query to get all devices along with their user for admin dashboard
    use crate::db::schema::{devices, users};
    use diesel::prelude::*;
    use crate::db_run;

    let results = db_run! { conn: {
        devices::table
            .inner_join(users::table.on(devices::user_uuid.eq(users::uuid)))
            .select((devices::all_columns, users::email))
            .load::<(Device, String)>(conn)
            .expect("Error loading devices")
    }};

    let mut devices_json = Vec::new();
    for (device, user_email) in results {
        let mut d = device.to_json();
        d["userEmail"] = Value::String(user_email);
        d["isTrusted"] = Value::Bool(device.is_trusted);
        d["mdmEnrolled"] = Value::Bool(device.mdm_enrolled);
        d["mdmCompliant"] = Value::Bool(device.mdm_compliant);
        d["certSubject"] = device.cert_subject.map_or(Value::Null, Value::String);
        d["certSerial"] = device.cert_serial.map_or(Value::Null, Value::String);
        d["certIssuer"] = device.cert_issuer.map_or(Value::Null, Value::String);
        if let Some(issued) = device.cert_expires_at {
             d["certExpiresAt"] = Value::String(crate::util::format_date(&issued));
        }
        if let Some(last_check) = device.mdm_last_check_at {
            d["mdmLastCheckAt"] = Value::String(crate::util::format_date(&last_check));
        }

        devices_json.push(d);
    }

    Ok(Json(serde_json::json!(devices_json)))
}

#[post("/devices/<uuid>/wipe")]
async fn wipe_device(uuid: String, _token: AdminToken, mut conn: DbConn) -> EmptyResult {
    let device_id = DeviceId::from(uuid);
    let device = match Device::find_by_uuid(&device_id, &mut conn).await {
        Some(d) => d,
        None => return Err(Error::new("Device not found", "The device could not be found.")),
    };

    let user = match User::find_by_uuid(&device.user_uuid, &mut conn).await {
        Some(u) => u,
        None => return Err(Error::new("User not found", "The user associated with device could not be found.")),
    };

    // Broadcast syncWipe logically mapped to logout pushing
    crate::api::push::push_logout(&user, Some(device_id.clone()), &mut conn).await;
    
    // Unregister push notifications if any
    crate::api::push::unregister_push_device(&device.push_uuid).await.ok();

    // Delete device from database
    use crate::db::schema::devices;
    use diesel::prelude::*;
    use crate::db_run;

    db_run! { conn: {
        diesel::delete(devices::table.filter(devices::uuid.eq(device_id)))
            .execute(conn)
            .expect("Error deleting device")
    }};

    Ok(())
}

#[post("/users/<uuid>/wipe-all-devices")]
async fn wipe_all_user_devices(uuid: String, _token: AdminToken, mut conn: DbConn) -> EmptyResult {
    let user_id = UserId::from(uuid);
    let user = match User::find_by_uuid(&user_id, &mut conn).await {
        Some(u) => u,
        None => return Err(Error::new("User not found", "The user could not be found.")),
    };

    let devices = Device::find_by_user(&user.uuid, &mut conn).await;
    
    for device in devices {
        crate::api::push::push_logout(&user, Some(device.uuid.clone()), &mut conn).await;
        crate::api::push::unregister_push_device(&device.push_uuid).await.ok();
    }

    Device::delete_all_by_user(&user.uuid, &mut conn).await.ok();
    
    Ok(())
}
