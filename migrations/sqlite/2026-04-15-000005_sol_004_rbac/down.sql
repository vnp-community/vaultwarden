-- SQLite does not support DROP COLUMN cleanly in old versions, but modern sqlite does
-- vaultwarden usually drops columns using standard ALTER TABLE DROP COLUMN since diesel covers sqlite 3.35+
ALTER TABLE users_organizations DROP COLUMN custom_role_uuid;

DROP TABLE sod_rules;
DROP TABLE approval_requests;
DROP TABLE break_glass_configs;
DROP TABLE ip_allowlists;
DROP TABLE access_schedules;
DROP TABLE custom_roles;
