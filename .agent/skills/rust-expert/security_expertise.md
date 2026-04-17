# Security Expertise — Vaultwarden Rust Expert

## 1. JWT Authentication (jsonwebtoken 10.x)

### Token Structure trong Vaultwarden
Project dùng RSA hoặc HMAC để sign JWT tùy config.

```rust
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User UUID
    pub iss: String,        // Issuer
    pub iat: i64,           // Issued at (Unix timestamp)
    pub exp: i64,           // Expiration
    pub nbf: i64,           // Not before
    pub email: String,
    pub scope: Vec<String>,
}

// Encode
fn create_token(user: &User, secret: &[u8]) -> Result<String, Error> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user.uuid.clone(),
        iss: "Vaultwarden".to_string(),
        iat: now,
        exp: now + CONFIG.token_expiration_seconds(),
        nbf: now,
        email: user.email.clone(),
        scope: vec!["api".to_string()],
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    ).map_err(|e| Error::Internal(format!("JWT encode: {e}")))
}

// Decode & Validate
fn validate_token(token: &str, secret: &[u8]) -> Result<Claims, Error> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 30; // 30 seconds clock skew tolerance
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        warn!("JWT validation failed: {e}");
        Error::Unauthorized("Invalid token".into())
    })
}
```

### RSA Key Management (JWT Signing Key Rotation)
```rust
use jsonwebtoken::{DecodingKey, EncodingKey};
use openssl::rsa::Rsa;

// Generate RSA key pair
let rsa = Rsa::generate(2048)?;
let private_pem = rsa.private_key_to_pem()?;
let public_pem = rsa.public_key_to_pem()?;

// Load keys
let encoding_key = EncodingKey::from_rsa_pem(&private_pem)?;
let decoding_key = DecodingKey::from_rsa_pem(&public_pem)?;

// Store in ArcSwap for hot-swap rotation
use arc_swap::ArcSwap;
static JWT_KEYS: Lazy<ArcSwap<JwtKeys>> = Lazy::new(|| 
    ArcSwap::from_pointee(JwtKeys::load_or_generate())
);

pub fn rotate_keys() {
    let new_keys = JwtKeys::generate();
    JWT_KEYS.store(Arc::new(new_keys));
}
```

---

## 2. Argon2 Password Hashing

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};

// Hash password (use in user creation/password change)
pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    
    // Use project-configured params (CPU-intensive by design)
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(
            CONFIG.password_iterations(),  // memory cost
            3,                              // time cost
            4,                              // parallelism
            None,
        )?,
    );
    
    argon2.hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| Error::Internal("Hashing failed".into()))
}

// Verify password
pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}
```

---

## 3. Cryptography (ring)

```rust
use ring::{digest, hmac, rand, signature};
use data_encoding::BASE64;

// Secure random bytes
pub fn secure_random_bytes(len: usize) -> Vec<u8> {
    let rng = rand::SystemRandom::new();
    let mut bytes = vec![0u8; len];
    rng.fill(&mut bytes).expect("RNG failed");
    bytes
}

// HMAC-SHA256
pub fn hmac_sign(key: &[u8], data: &[u8]) -> Vec<u8> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::sign(&key, data).as_ref().to_vec()
}

pub fn hmac_verify(key: &[u8], data: &[u8], signature: &[u8]) -> bool {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::verify(&key, data, signature).is_ok()
}

// SHA-256 hash
pub fn sha256(data: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, data).as_ref().to_vec()
}

// Constant-time comparison (CRITICAL for security)
use subtle::ConstantTimeEq;
pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    // NEVER use == for security-sensitive comparisons
    // Timing attack resistant
    a.ct_eq(b).into()
}
```

---

## 4. WebAuthn / FIDO2

```rust
use webauthn_rs::{Webauthn, WebauthnBuilder};
use webauthn_rs_proto::*;

// Initialize WebAuthn
pub fn build_webauthn(rp_id: &str, rp_origin: &Url) -> Result<Webauthn, Error> {
    WebauthnBuilder::new(rp_id, rp_origin)?
        .rp_name("Vaultwarden")
        .build()
        .map_err(|e| Error::Internal(format!("WebAuthn init: {e}")))
}

// Registration flow
pub async fn start_webauthn_registration(
    user: &User,
    webauthn: &Webauthn,
    conn: &mut DbConn,
) -> Result<(CreationChallengeResponse, PasskeyRegistration), Error> {
    let existing_creds = get_existing_credentials(user, conn).await?;
    
    webauthn.start_passkey_registration(
        uuid::Uuid::parse_str(&user.uuid)?,
        &user.email,
        &user.name,
        Some(existing_creds),
    ).map_err(|e| Error::Internal(format!("WebAuthn reg: {e}")))
}
```

---

## 5. OIDC / SSO Integration

```rust
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl,
    Nonce, PkceCodeChallenge, RedirectUrl, Scope,
};

// State machine pattern cho OIDC flow
pub enum SsoState {
    Initiating { nonce: Nonce, pkce_verifier: PkceCodeVerifier },
    Validating { code: AuthorizationCode },
    Complete { user_info: UserInfo },
}

// Discovery
pub async fn discover_provider(issuer: &str) -> Result<CoreProviderMetadata, Error> {
    CoreProviderMetadata::discover_async(
        IssuerUrl::new(issuer.to_string())?,
        async_http_client,
    ).await.map_err(|e| Error::Internal(format!("OIDC discovery: {e}")))
}
```

---

## 6. LDAP Integration

```rust
use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};

pub async fn ldap_authenticate(
    username: &str,
    password: &str,
) -> Result<LdapUser, Error> {
    let settings = LdapConnSettings::new()
        .set_starttls(CONFIG.ldap_ssl())
        .set_no_tls_verify(CONFIG.ldap_skip_cert_verify());
    
    let (conn, mut ldap) = LdapConnAsync::with_settings(
        settings,
        &CONFIG.ldap_host(),
    ).await?;
    
    ldap3::drive!(conn);
    
    // Bind con kết nối với service account
    ldap.simple_bind(&CONFIG.ldap_bind_dn(), &CONFIG.ldap_bind_password())
        .await?
        .success()?;
    
    // Search user
    let (entries, _) = ldap.search(
        &CONFIG.ldap_base_dn(),
        Scope::Subtree,
        &format!("({}={})", CONFIG.ldap_username_attr(), username),
        vec!["dn", "mail", "cn", "memberOf"],
    ).await?.success()?;
    
    let entry = entries.into_iter()
        .next()
        .ok_or_else(|| Error::Unauthorized("User not found in LDAP".into()))?;
    
    let entry = SearchEntry::construct(entry);
    
    // Bind với user credentials để verify password
    ldap.simple_bind(&entry.dn, password).await?.success()
        .map_err(|_| Error::Unauthorized("Invalid credentials".into()))?;
    
    Ok(LdapUser::from_entry(&entry))
}
```

---

## 7. Input Validation & Sanitization

```rust
use email_address::EmailAddress;
use url::Url;

// Email validation
pub fn validate_email(email: &str) -> Result<(), Error> {
    if !EmailAddress::is_valid(email) {
        return Err(Error::BadRequest("Invalid email format".into()));
    }
    Ok(())
}

// URL validation (for WebAuthn origins, OIDC, etc.)
pub fn validate_url(url_str: &str) -> Result<Url, Error> {
    Url::parse(url_str)
        .map_err(|_| Error::BadRequest(format!("Invalid URL: {url_str}")))
}

// UUID validation
pub fn validate_uuid(id: &str) -> Result<(), Error> {
    uuid::Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| Error::BadRequest(format!("Invalid UUID: {id}")))
}

// Strip HTML để tránh XSS trong user-supplied content
pub fn sanitize_string(input: &str) -> String {
    input.chars()
        .filter(|c| !matches!(c, '<' | '>' | '"' | '\''))
        .collect()
}
```

---

## 8. Security Headers & Rate Limiting

```rust
// Rate limited endpoint guard
use crate::ratelimit::RateLimitGuard;

#[post("/login", data = "<data>")]
async fn login(
    _rate_limit: RateLimitGuard<LoginRateLimit>,  // Auto-applied per IP
    data: Json<LoginData>,
    conn: DbConn,
) -> JsonResult {
    ...
}

// Trong src/ratelimit.rs — Governor-based implementation
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};

pub struct RateLimitGuard<Tag>(PhantomData<Tag>);

#[rocket::async_trait]
impl<'r, Tag: Send + Sync + 'static> FromRequest<'r> for RateLimitGuard<Tag> {
    type Error = Error;
    
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let limiter = req.rocket().state::<RateLimiterState>().unwrap();
        let ip = req.client_ip().unwrap_or(IpAddr::from([127, 0, 0, 1]));
        
        match limiter.check_key(&ip) {
            Ok(_) => Outcome::Success(RateLimitGuard(PhantomData)),
            Err(_) => Outcome::Error((Status::TooManyRequests, Error::RateLimited)),
        }
    }
}
```
