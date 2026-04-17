CREATE TABLE privileged_configs (
    uuid VARCHAR(64) PRIMARY KEY,
    cipher_uuid VARCHAR(64) NOT NULL REFERENCES ciphers(uuid),
    requires_approval BOOLEAN NOT NULL DEFAULT false,
    max_checkout_duration INTEGER,
    auto_rotate_after_checkout BOOLEAN NOT NULL DEFAULT false,
    rotation_target_type VARCHAR(32),
    rotation_target_config TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE checkouts (
    uuid VARCHAR(64) PRIMARY KEY,
    cipher_uuid VARCHAR(64) NOT NULL REFERENCES ciphers(uuid),
    user_uuid VARCHAR(64) NOT NULL REFERENCES users(uuid),
    justification TEXT NOT NULL,
    itsm_ticket VARCHAR(128),
    approval_request_uuid VARCHAR(64),
    checked_out_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP,
    checked_in_at TIMESTAMP,
    access_count INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL,
    rotation_triggered BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE rotation_history (
    uuid VARCHAR(64) PRIMARY KEY,
    cipher_uuid VARCHAR(64) NOT NULL REFERENCES ciphers(uuid),
    checkout_uuid VARCHAR(64),
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    status VARCHAR(32) NOT NULL,
    error_message TEXT
);

ALTER TABLE ciphers ADD COLUMN is_privileged BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE ciphers ADD COLUMN privileged_config_uuid VARCHAR(64);
