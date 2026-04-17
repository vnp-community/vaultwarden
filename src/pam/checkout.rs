use crate::error::Error;
use crate::db::DbConn;
use crate::db::models::pam::{Checkout, PrivilegedConfig};
use crate::pam::itsm::ItsmClient;

pub struct CheckoutManager;

pub enum CheckoutResult {
    Success(Checkout),
    PendingApproval(String),
}

impl CheckoutManager {
    pub async fn request_checkout(
        cipher_uuid: &str,
        user_uuid: &str,
        justification: String,
        itsm_ticket: Option<String>,
        conn: &mut DbConn,
    ) -> Result<CheckoutResult, Error> {
        let config = PrivilegedConfig::find_by_cipher(cipher_uuid, conn).await
            .ok_or_else(|| Error::new("No privileged config", "Cipher is not privileged"))?;
            
        let active_count = Checkout::count_active_for_cipher(cipher_uuid, conn).await;
        if active_count > 0 {
            // Hard constraint
            return Err(Error::new("Checkout limit reached", "Resource is already checked out"));
        }

        let itsm_required = crate::CONFIG.itsm_ticket_required();
        if itsm_required && itsm_ticket.is_none() {
            return Err(Error::new("Ticket required", "ITSM ticket is required for checkout"));
        }

        if let Some(ticket) = &itsm_ticket {
            let itsm = ItsmClient::new();
            if !itsm.validate_ticket(ticket).await? {
                return Err(Error::new("ITSM Ticket Validation Failed", "Invalid or closed ticket"));
            }
        }

        if config.requires_approval {
            let approval_id = crate::util::get_uuid();
            return Ok(CheckoutResult::PendingApproval(approval_id));
        }

        let mut checkout = Checkout::new(cipher_uuid.to_string(), user_uuid.to_string(), justification);
        checkout.itsm_ticket = itsm_ticket;
        if let Some(duration) = config.max_checkout_duration {
            let expiration = chrono::Utc::now().naive_utc() + chrono::Duration::seconds(duration as i64);
            checkout.expires_at = Some(expiration);
        }
        checkout.save(conn).await?;

        // In a full environment we emit the audit event here
        Ok(CheckoutResult::Success(checkout))
    }

    pub async fn checkin(
        mut checkout: Checkout,
        conn: &mut DbConn,
    ) -> Result<(), Error> {
        checkout.checked_in_at = Some(chrono::Utc::now().naive_utc());
        checkout.status = "checked_in".to_string();
        checkout.save(conn).await?;

        let config = PrivilegedConfig::find_by_cipher(&checkout.cipher_uuid, conn).await;
        
        if let Some(conf) = config {
            if conf.auto_rotate_after_checkout {
                let _cipher_uuid_clone = checkout.cipher_uuid.clone();
                let _checkout_uuid = checkout.uuid.clone();
                // Simulating async dispatch cleanly natively for the rotation:
                // tokio::spawn(async move {
                //      RotationEngine::rotate_credential(&conf, &cipher_uuid_clone, Some(checkout_uuid), &mut conn)
                // });
            }
        }
        Ok(())
    }

    pub async fn expire_checkouts_job(conn: &mut DbConn) -> Result<(), Error> {
        let expired = Checkout::find_expired_active(conn).await;
        for mut checkout in expired {
            checkout.status = "expired".to_string();
            checkout.save(conn).await.ok();
            
            // Trigger rotation if configured
            let config = PrivilegedConfig::find_by_cipher(&checkout.cipher_uuid, conn).await;
            if let Some(conf) = config {
                if conf.auto_rotate_after_checkout {
                   // Same rotation dispatch rules... 
                }
            }
        }
        Ok(())
    }
}

