use tokio::sync::mpsc;
use chrono::Utc;
use serde::Serialize;
use std::sync::OnceLock;
use sha2::{Sha256, Digest};

use crate::db::DbConn;
use crate::db::models::audit::{NewAuditEntry, AuditEntry};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub enum AuditEventType {
    LoginSuccess,
    LoginFailed,
    PasswordChanged,
    AttachmentUploaded,
    AttachmentDeleted,
    UserCreated,
    UserDeleted,
    GroupCreated,
    UserAddedToGroup,
}

impl AsRef<str> for AuditEventType {
    fn as_ref(&self) -> &str {
        match self {
            Self::LoginSuccess => "LoginSuccess",
            Self::LoginFailed => "LoginFailed",
            Self::PasswordChanged => "PasswordChanged",
            Self::AttachmentUploaded => "AttachmentUploaded",
            Self::AttachmentDeleted => "AttachmentDeleted",
            Self::UserCreated => "UserCreated",
            Self::UserDeleted => "UserDeleted",
            Self::GroupCreated => "GroupCreated",
            Self::UserAddedToGroup => "UserAddedToGroup",
        }
    }
}

pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub severity: String,
    pub actor_user_uuid: Option<String>,
    pub actor_email: Option<String>,
    pub target_resource: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub org_uuid: Option<String>,
    pub metadata: Option<String>,
}

pub static AUDIT_TX: OnceLock<mpsc::Sender<AuditEvent>> = OnceLock::new();

pub fn init_audit_log(pool: crate::db::DbPool) {
    let (tx, mut rx) = mpsc::channel::<AuditEvent>(1000);
    
    let _unused = AUDIT_TX.set(tx);

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let Ok(mut conn) = pool.get().await {
                write_audit_entry_with_chain(event, &mut conn).await;
            } else {
                error!("AUDIT ERROR: Cannot acquire DB connection to write audit log");
            }
        }
    });
}

pub async fn emit_audit_event(event: AuditEvent) {
    if crate::CONFIG.audit_log_enabled() {
        if let Some(tx) = AUDIT_TX.get() {
            let _unused = tx.send(event).await;
        }
    }
}

async fn write_audit_entry_with_chain(event: AuditEvent, conn: &mut DbConn) {
    let prev_entry = AuditEntry::get_latest(conn).await;
    let prev_hash = prev_entry.map(|e| e.entry_hash);
    
    let db_time = Utc::now().naive_utc();
    
    let mut hasher = Sha256::new();
    if let Some(ref ph) = prev_hash {
        hasher.update(ph);
    }
    hasher.update(db_time.and_utc().timestamp().to_be_bytes());
    hasher.update(event.event_type.as_ref().as_bytes());
    hasher.update(event.actor_user_uuid.as_deref().unwrap_or("").as_bytes());
    hasher.update(event.target_resource.as_deref().unwrap_or("").as_bytes());
    
    let entry_hash = hasher.finalize().to_vec();

    let new_entry = NewAuditEntry {
        timestamp: db_time,
        event_type: event.event_type.as_ref().to_string(),
        severity: event.severity,
        actor_user_uuid: event.actor_user_uuid,
        actor_email: event.actor_email,
        target_resource: event.target_resource,
        ip_address: event.ip_address,
        user_agent: event.user_agent,
        org_uuid: event.org_uuid,
        metadata: event.metadata,
        prev_hash,
        entry_hash,
        siem_delivered: false,
        siem_attempts: 0,
        tenant_uuid: crate::CONFIG.tenant_default_uuid(),
    };

    let _unused = AuditEntry::insert(new_entry, conn).await;
}

pub async fn archive_older_than_job(pool: crate::db::DbPool) {
    let conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("archive_older_than_job: DB pool error: {e:?}");
            return;
        }
    };

    let retention_days = crate::CONFIG.audit_retention_days();
    // ensure we don't go below minimum
    let min_days = crate::CONFIG.audit_retention_minimum_days();
    let actual_days = std::cmp::max(retention_days, min_days);

    let cutoff = (Utc::now() - chrono::TimeDelta::try_days(actual_days).unwrap()).naive_utc();

    match AuditEntry::archive_older_than(cutoff, &conn).await {
        Ok(count) if count > 0 => info!("[AUDIT] Archived {} old entries", count),
        Ok(_) => debug!("[AUDIT] No entries to archive"),
        Err(e) => error!("[AUDIT] Failed to archive old entries: {e:?}"),
    }
}
