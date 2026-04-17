use crate::db::DbConn;
use crate::db::models::{AccessSchedule, IpAllowlist};
use chrono::Utc;
use std::str::FromStr;
use std::net::IpAddr;

/// Checks if the current request IP is allowed according to the organization's or global IP allowlist rules.
/// If `IpAllowlist` is completely empty for the org and globals, access is GRANTED (fail-open for normal users without enterprise rules).
/// If any rule exists, and none matches the IP, access is DENIED.
pub async fn validate_ip_allowlist(ip: &IpAddr, org_uuid: Option<&str>, conn: &DbConn) -> Result<(), &'static str> {
    let mut allowlists = IpAllowlist::find_globals(conn).await;
    
    if let Some(org_id) = org_uuid {
        allowlists.extend(IpAllowlist::find_by_org(org_id, conn).await);
    }

    if allowlists.is_empty() {
        return Ok(());
    }

    let mut is_allowed = false;
    for allowlist in allowlists {
        for cidr_str in allowlist.cidr_ranges.split(',') {
            let cidr_str = cidr_str.trim();
            if cidr_str.is_empty() { continue; }
            if let Ok(network) = ipnetwork::IpNetwork::from_str(cidr_str) {
                if network.contains(*ip) {
                    is_allowed = true;
                    break;
                }
            }
        }
        if is_allowed { break; }
    }

    if is_allowed {
        Ok(())
    } else {
        Err("Your IP address is not permitted by enterprise access policies")
    }
}

/// Checks if the current time matches the access schedule rules defined for the user or organization.
pub async fn validate_access_schedules(user_uuid: &str, org_uuid: Option<&str>, conn: &DbConn) -> Result<(), &'static str> {
    let mut schedules = AccessSchedule::find_by_user(user_uuid, conn).await;
    
    if let Some(org_id) = org_uuid {
        schedules.extend(AccessSchedule::find_by_org(org_id, conn).await);
    }

    if schedules.is_empty() {
        return Ok(()); // Fail-open, no schedules defined
    }

    let now_utc = Utc::now();
    let current_weekday_bit = 1 << now_utc.format("%u").to_string().parse::<i32>().unwrap_or(0); // 1=Mon ... 7=Sun

    let current_naive_time = now_utc.time();

    let mut allowed = false;
    for schedule in schedules {
        // Bitwise check if current weekday is enabled
        if (schedule.allowed_days & current_weekday_bit) == 0 {
            continue;
        }

        // Check time range if specified (in UTC for simplicity, or handle chrono_tz if needed)
        // Here we just use naive UTC mapping
        if let (Some(start), Some(end)) = (schedule.allowed_time_from, schedule.allowed_time_until) {
            if current_naive_time >= start && current_naive_time <= end {
                allowed = true;
                break;
            }
        } else {
            // No time restrictions but day matches
            allowed = true;
            break;
        }
    }

    if allowed {
        Ok(())
    } else {
        Err("Your access is temporarily restricted due to time-based access policies")
    }
}
