use rocket::{http::Status, serde::json::Json, Route};
use serde_json::{json, Value};
use crate::{db::DbConn, CONFIG};

#[get("/alive")]
pub fn alive() -> &'static str {
    "OK"
}

#[get("/ready")]
pub async fn ready(_conn: DbConn) -> Result<&'static str, Status> {
    // _conn being successfully injected means DB pool is responsive
    if CONFIG.redis_enabled() {
        // Just a basic check that CACHE doesn't panic on operation
        drop(crate::cache::CACHE.get("__health_check__").await);
    }
    
    Ok("OK")
}

/// TASK-010-010: Detailed health endpoint with memory, metrics, job info
#[get("/detailed")]
pub async fn detailed(conn: DbConn) -> Result<Json<Value>, Status> {
    let db_ok = {
        use crate::db::schema::users;
        use diesel::prelude::*;
        use crate::db_run;
        // This will succeed or fail based on DB availability
        let result: Result<i64, _> = db_run! { conn: {
            users::table.count().get_result(conn)
        }};
        result.is_ok()
    };

    let metrics_snapshot = {
        let enabled = CONFIG.metrics_enabled();
        if enabled {
            let active_sessions = crate::metrics::METRICS.active_sessions.get();
            let ws_connections = crate::metrics::METRICS.websocket_connections.get();
            Some(json!({
                "active_sessions": active_sessions,
                "websocket_connections": ws_connections,
            }))
        } else {
            None
        }
    };

    let health = json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "version": crate::VERSION.unwrap_or("unknown"),
        "database": { "connected": db_ok },
        "redis": { "enabled": CONFIG.redis_enabled() },
        "metrics": metrics_snapshot,
        "Object": "health"
    });

    Ok(Json(health))
}

pub fn routes() -> Vec<Route> {
    routes![alive, ready, detailed]
}
