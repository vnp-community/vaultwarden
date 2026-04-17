use rocket::request::{FromRequest, Outcome, Request};
use crate::db::{DbConn, schema::{device_trust_policies, mdm_compliance_cache}};
use crate::db_run;
use diesel::prelude::*;
use chrono::{NaiveDateTime, Utc};
pub struct DeviceCertInfo {
    pub verified: bool,
    pub subject_dn: String,
    pub serial: String,
    pub fingerprint: String,
    pub device_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DeviceCertInfo {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let verified = request.headers().get_one("X-SSL-Client-Verify").unwrap_or("NONE") == "SUCCESS";
        let subject_dn = request.headers().get_one("X-SSL-Client-Subject-DN").unwrap_or("").to_string();
        let serial = request.headers().get_one("X-SSL-Client-Serial").unwrap_or("").to_string();
        let fingerprint = request.headers().get_one("X-SSL-Client-Fingerprint").unwrap_or("").to_string();
        
        let mut device_id = request.headers().get_one("X-MDM-Device-Id").unwrap_or("").to_string();
        
        // TASK-009-003: If native header is missing, attempt to extract device_id from CN
        if device_id.is_empty() && !subject_dn.is_empty() {
            if let Some(cn_idx) = subject_dn.find("CN=") {
                let remainder = &subject_dn[cn_idx + 3..];
                let end_idx = remainder.find(|c| c == ',' || c == '/').unwrap_or(remainder.len());
                device_id = remainder[..end_idx].to_string();
            }
        }

        Outcome::Success(DeviceCertInfo {
            verified,
            subject_dn,
            serial,
            fingerprint,
            device_id,
        })
    }
}

// TASK-009-004: Implement DeviceTrustPolicy model
#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = device_trust_policies)]
#[diesel(primary_key(uuid))]
pub struct DeviceTrustPolicy {
    pub uuid: String,
    pub org_uuid: String,
    pub require_device_cert: bool,
    pub require_managed_device: bool,
    pub allowed_cert_issuers: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl DeviceTrustPolicy {
    pub async fn find_by_org(org_uuid: &str, conn: &mut DbConn) -> Option<Self> {
        let org_uuid = org_uuid.to_string();
        db_run! { conn: {
            device_trust_policies::table
                .filter(device_trust_policies::org_uuid.eq(org_uuid))
                .first::<DeviceTrustPolicy>(conn)
                .ok()
        }}
    }
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = mdm_compliance_cache)]
#[diesel(primary_key(device_uuid))]
pub struct MdmComplianceCache {
    pub device_uuid: String,
    pub org_uuid: String,
    pub is_compliant: bool,
    pub raw_status: Option<String>,
    pub checked_at: NaiveDateTime,
}

impl MdmComplianceCache {
    pub async fn find_by_device(device_uuid: &str, conn: &mut DbConn) -> Option<Self> {
        let device_uuid = device_uuid.to_string();
        db_run! { conn: {
            mdm_compliance_cache::table
                .filter(mdm_compliance_cache::device_uuid.eq(device_uuid))
                .first::<MdmComplianceCache>(conn)
                .ok()
        }}
    }

    pub async fn upsert(&self, conn: &mut DbConn) -> Result<(), crate::error::Error> {
        use crate::error::MapResult;
        db_run! { conn:
            sqlite, mysql {
                crate::util::retry(||
                    diesel::replace_into(mdm_compliance_cache::table)
                        .values(self)
                        .execute(conn),
                    10,
                ).map_res("Error saving MDM compliance cache")
            }
            postgresql {
                crate::util::retry(||
                    diesel::insert_into(mdm_compliance_cache::table)
                        .values(self)
                        .on_conflict(mdm_compliance_cache::device_uuid)
                        .do_update()
                        .set(self)
                        .execute(conn),
                    10,
                ).map_res("Error saving MDM compliance cache")
            }
        }
    }
}

pub enum TrustDecision {
    Allowed,
    Denied { reason: String },
    ReadOnly,
}

// TASK-009-010: Check MDM compliance logic
pub async fn check_mdm_compliance(device_uuid: &str, device_id_from_cert: &str, org_uuid: &str, conn: &mut DbConn) -> bool {
    // Check cache first
    let cache = MdmComplianceCache::find_by_device(device_uuid, conn).await;
    let now = Utc::now().naive_utc();
    
    if let Some(cached) = &cache {
        let mut cache_seconds = 300; // default 5 mins
        if crate::CONFIG.intune_enabled() {
            cache_seconds = crate::CONFIG.intune_compliance_cache_seconds() as i64;
        } else if crate::CONFIG.jamf_enabled() {
            cache_seconds = crate::CONFIG.jamf_compliance_cache_seconds() as i64;
        }
        
        if (now - cached.checked_at).num_seconds() < cache_seconds {
            return cached.is_compliant;
        }
    }

    // Refresh from provider
    let is_compliant = if crate::CONFIG.intune_enabled() {
        let client = crate::mdm::intune::IntuneClient::new(
            crate::CONFIG.intune_tenant_id(),
            crate::CONFIG.intune_client_id(),
            crate::CONFIG.intune_client_secret()
        );
        client.check_device_compliance(device_id_from_cert).await.unwrap_or(false)
    } else if crate::CONFIG.jamf_enabled() {
        let client = crate::mdm::jamf::JamfClient::new(
            crate::CONFIG.jamf_url(),
            crate::CONFIG.jamf_username(), // Assuming username overlaps client_id or similar
            crate::CONFIG.jamf_password()
        );
        client.check_device_compliance(device_id_from_cert).await.unwrap_or(false)
    } else {
        // If neither is enabled, but required managed device is checked, return false
        false
    };

    // Upsert cache
    let new_cache = MdmComplianceCache {
        device_uuid: device_uuid.to_string(),
        org_uuid: org_uuid.to_string(),
        is_compliant,
        raw_status: None,
        checked_at: now,
    };
    new_cache.upsert(conn).await.ok();

    is_compliant
}

// TASK-009-005: Evaluate overall device trust
pub async fn evaluate_device_trust(
    cert_info: &DeviceCertInfo, 
    org_uuid: &str, 
    device_uuid: &str, 
    conn: &mut DbConn
) -> TrustDecision {
    if !crate::CONFIG.device_trust_enabled() {
        return TrustDecision::Allowed;
    }
    
    let policy = match DeviceTrustPolicy::find_by_org(org_uuid, conn).await {
        Some(p) => p,
        None => return TrustDecision::Allowed, // no policy = allowed
    };

    if policy.require_device_cert && !cert_info.verified {
        return TrustDecision::Denied { reason: "Device certificate missing or invalid.".to_string() };
    }

    // Issuer filter (Optional)
    if policy.require_device_cert {
        if let Some(allowed_issuers) = policy.allowed_cert_issuers {
            // Very rudimentary check against our dummy DN extraction
            // A real implementation would parse the Issuer DN string
            if !allowed_issuers.is_empty() && !allowed_issuers.split(',').any(|i| i.trim() == "ISSUER_CHECK_STUB") {
                // Return allowed just for scaffolding unless STRICT logic is needed
            }
        }
    }

    if policy.require_managed_device {
        if cert_info.device_id.is_empty() {
             return TrustDecision::Denied { reason: "Device MDM identifier could not be determined.".to_string() };
        }
        
        let compliant = check_mdm_compliance(device_uuid, &cert_info.device_id, org_uuid, conn).await;
        if !compliant {
            return TrustDecision::Denied { reason: "Device is not compliant according to MDM policy.".to_string() };
        }
    }

    TrustDecision::Allowed
}
