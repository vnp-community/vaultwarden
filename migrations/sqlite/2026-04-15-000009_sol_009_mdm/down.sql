ALTER TABLE devices DROP COLUMN is_trusted;
ALTER TABLE devices DROP COLUMN mdm_enrolled;
ALTER TABLE devices DROP COLUMN mdm_compliant;
ALTER TABLE devices DROP COLUMN mdm_last_check_at;
ALTER TABLE devices DROP COLUMN cert_subject;
ALTER TABLE devices DROP COLUMN cert_serial;
ALTER TABLE devices DROP COLUMN cert_expires_at;
ALTER TABLE devices DROP COLUMN cert_issuer;

DROP TABLE mdm_compliance_cache;
DROP TABLE device_trust_policies;
