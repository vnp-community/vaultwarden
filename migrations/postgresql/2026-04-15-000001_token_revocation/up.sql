-- TASK-SEC-HIGH-02-D: Revoked tokens table for opt-in JWT revocation.
-- Only needed when TOKEN_REVOCATION_ENABLED=true.
CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti        VARCHAR(40)  NOT NULL PRIMARY KEY,
    user_uuid  VARCHAR(40)  NOT NULL REFERENCES users(uuid) ON DELETE CASCADE,
    revoked_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ  NOT NULL
);

-- Index for the daily cleanup job (HIGH-02-G)
CREATE INDEX IF NOT EXISTS idx_revoked_tokens_expires_at ON revoked_tokens (expires_at);
