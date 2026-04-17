CREATE TABLE audit_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    actor_user_uuid TEXT,
    actor_email TEXT,
    target_resource TEXT,
    ip_address TEXT,
    user_agent TEXT,
    org_uuid TEXT,
    metadata TEXT,
    prev_hash BLOB,
    entry_hash BLOB NOT NULL,
    siem_delivered BOOLEAN NOT NULL DEFAULT 0,
    siem_attempts INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_audit_entries_org ON audit_entries(org_uuid);
CREATE INDEX idx_audit_entries_actor ON audit_entries(actor_user_uuid);
CREATE INDEX idx_audit_entries_timestamp ON audit_entries(timestamp);
CREATE INDEX idx_audit_entries_siem ON audit_entries(siem_delivered);

CREATE TABLE audit_entries_archive (
    id INTEGER PRIMARY KEY,
    timestamp DATETIME NOT NULL,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    actor_user_uuid TEXT,
    actor_email TEXT,
    target_resource TEXT,
    ip_address TEXT,
    user_agent TEXT,
    org_uuid TEXT,
    metadata TEXT,
    prev_hash BLOB,
    entry_hash BLOB NOT NULL
);
