# SOL-006: Giải Pháp Thực Hiện — Disaster Recovery & Business Continuity

> **Giải pháp cho**: CR-006  
> **Ngày**: 2026-04-12  
> **Trạng thái**: ✅ Implemented  
> **Kiến trúc thay đổi**: Tối thiểu — thêm backup subsystem, không thay đổi core  
> **Cập nhật**: 2026-04-17 — Verified full implementation in codebase

---

## 1. Tổng Quan Giải Pháp

Vaultwarden hiện có SQLite backup cơ bản (SIGUSR1 + `SEND_PURGE_SCHEDULE`). Giải pháp **mở rộng** với:

1. **Backup module** mới `src/backup.rs` — orchestrate pg_dump, S3 upload, verification
2. **Job scheduler** hiện có — thêm backup và verification jobs
3. **OpenDAL** hiện có — dùng cho S3 upload
4. **Admin API** mới cho backup management
5. **Backup manifest** với SHA-256 và digital signature

---

## 2. Thay Đổi Kiến Trúc

### 2.1 Modules Mới

| File | Mô tả |
|------|-------|
| `src/backup.rs` | Backup orchestration (pg_dump, WAL, S3 upload, verification) |
| `src/api/admin/backup.rs` | Admin REST API cho backup management |

### 2.2 Files Hiện Có Cần Sửa

| File | Thay đổi |
|------|---------|
| `src/main.rs` | Thêm backup + verification jobs vào scheduler |
| `src/config.rs` | Thêm BACKUP_* config keys |
| `src/api/admin.rs` | Mount backup API routes |
| `src/db/mod.rs` | Expose record count queries cho backup manifest |

### 2.3 Database Migration

```sql
-- migrations/postgresql/YYYYMMDD_backup/up.sql

CREATE TABLE backup_runs (
    id              VARCHAR(40) PRIMARY KEY,    -- 'bkp-YYYYMMDD-HHMMSS'
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    status          VARCHAR(20) NOT NULL DEFAULT 'running',  -- running, success, failed
    backup_type     VARCHAR(20) NOT NULL,       -- pg_dump, wal_archive, sqlite_copy
    destination     TEXT NOT NULL,              -- S3 path or local path
    size_bytes      BIGINT,
    sha256          VARCHAR(64),
    manifest_json   JSONB,
    error_message   TEXT,
    
    -- Verification
    verified_at     TIMESTAMPTZ,
    verification_status VARCHAR(20),            -- passed, failed, pending
    verification_error  TEXT
);

CREATE INDEX idx_backup_runs_started ON backup_runs(started_at DESC);
```

---

## 3. Thiết Kế Chi Tiết

### 3.1 Backup Orchestration

**File**: `src/backup.rs`

```rust
pub struct BackupManager {
    config: BackupConfig,
    storage: Operator,          // OpenDAL — đã có trong codebase
}

impl BackupManager {
    pub async fn run_backup(&self, conn: &DbConn) -> Result<BackupRun, Error> {
        let backup_id = format!("bkp-{}", Utc::now().format("%Y%m%d-%H%M%S"));
        
        // Tạo backup run record
        let run = BackupRun {
            id: backup_id.clone(),
            backup_type: CONFIG.backup_type().to_string(),
            destination: self.destination_path(&backup_id),
            ..Default::default()
        };
        run.insert(conn).await?;
        
        let result = match CONFIG.backup_type() {
            "pg_dump"       => self.run_pg_dump(&backup_id, conn).await,
            "pg_basebackup" => self.run_pg_basebackup(&backup_id).await,
            "sqlite_copy"   => self.run_sqlite_copy(&backup_id).await,
            "mysqldump"     => self.run_mysqldump(&backup_id).await,
            t => Err(Error::new(&format!("Unknown backup type: {t}"), "")),
        };
        
        match result {
            Ok(backup_data) => {
                // Tính SHA-256
                let sha256 = compute_sha256(&backup_data);
                
                // Tạo manifest
                let manifest = self.create_manifest(&backup_id, &sha256, conn).await?;
                
                // Upload backup + manifest lên S3
                self.upload_backup(&backup_id, &backup_data).await?;
                self.upload_manifest(&backup_id, &manifest).await?;
                
                // Nếu có secondary destination
                if CONFIG.backup_cross_region_enabled() {
                    self.replicate_to_secondary(&backup_id).await.ok();
                }
                
                // Update record
                BackupRun::mark_success(&backup_id, sha256, manifest, conn).await?;
                
                // Audit log
                audit::emit(AuditEntry {
                    event_type: AuditEventType::BackupCompleted,
                    metadata: json!({
                        "backup_id": backup_id,
                        "size_bytes": backup_data.len(),
                        "sha256": sha256,
                    }),
                    ..Default::default()
                });
                
                BackupRun::find_by_id(&backup_id, conn).await
            }
            Err(e) => {
                BackupRun::mark_failed(&backup_id, &e.to_string(), conn).await.ok();
                
                // Alert email
                if !CONFIG.backup_verify_alert_email().is_empty() {
                    mail::send_backup_failure_alert(
                        CONFIG.backup_verify_alert_email(),
                        &backup_id, &e.to_string()
                    ).await.ok();
                }
                
                // SIEM event
                audit::emit(AuditEntry {
                    event_type: AuditEventType::BackupFailed,
                    severity: Severity::Critical,
                    metadata: json!({"backup_id": backup_id, "error": e.to_string()}),
                    ..Default::default()
                });
                
                Err(e)
            }
        }
    }
    
    async fn run_pg_dump(&self, backup_id: &str, conn: &DbConn) -> Result<Vec<u8>, Error> {
        // Sử dụng std::process::Command để gọi pg_dump
        let db_url = CONFIG.database_url();
        
        let output = tokio::process::Command::new("pg_dump")
            .arg("--format=custom")
            .arg("--no-password")
            .arg("--compress=6")
            .arg(&db_url)
            .output()
            .await
            .map_err(|e| Error::new(&format!("pg_dump failed: {e}"), ""))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::new(&format!("pg_dump error: {stderr}"), ""));
        }
        
        // Optional: encrypt backup
        if !CONFIG.backup_encryption_key_id().is_empty() {
            return self.encrypt_backup(output.stdout).await;
        }
        
        Ok(output.stdout)
    }
    
    async fn create_manifest(&self, backup_id: &str, sha256: &str, conn: &DbConn) 
        -> Result<BackupManifest, Error> 
    {
        let counts = conn.run(|c| {
            (
                c.query_row("SELECT COUNT(*) FROM users", [], |r| r.get::<_, i64>(0)),
                c.query_row("SELECT COUNT(*) FROM ciphers", [], |r| r.get::<_, i64>(0)),
                c.query_row("SELECT COUNT(*) FROM organizations", [], |r| r.get::<_, i64>(0)),
            )
        }).await;
        
        let manifest = BackupManifest {
            backup_id: backup_id.to_string(),
            timestamp: Utc::now(),
            db_version: get_schema_version(conn).await?,
            record_counts: RecordCounts {
                users: counts.0.unwrap_or(0),
                ciphers: counts.1.unwrap_or(0),
                organizations: counts.2.unwrap_or(0),
            },
            sha256: sha256.to_string(),
            backup_type: CONFIG.backup_type().to_string(),
            // Ký manifest bằng server RSA key
            signature: self.sign_manifest(&sha256).await?,
            verified_at: None,
            verification_status: "pending".to_string(),
        };
        
        Ok(manifest)
    }
}
```

### 3.2 Automated Backup Verification

```rust
pub async fn verify_backup(backup_id: &str, conn: &DbConn) -> Result<VerificationResult, Error> {
    // 1. Download backup từ S3
    let backup_data = download_from_storage(backup_id).await?;
    
    // 2. Verify SHA-256 checksum
    let actual_sha256 = compute_sha256(&backup_data);
    let manifest = BackupManifest::load(backup_id).await?;
    
    if actual_sha256 != manifest.sha256 {
        return Ok(VerificationResult {
            passed: false,
            error: Some("SHA-256 checksum mismatch — backup may be corrupted".to_string()),
        });
    }
    
    // 3. Restore vào ephemeral test database
    // Dùng temporary PostgreSQL schema hoặc SQLite in-memory
    let test_db = create_ephemeral_test_db().await?;
    
    let restore_result = tokio::process::Command::new("pg_restore")
        .arg("--dbname").arg(&test_db.connection_string)
        .arg("--no-password")
        .stdin(std::process::Stdio::piped())
        .output()
        .await?;
    
    if !restore_result.status.success() {
        return Ok(VerificationResult {
            passed: false,
            error: Some(format!("Restore failed: {}", 
                String::from_utf8_lossy(&restore_result.stderr))),
        });
    }
    
    // 4. Verify record counts khớp với manifest
    let actual_counts = query_test_db_counts(&test_db).await?;
    
    let counts_match = actual_counts.users == manifest.record_counts.users
        && actual_counts.ciphers == manifest.record_counts.ciphers
        && actual_counts.organizations == manifest.record_counts.organizations;
    
    // 5. Cleanup ephemeral DB
    test_db.drop().await.ok();
    
    // 6. Log verification result
    BackupRun::update_verification(backup_id, counts_match, conn).await?;
    
    audit::emit(AuditEntry {
        event_type: if counts_match { 
            AuditEventType::BackupVerificationPassed 
        } else { 
            AuditEventType::BackupVerificationFailed 
        },
        severity: if counts_match { Severity::Info } else { Severity::Critical },
        metadata: json!({
            "backup_id": backup_id,
            "sha256_ok": true,
            "counts_match": counts_match,
            "expected": manifest.record_counts,
            "actual": actual_counts,
        }),
        ..Default::default()
    });
    
    // Alert nếu failed
    if !counts_match && !CONFIG.backup_verify_alert_email().is_empty() {
        mail::send_backup_verification_failure(
            CONFIG.backup_verify_alert_email(),
            backup_id
        ).await.ok();
    }
    
    Ok(VerificationResult { passed: counts_match, error: None })
}
```

### 3.3 Admin Backup API

```rust
// POST /api/admin/backup/trigger
#[post("/admin/backup/trigger")]
async fn trigger_backup(
    _admin: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    if !CONFIG.backup_enabled() {
        err!("Backup is not configured");
    }
    
    let manager = BackupManager::from_config();
    let run = manager.run_backup(&conn).await?;
    
    Ok(Json(json!({
        "backup_id": run.id,
        "status": run.status,
        "started_at": run.started_at,
    })))
}

// POST /api/admin/backup/verify
#[post("/admin/backup/verify", data = "<body>")]
async fn trigger_verification(
    body: Json<VerifyRequest>,
    _admin: AdminHeaders,
    conn: DbConn,
) -> JsonResult {
    let backup_id = body.backup_id.as_deref()
        .unwrap_or_else(|| "latest");  // Mặc định verify latest backup
    
    let actual_id = if backup_id == "latest" {
        BackupRun::find_latest_successful(&conn).await?
            .ok_or_else(|| Error::new("No successful backup found", ""))?
            .id
    } else {
        backup_id.to_string()
    };
    
    let result = verify_backup(&actual_id, &conn).await?;
    
    Ok(Json(json!({
        "backup_id": actual_id,
        "passed": result.passed,
        "error": result.error,
    })))
}

// GET /api/admin/backup/status
#[get("/admin/backup/status")]
async fn backup_status(_admin: AdminHeaders, conn: DbConn) -> JsonResult {
    let latest = BackupRun::find_latest(&conn).await?;
    let latest_verified = BackupRun::find_latest_verified(&conn).await?;
    
    Ok(Json(json!({
        "latest_backup": latest,
        "latest_verified_backup": latest_verified,
        "backup_enabled": CONFIG.backup_enabled(),
        "backup_schedule": CONFIG.backup_schedule(),
        "next_backup_at": calculate_next_cron(CONFIG.backup_schedule()),
        "retention_days": CONFIG.backup_retention_days(),
    })))
}

// GET /api/admin/dr-runbook?format=html|json
#[get("/admin/dr-runbook?<format>")]
async fn dr_runbook(
    format: Option<&str>,
    _admin: AdminHeaders,
    conn: DbConn,
) -> Result<String, Error> {
    let latest_backup = BackupRun::find_latest_successful(&conn).await?.ok_or_else(
        || Error::new("No backup available", "")
    )?;
    
    let runbook_data = json!({
        "generated_at": Utc::now().to_rfc3339(),
        "deployment": {
            "database_type": detect_db_type(),
            "backup_location": CONFIG.backup_destination(),
            "backup_encryption": !CONFIG.backup_encryption_key_id().is_empty(),
            "ha_mode": CONFIG.cluster_mode(),
        },
        "latest_backup": {
            "id": latest_backup.id,
            "timestamp": latest_backup.completed_at,
            "sha256": latest_backup.sha256,
            "verified": latest_backup.verification_status == "passed",
        },
        "restore_steps": generate_restore_steps(),
        "contacts": [CONFIG.backup_verify_alert_email()],
    });
    
    match format.unwrap_or("json") {
        "json" => Ok(serde_json::to_string_pretty(&runbook_data).unwrap()),
        _ => Ok(render_runbook_html(&runbook_data)),
    }
}
```

---

## 4. Job Scheduler Integration

Thêm vào `src/main.rs`:

```rust
// Backup job
if CONFIG.backup_enabled() {
    sched.add(Job::new(CONFIG.backup_schedule(), |_, _| {
        tokio::spawn(async move {
            let pool = DB_POOL.get().expect("pool");
            let conn = pool.get().expect("conn");
            let manager = BackupManager::from_config();
            manager.run_backup(&conn).await.ok();
        });
    })?)?;
}

// Verification job
if CONFIG.backup_verify_enabled() {
    sched.add(Job::new(CONFIG.backup_verify_schedule(), |_, _| {
        tokio::spawn(async move {
            let pool = DB_POOL.get().expect("pool");
            let conn = pool.get().expect("conn");
            if let Some(latest) = BackupRun::find_latest_successful(&conn).await.ok().flatten() {
                verify_backup(&latest.id, &conn).await.ok();
            }
        });
    })?)?;
}
```

---

## 5. Config Variables Mới

```bash
# Core backup
BACKUP_ENABLED=false
BACKUP_TYPE=pg_dump                 # pg_dump|pg_basebackup|sqlite_copy|mysqldump
BACKUP_DESTINATION=s3://bucket/vaultwarden/
BACKUP_S3_REGION=ap-southeast-1
BACKUP_SCHEDULE=0 */1 * * *        # Hourly
BACKUP_RETENTION_DAYS=30
BACKUP_ENCRYPTION_KEY_ID=""        # KMS key ARN (optional)

# WAL Archiving (PostgreSQL)
BACKUP_WAL_ARCHIVE_ENABLED=false
BACKUP_WAL_ARCHIVE_DESTINATION=s3://bucket/vaultwarden/wal/

# Verification
BACKUP_VERIFY_ENABLED=false
BACKUP_VERIFY_SCHEDULE=0 2 * * *   # Daily at 2am
BACKUP_VERIFY_ALERT_EMAIL=""
BACKUP_VERIFY_TIMEOUT_SECONDS=3600

# PITR
BACKUP_PITR_ENABLED=false
BACKUP_PITR_RETENTION_HOURS=168    # 7 days

# Multi-region
BACKUP_CROSS_REGION_ENABLED=false
BACKUP_SECONDARY_DESTINATION=""
BACKUP_SECONDARY_REGION=""
```

---

## 6. Phụ Thuộc Mới

| Crate | Phiên bản | Lý do |
|-------|-----------|-------|
| Không có | - | pg_dump/pg_restore là external binaries, gọi via `tokio::process::Command` |

> `opendal` đã có sẵn cho S3 upload.  
> `sha2` đã dùng trong CR-002 solution.  
> `openssl`/`ring` đã có sẵn cho manifest signing.

---

## 7. Kế Hoạch Triển Khai

### Sprint 1–2: Core Backup (pg_dump + S3)
- `src/backup.rs` — BackupManager
- DB migration cho `backup_runs`
- S3 upload via OpenDAL
- Admin API: trigger, status

### Sprint 3–4: WAL Archiving
- WAL archive configuration
- PITR documentation

### Sprint 5–7: Verification Pipeline
- Ephemeral test DB approach
- Restore + count verification
- Alert system

### Sprint 8: Manifest + Signing
- SHA-256 + RSA signature
- Manifest JSON format

### Sprint 9: DR Runbook
- Runbook generator
- Multi-region replication

---

*Status: ✅ Implemented | Ngày cập nhật: 2026-04-17*

## Implementation Notes
- `src/backup.rs` (275 lines) — BackupManager: pg_dump, SQLite copy, S3 upload via OpenDAL, manifest + SHA-256
- `src/api/admin/backup.rs` — Admin API: trigger, verify, status, DR runbook endpoints
- `src/db/models/backup_run.rs` — BackupRun model
- DB migration: `2026-04-15-000006_sol_006_backup` — backup_runs table with verification tracking
- Tests: `tests/backup_tests.rs` (67 lines)
- Automated verification job: pg_restore to ephemeral schema, record count check
- Alert email on backup failure / verification failure
- Backup + verification cron jobs registered in `src/main.rs`
