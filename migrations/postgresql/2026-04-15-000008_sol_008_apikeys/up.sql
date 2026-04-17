CREATE TABLE api_keys_v2 (
    uuid VARCHAR(64) PRIMARY KEY,
    org_uuid VARCHAR(64) NOT NULL REFERENCES organizations(uuid),
    client_id VARCHAR(64) NOT NULL,
    secret_hash TEXT NOT NULL,
    name VARCHAR(128) NOT NULL,
    scopes TEXT NOT NULL,
    allowed_ips TEXT,
    rate_limit_minute INTEGER,
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE api_key_usage (
    id VARCHAR(64) PRIMARY KEY,
    api_key_uuid VARCHAR(64) NOT NULL REFERENCES api_keys_v2(uuid),
    endpoint VARCHAR(128) NOT NULL,
    method VARCHAR(16) NOT NULL,
    status_code INTEGER NOT NULL,
    response_ms INTEGER NOT NULL,
    timestamp TIMESTAMP NOT NULL
);

CREATE TABLE webhooks (
    uuid VARCHAR(64) PRIMARY KEY,
    org_uuid VARCHAR(64) NOT NULL REFERENCES organizations(uuid),
    name VARCHAR(128) NOT NULL,
    url TEXT NOT NULL,
    secret_hash TEXT NOT NULL,
    events TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    retry_count INTEGER NOT NULL DEFAULT 3,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE webhook_deliveries (
    uuid VARCHAR(64) PRIMARY KEY,
    webhook_uuid VARCHAR(64) NOT NULL REFERENCES webhooks(uuid),
    event_type VARCHAR(64) NOT NULL,
    payload TEXT NOT NULL,
    status VARCHAR(32) NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMP,
    next_attempt_at TIMESTAMP,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL
);

ALTER TABLE ciphers ADD COLUMN is_secret BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE ciphers ADD COLUMN secret_project VARCHAR(128);
