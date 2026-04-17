use chrono::NaiveDateTime;
use diesel::prelude::*;
use crate::db::schema::{audit_entries, audit_entries_archive};
use crate::{db::DbConn, error::Error};

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = audit_entries)]
#[diesel(treat_none_as_null = true)]
pub struct AuditEntry {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub event_type: String,
    pub severity: String,
    pub actor_user_uuid: Option<String>,
    pub actor_email: Option<String>,
    pub target_resource: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub org_uuid: Option<String>,
    pub metadata: Option<String>,
    pub prev_hash: Option<Vec<u8>>,
    pub entry_hash: Vec<u8>,
    pub siem_delivered: bool,
    pub siem_attempts: i32,
    pub tenant_uuid: String,
}

#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = audit_entries)]
#[diesel(treat_none_as_null = true)]
pub struct NewAuditEntry {
    pub timestamp: NaiveDateTime,
    pub event_type: String,
    pub severity: String,
    pub actor_user_uuid: Option<String>,
    pub actor_email: Option<String>,
    pub target_resource: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub org_uuid: Option<String>,
    pub metadata: Option<String>,
    pub prev_hash: Option<Vec<u8>>,
    pub entry_hash: Vec<u8>,
    pub siem_delivered: bool,
    pub siem_attempts: i32,
    pub tenant_uuid: String,
}

#[derive(Queryable, Identifiable)]
#[diesel(table_name = audit_entries_archive)]
#[diesel(treat_none_as_null = true)]
pub struct AuditEntryArchive {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub event_type: String,
    pub severity: String,
    pub actor_user_uuid: Option<String>,
    pub actor_email: Option<String>,
    pub target_resource: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub org_uuid: Option<String>,
    pub metadata: Option<String>,
    pub prev_hash: Option<Vec<u8>>,
    pub entry_hash: Vec<u8>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = audit_entries_archive)]
#[diesel(treat_none_as_null = true)]
pub struct NewAuditEntryArchive {
    pub timestamp: NaiveDateTime,
    pub event_type: String,
    pub severity: String,
    pub actor_user_uuid: Option<String>,
    pub actor_email: Option<String>,
    pub target_resource: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub org_uuid: Option<String>,
    pub metadata: Option<String>,
    pub prev_hash: Option<Vec<u8>>,
    pub entry_hash: Vec<u8>,
}

impl AuditEntry {
    pub async fn get_latest(conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            audit_entries::table
                .order_by(audit_entries::id.desc())
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn insert(entry: NewAuditEntry, conn: &DbConn) -> Result<Self, Error> {
        db_run! { conn: {
            diesel::insert_into(audit_entries::table)
                .values(&entry)
                // Vaultwarden supports sqlite and postgres natively returning the row
                // For MySQL, we might have to use last_insert_id but Vaultwarden has polyfills or we do a re-query
                // Or we can just insert and not return the ID
                .execute(conn)?;
            
            audit_entries::table
                .order_by(audit_entries::id.desc())
                .first::<Self>(conn)
                .map_err(Into::into)
        }}
    }

    pub async fn archive_older_than(cutoff: NaiveDateTime, conn: &DbConn) -> Result<usize, Error> {
        db_run! { conn: {
            let old_entries = audit_entries::table
                .filter(audit_entries::timestamp.lt(cutoff))
                .load::<Self>(conn)?;
            
            if old_entries.is_empty() {
                return Ok(0);
            }

            let mut archive_entries = Vec::with_capacity(old_entries.len());
            let mut ids_to_delete = Vec::with_capacity(old_entries.len());

            for entry in old_entries {
                ids_to_delete.push(entry.id);
                archive_entries.push(NewAuditEntryArchive {
                    timestamp: entry.timestamp,
                    event_type: entry.event_type,
                    severity: entry.severity,
                    actor_user_uuid: entry.actor_user_uuid,
                    actor_email: entry.actor_email,
                    target_resource: entry.target_resource,
                    ip_address: entry.ip_address,
                    user_agent: entry.user_agent,
                    org_uuid: entry.org_uuid,
                    metadata: entry.metadata,
                    prev_hash: entry.prev_hash,
                    entry_hash: entry.entry_hash,
                });
            }

            // Insert into archive table
            for archive_entry in archive_entries {
                diesel::insert_into(audit_entries_archive::table)
                    .values(&archive_entry)
                    .execute(conn)?;
            }

            // Delete from main table
            let deleted = diesel::delete(
                audit_entries::table.filter(audit_entries::id.eq_any(ids_to_delete))
            ).execute(conn)?;

            Ok(deleted)
        }}
    }
}
