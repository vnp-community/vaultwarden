CREATE TABLE ldap_sync_state (
    id SERIAL PRIMARY KEY,
    last_sync_at TIMESTAMP NOT NULL,
    status VARCHAR(50) NOT NULL,
    users_synced INTEGER NOT NULL,
    groups_synced INTEGER NOT NULL,
    error_message TEXT
);

CREATE TABLE ldap_group_mappings (
    id SERIAL PRIMARY KEY,
    ldap_group_dn VARCHAR(255) NOT NULL,
    collection_uuid VARCHAR(40) NOT NULL,
    org_uuid VARCHAR(40) NOT NULL
);

CREATE TABLE access_reviews (
    id SERIAL PRIMARY KEY,
    org_uuid VARCHAR(40) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    deadline_at TIMESTAMP NOT NULL,
    status VARCHAR(50) NOT NULL
);

CREATE TABLE access_review_items (
    id SERIAL PRIMARY KEY,
    access_review_id INTEGER NOT NULL REFERENCES access_reviews(id) ON DELETE CASCADE,
    collection_uuid VARCHAR(40) NOT NULL,
    user_uuid VARCHAR(40) NOT NULL,
    reviewed_by VARCHAR(40),
    reviewed_at TIMESTAMP,
    decision VARCHAR(50)
);

CREATE TABLE scim_tokens (
    id SERIAL PRIMARY KEY,
    token_hash BYTEA NOT NULL,
    org_uuid VARCHAR(40) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP
);

ALTER TABLE users ADD COLUMN provisioning_source VARCHAR(50);
ALTER TABLE users ADD COLUMN provisioning_external_id VARCHAR(255);
ALTER TABLE users ADD COLUMN suspension_scheduled_at TIMESTAMP;

ALTER TABLE organizations ADD COLUMN ldap_group_dn VARCHAR(255);
