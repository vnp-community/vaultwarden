CREATE TABLE ldap_sync_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    last_sync_at DATETIME NOT NULL,
    status TEXT NOT NULL,
    users_synced INTEGER NOT NULL,
    groups_synced INTEGER NOT NULL,
    error_message TEXT
);

CREATE TABLE ldap_group_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ldap_group_dn TEXT NOT NULL,
    collection_uuid TEXT NOT NULL,
    org_uuid TEXT NOT NULL
);

CREATE TABLE access_reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    org_uuid TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    deadline_at DATETIME NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE access_review_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    access_review_id INTEGER NOT NULL REFERENCES access_reviews(id) ON DELETE CASCADE,
    collection_uuid TEXT NOT NULL,
    user_uuid TEXT NOT NULL,
    reviewed_by TEXT,
    reviewed_at DATETIME,
    decision TEXT
);

CREATE TABLE scim_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token_hash BLOB NOT NULL,
    org_uuid TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    last_used_at DATETIME
);

ALTER TABLE users ADD COLUMN provisioning_source TEXT;
ALTER TABLE users ADD COLUMN provisioning_external_id TEXT;
ALTER TABLE users ADD COLUMN suspension_scheduled_at DATETIME;

ALTER TABLE organizations ADD COLUMN ldap_group_dn TEXT;
