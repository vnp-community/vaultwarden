CREATE TABLE audit_entries (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    severity VARCHAR(50) NOT NULL,
    actor_user_uuid VARCHAR(40),
    actor_email VARCHAR(255),
    target_resource VARCHAR(255),
    ip_address VARCHAR(255),
    user_agent TEXT,
    org_uuid VARCHAR(40),
    metadata TEXT,
    prev_hash BYTEA,
    entry_hash BYTEA NOT NULL,
    siem_delivered BOOLEAN NOT NULL DEFAULT false,
    siem_attempts INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_audit_entries_org ON audit_entries(org_uuid);
CREATE INDEX idx_audit_entries_actor ON audit_entries(actor_user_uuid);
CREATE INDEX idx_audit_entries_timestamp ON audit_entries(timestamp);
CREATE INDEX idx_audit_entries_siem ON audit_entries(siem_delivered);

CREATE TABLE audit_entries_archive (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    severity VARCHAR(50) NOT NULL,
    actor_user_uuid VARCHAR(40),
    actor_email VARCHAR(255),
    target_resource VARCHAR(255),
    ip_address VARCHAR(255),
    user_agent TEXT,
    org_uuid VARCHAR(40),
    metadata TEXT,
    prev_hash BYTEA,
    entry_hash BYTEA NOT NULL
);
