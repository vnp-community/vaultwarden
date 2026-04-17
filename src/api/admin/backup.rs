#![allow(dead_code)]

use rocket::Route;
use crate::api::EmptyResult;
use crate::backup::BackupManager;
use crate::db::DbConn;

pub fn routes() -> Vec<Route> {
    routes![trigger_backup, get_backup_runs, get_dr_runbook]
}

#[post("/backups/trigger")]
pub async fn trigger_backup(mut conn: DbConn) -> EmptyResult {
    let backup_manager = BackupManager::new();
    
    // Asynchronous background execution so we don't timeout the HTTP response
    tokio::spawn(async move {
        match backup_manager.run_backup(&mut conn).await {
            Ok(_) => info!("Administrative backup triggered successfully."),
            Err(e) => error!("Administrative backup failed: {}", e),
        }
    });

    Ok(())
}

#[get("/backups")]
pub async fn get_backup_runs() -> EmptyResult {
    // Scaffold: List backup history
    Ok(())
}

#[get("/backups/runbook")]
pub async fn get_dr_runbook() -> crate::api::JsonResult {
    let runbook = serde_json::json!({
        "status": "ready",
        "primary_destination": crate::CONFIG.backup_destination(),
        "secondary_destination": crate::CONFIG.backup_secondary_destination(),
        "pitr_enabled": crate::CONFIG.backup_pitr_enabled(),
        "wal_archive": crate::CONFIG.backup_wal_archive_destination(),
        "recovery_steps": [
            "1. Download closest full `.sqldump` along with `.manifest.json` from destination bucket.",
            "2. Ensure Manifest digital signature is intact.",
            "3. Restore format: Initialize PostgreSQL cluster via `pg_restore` (or direct copy inside Vaultwarden if SQLite).",
            "4. If PITR enabled, apply the recovered WAL chunks to achieve point-in-time recovery target."
        ]
    });
    Ok(crate::api::Json(runbook))
}
