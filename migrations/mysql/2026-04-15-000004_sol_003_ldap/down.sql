ALTER TABLE organizations DROP COLUMN ldap_group_dn;
ALTER TABLE users DROP COLUMN suspension_scheduled_at;
ALTER TABLE users DROP COLUMN provisioning_external_id;
ALTER TABLE users DROP COLUMN provisioning_source;

DROP TABLE scim_tokens;
DROP TABLE access_review_items;
DROP TABLE access_reviews;
DROP TABLE ldap_group_mappings;
DROP TABLE ldap_sync_state;
