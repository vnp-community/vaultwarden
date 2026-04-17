ALTER TABLE users DROP COLUMN tenant_uuid;
ALTER TABLE organizations DROP COLUMN tenant_uuid;
ALTER TABLE audit_entries DROP COLUMN tenant_uuid;

DROP TABLE tenant_admins;
DROP TABLE tenants;
