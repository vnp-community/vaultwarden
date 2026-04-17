CREATE TABLE privileged_configs (
    uuid VARCHAR(64) PRIMARY KEY,
    cipher_uuid VARCHAR(64) NOT NULL REFERENCES ciphers(uuid),
    requires_approval BOOLEAN NOT NULL DEFAULT 0,
    max_checkout_duration INTEGER,
    auto_rotate_after_checkout BOOLEAN NOT NULL DEFAULT 0,
    rotation_target_type VARCHAR(32),
    rotation_target_config TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

CREATE TABLE checkouts (
    uuid VARCHAR(64) PRIMARY KEY,
    cipher_uuid VARCHAR(64) NOT NULL REFERENCES ciphers(uuid),
    user_uuid VARCHAR(64) NOT NULL REFERENCES users(uuid),
    justification TEXT NOT NULL,
    itsm_ticket VARCHAR(128),
    approval_request_uuid VARCHAR(64),
    checked_out_at DATETIME NOT NULL,
    expires_at DATETIME,
    checked_in_at DATETIME,
    access_count INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL,
    rotation_triggered BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE rotation_history (
    uuid VARCHAR(64) PRIMARY KEY,
    cipher_uuid VARCHAR(64) NOT NULL REFERENCES ciphers(uuid),
    checkout_uuid VARCHAR(64),
    started_at DATETIME NOT NULL,
    completed_at DATETIME,
    status VARCHAR(32) NOT NULL,
    error_message TEXT
);

ALTER TABLE ciphers ADD COLUMN is_privileged BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE ciphers ADD COLUMN privileged_config_uuid VARCHAR(64);
