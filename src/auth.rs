use std::{
    env,
    net::IpAddr,
    sync::{LazyLock, RwLock},
};

use chrono::{DateTime, TimeDelta, Utc};
use jsonwebtoken::{errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header};
use num_traits::FromPrimitive;
use openssl::rsa::Rsa;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use crate::{
    api::ApiResult,
    config::PathType,
    db::models::{
        AttachmentId, CipherId, CollectionId, DeviceId, DeviceType, EmergencyAccessId, MembershipId, OrgApiKeyId,
        OrganizationId, SendFileId, SendId, UserId,
    },
    error::Error,
    sso, CONFIG,
};

const JWT_ALGORITHM: Algorithm = Algorithm::RS256;

// Limit when BitWarden consider the token as expired
pub static BW_EXPIRATION: LazyLock<TimeDelta> = LazyLock::new(|| TimeDelta::try_minutes(5).unwrap());

pub static DEFAULT_REFRESH_VALIDITY: LazyLock<TimeDelta> =
    LazyLock::new(|| TimeDelta::try_days(i64::from(CONFIG.refresh_token_validity_days())).unwrap());
pub static MOBILE_REFRESH_VALIDITY: LazyLock<TimeDelta> =
    LazyLock::new(|| TimeDelta::try_days(i64::from(CONFIG.mobile_refresh_token_validity_days())).unwrap());
pub static DEFAULT_ACCESS_VALIDITY: LazyLock<TimeDelta> =
    LazyLock::new(|| TimeDelta::try_hours(i64::from(CONFIG.access_token_validity_hours())).unwrap());
static JWT_HEADER: LazyLock<Header> = LazyLock::new(|| Header::new(JWT_ALGORITHM));

pub static JWT_LOGIN_ISSUER: LazyLock<String> = LazyLock::new(|| format!("{}|login", CONFIG.domain_origin()));
static JWT_INVITE_ISSUER: LazyLock<String> = LazyLock::new(|| format!("{}|invite", CONFIG.domain_origin()));
static JWT_EMERGENCY_ACCESS_INVITE_ISSUER: LazyLock<String> =
    LazyLock::new(|| format!("{}|emergencyaccessinvite", CONFIG.domain_origin()));
static JWT_DELETE_ISSUER: LazyLock<String> = LazyLock::new(|| format!("{}|delete", CONFIG.domain_origin()));
static JWT_VERIFYEMAIL_ISSUER: LazyLock<String> = LazyLock::new(|| format!("{}|verifyemail", CONFIG.domain_origin()));
static JWT_ADMIN_ISSUER: LazyLock<String> = LazyLock::new(|| format!("{}|admin", CONFIG.domain_origin()));
static JWT_SEND_ISSUER: LazyLock<String> = LazyLock::new(|| format!("{}|send", CONFIG.domain_origin()));
static JWT_ORG_API_KEY_ISSUER: LazyLock<String> =
    LazyLock::new(|| format!("{}|api.organization", CONFIG.domain_origin()));
static JWT_FILE_DOWNLOAD_ISSUER: LazyLock<String> =
    LazyLock::new(|| format!("{}|file_download", CONFIG.domain_origin()));
static JWT_REGISTER_VERIFY_ISSUER: LazyLock<String> =
    LazyLock::new(|| format!("{}|register_verify", CONFIG.domain_origin()));

// TASK-SEC-LOW-01-A: Changed from OnceLock to RwLock to support JWT signing key rotation without restart.
// Reads acquire a read guard; rotation takes a brief write lock to swap both keys atomically.
static PRIVATE_RSA_KEY: RwLock<Option<EncodingKey>> = RwLock::new(None);
static PUBLIC_RSA_KEY: RwLock<Option<DecodingKey>> = RwLock::new(None);

/// Encrypt RSA PEM bytes with AES-256-GCM using the master key from config.
/// Storage format: `[12-byte nonce][ciphertext]`.
fn encrypt_rsa_key(pem: &[u8], master_key: &str) -> Result<Vec<u8>, Error> {
    use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
    use ring::digest::{digest, SHA256};
    use ring::rand::{SecureRandom, SystemRandom};

    // Derive a 32-byte key from the master secret via SHA-256
    let key_bytes = digest(&SHA256, master_key.as_bytes());
    let unbound = UnboundKey::new(&AES_256_GCM, key_bytes.as_ref())
        .map_err(|_| Error::new("Failed to create AES key", ""))?;
    let key = LessSafeKey::new(unbound);

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes).map_err(|_| Error::new("Failed to generate nonce", ""))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut ciphertext = pem.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
        .map_err(|_| Error::new("AES-GCM encryption failed", ""))?;

    let mut out = nonce_bytes.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt RSA PEM bytes previously encrypted by `encrypt_rsa_key`.
fn decrypt_rsa_key(data: &[u8], master_key: &str) -> Result<Vec<u8>, Error> {
    use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
    use ring::digest::{digest, SHA256};

    if data.len() < 12 {
        return Err(Error::new("Encrypted RSA key too short", ""));
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes.try_into().unwrap());

    let key_bytes = digest(&SHA256, master_key.as_bytes());
    let unbound = UnboundKey::new(&AES_256_GCM, key_bytes.as_ref())
        .map_err(|_| Error::new("Failed to create AES key", ""))?;
    let key = LessSafeKey::new(unbound);

    let mut plaintext = ciphertext.to_vec();
    let decrypted = key.open_in_place(nonce, Aad::empty(), &mut plaintext)
        .map_err(|_| Error::new("AES-GCM decryption failed — wrong RSA_KEY_ENCRYPTION_KEY or corrupted key file", ""))?;
    Ok(decrypted.to_vec())
}

pub async fn initialize_keys() -> Result<(), Error> {
    use std::io::Error as IoError;

    let rsa_key_filename = std::path::PathBuf::from(CONFIG.private_rsa_key())
        .file_name()
        .ok_or_else(|| IoError::other("Private RSA key path missing filename"))?
        .to_str()
        .ok_or_else(|| IoError::other("Private RSA key path filename is not valid UTF-8"))?
        .to_string();

    let operator = CONFIG.opendal_operator_for_path_type(&PathType::RsaKey).map_err(IoError::other)?;

    let raw_file_bytes = match operator.read(&rsa_key_filename).await {
        Ok(buffer) => Some(buffer.to_vec()),
        Err(e) if e.kind() == opendal::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };

    let master_key = CONFIG.rsa_key_encryption_key();
    let encryption_enabled = !master_key.is_empty();

    let (priv_key, priv_key_pem) = if let Some(raw) = raw_file_bytes {
        // Decrypt if master key is set; otherwise treat as raw PEM
        let pem = if encryption_enabled {
            decrypt_rsa_key(&raw, &master_key)?
        } else {
            raw
        };
        (Rsa::private_key_from_pem(&pem)?, pem)
    } else {
        let rsa_key = Rsa::generate(2048)?;
        let pem = rsa_key.private_key_to_pem()?;
        let to_store = if encryption_enabled {
            encrypt_rsa_key(&pem, &master_key)?
        } else {
            pem.clone()
        };
        operator.write(&rsa_key_filename, to_store).await?;
        info!("Private key '{}' created correctly", CONFIG.private_rsa_key());
        (rsa_key, pem)
    };

    // Startup warning if key is stored unencrypted
    if !encryption_enabled {
        warn!(
            "SECURITY: RSA private key is stored unencrypted. \
             Set RSA_KEY_ENCRYPTION_KEY to encrypt it at rest."
        );
    }

    let pub_key_buffer = priv_key.public_key_to_pem()?;

    let enc = EncodingKey::from_rsa_pem(&priv_key_pem)?;
    let dec: DecodingKey = DecodingKey::from_rsa_pem(&pub_key_buffer)?;
    *PRIVATE_RSA_KEY.write().expect("PRIVATE_RSA_KEY poisoned") = Some(enc);
    *PUBLIC_RSA_KEY.write().expect("PUBLIC_RSA_KEY poisoned") = Some(dec);
    Ok(())
}

/// TASK-SEC-LOW-01-A: Hot-rotate the JWT signing RSA key without server restart.
/// Steps:
///  1. Archive the current key file as `{filename}.{timestamp}.bak`
///  2. Generate a new RSA-2048 key pair
///  3. Persist the new private key (encrypted if RSA_KEY_ENCRYPTION_KEY is set)
///  4. Hot-swap both global statics under write lock — new tokens use new key; old tokens
///     will fail validation on the next request (security_stamp rotation is done by caller)
///
/// Returns the new public key PEM string for admin confirmation.
pub async fn rotate_jwt_signing_key() -> Result<String, Error> {
    use std::io::Error as IoError;

    let rsa_key_filename = std::path::PathBuf::from(CONFIG.private_rsa_key())
        .file_name()
        .ok_or_else(|| IoError::other("Private RSA key path missing filename"))?
        .to_str()
        .ok_or_else(|| IoError::other("Private RSA key path filename is not valid UTF-8"))?
        .to_string();

    let operator = CONFIG.opendal_operator_for_path_type(&PathType::RsaKey).map_err(IoError::other)?;

    // 1. Archive current key (best-effort; failure is non-fatal)
    let archive_name = format!(
        "{}.{}.bak",
        rsa_key_filename,
        Utc::now().format("%Y%m%dT%H%M%SZ")
    );
    if let Ok(current_bytes) = operator.read(&rsa_key_filename).await {
        if let Err(e) = operator.write(&archive_name, current_bytes.to_vec()).await {
            warn!("[KeyRotation] Failed to archive old RSA key as {archive_name}: {e}");
        } else {
            info!("[KeyRotation] Old RSA key archived as {archive_name}");
        }
    }

    // 2. Generate new RSA-2048 key pair
    let new_rsa = Rsa::generate(2048)?;
    let new_pem = new_rsa.private_key_to_pem()?;
    let new_pub_pem = new_rsa.public_key_to_pem()?;

    // 3. Persist (encrypt if master key configured)
    let master_key = CONFIG.rsa_key_encryption_key();
    let to_store = if !master_key.is_empty() {
        encrypt_rsa_key(&new_pem, &master_key)?
    } else {
        new_pem.clone()
    };
    operator.write(&rsa_key_filename, to_store).await.map_err(IoError::other)?;
    info!("[KeyRotation] New RSA key written to {rsa_key_filename}");

    // 4. Hot-swap under write lock (brief exclusive window)
    let new_enc = EncodingKey::from_rsa_pem(&new_pem)?;
    let new_dec = DecodingKey::from_rsa_pem(&new_pub_pem)?;
    *PRIVATE_RSA_KEY.write().expect("PRIVATE_RSA_KEY poisoned") = Some(new_enc);
    *PUBLIC_RSA_KEY.write().expect("PUBLIC_RSA_KEY poisoned") = Some(new_dec);

    let pub_pem_str = String::from_utf8(new_pub_pem).map_err(|e| IoError::other(e.to_string()))?;
    info!("[KeyRotation] JWT signing key rotated successfully");
    Ok(pub_pem_str)
}

pub fn encode_jwt<T: Serialize>(claims: &T) -> Result<String, Error> {
    let key_guard = PRIVATE_RSA_KEY.read().expect("PRIVATE_RSA_KEY poisoned");
    let key = key_guard.as_ref().ok_or_else(|| Error::new("RSA key not initialized", ""))?;
    match jsonwebtoken::encode(&JWT_HEADER, claims, key) {
        Ok(token) => Ok(token),
        Err(e) => {
            error!("JWT encoding failed: {e}");
            Err(Error::new("JWT encoding failed", e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Initialize RSA test keys once for all tests in this module.
    /// Uses `get_or_init` so it is safe to call from multiple tests.
    fn init_test_keys() {
        use openssl::rsa::Rsa;
        let mut guard = PRIVATE_RSA_KEY.write().expect("PRIVATE_RSA_KEY poisoned");
        if guard.is_none() {
            let rsa = Rsa::generate(2048).expect("RSA key generation failed");
            let priv_pem = rsa.private_key_to_pem().expect("priv pem");
            let pub_pem = rsa.public_key_to_pem().expect("pub pem");
            let enc = EncodingKey::from_rsa_pem(&priv_pem).expect("encoding key");
            let dec = DecodingKey::from_rsa_pem(&pub_pem).expect("decoding key");
            *guard = Some(enc);
            *PUBLIC_RSA_KEY.write().expect("PUBLIC_RSA_KEY poisoned") = Some(dec);
        }
    }

    fn make_test_claims(exp_offset_secs: i64) -> BasicJwtClaims {
        let now = Utc::now().timestamp();
        BasicJwtClaims {
            nbf: now - 1,
            exp: now + exp_offset_secs,
            iss: "test|delete".to_string(),
            sub: "test-user-uuid".to_string(),
        }
    }

    // -------------------------------------------------------------------------
    // TASK-RUSTDEV-CRIT-01-C / TASK-RUSTDEV-LOW-02-A
    // -------------------------------------------------------------------------

    /// encode_jwt must not panic and must return Ok for valid claims.
    #[test]
    fn test_encode_jwt_returns_ok() {
        init_test_keys();
        let claims = make_test_claims(3600);
        let result = encode_jwt(&claims);
        assert!(result.is_ok(), "encode_jwt should return Ok, got: {:?}", result);
        let token = result.unwrap();
        assert!(!token.is_empty(), "token should not be empty");
        // JWT has three dot-separated parts
        assert_eq!(token.split('.').count(), 3, "JWT should have 3 parts");
    }

    /// Encode then decode must round-trip all claim fields.
    #[test]
    fn test_encode_decode_roundtrip() {
        init_test_keys();
        let claims = make_test_claims(3600);
        let token = encode_jwt(&claims).expect("encode should succeed");

        let mut validation = jsonwebtoken::Validation::new(JWT_ALGORITHM);
        validation.leeway = 30;
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.set_issuer(&["test|delete"]);

        let decoded: BasicJwtClaims = {
            let key_guard = PUBLIC_RSA_KEY.read().expect("PUBLIC_RSA_KEY poisoned");
            jsonwebtoken::decode(&token, key_guard.as_ref().unwrap(), &validation)
                .expect("decode should succeed")
                .claims
        };

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.iss, claims.iss);
        assert_eq!(decoded.nbf, claims.nbf);
        assert_eq!(decoded.exp, claims.exp);
    }

    /// A token with exp in the past must be rejected by the decoder.
    #[test]
    fn test_expired_jwt_rejected() {
        init_test_keys();
        // exp 60s in the past, nbf before that
        let now = Utc::now().timestamp();
        let claims = BasicJwtClaims {
            nbf: now - 120,
            exp: now - 60,
            iss: "test|delete".to_string(),
            sub: "test-user-uuid".to_string(),
        };
        let token = encode_jwt(&claims).expect("encode should succeed");

        let mut validation = jsonwebtoken::Validation::new(JWT_ALGORITHM);
        validation.leeway = 0;
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.set_issuer(&["test|delete"]);

        let result: Result<jsonwebtoken::TokenData<BasicJwtClaims>, _> = {
            let key_guard = PUBLIC_RSA_KEY.read().expect("PUBLIC_RSA_KEY poisoned");
            jsonwebtoken::decode(&token, key_guard.as_ref().unwrap(), &validation)
        };
        assert!(result.is_err(), "expired JWT should be rejected");
        let kind = result.unwrap_err().into_kind();
        assert!(
            matches!(kind, ErrorKind::ExpiredSignature),
            "error should be ExpiredSignature, got: {:?}",
            kind
        );
    }

    /// Tampering with the signature bytes must cause decode to fail.
    #[test]
    fn test_tampered_jwt_rejected() {
        init_test_keys();
        let claims = make_test_claims(3600);
        let token = encode_jwt(&claims).expect("encode should succeed");

        // Flip bytes in the signature (third part)
        let mut parts: Vec<&str> = token.split('.').collect();
        let mut sig = parts[2].as_bytes().to_vec();
        // XOR the first byte to corrupt the signature
        sig[0] ^= 0xFF;
        let bad_sig = String::from_utf8_lossy(&sig).into_owned();
        parts[2] = Box::leak(bad_sig.into_boxed_str());
        let tampered = parts.join(".");

        let mut validation = jsonwebtoken::Validation::new(JWT_ALGORITHM);
        validation.leeway = 30;
        validation.validate_exp = true;
        validation.set_issuer(&["test|delete"]);

        let result: Result<jsonwebtoken::TokenData<BasicJwtClaims>, _> = {
            let key_guard = PUBLIC_RSA_KEY.read().expect("PUBLIC_RSA_KEY poisoned");
            jsonwebtoken::decode(&tampered, key_guard.as_ref().unwrap(), &validation)
        };
        assert!(result.is_err(), "tampered JWT should be rejected");
    }

    // -------------------------------------------------------------------------
    // AES-256-GCM RSA key encryption (TASK-RUSTDEV-MED-02-B/C)
    // -------------------------------------------------------------------------

    /// Encrypt then decrypt of RSA PEM must produce the original bytes.
    #[test]
    fn test_rsa_key_encrypt_decrypt_roundtrip() {
        let pem = b"fake-pem-bytes-for-testing-purposes";
        let master = "my-test-master-key";
        let encrypted = encrypt_rsa_key(pem, master).expect("encrypt should succeed");
        // Encrypted blob is nonce (12) + ciphertext + tag
        assert!(encrypted.len() > 12 + pem.len(), "encrypted output should be longer than input");
        let decrypted = decrypt_rsa_key(&encrypted, master).expect("decrypt should succeed");
        assert_eq!(decrypted, pem);
    }

    /// Decrypting with wrong key must return Err.
    #[test]
    fn test_rsa_key_decrypt_wrong_key_fails() {
        let pem = b"fake-pem-bytes";
        let encrypted = encrypt_rsa_key(pem, "correct-key").expect("encrypt");
        let result = decrypt_rsa_key(&encrypted, "wrong-key");
        assert!(result.is_err(), "wrong key must fail decryption");
    }

    /// Truncated data (< 12 bytes) must fail gracefully without panic.
    #[test]
    fn test_rsa_key_decrypt_too_short_fails() {
        let result = decrypt_rsa_key(b"short", "any-key");
        assert!(result.is_err());
    }
}

pub fn decode_jwt<T: DeserializeOwned>(token: &str, issuer: String) -> Result<T, Error> {
    let mut validation = jsonwebtoken::Validation::new(JWT_ALGORITHM);
    validation.leeway = 30; // 30 seconds
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.set_issuer(&[issuer]);

    let token = token.replace(char::is_whitespace, "");
    let key_guard = PUBLIC_RSA_KEY.read().expect("PUBLIC_RSA_KEY poisoned");
    let key = key_guard.as_ref().ok_or_else(|| Error::new("RSA key not initialized", ""))?;
    match jsonwebtoken::decode(&token, key, &validation) {
        Ok(d) => Ok(d.claims),
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => err!("Token is invalid"),
            ErrorKind::InvalidIssuer => err!("Issuer is invalid"),
            ErrorKind::ExpiredSignature => err!("Token has expired"),
            _ => err!(format!("Error decoding JWT: {:?}", err)),
        },
    }
}

pub fn decode_refresh(token: &str) -> Result<RefreshJwtClaims, Error> {
    decode_jwt(token, JWT_LOGIN_ISSUER.to_string())
}

pub fn decode_login(token: &str) -> Result<LoginJwtClaims, Error> {
    decode_jwt(token, JWT_LOGIN_ISSUER.to_string())
}

pub fn decode_invite(token: &str) -> Result<InviteJwtClaims, Error> {
    decode_jwt(token, JWT_INVITE_ISSUER.to_string())
}

pub fn decode_emergency_access_invite(token: &str) -> Result<EmergencyAccessInviteJwtClaims, Error> {
    decode_jwt(token, JWT_EMERGENCY_ACCESS_INVITE_ISSUER.to_string())
}

pub fn decode_delete(token: &str) -> Result<BasicJwtClaims, Error> {
    decode_jwt(token, JWT_DELETE_ISSUER.to_string())
}

pub fn decode_verify_email(token: &str) -> Result<BasicJwtClaims, Error> {
    decode_jwt(token, JWT_VERIFYEMAIL_ISSUER.to_string())
}

pub fn decode_admin(token: &str) -> Result<BasicJwtClaims, Error> {
    decode_jwt(token, JWT_ADMIN_ISSUER.to_string())
}

pub fn decode_send(token: &str) -> Result<BasicJwtClaims, Error> {
    decode_jwt(token, JWT_SEND_ISSUER.to_string())
}

pub fn decode_api_org(token: &str) -> Result<OrgApiKeyLoginJwtClaims, Error> {
    decode_jwt(token, JWT_ORG_API_KEY_ISSUER.to_string())
}

pub fn decode_file_download(token: &str) -> Result<FileDownloadClaims, Error> {
    decode_jwt(token, JWT_FILE_DOWNLOAD_ISSUER.to_string())
}

pub fn decode_register_verify(token: &str) -> Result<RegisterVerifyClaims, Error> {
    decode_jwt(token, JWT_REGISTER_VERIFY_ISSUER.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginJwtClaims {
    // Not before
    pub nbf: i64,
    // Expiration time
    pub exp: i64,
    // Issuer
    pub iss: String,
    // Subject
    pub sub: UserId,

    pub premium: bool,
    pub name: String,
    pub email: String,
    pub email_verified: bool,

    // ---
    // Disabled these keys to be added to the JWT since they could cause the JWT to get too large
    // Also These key/value pairs are not used anywhere by either Vaultwarden or Bitwarden Clients
    // Because these might get used in the future, and they are added by the Bitwarden Server, lets keep it, but then commented out
    // See: https://github.com/dani-garcia/vaultwarden/issues/4156
    // ---
    // pub orgowner: Vec<String>,
    // pub orgadmin: Vec<String>,
    // pub orguser: Vec<String>,
    // pub orgmanager: Vec<String>,

    // user security_stamp
    pub sstamp: String,
    // device uuid
    pub device: DeviceId,
    // what kind of device, like FirefoxBrowser or Android derived from DeviceType
    pub devicetype: String,
    // the type of client_id, like web, cli, desktop, browser or mobile
    pub client_id: String,

    // [ "api", "offline_access" ]
    pub scope: Vec<String>,
    // [ "Application" ]
    pub amr: Vec<String>,

    // TASK-SEC-HIGH-02-F: JWT ID for opt-in token revocation.
    // Only included in the JWT when TOKEN_REVOCATION_ENABLED=true.
    // Using Option + skip_serializing_if keeps backward-compat tokens when revocation is off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,

    // SOL-011: Multi-Tenancy fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_uuid: Option<String>,
    #[serde(default)]
    pub is_tenant_admin: bool,
    #[serde(default)]
    pub is_system_admin: bool,
}

impl LoginJwtClaims {
    pub fn new(
        device: &Device,
        user: &User,
        nbf: i64,
        exp: i64,
        scope: Vec<String>,
        client_id: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        // ---
        // Disabled these keys to be added to the JWT since they could cause the JWT to get too large
        // Also These key/value pairs are not used anywhere by either Vaultwarden or Bitwarden Clients
        // Because these might get used in the future, and they are added by the Bitwarden Server, lets keep it, but then commented out
        // ---
        // fn arg: orgs: Vec<super::UserOrganization>,
        // ---
        // let orgowner: Vec<_> = orgs.iter().filter(|o| o.atype == 0).map(|o| o.org_uuid.clone()).collect();
        // let orgadmin: Vec<_> = orgs.iter().filter(|o| o.atype == 1).map(|o| o.org_uuid.clone()).collect();
        // let orguser: Vec<_> = orgs.iter().filter(|o| o.atype == 2).map(|o| o.org_uuid.clone()).collect();
        // let orgmanager: Vec<_> = orgs.iter().filter(|o| o.atype == 3).map(|o| o.org_uuid.clone()).collect();

        if exp <= (now + *BW_EXPIRATION).timestamp() {
            warn!("Raise access_token lifetime to more than 5min.")
        }

        // Create the JWT claims struct, to send to the client
        Self {
            nbf,
            exp,
            iss: JWT_LOGIN_ISSUER.to_string(),
            sub: user.uuid.clone(),
            premium: true,
            name: user.name.clone(),
            email: user.email.clone(),
            email_verified: !CONFIG.mail_enabled() || user.verified_at.is_some(),

            // ---
            // Disabled these keys to be added to the JWT since they could cause the JWT to get too large
            // Also These key/value pairs are not used anywhere by either Vaultwarden or Bitwarden Clients
            // Because these might get used in the future, and they are added by the Bitwarden Server, lets keep it, but then commented out
            // See: https://github.com/dani-garcia/vaultwarden/issues/4156
            // ---
            // orgowner,
            // orgadmin,
            // orguser,
            // orgmanager,
            sstamp: user.security_stamp.clone(),
            device: device.uuid.clone(),
            devicetype: DeviceType::from_i32(device.atype).to_string(),
            client_id: client_id.unwrap_or("undefined".to_string()),
            scope,
            amr: vec!["Application".into()],
            // TASK-SEC-HIGH-02-F: inject jti only when revocation is enabled.
            jti: if CONFIG.token_revocation_enabled() {
                Some(crate::util::get_uuid())
            } else {
                None
            },
            tenant_uuid: None,
            is_tenant_admin: false,
            is_system_admin: false,
        }
    }

    pub fn default(device: &Device, user: &User, auth_method: &AuthMethod, client_id: Option<String>) -> Self {
        let time_now = Utc::now();
        Self::new(
            device,
            user,
            time_now.timestamp(),
            (time_now + *DEFAULT_ACCESS_VALIDITY).timestamp(),
            auth_method.scope_vec(),
            client_id,
            time_now,
        )
    }

    pub fn token(&self) -> Result<String, Error> {
        encode_jwt(&self)
    }

    pub fn expires_in(&self) -> i64 {
        self.exp - Utc::now().timestamp()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteJwtClaims {
    // Not before
    pub nbf: i64,
    // Expiration time
    pub exp: i64,
    // Issuer
    pub iss: String,
    // Subject
    pub sub: UserId,

    pub email: String,
    pub org_id: OrganizationId,
    pub member_id: MembershipId,
    pub invited_by_email: Option<String>,
}

pub fn generate_invite_claims(
    user_id: UserId,
    email: String,
    org_id: OrganizationId,
    member_id: MembershipId,
    invited_by_email: Option<String>,
) -> InviteJwtClaims {
    let time_now = Utc::now();
    let expire_hours = i64::from(CONFIG.invitation_expiration_hours());
    InviteJwtClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_hours(expire_hours).unwrap()).timestamp(),
        iss: JWT_INVITE_ISSUER.to_string(),
        sub: user_id,
        email,
        org_id,
        member_id,
        invited_by_email,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmergencyAccessInviteJwtClaims {
    // Not before
    pub nbf: i64,
    // Expiration time
    pub exp: i64,
    // Issuer
    pub iss: String,
    // Subject
    pub sub: UserId,

    pub email: String,
    pub emer_id: EmergencyAccessId,
    pub grantor_name: String,
    pub grantor_email: String,
}

pub fn generate_emergency_access_invite_claims(
    user_id: UserId,
    email: String,
    emer_id: EmergencyAccessId,
    grantor_name: String,
    grantor_email: String,
) -> EmergencyAccessInviteJwtClaims {
    let time_now = Utc::now();
    let expire_hours = i64::from(CONFIG.invitation_expiration_hours());
    EmergencyAccessInviteJwtClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_hours(expire_hours).unwrap()).timestamp(),
        iss: JWT_EMERGENCY_ACCESS_INVITE_ISSUER.to_string(),
        sub: user_id,
        email,
        emer_id,
        grantor_name,
        grantor_email,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrgApiKeyLoginJwtClaims {
    // Not before
    pub nbf: i64,
    // Expiration time
    pub exp: i64,
    // Issuer
    pub iss: String,
    // Subject
    pub sub: OrgApiKeyId,

    pub client_id: String,
    pub client_sub: OrganizationId,
    pub scope: Vec<String>,
}

pub fn generate_organization_api_key_login_claims(
    org_api_key_uuid: OrgApiKeyId,
    org_id: OrganizationId,
) -> OrgApiKeyLoginJwtClaims {
    let time_now = Utc::now();
    OrgApiKeyLoginJwtClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_hours(1).unwrap()).timestamp(),
        iss: JWT_ORG_API_KEY_ISSUER.to_string(),
        sub: org_api_key_uuid,
        client_id: format!("organization.{org_id}"),
        client_sub: org_id,
        scope: vec!["api.organization".into()],
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDownloadClaims {
    // Not before
    pub nbf: i64,
    // Expiration time
    pub exp: i64,
    // Issuer
    pub iss: String,
    // Subject
    pub sub: CipherId,

    pub file_id: AttachmentId,
}

pub fn generate_file_download_claims(cipher_id: CipherId, file_id: AttachmentId) -> FileDownloadClaims {
    let time_now = Utc::now();
    FileDownloadClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_minutes(5).unwrap()).timestamp(),
        iss: JWT_FILE_DOWNLOAD_ISSUER.to_string(),
        sub: cipher_id,
        file_id,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterVerifyClaims {
    // Not before
    pub nbf: i64,
    // Expiration time
    pub exp: i64,
    // Issuer
    pub iss: String,
    // Subject
    pub sub: String,

    pub name: Option<String>,
    pub verified: bool,
}

pub fn generate_register_verify_claims(email: String, name: Option<String>, verified: bool) -> RegisterVerifyClaims {
    let time_now = Utc::now();
    RegisterVerifyClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_minutes(30).unwrap()).timestamp(),
        iss: JWT_REGISTER_VERIFY_ISSUER.to_string(),
        sub: email,
        name,
        verified,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicJwtClaims {
    // Not before
    pub nbf: i64,
    // Expiration time
    pub exp: i64,
    // Issuer
    pub iss: String,
    // Subject
    pub sub: String,
}

pub fn generate_delete_claims(uuid: String) -> BasicJwtClaims {
    let time_now = Utc::now();
    let expire_hours = i64::from(CONFIG.invitation_expiration_hours());
    BasicJwtClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_hours(expire_hours).unwrap()).timestamp(),
        iss: JWT_DELETE_ISSUER.to_string(),
        sub: uuid,
    }
}

pub fn generate_verify_email_claims(user_id: &UserId) -> BasicJwtClaims {
    let time_now = Utc::now();
    let expire_hours = i64::from(CONFIG.invitation_expiration_hours());
    BasicJwtClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_hours(expire_hours).unwrap()).timestamp(),
        iss: JWT_VERIFYEMAIL_ISSUER.to_string(),
        sub: user_id.to_string(),
    }
}

pub fn generate_admin_claims() -> BasicJwtClaims {
    let time_now = Utc::now();
    BasicJwtClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_minutes(CONFIG.admin_session_lifetime()).unwrap()).timestamp(),
        iss: JWT_ADMIN_ISSUER.to_string(),
        sub: "admin_panel".to_string(),
    }
}

pub fn generate_send_claims(send_id: &SendId, file_id: &SendFileId) -> BasicJwtClaims {
    let time_now = Utc::now();
    BasicJwtClaims {
        nbf: time_now.timestamp(),
        exp: (time_now + TimeDelta::try_minutes(2).unwrap()).timestamp(),
        iss: JWT_SEND_ISSUER.to_string(),
        sub: format!("{send_id}/{file_id}"),
    }
}

//
// Bearer token authentication
//
use rocket::{
    outcome::try_outcome,
    request::{FromRequest, Outcome, Request},
};

use crate::db::{
    models::{Collection, Device, Membership, MembershipStatus, MembershipType, User, UserStampException},
    DbConn,
};

pub struct Host {
    pub host: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Host {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();

        // Get host
        let host = if CONFIG.domain_set() {
            CONFIG.domain()
        } else if let Some(referer) = headers.get_one("Referer") {
            referer.to_string()
        } else {
            // Try to guess from the headers
            let protocol = if let Some(proto) = headers.get_one("X-Forwarded-Proto") {
                proto
            } else if env::var("ROCKET_TLS").is_ok() {
                "https"
            } else {
                "http"
            };

            let host = if let Some(host) = headers.get_one("X-Forwarded-Host") {
                host
            } else {
                headers.get_one("Host").unwrap_or_default()
            };

            format!("{protocol}://{host}")
        };

        Outcome::Success(Host {
            host,
        })
    }
}

pub struct ClientHeaders {
    pub device_type: i32,
    pub ip: ClientIp,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientHeaders {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let ip = match ClientIp::from_request(request).await {
            Outcome::Success(ip) => ip,
            _ => err_handler!("Error getting Client IP"),
        };
        // When unknown or unable to parse, return 14, which is 'Unknown Browser'
        let device_type: i32 =
            request.headers().get_one("device-type").map(|d| d.parse().unwrap_or(14)).unwrap_or_else(|| 14);

        Outcome::Success(ClientHeaders {
            device_type,
            ip,
        })
    }
}

pub struct Headers {
    pub host: String,
    pub device: Device,
    pub user: User,
    pub ip: ClientIp,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Headers {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();

        let host = try_outcome!(Host::from_request(request).await).host;
        let ip = match ClientIp::from_request(request).await {
            Outcome::Success(ip) => ip,
            _ => err_handler!("Error getting Client IP"),
        };

        // Get access_token
        let access_token: &str = match headers.get_one("Authorization") {
            Some(a) => match a.rsplit("Bearer ").next() {
                Some(split) => split,
                None => err_handler!("No access token provided"),
            },
            None => err_handler!("No access token provided"),
        };

        // Check JWT token is valid and get device and user from it
        let Ok(claims) = decode_login(access_token) else {
            err_handler!("Invalid claim")
        };

        let device_id = claims.device;
        let user_id = claims.sub;

        let conn = match DbConn::from_request(request).await {
            Outcome::Success(conn) => conn,
            _ => err_handler!("Error getting DB"),
        };

        let Some(device) = Device::find_by_uuid_and_user(&device_id, &user_id, &conn).await else {
            err_handler!("Invalid device id")
        };

        let Some(user) = User::find_by_uuid(&user_id, &conn).await else {
            err_handler!("Device has no user associated")
        };

        if user.security_stamp != claims.sstamp {
            if let Some(stamp_exception) =
                user.stamp_exception.as_deref().and_then(|s| serde_json::from_str::<UserStampException>(s).ok())
            {
                let Some(current_route) = request.route().and_then(|r| r.name.as_deref()) else {
                    err_handler!("Error getting current route for stamp exception")
                };

                // Check if the stamp exception has expired first.
                // Then, check if the current route matches any of the allowed routes.
                // After that check the stamp in exception matches the one in the claims.
                if Utc::now().timestamp() > stamp_exception.expire {
                    // If the stamp exception has been expired remove it from the database.
                    // This prevents checking this stamp exception for new requests.
                    let mut user = user;
                    user.reset_stamp_exception();
                    if let Err(e) = user.save(&conn).await {
                        error!("Error updating user: {e:#?}");
                    }
                    err_handler!("Stamp exception is expired")
                } else if !stamp_exception.routes.contains(&current_route.to_string()) {
                    err_handler!("Invalid security stamp: Current route and exception route do not match")
                } else if stamp_exception.security_stamp != claims.sstamp {
                    err_handler!("Invalid security stamp for matched stamp exception")
                }
            } else {
                err_handler!("Invalid security stamp")
            }
        }

        // TASK-SEC-HIGH-02-F: JTI revocation check (opt-in).
        // Only performs a DB lookup when TOKEN_REVOCATION_ENABLED=true AND the token has a jti.
        if CONFIG.token_revocation_enabled() {
            if let Some(jti) = &claims.jti {
                use crate::db::models::RevokedToken;
                if RevokedToken::exists(jti, &conn).await {
                    err_handler!("Token has been revoked")
                }
            }
        }

        if let Err(e) = crate::access_control::validate_ip_allowlist(&ip.ip, None, &conn).await {
            err_handler!(e);
        }

        Outcome::Success(Headers {
            host,
            device,
            user,
            ip,
        })
    }
}

pub struct OrgHeaders {
    pub host: String,
    pub device: Device,
    pub user: User,
    pub membership_type: MembershipType,
    pub membership_status: MembershipStatus,
    pub membership: Membership,
    pub ip: ClientIp,
}

impl OrgHeaders {
    fn is_member(&self) -> bool {
        self.membership_type >= MembershipType::User
    }
    fn is_confirmed_and_admin(&self) -> bool {
        if let Some(_) = &self.membership.custom_role_uuid {
            // Soft fallback if using custom roles, normally permissions matrix handles granular action
            self.membership_status == MembershipStatus::Confirmed
        } else {
            self.membership_status == MembershipStatus::Confirmed && self.membership_type >= MembershipType::Admin
        }
    }
    fn is_confirmed_and_manager(&self) -> bool {
        if let Some(_) = &self.membership.custom_role_uuid {
            self.membership_status == MembershipStatus::Confirmed
        } else {
            self.membership_status == MembershipStatus::Confirmed && self.membership_type >= MembershipType::Manager
        }
    }
    fn is_confirmed_and_owner(&self) -> bool {
        if let Some(_) = &self.membership.custom_role_uuid {
            self.membership_status == MembershipStatus::Confirmed
        } else {
            self.membership_status == MembershipStatus::Confirmed && self.membership_type == MembershipType::Owner
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OrgHeaders {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = try_outcome!(Headers::from_request(request).await);

        // org_id is usually the second path param ("/organizations/<org_id>"),
        // but there are cases where it is a query value.
        // First check the path, if this is not a valid uuid, try the query values.
        let url_org_id: Option<OrganizationId> = {
            if let Some(Ok(org_id)) = request.param::<OrganizationId>(1) {
                Some(org_id)
            } else if let Some(Ok(org_id)) = request.query_value::<OrganizationId>("organizationId") {
                Some(org_id)
            } else {
                None
            }
        };

        match url_org_id {
            Some(org_id) if uuid::Uuid::parse_str(&org_id).is_ok() => {
                let conn = match DbConn::from_request(request).await {
                    Outcome::Success(conn) => conn,
                    _ => err_handler!("Error getting DB"),
                };

                let user = headers.user;
                let Some(membership) = Membership::find_by_user_and_org(&user.uuid, &org_id, &conn).await else {
                    err_handler!("The current user isn't member of the organization");
                };

                if let Err(e) = crate::access_control::validate_ip_allowlist(&headers.ip.ip, Some(&org_id), &conn).await {
                    err_handler!(e);
                }

                if let Err(e) = crate::access_control::validate_access_schedules(&user.uuid, Some(&org_id), &conn).await {
                    err_handler!(e);
                }

                Outcome::Success(Self {
                    host: headers.host,
                    device: headers.device,
                    user,
                    membership_type: {
                        if let Some(member_type) = MembershipType::from_i32(membership.atype) {
                            member_type
                        } else {
                            // This should only happen if the DB is corrupted
                            err_handler!("Unknown user type in the database")
                        }
                    },
                    membership_status: {
                        if let Some(member_status) = MembershipStatus::from_i32(membership.status) {
                            // NOTE: add additional check for revoked if from_i32 is ever changed
                            // to return Revoked status.
                            member_status
                        } else {
                            err_handler!("User status is either revoked or invalid.")
                        }
                    },
                    membership,
                    ip: headers.ip,
                })
            }
            _ => err_handler!("Error getting the organization id"),
        }
    }
}

pub struct AdminHeaders {
    pub host: String,
    pub device: Device,
    pub user: User,
    pub membership_type: MembershipType,
    pub ip: ClientIp,
    pub org_id: OrganizationId,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminHeaders {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = try_outcome!(OrgHeaders::from_request(request).await);
        if headers.is_confirmed_and_admin() {
            Outcome::Success(Self {
                host: headers.host,
                device: headers.device,
                user: headers.user,
                membership_type: headers.membership_type,
                ip: headers.ip,
                org_id: headers.membership.org_uuid,
            })
        } else {
            err_handler!("You need to be Admin or Owner to call this endpoint")
        }
    }
}

// col_id is usually the fourth path param ("/organizations/<org_id>/collections/<col_id>"),
// but there could be cases where it is a query value.
// First check the path, if this is not a valid uuid, try the query values.
fn get_col_id(request: &Request<'_>) -> Option<CollectionId> {
    if let Some(Ok(col_id)) = request.param::<String>(3) {
        if uuid::Uuid::parse_str(&col_id).is_ok() {
            return Some(col_id.into());
        }
    }

    if let Some(Ok(col_id)) = request.query_value::<String>("collectionId") {
        if uuid::Uuid::parse_str(&col_id).is_ok() {
            return Some(col_id.into());
        }
    }

    None
}

/// The ManagerHeaders are used to check if you are at least a Manager
/// and have access to the specific collection provided via the <col_id>/collections/collectionId.
/// This does strict checking on the collection_id, ManagerHeadersLoose does not.
pub struct ManagerHeaders {
    pub host: String,
    pub device: Device,
    pub user: User,
    pub ip: ClientIp,
    pub org_id: OrganizationId,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ManagerHeaders {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = try_outcome!(OrgHeaders::from_request(request).await);
        if headers.is_confirmed_and_manager() {
            match get_col_id(request) {
                Some(col_id) => {
                    let conn = match DbConn::from_request(request).await {
                        Outcome::Success(conn) => conn,
                        _ => err_handler!("Error getting DB"),
                    };

                    if !Collection::can_access_collection(&headers.membership, &col_id, &conn).await {
                        err_handler!("The current user isn't a manager for this collection")
                    }
                }
                _ => err_handler!("Error getting the collection id"),
            }

            Outcome::Success(Self {
                host: headers.host,
                device: headers.device,
                user: headers.user,
                ip: headers.ip,
                org_id: headers.membership.org_uuid,
            })
        } else {
            err_handler!("You need to be a Manager, Admin or Owner to call this endpoint")
        }
    }
}

impl From<ManagerHeaders> for Headers {
    fn from(h: ManagerHeaders) -> Headers {
        Headers {
            host: h.host,
            device: h.device,
            user: h.user,
            ip: h.ip,
        }
    }
}

/// The ManagerHeadersLoose is used when you at least need to be a Manager,
/// but there is no collection_id sent with the request (either in the path or as form data).
pub struct ManagerHeadersLoose {
    pub host: String,
    pub device: Device,
    pub user: User,
    pub membership: Membership,
    pub ip: ClientIp,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ManagerHeadersLoose {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = try_outcome!(OrgHeaders::from_request(request).await);
        if headers.is_confirmed_and_manager() {
            Outcome::Success(Self {
                host: headers.host,
                device: headers.device,
                user: headers.user,
                membership: headers.membership,
                ip: headers.ip,
            })
        } else {
            err_handler!("You need to be a Manager, Admin or Owner to call this endpoint")
        }
    }
}

impl From<ManagerHeadersLoose> for Headers {
    fn from(h: ManagerHeadersLoose) -> Headers {
        Headers {
            host: h.host,
            device: h.device,
            user: h.user,
            ip: h.ip,
        }
    }
}

impl ManagerHeaders {
    pub async fn from_loose(
        h: ManagerHeadersLoose,
        collections: &Vec<CollectionId>,
        conn: &DbConn,
    ) -> Result<ManagerHeaders, Error> {
        for col_id in collections {
            if uuid::Uuid::parse_str(col_id.as_ref()).is_err() {
                err!("Collection Id is malformed!");
            }
            if !Collection::can_access_collection(&h.membership, col_id, conn).await {
                err!("You don't have access to all collections!");
            }
        }

        Ok(ManagerHeaders {
            host: h.host,
            device: h.device,
            user: h.user,
            ip: h.ip,
            org_id: h.membership.org_uuid,
        })
    }
}

pub struct OwnerHeaders {
    pub device: Device,
    pub user: User,
    pub ip: ClientIp,
    pub org_id: OrganizationId,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OwnerHeaders {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = try_outcome!(OrgHeaders::from_request(request).await);
        if headers.is_confirmed_and_owner() {
            Outcome::Success(Self {
                device: headers.device,
                user: headers.user,
                ip: headers.ip,
                org_id: headers.membership.org_uuid,
            })
        } else {
            err_handler!("You need to be Owner to call this endpoint")
        }
    }
}

pub struct OrgMemberHeaders {
    pub host: String,
    pub device: Device,
    pub user: User,
    pub membership: Membership,
    pub ip: ClientIp,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OrgMemberHeaders {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = try_outcome!(OrgHeaders::from_request(request).await);
        if headers.is_member() {
            Outcome::Success(Self {
                host: headers.host,
                device: headers.device,
                user: headers.user,
                membership: headers.membership,
                ip: headers.ip,
            })
        } else {
            err_handler!("You need to be a Member of the Organization to call this endpoint")
        }
    }
}

impl From<OrgMemberHeaders> for Headers {
    fn from(h: OrgMemberHeaders) -> Headers {
        Headers {
            host: h.host,
            device: h.device,
            user: h.user,
            ip: h.ip,
        }
    }
}

//
// Client IP address detection
//

pub struct ClientIp {
    pub ip: IpAddr,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientIp {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // SEC-HIGH-04-E: When TRUSTED_PROXIES is configured, use the secure
        // get_real_ip() which only honours XFF from trusted CIDR ranges.
        let ip = if !CONFIG.trusted_proxies().trim().is_empty() {
            crate::util::get_real_ip(req)
        } else if CONFIG._ip_header_enabled() {
            req.headers()
                .get_one(&CONFIG.ip_header())
                .and_then(|ip| {
                    match ip.find(',') {
                        Some(idx) => &ip[..idx],
                        None => ip,
                    }
                    .parse()
                    .map_err(|_| warn!("'{}' header is malformed: {ip}", CONFIG.ip_header()))
                    .ok()
                })
                .or_else(|| req.remote().map(|r| r.ip()))
                .unwrap_or_else(|| "0.0.0.0".parse().unwrap())
        } else {
            req.remote().map(|r| r.ip()).unwrap_or_else(|| "0.0.0.0".parse().unwrap())
        };

        Outcome::Success(ClientIp {
            ip,
        })
    }
}

pub struct Secure {
    pub https: bool,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Secure {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();

        // Try to guess from the headers
        let protocol = match headers.get_one("X-Forwarded-Proto") {
            Some(proto) => proto,
            None => {
                if env::var("ROCKET_TLS").is_ok() {
                    "https"
                } else {
                    "http"
                }
            }
        };

        Outcome::Success(Secure {
            https: protocol == "https",
        })
    }
}

pub struct WsAccessTokenHeader {
    pub access_token: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WsAccessTokenHeader {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();

        // Get access_token
        let access_token = match headers.get_one("Authorization") {
            Some(a) => a.rsplit("Bearer ").next().map(String::from),
            None => None,
        };

        Outcome::Success(Self {
            access_token,
        })
    }
}

pub struct ClientVersion(pub semver::Version);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientVersion {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();

        let Some(version) = headers.get_one("Bitwarden-Client-Version") else {
            err_handler!("No Bitwarden-Client-Version header provided")
        };

        let Ok(version) = semver::Version::parse(version) else {
            err_handler!("Invalid Bitwarden-Client-Version header provided")
        };

        Outcome::Success(ClientVersion(version))
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    OrgApiKey,
    Password,
    Sso,
    UserApiKey,
}

impl AuthMethod {
    pub fn scope(&self) -> String {
        match self {
            AuthMethod::OrgApiKey => "api.organization".to_string(),
            AuthMethod::Password => "api offline_access".to_string(),
            AuthMethod::Sso => "api offline_access".to_string(),
            AuthMethod::UserApiKey => "api".to_string(),
        }
    }

    pub fn scope_vec(&self) -> Vec<String> {
        self.scope().split_whitespace().map(str::to_string).collect()
    }

    pub fn check_scope(&self, scope: Option<&String>) -> ApiResult<String> {
        let method_scope = self.scope();
        match scope {
            None => err!("Missing scope"),
            Some(scope) if scope == &method_scope => Ok(method_scope),
            Some(scope) => err!(format!("Scope ({scope}) not supported")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TokenWrapper {
    Access(String),
    Refresh(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshJwtClaims {
    // Not before
    pub nbf: i64,
    // Expiration time
    pub exp: i64,
    // Issuer
    pub iss: String,
    // Subject
    pub sub: AuthMethod,

    pub device_token: String,

    pub token: Option<TokenWrapper>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokens {
    pub refresh_claims: RefreshJwtClaims,
    pub access_claims: LoginJwtClaims,
}

impl AuthTokens {
    pub fn refresh_token(&self) -> Result<String, Error> {
        encode_jwt(&self.refresh_claims)
    }

    pub fn access_token(&self) -> Result<String, Error> {
        self.access_claims.token()
    }

    pub fn expires_in(&self) -> i64 {
        self.access_claims.expires_in()
    }

    pub fn scope(&self) -> String {
        self.refresh_claims.sub.scope()
    }

    // Create refresh_token and access_token with default validity
    pub fn new(device: &Device, user: &User, sub: AuthMethod, client_id: Option<String>) -> Self {
        let time_now = Utc::now();

        let access_claims = LoginJwtClaims::default(device, user, &sub, client_id);

        let validity = if device.is_mobile() {
            *MOBILE_REFRESH_VALIDITY
        } else {
            *DEFAULT_REFRESH_VALIDITY
        };

        let refresh_claims = RefreshJwtClaims {
            nbf: time_now.timestamp(),
            exp: (time_now + validity).timestamp(),
            iss: JWT_LOGIN_ISSUER.to_string(),
            sub,
            device_token: device.refresh_token.clone(),
            token: None,
        };

        Self {
            refresh_claims,
            access_claims,
        }
    }
}

pub async fn refresh_tokens(
    ip: &ClientIp,
    refresh_token: &str,
    client_id: Option<String>,
    conn: &DbConn,
) -> ApiResult<(Device, AuthTokens)> {
    let refresh_claims = match decode_refresh(refresh_token) {
        Err(err) => {
            debug!("Failed to decode {} refresh_token: {refresh_token}", ip.ip);
            err_silent!(format!("Impossible to read refresh_token: {}", err.message()))
        }
        Ok(claims) => claims,
    };

    // Get device by refresh token
    let mut device = match Device::find_by_refresh_token(&refresh_claims.device_token, conn).await {
        None => err!("Invalid refresh token"),
        Some(device) => device,
    };

    // Save to update `updated_at`.
    device.save(conn).await?;

    let user = match User::find_by_uuid(&device.user_uuid, conn).await {
        None => err!("Impossible to find user"),
        Some(user) => user,
    };

    let auth_tokens = match refresh_claims.sub {
        AuthMethod::Sso if CONFIG.sso_enabled() && CONFIG.sso_auth_only_not_session() => {
            AuthTokens::new(&device, &user, refresh_claims.sub, client_id)
        }
        AuthMethod::Sso if CONFIG.sso_enabled() => {
            sso::exchange_refresh_token(&device, &user, client_id, refresh_claims).await?
        }
        AuthMethod::Sso => err!("SSO is now disabled, Login again using email and master password"),
        AuthMethod::Password if CONFIG.sso_enabled() && CONFIG.sso_only() => err!("SSO is now required, Login again"),
        AuthMethod::Password => AuthTokens::new(&device, &user, refresh_claims.sub, client_id),
        _ => err!("Invalid auth method, cannot refresh token"),
    };

    Ok((device, auth_tokens))
}

pub struct ApiKeyAuth {
    #[allow(dead_code)]
    pub org_uuid: String,
    #[allow(dead_code)]
    pub api_key_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKeyAuth {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use rocket::http::Status;
        if !CONFIG.api_key_v2_enabled() {
            return Outcome::Error((Status::Forbidden, "API Keys V2 disabled"));
        }

        let mut conn = match DbConn::from_request(request).await {
            Outcome::Success(conn) => conn,
            _ => return Outcome::Error((Status::InternalServerError, "Database connection failed")),
        };

        let headers = request.headers();
        let _token = if let Some(auth) = headers.get_one("Authorization") {
            if auth.starts_with("Bearer ") {
                &auth[7..]
            } else {
                return Outcome::Error((Status::Unauthorized, "Invalid Authorization header"));
            }
        } else {
            return Outcome::Error((Status::Unauthorized, "Missing Authorization header"));
        };

        // "client_id:secret_candidate" format inside Bearer token
        let parts: Vec<&str> = _token.split(':').collect();
        if parts.len() != 2 {
            return Outcome::Error((Status::Unauthorized, "Invalid token format"));
        }
        let client_id = parts[0];
        let secret_candidate = parts[1];

        let mut api_key = match crate::db::models::ApiKeyV2::find_by_client_id(client_id, &mut conn).await {
            Some(k) => k,
            None => return Outcome::Error((Status::Unauthorized, "Invalid API Key")),
        };

        if !api_key.is_active {
            return Outcome::Error((Status::Unauthorized, "API Key is disabled"));
        }

        if !api_key.verify_token(secret_candidate) {
            return Outcome::Error((Status::Unauthorized, "Invalid API Key"));
        }

        // TODO: Check rate limit & allowed IPs

        // Touch the key in the background
        if let Err(e) = api_key.touch(&mut conn).await {
            error!("Failed to touch API key: {:?}", e);
        }

        Outcome::Success(ApiKeyAuth {
            org_uuid: api_key.org_uuid.clone(),
            api_key_id: api_key.uuid.clone(),
        })
    }
}

#[allow(dead_code)]
pub fn require_scope(api_key: &crate::db::models::ApiKeyV2, scope: &str) -> crate::api::EmptyResult {
    let scopes: Vec<&str> = api_key.scopes.split(',').collect();
    if scopes.contains(&scope) {
        Ok(())
    } else {
        err!("API Key missing required scope")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-011-017: SystemAdminHeaders request guard
// ─────────────────────────────────────────────────────────────────────────────

/// Guard that validates the X-System-Admin-Token header via constant-time SHA-256.
pub struct SystemAdminHeaders;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SystemAdminHeaders {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use rocket::http::Status;
        let stored_token = CONFIG.system_admin_token();
        if stored_token.is_empty() {
            return Outcome::Error((Status::Unauthorized, "System admin token not configured"));
        }

        if let Some(provided) = request.headers().get_one("X-System-Admin-Token") {
            use ring::digest;
            use data_encoding::HEXLOWER;
            let p_hash = HEXLOWER.encode(digest::digest(&digest::SHA256, provided.as_bytes()).as_ref());
            let s_hash = HEXLOWER.encode(digest::digest(&digest::SHA256, stored_token.as_bytes()).as_ref());
            if p_hash == s_hash {
                return Outcome::Success(SystemAdminHeaders);
            }
        }

        Outcome::Error((Status::Unauthorized, "Invalid system admin token"))
    }
}
