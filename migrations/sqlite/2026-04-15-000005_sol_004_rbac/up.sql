CREATE TABLE custom_roles (
    uuid TEXT PRIMARY KEY,
    org_uuid TEXT NOT NULL REFERENCES organizations(uuid),
    name TEXT NOT NULL,
    permissions TEXT NOT NULL DEFAULT '[]',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE access_schedules (
    uuid TEXT PRIMARY KEY,
    org_uuid TEXT REFERENCES organizations(uuid),
    user_uuid TEXT REFERENCES users(uuid),
    timezone TEXT NOT NULL DEFAULT 'UTC',
    allowed_days INTEGER NOT NULL DEFAULT 127,
    allowed_time_from TEXT,
    allowed_time_until TEXT
);

CREATE TABLE ip_allowlists (
    uuid TEXT PRIMARY KEY,
    org_uuid TEXT REFERENCES organizations(uuid),
    cidr_ranges TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE break_glass_configs (
    uuid TEXT PRIMARY KEY,
    user_uuid TEXT UNIQUE NOT NULL REFERENCES users(uuid),
    witness_uuids TEXT NOT NULL DEFAULT '[]',
    notification_emails TEXT NOT NULL DEFAULT '[]',
    session_duration_hours INTEGER NOT NULL DEFAULT 2
);

CREATE TABLE approval_requests (
    uuid TEXT PRIMARY KEY,
    requester_user_uuid TEXT NOT NULL REFERENCES users(uuid),
    resource_uuid TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'pending',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME
);

CREATE TABLE sod_rules (
    uuid TEXT PRIMARY KEY,
    org_uuid TEXT NOT NULL REFERENCES organizations(uuid),
    role_a_uuid TEXT NOT NULL REFERENCES custom_roles(uuid),
    role_b_uuid TEXT NOT NULL REFERENCES custom_roles(uuid),
    enforcement TEXT NOT NULL DEFAULT 'soft'
);

ALTER TABLE users_organizations ADD COLUMN custom_role_uuid TEXT REFERENCES custom_roles(uuid);
