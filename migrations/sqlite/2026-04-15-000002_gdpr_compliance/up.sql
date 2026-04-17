-- TASK-001-005: GDPR Erasure Pipeline — SQLite migration
-- Creates erasure_logs (append-only audit chain with SHA-256 prev_hash linkage)
-- and adds PII erasure scheduling columns to users table.

CREATE TABLE IF NOT EXISTS erasure_logs (
    uuid          TEXT NOT NULL PRIMARY KEY,
    user_uuid     TEXT NOT NULL,               -- May reference deleted users; no FK ON DELETE CASCADE
    requested_at  DATETIME NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%f', 'NOW')),
    scheduled_at  DATETIME NOT NULL,           -- When PII will/was erased (requested_at + GDPR_ERASURE_DELAY_DAYS)
    completed_at  DATETIME,                    -- NULL until erasure actually runs
    requestor_ip  TEXT NOT NULL DEFAULT '',    -- IP of user who requested erasure
    prev_hash     TEXT NOT NULL DEFAULT '',    -- SHA-256 of previous entry for hash chain integrity
    entry_hash    TEXT NOT NULL DEFAULT ''     -- SHA-256 of this entry (computed after insert)
);

-- Columns for scheduling PII erasure on users
ALTER TABLE users ADD COLUMN pii_erasure_scheduled_at DATETIME;
ALTER TABLE users ADD COLUMN pii_erased_at DATETIME;
