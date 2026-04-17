CREATE TABLE tenants (
    uuid CHAR(36) NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    domain_restriction TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    max_users INTEGER,
    max_organizations INTEGER,
    max_vault_items INTEGER,
    max_storage_bytes BIGINT,
    config_overrides JSONB,
    branding JSONB,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
CREATE UNIQUE INDEX ix_tenants_slug ON tenants (slug);

-- Create a single default tenant
INSERT INTO tenants (uuid, name, slug, is_active, created_at, updated_at)
VALUES ('00000000-0000-0000-0000-000000000001', 'Default Tenant', 'default', true, NOW(), NOW());

CREATE TABLE tenant_admins (
    tenant_uuid CHAR(36) NOT NULL,
    user_uuid CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY(tenant_uuid, user_uuid),
    FOREIGN KEY(tenant_uuid) REFERENCES tenants(uuid),
    FOREIGN KEY(user_uuid) REFERENCES users(uuid)
);

ALTER TABLE users ADD COLUMN tenant_uuid CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001';
CREATE INDEX ix_users_tenant_uuid ON users (tenant_uuid);

ALTER TABLE organizations ADD COLUMN tenant_uuid CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001';
CREATE INDEX ix_organizations_tenant_uuid ON organizations (tenant_uuid);

ALTER TABLE audit_entries ADD COLUMN tenant_uuid CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001';
CREATE INDEX ix_audit_entries_tenant_uuid ON audit_entries (tenant_uuid);
