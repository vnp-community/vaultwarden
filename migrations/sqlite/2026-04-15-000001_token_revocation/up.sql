-- TASK-SEC-HIGH-02-D: Revoked tokens table for opt-in JWT revocation.
-- Only needed when TOKEN_REVOCATION_ENABLED=true.
-- The jti (JWT ID) is a UUID that uniquely identifies each issued token.
-- revoked_at: when the token was explicitly revoked
-- expires_at: mirrors the JWT exp claim; used for cleanup job (HIGH-02-G)
CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti        TEXT NOT NULL PRIMARY KEY,
    user_uuid  TEXT NOT NULL REFERENCES users(uuid) ON DELETE CASCADE,
    revoked_at DATETIME NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%f', 'NOW')),
    expires_at DATETIME NOT NULL
);

-- Index for the daily cleanup job (HIGH-02-G): delete WHERE expires_at < NOW()
CREATE INDEX IF NOT EXISTS idx_revoked_tokens_expires_at ON revoked_tokens (expires_at);
