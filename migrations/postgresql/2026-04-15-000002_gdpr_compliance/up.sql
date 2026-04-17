-- TASK-001-005: GDPR Erasure Pipeline — PostgreSQL migration
CREATE TABLE IF NOT EXISTS erasure_logs (
    uuid          TEXT        NOT NULL PRIMARY KEY,
    user_uuid     TEXT        NOT NULL,
    requested_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scheduled_at  TIMESTAMPTZ NOT NULL,
    completed_at  TIMESTAMPTZ,
    requestor_ip  TEXT        NOT NULL DEFAULT '',
    prev_hash     TEXT        NOT NULL DEFAULT '',
    entry_hash    TEXT        NOT NULL DEFAULT ''
);

ALTER TABLE users ADD COLUMN IF NOT EXISTS pii_erasure_scheduled_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS pii_erased_at TIMESTAMPTZ;
