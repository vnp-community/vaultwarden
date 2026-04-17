# CR-006: Disaster Recovery & Business Continuity Planning

> **Change Request ID**: CR-006  
> **Title**: Disaster Recovery, RTO/RPO Guarantees & Automated Backup Verification  
> **Priority**: P1 — Critical  
> **Target Release**: v2.0  
> **Driven By**: [specs/crs/product-market-analysis.md §2.3 RTO/RPO]  
> **Affects**: PRD §9.3, URD §4.8, SRS §5.3

---

## 1. Problem Statement

- Không có formal DR plan hoặc documented RTO/RPO
- SQLite backup chỉ là point-in-time, không có WAL-based streaming
- PostgreSQL/MySQL replication là database-level concern không được document trong product
- Không có backup verification — không biết backup có restore được không
- Banking yêu cầu: RTO ≤ 4 giờ, RPO ≤ 1 giờ (thường là yêu cầu tối thiểu)

---

## 2. Scope of Change

### 2.1 Backup Architecture

#### PostgreSQL (Production Recommended)

```
NEW CONFIG:
BACKUP_ENABLED=true
BACKUP_TYPE=pg_basebackup|pg_dump|wal_archive
BACKUP_DESTINATION=s3://backup-bucket/vaultwarden/
BACKUP_S3_REGION=ap-southeast-1
BACKUP_SCHEDULE=0 */1 * * *              # Hourly
BACKUP_RETENTION_DAYS=30
BACKUP_ENCRYPTION_KEY_ID=<KMS key>       # Encrypt backups at rest
BACKUP_VERIFY_SCHEDULE=0 2 * * *         # Daily verification at 2am
```

**WAL archiving support** (PostgreSQL continuous archiving):
```
BACKUP_WAL_ARCHIVE_ENABLED=true
BACKUP_WAL_ARCHIVE_DESTINATION=s3://backup-bucket/vaultwarden/wal/
```

RPO với WAL archiving: **< 5 phút** (standard WAL archive interval)

#### MySQL/MariaDB

```
BACKUP_TYPE=mysqldump|xtrabackup
```

#### SQLite

```
BACKUP_TYPE=sqlite_copy                  # Existing SIGUSR1 mechanism
```

### 2.2 Automated Backup Verification

Đây là tính năng quan trọng nhất — ngân hàng không tin backup nếu không test restore.

**Backup verification pipeline**:
```
Every BACKUP_VERIFY_SCHEDULE:
    1. Download latest backup from storage
    2. Restore to ephemeral test database (in-memory or temp container)
    3. Run verification queries:
       - User count matches pre-backup snapshot
       - Cipher count matches
       - Organization count matches
       - Schema version matches
    4. Log verification result to audit trail
    5. Alert if verification fails (email + SIEM event)
    6. Destroy ephemeral test database
```

**Config**:
```
NEW CONFIG:
BACKUP_VERIFY_ENABLED=true
BACKUP_VERIFY_SCHEDULE=0 2 * * *
BACKUP_VERIFY_ALERT_EMAIL=ops@example.com
BACKUP_VERIFY_TIMEOUT_SECONDS=3600
```

**API**:
```
POST /api/admin/backup/trigger           # Trigger manual backup
POST /api/admin/backup/verify            # Trigger manual verification
GET  /api/admin/backup/status            # Latest backup status + hash
GET  /api/admin/backup/history           # List of backup runs + verification results
```

### 2.3 Point-In-Time Recovery (PITR)

Cho phép restore đến bất kỳ điểm nào trong quá khứ (trong retention window):

**Config**:
```
BACKUP_PITR_ENABLED=true                 # Requires WAL archiving
BACKUP_PITR_RETENTION_HOURS=168          # 7 days of PITR
```

**Admin endpoint**:
```
POST /api/admin/backup/restore
{
  "target_time": "2026-04-11T14:30:00Z",
  "confirm": true,
  "justification": "Ransomware recovery - ticket #INC-20260411-001"
}
```

### 2.4 Multi-Region / Geo-Redundant Backup

```
BACKUP_SECONDARY_DESTINATION=s3://backup-bucket-dr/vaultwarden/
BACKUP_SECONDARY_REGION=ap-northeast-1   # Different region from primary
BACKUP_CROSS_REGION_ENABLED=true
```

### 2.5 RTO/RPO SLA Documentation

| Deployment Mode | RTO | RPO | Mechanism |
|----------------|-----|-----|-----------|
| Single SQLite (development) | Best effort | 24h | Manual backup |
| PostgreSQL + hourly backup | < 4h | 1h | pg_dump + S3 |
| PostgreSQL + WAL archiving | < 1h | 5min | WAL + base backup |
| PostgreSQL + Streaming Replication + WAL | < 15min | Near-zero | Standby + WAL |
| HA Cluster (CR-005) + Streaming Replication | < 5min | Near-zero | Hot standby |

### 2.6 Disaster Recovery Runbook (Template)

Documentation template generated at:
```
GET /api/admin/dr-runbook?format=pdf|html
```

Includes:
- Current deployment topology
- Backup location and encryption key references
- Step-by-step restore procedure
- Verification checklist
- Contact list (auto-populated from admin config)
- Last successful backup timestamp and hash

### 2.7 Backup Integrity Verification

Mỗi backup kèm theo:
- SHA-256 checksum file
- Digital signature (signed với server's RSA key)
- Manifest JSON: backup timestamp, source DB version, record counts

```json
{
  "backup_id": "bkp-20260412-020000",
  "timestamp": "2026-04-12T02:00:00Z",
  "db_version": "20260101000000",
  "record_counts": {
    "users": 1247,
    "ciphers": 45231,
    "organizations": 23
  },
  "sha256": "a1b2c3...",
  "verified_at": "2026-04-12T02:15:00Z",
  "verification_status": "passed"
}
```

---

## 3. Acceptance Criteria

- [ ] Automated backup runs on schedule; failure triggers alert email within 5 minutes
- [ ] Backup verification restores and validates successfully in < 60 minutes
- [ ] Failed backup verification generates SIEM event (CR-002)
- [ ] WAL archiving achieves RPO < 5 minutes (measured in load test)
- [ ] DR runbook PDF generated with accurate current configuration
- [ ] Backup manifest SHA-256 matches downloaded file checksum
- [ ] Multi-region backup replicates to secondary location within 15 minutes

---

## 4. Estimated Effort

| Area | Effort |
|------|--------|
| Backup scheduler (pg_dump, S3) | 2 sprints |
| WAL archiving integration | 2 sprints |
| Backup verification pipeline | 3 sprints |
| PITR admin API | 2 sprints |
| Backup integrity signing | 1 sprint |
| DR runbook generator | 1 sprint |
| Multi-region replication | 1 sprint |

---

*Status: ✅ Implemented | Author: Product Team | Date: 2026-04-12 | Cập nhật: 2026-04-17*

> **Implementation**: [SOL-006](solutions/SOL-006-disaster-recovery.md) — `src/backup.rs` (275L), `src/api/admin/backup.rs`, `src/db/models/backup_run.rs`, DB migration `2026-04-15-000006_sol_006_backup`, tests `backup_tests.rs` (67L)
