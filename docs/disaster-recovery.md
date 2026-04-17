# Disaster Recovery Runbook: PostgreSQL WAL Archiving

This document describes how to configure your external PostgreSQL database to archive Write-Ahead Logs (WAL) for Vaultwarden's deep PITR (Point-In-Time Recovery) subsystem.

## Context
Vaultwarden automatically dispatches custom-formatted logical backups using `pg_dump`. However, purely logical exports only capture the database exact state exactly at the chronometer tick of the operation. Hardware failures or targeted corruptions spanning outside the chronometer window will suffer RPO (Recovery Point Objective) losses.

Therefore, enabling WAL Archiving lets administrators roll the cluster backwards to an exact minute and second.

## 1. Primary PostgreSQL Verification
Ensure your upstream instance is configured appropriately. Inside `postgresql.conf`:

```ini
wal_level = replica
archive_mode = on
# Example: Send the segment to S3 using AWS CLI or pgBackRest
archive_command = 'aws s3 cp %p s3://my-vaultwarden-dr-bucket/wal/%f'
archive_timeout = 60
```

*Note: Restart PostgreSQL after altering `wal_level` or `archive_mode`.*

## 2. Vaultwarden Integration Configuration
Set the following properties in your Vaultwarden `.env` file (or orchestrator configuration overlay) to activate the WAL integrations logic inside `BackupManager`:

```env
BACKUP_WAL_ARCHIVE_ENABLED=true
BACKUP_WAL_ARCHIVE_DESTINATION="s3://my-vaultwarden-dr-bucket/wal"
BACKUP_PITR_ENABLED=true
BACKUP_PITR_RETENTION_HOURS=72
```

## 3. Recovery Protocol
If an incident destroys the current storage slice:
1. Initialize a vanilla Postgres instance from your latest `pg_dump` blob (`BackupManager` custom format).
2. Apply the relevant `recovery.conf` directing to your `s3://my-vaultwarden-dr-bucket/wal` directory.
3. Boot the cluster and let the replay parser inject the historical metrics.
4. Update Vaultwarden's `DATABASE_URL` routing the backend to your new instance.
