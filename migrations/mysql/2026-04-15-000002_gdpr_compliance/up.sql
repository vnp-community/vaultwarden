-- TASK-001-005: GDPR Erasure Pipeline — MySQL migration
CREATE TABLE IF NOT EXISTS erasure_logs (
    uuid          VARCHAR(36)  NOT NULL PRIMARY KEY,
    user_uuid     VARCHAR(36)  NOT NULL,
    requested_at  DATETIME(6)  NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    scheduled_at  DATETIME(6)  NOT NULL,
    completed_at  DATETIME(6),
    requestor_ip  VARCHAR(50)  NOT NULL DEFAULT '',
    prev_hash     VARCHAR(64)  NOT NULL DEFAULT '',
    entry_hash    VARCHAR(64)  NOT NULL DEFAULT ''
) ENGINE=InnoDB CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

ALTER TABLE users ADD COLUMN pii_erasure_scheduled_at DATETIME(6);
ALTER TABLE users ADD COLUMN pii_erased_at DATETIME(6);
