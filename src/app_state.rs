/// TASK-RUSTDEV-MED-03: AppState — dependency-injectable application state.
///
/// `AppState` is registered with Rocket via `.manage(app_state)` and can be
/// injected into route handlers as `state: &State<AppState>`. This enables
/// replacing global statics (CONFIG, LIMITER_LOGIN) with testable, injectable
/// dependencies — without requiring a full migration of every handler up front.
///
/// The `RateLimiter` trait abstracts the rate-limiting backend so tests can
/// inject a `NoopRateLimiter` or `CountingRateLimiter` instead of the real
/// governor-backed limiter.
use std::{net::IpAddr, sync::Arc};

use crate::Error;

/// Abstraction over a rate-limiting backend.
/// Implementations: `IpRateLimiter` (production), `NoopRateLimiter` (tests).
#[rocket::async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check whether the given key is within rate limits.
    /// Returns `Ok(())` if the request is allowed, or `Err` if it should be blocked.
    async fn check_login(&self, ip: &IpAddr) -> Result<(), Error>;
    /// Check admin panel rate limit.
    /// Used by the admin panel handler once it migrates to AppState (Sprint 4).
    #[allow(dead_code)]
    async fn check_admin(&self, ip: &IpAddr) -> Result<(), Error>;
}

/// Production rate limiter that delegates to the existing global `LIMITER_LOGIN`
/// and `LIMITER_ADMIN` in `ratelimit.rs`.  No behavior change — this is a thin
/// wrapper so handlers can use the trait instead of calling `check_limit_login`
/// directly.
pub struct IpRateLimiter;

#[rocket::async_trait]
impl RateLimiter for IpRateLimiter {
    async fn check_login(&self, ip: &IpAddr) -> Result<(), Error> {
        crate::ratelimit::check_limit_login(ip).await
    }

    async fn check_admin(&self, ip: &IpAddr) -> Result<(), Error> {
        crate::ratelimit::check_limit_admin(ip).await
    }
}

/// Application-wide shared state managed by Rocket.
///
/// # Adding fields
/// Add new injectable dependencies here.  Keep the existing global statics
/// (CONFIG etc.) for backward compat during the incremental migration — handlers
/// can be updated one by one to read from `AppState` instead of the globals.
pub struct AppState {
    pub rate_limiter: Arc<dyn RateLimiter>,
}

impl AppState {
    /// Construct the production `AppState`.
    pub fn new() -> Self {
        Self {
            rate_limiter: Arc::new(IpRateLimiter),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod test_utils {
    //! TASK-RUSTDEV-MED-03-D: Test utilities for injecting mock rate limiters.
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Rate limiter that always allows requests — for use in unit/integration tests.
    pub struct NoopRateLimiter;

    #[rocket::async_trait]
    impl RateLimiter for NoopRateLimiter {
        async fn check_login(&self, _ip: &IpAddr) -> Result<(), Error> {
            Ok(())
        }

        async fn check_admin(&self, _ip: &IpAddr) -> Result<(), Error> {
            Ok(())
        }
    }

    /// Rate limiter that counts invocations — for tests asserting rate limiting is called.
    pub struct CountingRateLimiter {
        pub login_count: AtomicU32,
        pub admin_count: AtomicU32,
    }

    impl CountingRateLimiter {
        pub fn new() -> Self {
            Self {
                login_count: AtomicU32::new(0),
                admin_count: AtomicU32::new(0),
            }
        }
    }

    impl Default for CountingRateLimiter {
        fn default() -> Self {
            Self::new()
        }
    }

    #[rocket::async_trait]
    impl RateLimiter for CountingRateLimiter {
        async fn check_login(&self, _ip: &IpAddr) -> Result<(), Error> {
            self.login_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        async fn check_admin(&self, _ip: &IpAddr) -> Result<(), Error> {
            self.admin_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::*;
    use std::net::Ipv4Addr;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn test_noop_rate_limiter_always_ok() {
        let limiter = NoopRateLimiter;
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        assert!(limiter.check_login(&ip).await.is_ok());
        assert!(limiter.check_admin(&ip).await.is_ok());
    }

    #[tokio::test]
    async fn test_counting_rate_limiter_increments() {
        let limiter = CountingRateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        limiter.check_login(&ip).await.unwrap();
        limiter.check_login(&ip).await.unwrap();
        limiter.check_admin(&ip).await.unwrap();
        assert_eq!(limiter.login_count.load(Ordering::Relaxed), 2);
        assert_eq!(limiter.admin_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_app_state_default_constructs() {
        let _state = AppState::default();
    }
}
