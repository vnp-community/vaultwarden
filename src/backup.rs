// SOL-006 Scaffold: Disaster Recovery and Backup Manager
use std::process::Stdio;
use tokio::process::Command;
use chrono::Utc;
use sha2::{Sha256, Digest};
use opendal::{Operator, services};
use serde_json::json;

use crate::db::DbConn;
use crate::error::Error;

pub struct BackupManager {
    pub enabled: bool,
    pub backup_destination: String,
    pub backup_type: String, // e.g. pg_dump
}

impl BackupManager {
    pub fn new() -> Self {
        Self {
            enabled: crate::CONFIG.backup_enabled(),
            backup_destination: crate::CONFIG.backup_destination(),
            backup_type: crate::CONFIG.backup_type(),
        }
    }

    pub async fn run_backup(&self, _conn: &mut DbConn) -> Result<(), Error> {
        if !self.enabled {
            return Ok(());
        }

        // Generate ID
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        let backup_id = format!("bkp-{}", timestamp);
        
        info!("Starting backup run: {}", backup_id);

        // TASK-006-004 / 005: Execute database dump
        let dump_bytes = match self.backup_type.as_str() {
            "pg_dump" => self.run_pg_dump().await?,
            "sqlite" => self.run_sqlite_copy().await?,
            "mysql" => self.run_mysqldump().await?,
            _ => return Err(Error::new("Unsupported backup_type", "")),
        };

        // Calculate SHA256 (TASK-006-006)
        let mut hasher = Sha256::new();
        hasher.update(&dump_bytes);
        let hash = data_encoding::HEXLOWER.encode(&hasher.finalize());

        let size = dump_bytes.len();

        let operator = self.create_operator().await?;
        let backup_filename = format!("{}.sqldump", backup_id);
        
        // Upload backup (TASK-006-006)
        operator.write(&backup_filename, dump_bytes.clone()).await
            .map_err(|e| Error::new("Upload failed", e.to_string()))?;

        // Manifest Generation (TASK-006-006 & 013)
        let signature = self.sign_manifest(&hash);
        let manifest = json!({
            "version": "1.0",
            "backup_id": backup_id,
            "type": self.backup_type,
            "size": size,
            "sha256": hash,
            "signature": signature,
            "timestamp": timestamp.to_string()
        });
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        let manifest_filename = format!("{}.manifest.json", backup_id);
        
        operator.write(&manifest_filename, manifest_bytes).await
            .map_err(|e| Error::new("Manifest upload failed", e.to_string()))?;

        info!("Backup {} completed and uploaded successfully.", backup_id);
        
        // Replicate to secondary (TASK-006-012)
        if crate::CONFIG.backup_cross_region_enabled() && !crate::CONFIG.backup_secondary_destination().is_empty() {
            if let Err(e) = self.replicate_to_secondary(&backup_filename, &dump_bytes).await {
                error!("Secondary backup replication failed: {}", e);
            }
            if let Err(e) = self.replicate_to_secondary(&manifest_filename, &serde_json::to_vec(&manifest).unwrap()).await {
                error!("Secondary manifest replication failed: {}", e);
            }
        }
        
        Ok(())
    }

    // TASK-006-013: Manifest digital signature
    fn sign_manifest(&self, hash: &str) -> Option<String> {
        let key_path = "data/backup_signing_key.pem";
        if let Ok(pem) = std::fs::read(key_path) {
            use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};
            #[derive(serde::Serialize)]
            struct Claims { hash: String }
            let claims = Claims { hash: hash.to_string() };
            if let Ok(key) = EncodingKey::from_rsa_pem(&pem) {
                return encode(&Header::new(Algorithm::RS256), &claims, &key).ok();
            }
        }
        None
    }

    // TASK-006-012: Cross-region replication
    #[cfg(feature = "s3")]
    async fn replicate_to_secondary(&self, object_path: &str, data: &[u8]) -> Result<(), Error> {
        let sec_dest = crate::CONFIG.backup_secondary_destination();
        let clean_dest = sec_dest.replace("s3://", "");
        let parts: Vec<&str> = clean_dest.splitn(2, '/').collect();
        let bucket = parts[0];
        let root = if parts.len() > 1 { parts[1] } else { "" };
        
        let builder = services::S3::default()
            .bucket(bucket)
            .root(root)
            .region(&crate::CONFIG.backup_s3_region()); // Simplified: reuse primary region logic if unspecified
            
        let op_secondary = Operator::new(builder).map_err(|e| Error::new("Secondary S3 config invalid", e.to_string()))?.finish();
        
        op_secondary.write(object_path, data.to_vec()).await
            .map_err(|e| Error::new("Secondary replication write failed", e.to_string()))?;
        Ok(())
    }

    #[cfg(not(feature = "s3"))]
    async fn replicate_to_secondary(&self, object_path: &str, data: &[u8]) -> Result<(), Error> {
        let sec_dest = crate::CONFIG.backup_secondary_destination();
        let builder = services::Fs::default().root(&sec_dest);
        let op_secondary = Operator::new(builder).map_err(|e| Error::new("Secondary FS config invalid", e.to_string()))?.finish();
        op_secondary.write(object_path, data.to_vec()).await
            .map_err(|e| Error::new("Secondary FS write failed", e.to_string()))?;
        Ok(())
    }

    // TASK-006-011: Verify backup checksum and optionally signature
    pub async fn verify_backup(&self, backup_id: &str) -> Result<bool, Error> {
        let operator = self.create_operator().await?;
        
        let db_filename = format!("{}.sqldump", backup_id);
        let manifest_filename = format!("{}.manifest.json", backup_id);

        let manifest_bytes = operator.read(&manifest_filename).await
            .map_err(|e| Error::new("Failed to read manifest", e.to_string()))?.to_vec();
        
        let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)
            .map_err(|_| Error::new("Invalid manifest JSON format", ""))?;
        
        let expected_hash = manifest["sha256"].as_str().unwrap_or_default();
        
        let db_bytes = operator.read(&db_filename).await
            .map_err(|e| Error::new("Failed to read DB dump", e.to_string()))?.to_vec();
            
        let mut hasher = Sha256::new();
        hasher.update(&db_bytes);
        let actual_hash = data_encoding::HEXLOWER.encode(&hasher.finalize());
        
        if expected_hash != actual_hash {
            error!("Backup verification failed: hash mismatch for {}", backup_id);
            return Ok(false);
        }

        // Verify RSA signature if present
        if let Some(signature) = manifest["signature"].as_str() {
            let key_path = "data/backup_signing_key.pem";
            if let Ok(pem) = std::fs::read(key_path) {
                use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
                #[derive(serde::Deserialize)]
                struct Claims { hash: String }
                
                let mut validation = Validation::new(Algorithm::RS256);
                validation.required_spec_claims.clear(); // Disable expirations checks
                validation.validate_exp = false;
                
                if let Ok(key) = DecodingKey::from_rsa_pem(&pem) {
                    if let Ok(token_data) = decode::<Claims>(signature, &key, &validation) {
                        if token_data.claims.hash != actual_hash {
                            error!("Backup verification failed: signature hash mismatch for {}", backup_id);
                            return Ok(false);
                        }
                    } else {
                        error!("Backup verification failed: invalid signature for {}", backup_id);
                        return Ok(false);
                    }
                }
            }
        }
        
        info!("Backup {} perfectly verified (SHA256).", backup_id);
        Ok(true)
    }

    // TASK-006-003: OpenDAL Operator
    #[cfg(feature = "s3")]
    async fn create_operator(&self) -> Result<Operator, Error> {
        if self.backup_destination.starts_with("s3://") {
            let clean_dest = self.backup_destination.replace("s3://", "");
            let parts: Vec<&str> = clean_dest.splitn(2, '/').collect();
            let bucket = parts[0];
            let root = if parts.len() > 1 { parts[1] } else { "" };
            
            let builder = services::S3::default()
                .bucket(bucket)
                .root(root)
                .region(&crate::CONFIG.backup_s3_region());
            
            let op = Operator::new(builder).map_err(|e| Error::new("S3 config invalid", e.to_string()))?.finish();
            Ok(op)
        } else {
            // Local fallback
            let mut builder = services::Fs::default();
            builder.root(&self.backup_destination);
            let op = Operator::new(builder).map_err(|e| Error::new("FS init failed", e.to_string()))?.finish();
            Ok(op)
        }
    }

    #[cfg(not(feature = "s3"))]
    async fn create_operator(&self) -> Result<Operator, Error> {
        let builder = services::Fs::default().root(&self.backup_destination);
        let op = Operator::new(builder).map_err(|e| Error::new("FS init failed", e.to_string()))?.finish();
        Ok(op)
    }

    // TASK-006-004: pg_dump wrapper
    async fn run_pg_dump(&self) -> Result<Vec<u8>, Error> {
        let db_url = crate::CONFIG.database_url();
        let output = Command::new("pg_dump")
            .arg("--format=custom")
            // Compress requires pg_dump 16+ for zstd natively, using custom format defaults to gzip
            // .arg("--compress=6")
            .arg(&db_url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| Error::new("pg_dump process failed", e.to_string()))?;

        if !output.status.success() {
            let err_str = String::from_utf8_lossy(&output.stderr);
            return Err(Error::new("pg_dump execution returned error", err_str.into_owned()));
        }

        Ok(output.stdout)
    }

    // TASK-006-005
    async fn run_sqlite_copy(&self) -> Result<Vec<u8>, Error> {
        let db_url = crate::CONFIG.database_url();
        let output = tokio::fs::read(&db_url).await
            .map_err(|e| Error::new("Failed reading sqlite DB", e.to_string()))?;
        Ok(output)
    }

    // TASK-006-005
    async fn run_mysqldump(&self) -> Result<Vec<u8>, Error> {
        let db_url = crate::CONFIG.database_url();
        let output = Command::new("mysqldump")
            .arg(&db_url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| Error::new("mysqldump process failed", e.to_string()))?;

        if !output.status.success() {
            let err_str = String::from_utf8_lossy(&output.stderr);
            return Err(Error::new("mysqldump execution returned error", err_str.into_owned()));
        }

        Ok(output.stdout)
    }
}
