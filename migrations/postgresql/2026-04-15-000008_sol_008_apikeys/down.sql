ALTER TABLE ciphers DROP COLUMN secret_project;
ALTER TABLE ciphers DROP COLUMN is_secret;
DROP TABLE webhook_deliveries;
DROP TABLE webhooks;
DROP TABLE api_key_usage;
DROP TABLE api_keys_v2;
