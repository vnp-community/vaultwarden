CREATE TABLE backup_runs (
    id VARCHAR(64) PRIMARY KEY,
    started_at DATETIME NOT NULL,
    completed_at DATETIME,
    status VARCHAR(32) NOT NULL,
    backup_type VARCHAR(32) NOT NULL,
    destination VARCHAR(128) NOT NULL,
    size_bytes BIGINT,
    sha256 VARCHAR(64),
    manifest_json TEXT,
    error_message TEXT,
    verified_at DATETIME,
    verification_status VARCHAR(32),
    verification_error TEXT
);
CREATE INDEX ix_backup_runs_started_at ON backup_runs (started_at);
