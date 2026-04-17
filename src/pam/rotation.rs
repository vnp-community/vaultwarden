use crate::error::Error;
use crate::db::models::pam::{RotationHistory, PrivilegedConfig};
use crate::db::DbConn;
use std::process::Command;
use rand::{distr::Alphanumeric, Rng};

pub struct RotationEngine;

impl RotationEngine {
    pub async fn rotate_credential(config: &PrivilegedConfig, cipher_uuid: &str, checkout_uuid: Option<String>, conn: &mut DbConn) -> Result<String, Error> {
        let history = RotationHistory::new(cipher_uuid.to_string(), checkout_uuid);
        history.insert(conn).await?;

        let new_password = Self::generate_secure_password(32);
        
        let target_type = config.rotation_target_type.as_deref().unwrap_or("");
        
        let result = match target_type {
            "ssh" => Self::rotate_ssh(config, &new_password),
            "mysql" => Self::rotate_mysql(config, &new_password),
            "postgresql" => Self::rotate_postgresql(config, &new_password),
            _ => Err(Error::new("Unsupported target type", "Rotation Failed")),
        };

        // We update the history record with the completion status.
        // Needs a fresh copy or we just update fields for saving.
        let mut completed_history = history;
        if let Err(e) = &result {
            completed_history.status = "failed".to_string();
            completed_history.error_message = Some(format!("{}", e));
        } else {
            completed_history.status = "success".to_string();
        }
        completed_history.completed_at = Some(chrono::Utc::now().naive_utc());
        completed_history.save(conn).await?;
        
        result.map(|_| new_password)
    }

    fn rotate_ssh(config: &PrivilegedConfig, new_password: &str) -> Result<(), Error> {
        let rotation_conf = config.get_rotation_config()
            .ok_or_else(|| Error::new("Missing config", "Rotation config is empty"))?;
        
        let host = rotation_conf.host.as_deref().unwrap_or("");
        let user = rotation_conf.user.as_deref().unwrap_or("root");
        
        let cmd = format!("echo '{}:{}' | sudo chpasswd", user, new_password);
        
        let output = Command::new("ssh")
            .arg("-o").arg("StrictHostKeyChecking=no")
            .arg("-o").arg("ConnectTimeout=10")
            .arg("-i").arg(crate::CONFIG.pam_rotation_ssh_key_path())
            .arg(format!("{}@{}", user, host))
            .arg(&cmd)
            .output()
            .map_err(|e| Error::new(e.to_string(), "SSH Execution Failed"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::new(stderr, "SSH Command Failed"));
        }
        Ok(())
    }

    fn rotate_mysql(config: &PrivilegedConfig, new_password: &str) -> Result<(), Error> {
        let rotation_conf = config.get_rotation_config()
            .ok_or_else(|| Error::new("Missing config", "Rotation config is empty"))?;
        let host = rotation_conf.host.as_deref().unwrap_or("127.0.0.1");
        let user = rotation_conf.user.as_deref().unwrap_or("root");
        
        let sql = format!("ALTER USER '{}'@'%' IDENTIFIED BY '{}';", user, new_password);

        let output = Command::new("mysql")
            .arg("-h").arg(host)
            .arg("-u").arg("admin") // Ideally managed by environment setup or .my.cnf
            .arg("-e").arg(&sql)
            .output()
            .map_err(|e| Error::new(e.to_string(), "MySQL Execution Failed"))?;

        if !output.status.success() {
            return Err(Error::new("MySQL command failed", "MySQL execution failed"));
        }
        Ok(())
    }

    fn rotate_postgresql(config: &PrivilegedConfig, new_password: &str) -> Result<(), Error> {
        let rotation_conf = config.get_rotation_config()
            .ok_or_else(|| Error::new("Missing config", "Rotation config is empty"))?;
        let host = rotation_conf.host.as_deref().unwrap_or("127.0.0.1");
        let user = rotation_conf.user.as_deref().unwrap_or("postgres");
        
        let sql = format!("ALTER USER {} PASSWORD '{}';", user, new_password);

        let output = Command::new("psql")
            .arg("-h").arg(host)
            .arg("-U").arg("postgres")
            .arg("-c").arg(&sql)
            .output()
            .map_err(|e| Error::new(e.to_string(), "PostgreSQL Execution Failed"))?;

        if !output.status.success() {
            return Err(Error::new("PostgreSQL command failed", "PostgreSQL execution failed"));
        }
        Ok(())
    }

    fn generate_secure_password(length: usize) -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
}
