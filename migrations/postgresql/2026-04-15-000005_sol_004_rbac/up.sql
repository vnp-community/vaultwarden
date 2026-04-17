CREATE TABLE custom_roles (
    uuid CHAR(36) PRIMARY KEY,
    org_uuid CHAR(36) NOT NULL REFERENCES organizations(uuid),
    name VARCHAR(255) NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE access_schedules (
    uuid CHAR(36) PRIMARY KEY,
    org_uuid CHAR(36) NULL REFERENCES organizations(uuid),
    user_uuid CHAR(36) NULL REFERENCES users(uuid),
    timezone VARCHAR(128) NOT NULL DEFAULT 'UTC',
    allowed_days INTEGER NOT NULL DEFAULT 127,
    allowed_time_from TIME NULL,
    allowed_time_until TIME NULL
);

CREATE TABLE ip_allowlists (
    uuid CHAR(36) PRIMARY KEY,
    org_uuid CHAR(36) NULL REFERENCES organizations(uuid),
    cidr_ranges JSONB NOT NULL DEFAULT '[]'
);

CREATE TABLE break_glass_configs (
    uuid CHAR(36) PRIMARY KEY,
    user_uuid CHAR(36) UNIQUE NOT NULL REFERENCES users(uuid),
    witness_uuids JSONB NOT NULL DEFAULT '[]',
    notification_emails JSONB NOT NULL DEFAULT '[]',
    session_duration_hours INTEGER NOT NULL DEFAULT 2
);

CREATE TABLE approval_requests (
    uuid CHAR(36) PRIMARY KEY,
    requester_user_uuid CHAR(36) NOT NULL REFERENCES users(uuid),
    resource_uuid CHAR(36) NOT NULL,
    state VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NULL
);

CREATE TABLE sod_rules (
    uuid CHAR(36) PRIMARY KEY,
    org_uuid CHAR(36) NOT NULL REFERENCES organizations(uuid),
    role_a_uuid CHAR(36) NOT NULL REFERENCES custom_roles(uuid),
    role_b_uuid CHAR(36) NOT NULL REFERENCES custom_roles(uuid),
    enforcement VARCHAR(50) NOT NULL DEFAULT 'soft'
);

ALTER TABLE users_organizations ADD COLUMN custom_role_uuid CHAR(36) NULL REFERENCES custom_roles(uuid);
