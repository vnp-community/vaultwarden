ALTER TABLE devices ADD COLUMN is_trusted BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE devices ADD COLUMN mdm_enrolled BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE devices ADD COLUMN mdm_compliant BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE devices ADD COLUMN mdm_last_check_at DATETIME;
ALTER TABLE devices ADD COLUMN cert_subject TEXT;
ALTER TABLE devices ADD COLUMN cert_serial TEXT;
ALTER TABLE devices ADD COLUMN cert_expires_at DATETIME;
ALTER TABLE devices ADD COLUMN cert_issuer TEXT;

CREATE TABLE device_trust_policies (
    uuid TEXT NOT NULL PRIMARY KEY,
    org_uuid TEXT NOT NULL,
    require_device_cert BOOLEAN NOT NULL DEFAULT 0,
    require_managed_device BOOLEAN NOT NULL DEFAULT 0,
    allowed_cert_issuers TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY(org_uuid) REFERENCES organizations(uuid)
);
CREATE UNIQUE INDEX ix_device_trust_policies_org_uuid ON device_trust_policies (org_uuid);

CREATE TABLE mdm_compliance_cache (
    device_uuid TEXT NOT NULL PRIMARY KEY,
    org_uuid TEXT NOT NULL,
    is_compliant BOOLEAN NOT NULL,
    raw_status TEXT,
    checked_at DATETIME NOT NULL,
    FOREIGN KEY(device_uuid) REFERENCES devices(uuid),
    FOREIGN KEY(org_uuid) REFERENCES organizations(uuid)
);
