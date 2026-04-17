ALTER TABLE ciphers DROP COLUMN privileged_config_uuid;
ALTER TABLE ciphers DROP COLUMN is_privileged;
DROP TABLE rotation_history;
DROP TABLE checkouts;
DROP TABLE privileged_configs;
