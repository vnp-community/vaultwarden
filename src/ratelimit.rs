use std::{
    collections::HashSet,
    net::IpAddr,
    num::NonZeroU32,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use governor::{clock::DefaultClock, state::keyed::DashMapStateStore, Quota, RateLimiter};

use crate::{Error, CONFIG, cache::CACHE};

type Limiter<T = IpAddr> = RateLimiter<T, DashMapStateStore<T>, DefaultClock>;
// Per-account rate limiter uses a hashed email string as key (not raw PII)
type AccountLimiter = RateLimiter<String, DashMapStateStore<String>, DefaultClock>;

static LIMITER_LOGIN: LazyLock<Limiter> = LazyLock::new(|| {
    let seconds = Duration::from_secs(CONFIG.login_ratelimit_seconds());
    let burst = NonZeroU32::new(CONFIG.login_ratelimit_max_burst()).expect("Non-zero login ratelimit burst");
    RateLimiter::keyed(Quota::with_period(seconds).expect("Non-zero login ratelimit seconds").allow_burst(burst))
});

static LIMITER_ADMIN: LazyLock<Limiter> = LazyLock::new(|| {
    let seconds = Duration::from_secs(CONFIG.admin_ratelimit_seconds());
    let burst = NonZeroU32::new(CONFIG.admin_ratelimit_max_burst()).expect("Non-zero admin ratelimit burst");
    RateLimiter::keyed(Quota::with_period(seconds).expect("Non-zero admin ratelimit seconds").allow_burst(burst))
});

static LIMITER_ACCOUNT: LazyLock<AccountLimiter> = LazyLock::new(|| {
    let seconds = Duration::from_secs(CONFIG.login_ratelimit_seconds());
    let burst = NonZeroU32::new(CONFIG.login_ratelimit_max_burst()).expect("Non-zero login ratelimit burst");
    RateLimiter::keyed(Quota::with_period(seconds).expect("Non-zero account ratelimit seconds").allow_burst(burst))
});

static CRED_STUFF_TRACKER: LazyLock<Mutex<std::collections::HashMap<String, (Instant, HashSet<IpAddr>)>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

fn hash_email(email: &str) -> String {
    use ring::digest;
    let hash = digest::digest(&digest::SHA256, email.to_lowercase().as_bytes());
    data_encoding::HEXLOWER.encode(hash.as_ref())
}

async fn check_rate_limit(prefix: &str, identifier: &str, burst: u32, window_secs: u64, msg: &str) -> Result<(), Error> {
    if CONFIG.cluster_mode() {
        let key = format!("rl:{}:{}", prefix, identifier);
        match CACHE.increment(&key, 1, window_secs).await {
            Ok(count) => {
                if count > burst as u64 {
                    err_code!(msg, 429);
                }
            }
            Err(_) => {
                // Fail open or closed? Typically open to not block legit users if Redis is down momentarily
                warn!("Failed to check rate limit in cluster mode for {}", key);
            }
        }
        Ok(())
    } else {
        // Handled by traditional governor limits
        Ok(())
    }
}

pub async fn check_limit_login(ip: &IpAddr) -> Result<(), Error> {
    if CONFIG.cluster_mode() {
        check_rate_limit("ip_login", &ip.to_string(), CONFIG.login_ratelimit_max_burst(), CONFIG.login_ratelimit_seconds(), "Too many login requests").await
    } else {
        match LIMITER_LOGIN.check_key(ip) {
            Ok(_) => Ok(()),
            Err(_e) => {
                if CONFIG.metrics_enabled() {
                    crate::metrics::METRICS.rate_limit_triggers
                        .get_or_create(&crate::metrics::RateLimitLabels { route: "login".to_string() })
                        .inc();
                }
                err_code!("Too many login requests", 429);
            }
        }
    }
}

pub async fn check_limit_login_account(email: &str) -> Result<(), Error> {
    let hashed = hash_email(email);
    if CONFIG.cluster_mode() {
        check_rate_limit("acct_login", &hashed, CONFIG.login_ratelimit_max_burst(), CONFIG.login_ratelimit_seconds(), "Too many login attempts for this account").await
    } else {
        match LIMITER_ACCOUNT.check_key(&hashed) {
            Ok(_) => Ok(()),
            Err(_) => {
                err_code!("Too many login attempts for this account", 429);
            }
        }
    }
}

pub fn detect_credential_stuffing(email: &str, ip: &IpAddr) {
    const WINDOW_SECS: u64 = 900; // 15 minutes
    const UNIQUE_IP_THRESHOLD: usize = 5;

    let key = hash_email(email);
    let mut tracker = match CRED_STUFF_TRACKER.lock() {
        Ok(t) => t,
        Err(e) => {
            error!("CRED_STUFF_TRACKER lock poisoned: {e}");
            return;
        }
    };

    let now = Instant::now();
    let entry = tracker.entry(key).or_insert_with(|| (now, HashSet::new()));

    // Slide the window: reset if older than WINDOW_SECS
    if now.duration_since(entry.0).as_secs() > WINDOW_SECS {
        entry.0 = now;
        entry.1.clear();
    }

    entry.1.insert(*ip);
    let unique_count = entry.1.len();

    if unique_count >= UNIQUE_IP_THRESHOLD {
        error!(
            "SECURITY AUDIT [CredentialStuffing]: {} unique source IPs targeted the same account \
             in the last 15 minutes. This is a strong indicator of a distributed credential stuffing \
             attack. Consider enabling ACCOUNT_LOCKOUT or reviewing access logs.",
            unique_count
        );
    }
}

pub async fn check_limit_admin(ip: &IpAddr) -> Result<(), Error> {
    if CONFIG.cluster_mode() {
        check_rate_limit("admin", &ip.to_string(), CONFIG.admin_ratelimit_max_burst(), CONFIG.admin_ratelimit_seconds(), "Too many admin requests").await
    } else {
        match LIMITER_ADMIN.check_key(ip) {
            Ok(_) => Ok(()),
            Err(_e) => {
                if CONFIG.metrics_enabled() {
                    crate::metrics::METRICS.rate_limit_triggers
                        .get_or_create(&crate::metrics::RateLimitLabels { route: "admin".to_string() })
                        .inc();
                }
                err_code!("Too many admin requests", 429);
            }
        }
    }
}

static LIMITER_ANON_WS: LazyLock<Limiter> = LazyLock::new(|| {
    let burst = std::env::var("WS_ANON_RATELIMIT_BURST")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .and_then(NonZeroU32::new)
        .unwrap_or(NonZeroU32::new(10).unwrap());
    let window = Duration::from_secs(60);
    RateLimiter::keyed(Quota::with_period(window).expect("Non-zero anon WS window").allow_burst(burst))
});

pub async fn check_limit_anon_ws(ip: &IpAddr) -> Result<(), Error> {
    let burst = std::env::var("WS_ANON_RATELIMIT_BURST")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(10);
    
    if CONFIG.cluster_mode() {
        check_rate_limit("anon_ws", &ip.to_string(), burst, 60, "Too many anonymous WebSocket connection attempts from this IP").await
    } else {
        match LIMITER_ANON_WS.check_key(ip) {
            Ok(_) => Ok(()),
            Err(_) => {
                err_code!("Too many anonymous WebSocket connection attempts from this IP", 429)
            }
        }
    }
}
