// TASK-010-015: Security Alerting Engine
// Sliding window counters for failed logins and rate limit triggers.
// Background task checks thresholds every 10 seconds.

use std::sync::{LazyLock, Mutex};
use std::collections::VecDeque;
use std::time::Instant;

/// A sliding-window rate tracker
struct SlidingWindow {
    events: VecDeque<Instant>,
    window_secs: u64,
}

impl SlidingWindow {
    fn new(window_secs: u64) -> Self {
        Self { events: VecDeque::new(), window_secs }
    }

    fn push(&mut self) {
        let now = Instant::now();
        self.events.push_back(now);
        // evict old events
        while let Some(&front) = self.events.front() {
            if now.duration_since(front).as_secs() > self.window_secs {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    fn count(&self) -> usize {
        let now = Instant::now();
        self.events.iter().filter(|&&t| now.duration_since(t).as_secs() <= self.window_secs).count()
    }
}

static FAILED_LOGINS_WINDOW: LazyLock<Mutex<SlidingWindow>> =
    LazyLock::new(|| Mutex::new(SlidingWindow::new(60)));

static RATE_LIMIT_WINDOW: LazyLock<Mutex<SlidingWindow>> =
    LazyLock::new(|| Mutex::new(SlidingWindow::new(60)));

/// Call this whenever a login fails
pub fn record_failed_login() {
    if let Ok(mut w) = FAILED_LOGINS_WINDOW.lock() {
        w.push();
    }
}

/// Call this whenever a rate limit is triggered
pub fn record_rate_limit() {
    if let Ok(mut w) = RATE_LIMIT_WINDOW.lock() {
        w.push();
    }
}

/// Send an alert via email and/or Slack webhook
async fn send_alert(subject: &str, message: &str) {
    // Email alert
    let email = crate::CONFIG.security_alert_email();
    if !email.is_empty() {
        warn!("[SecurityAlert] {} - {} (Email alerts not yet wired to mail::)", subject, message);
    }

    // Slack/Teams webhook
    let webhook_url = crate::CONFIG.security_alert_webhook_url();
    if !webhook_url.is_empty() {
        let body = serde_json::json!({ "text": format!("*{}*\n{}", subject, message) });
        let client = reqwest::Client::new();
        drop(client.post(&webhook_url)
            .json(&body)
            .send()
            .await);
    }
}

/// Background task: evaluate thresholds every 10 seconds
pub async fn start_alerting_engine() {
    if !crate::CONFIG.security_alerts_enabled() {
        return;
    }

    let failed_logins_threshold = crate::CONFIG.alert_failed_logins_per_minute();
    let rate_limit_threshold = crate::CONFIG.alert_rate_limit_per_minute();

    let mut last_failed_alert = Instant::now();
    let mut last_rl_alert = Instant::now();
    let cooldown = std::time::Duration::from_secs(300); // 5-min alert cooldown

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        // Check failed logins
        let failed = FAILED_LOGINS_WINDOW.lock().map(|w| w.count()).unwrap_or(0);
        if failed as u32 >= failed_logins_threshold && last_failed_alert.elapsed() > cooldown {
            warn!("[SecurityAlert] High failed login rate: {failed}/min (threshold: {failed_logins_threshold})");
            send_alert(
                "High Failed Login Rate",
                &format!("{failed} failed logins in the last minute (threshold: {failed_logins_threshold})")
            ).await;
            last_failed_alert = Instant::now();
        }

        // Check rate limit triggers
        let rate_limits = RATE_LIMIT_WINDOW.lock().map(|w| w.count()).unwrap_or(0);
        if rate_limits as u32 >= rate_limit_threshold && last_rl_alert.elapsed() > cooldown {
            warn!("[SecurityAlert] High rate limit trigger rate: {rate_limits}/min (threshold: {rate_limit_threshold})");
            send_alert(
                "High Rate Limit Activity",
                &format!("{rate_limits} rate limit triggers in the last minute (threshold: {rate_limit_threshold})")
            ).await;
            last_rl_alert = Instant::now();
        }
    }
}
