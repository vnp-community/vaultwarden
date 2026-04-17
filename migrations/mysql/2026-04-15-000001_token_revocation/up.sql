-- TASK-SEC-HIGH-02-D: Revoked tokens table for opt-in JWT revocation.
-- Only needed when TOKEN_REVOCATION_ENABLED=true.
CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti        VARCHAR(40)  NOT NULL PRIMARY KEY,
    user_uuid  VARCHAR(40)  NOT NULL,
    revoked_at DATETIME(6)  NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    expires_at DATETIME(6)  NOT NULL,
    CONSTRAINT fk_revoked_tokens_user FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
);

-- Index for the daily cleanup job (HIGH-02-G)
CREATE INDEX idx_revoked_tokens_expires_at ON revoked_tokens (expires_at);
