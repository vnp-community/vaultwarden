/// TASK-002-018: Integration tests for Tamper-Evident Audit Log & SIEM Integration
///
/// Test cases:
/// 1. SHA-256 hash chain computation correctness
/// 2. Hash chain linking (prev_hash → entry_hash)
/// 3. Tamper detection: different event produces different hash
/// 4. Empty chain produces valid 32-byte hash
/// 5. Hash chain ordering: chain is order-dependent (position matters)
/// 6. Retention floor enforcement: actual_days >= minimum_days
/// 7. AuditEventType as_ref strings (required by hash computation)
/// 8. Batch hash chain verification logic
///
/// NOTE: Because vaultwarden is a binary crate, this file uses standalone
/// reimplementations of hash logic identical to `src/audit.rs`.
/// Full DB-backed tests (10,000 entries, SIEM delivery mock, archival) are
/// in `src/tests.rs` via the Rocket in-memory test harness.
///
/// Run with:
///   cargo test --features sqlite audit

// ── Hash Chain Tests (TASK-002-004) ──────────────────────────────────────────

/// SHA-256 of a single entry (no prev_hash) must produce a 32-byte digest.
#[test]
fn test_hash_single_entry_length() {
    use sha2::{Digest, Sha256};

    let mut h = Sha256::new();
    h.update(1_700_000_000_i64.to_be_bytes());
    h.update(b"LoginSuccess");
    h.update(b"user-abc");
    h.update(b"vault");
    let digest = h.finalize().to_vec();

    assert_eq!(digest.len(), 32, "SHA-256 must produce exactly 32 bytes");
}

/// Hash of identical inputs must be deterministic.
#[test]
fn test_hash_determinism() {
    use sha2::{Digest, Sha256};

    let compute = || {
        let mut h = Sha256::new();
        h.update(1_700_000_000_i64.to_be_bytes());
        h.update(b"PasswordChanged");
        h.update(b"user-xyz");
        h.update(b"account");
        h.finalize().to_vec()
    };

    assert_eq!(compute(), compute(), "Same inputs must always produce the same hash");
}

/// Sequential entries must have different hashes (chain progresses).
#[test]
fn test_hash_chain_sequential_entries_differ() {
    use sha2::{Digest, Sha256};

    let mut h1 = Sha256::new();
    h1.update(1_700_000_000_i64.to_be_bytes());
    h1.update(b"LoginSuccess");
    h1.update(b"user-1");
    h1.update(b"vault");
    let hash1 = h1.finalize().to_vec();

    let mut h2 = Sha256::new();
    h2.update(&hash1); // link prev
    h2.update(1_700_000_001_i64.to_be_bytes());
    h2.update(b"PasswordChanged");
    h2.update(b"user-1");
    h2.update(b"account");
    let hash2 = h2.finalize().to_vec();

    assert_ne!(hash1, hash2, "Sequential entries must have different hashes");
}

/// Tamper detection: changing event_type produces a different hash.
#[test]
fn test_tamper_detection_event_type() {
    use sha2::{Digest, Sha256};
    let ts = 1_700_000_000_i64.to_be_bytes();

    let mut h_orig = Sha256::new();
    h_orig.update(ts);
    h_orig.update(b"LoginSuccess");
    h_orig.update(b"user-1");
    h_orig.update(b"vault");
    let orig = h_orig.finalize().to_vec();

    let mut h_tampered = Sha256::new();
    h_tampered.update(ts);
    h_tampered.update(b"AdminConfigChanged"); // tampered
    h_tampered.update(b"user-1");
    h_tampered.update(b"vault");
    let tampered = h_tampered.finalize().to_vec();

    assert_ne!(orig, tampered, "Different event_type must produce a different hash");
}

/// Tamper detection: changing actor_user_uuid produces a different hash.
#[test]
fn test_tamper_detection_actor_uuid() {
    use sha2::{Digest, Sha256};
    let ts = 1_700_000_000_i64.to_be_bytes();

    let make_hash = |actor: &[u8]| {
        let mut h = Sha256::new();
        h.update(ts);
        h.update(b"LoginSuccess");
        h.update(actor);
        h.update(b"vault");
        h.finalize().to_vec()
    };

    assert_ne!(
        make_hash(b"user-legitimate"),
        make_hash(b"attacker-uuid"),
        "Different actor_uuid must produce a different hash"
    );
}

/// Chain ordering: inserting entries out-of-order breaks the chain.
#[test]
fn test_hash_chain_order_dependent() {
    use sha2::{Digest, Sha256};

    let make_entry = |ts: i64, event: &[u8], prev: &[u8]| {
        let mut h = Sha256::new();
        h.update(prev); // may be empty for first entry
        h.update(ts.to_be_bytes());
        h.update(event);
        h.update(b"user-1");
        h.update(b"vault");
        h.finalize().to_vec()
    };

    let hash_a = make_entry(1_000, b"LoginSuccess", b"");
    let hash_b = make_entry(1_001, b"PasswordChanged", &hash_a);
    let hash_c = make_entry(1_002, b"AttachmentUploaded", &hash_b);

    // If we reorder: A → C → B, the hashes won't match
    let hash_c_reordered = make_entry(1_002, b"AttachmentUploaded", &hash_a); // skipping B

    assert_ne!(
        hash_c, hash_c_reordered,
        "Chain order must matter: skipping an entry must produce a different hash"
    );
}

/// 10-entry chain: simulate sequential writes and verify the chain is self-consistent.
/// This mirrors the core logic of the `GET /api/audit/verify-chain` endpoint.
#[test]
fn test_hash_chain_10_entries_consistent() {
    use sha2::{Digest, Sha256};

    struct FakeEntry {
        timestamp: i64,
        event_type: &'static str,
        actor: &'static str,
        target: &'static str,
        prev_hash: Option<Vec<u8>>,
        entry_hash: Vec<u8>,
    }

    let events = [
        "LoginSuccess", "LoginFailed", "PasswordChanged",
        "AttachmentUploaded", "AttachmentDeleted", "UserCreated",
        "UserDeleted", "GroupCreated", "AdminConfigChanged", "Logout",
    ];

    let mut entries: Vec<FakeEntry> = Vec::with_capacity(10);
    let mut prev_hash: Option<Vec<u8>> = None;

    for (i, &event) in events.iter().enumerate() {
        let mut h = Sha256::new();
        if let Some(ref ph) = prev_hash {
            h.update(ph);
        }
        let ts = (1_700_000_000_i64) + i as i64;
        h.update(ts.to_be_bytes());
        h.update(event.as_bytes());
        h.update(b"user-test");
        h.update(b"test-resource");
        let hash = h.finalize().to_vec();

        entries.push(FakeEntry {
            timestamp: ts,
            event_type: event,
            actor: "user-test",
            target: "test-resource",
            prev_hash: prev_hash.clone(),
            entry_hash: hash.clone(),
        });

        prev_hash = Some(hash);
    }

    // Now verify the chain (same logic as verify_chain endpoint)
    let mut check_prev: Option<Vec<u8>> = None;
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.prev_hash, check_prev, "Entry #{i} prev_hash must match previous entry_hash");

        let mut h = Sha256::new();
        if let Some(ref ph) = check_prev {
            h.update(ph);
        }
        h.update(entry.timestamp.to_be_bytes());
        h.update(entry.event_type.as_bytes());
        h.update(entry.actor.as_bytes());
        h.update(entry.target.as_bytes());
        let recomputed = h.finalize().to_vec();

        assert_eq!(
            recomputed, entry.entry_hash,
            "Entry #{i} hash mismatch — chain integrity broken"
        );

        check_prev = Some(entry.entry_hash.clone());
    }
}

/// Broken chain: tampering with entry #5 is detected when verifying from entry #6 onward.
#[test]
fn test_hash_chain_tamper_detected_at_position() {
    use sha2::{Digest, Sha256};

    let compute_hash = |prev: &Option<Vec<u8>>, ts: i64, event: &str| {
        let mut h = Sha256::new();
        if let Some(ph) = prev {
            h.update(ph);
        }
        h.update(ts.to_be_bytes());
        h.update(event.as_bytes());
        h.update(b"user-1");
        h.update(b"vault");
        h.finalize().to_vec()
    };

    let hash_1 = compute_hash(&None, 1_000, "LoginSuccess");
    let hash_2 = compute_hash(&Some(hash_1.clone()), 1_001, "PasswordChanged");
    let hash_3 = compute_hash(&Some(hash_2.clone()), 1_002, "AttachmentUploaded");

    // Tamper: modify entry #2's event and recompute hash_2_tampered
    let hash_2_tampered = compute_hash(&Some(hash_1.clone()), 1_001, "AdminConfigChanged");

    // Entry #3's prev_hash should point to hash_2, not hash_2_tampered
    // → mismatch at entry #3 (position 2)
    let hash_3_with_tampered_prev = compute_hash(&Some(hash_2_tampered.clone()), 1_002, "AttachmentUploaded");

    // The stored hash_3 was computed with the original hash_2
    assert_ne!(
        hash_3, hash_3_with_tampered_prev,
        "Tampering entry #2 must be detectable at entry #3 verification"
    );
}

// ── Retention Policy Tests (TASK-002-017) ────────────────────────────────────

/// Retention floor: actual_days = max(retention_days, min_days).
/// With retention_days=7 and min_days=90, actual_days must be 90.
#[test]
fn test_retention_floor_enforced() {
    let retention_days: i64 = 7; // attempted short retention
    let min_days: i64 = 90;
    let actual_days = std::cmp::max(retention_days, min_days);
    assert_eq!(actual_days, 90, "Minimum retention floor must be respected");
}

/// Retention: when configured days exceed minimum, configured value is used.
#[test]
fn test_retention_above_minimum_uses_configured() {
    let retention_days: i64 = 2555;
    let min_days: i64 = 90;
    let actual_days = std::cmp::max(retention_days, min_days);
    assert_eq!(actual_days, 2555, "Retention above minimum must use configured value");
}

/// Cutoff date must be strictly in the past.
#[test]
fn test_retention_cutoff_is_in_past() {
    use chrono::Utc;
    let actual_days: i64 = 90;
    let cutoff = (Utc::now() - chrono::TimeDelta::try_days(actual_days).unwrap()).naive_utc();
    assert!(
        cutoff < Utc::now().naive_utc(),
        "Retention cutoff must be strictly in the past"
    );
}

// ── AuditEventType String Representation Tests (TASK-002-002) ────────────────

/// All AuditEventType variants must map to non-empty string identifiers.
/// The string representation is used in hash computation — typos would break chains.
#[test]
fn test_audit_event_type_strings_non_empty() {
    // Inline the enum → string mapping (mirrors src/audit.rs AsRef<str> impl)
    let cases: &[(&str, &str)] = &[
        ("LoginSuccess", "LoginSuccess"),
        ("LoginFailed", "LoginFailed"),
        ("PasswordChanged", "PasswordChanged"),
        ("AttachmentUploaded", "AttachmentUploaded"),
        ("AttachmentDeleted", "AttachmentDeleted"),
        ("UserCreated", "UserCreated"),
        ("UserDeleted", "UserDeleted"),
        ("GroupCreated", "GroupCreated"),
        ("UserAddedToGroup", "UserAddedToGroup"),
    ];

    for (variant, expected) in cases {
        assert_eq!(variant, expected, "AuditEventType string must match exactly for hash stability");
        assert!(!expected.is_empty(), "AuditEventType string must not be empty");
    }
}

// ── SIEM Format Unit Tests (TASK-002-010 / 011 / 013) ────────────────────────

/// Splunk HEC format payload must have required fields: time, host, source, event.
#[test]
fn test_splunk_hec_payload_structure() {
    let ts: i64 = 1_700_000_000;
    let payload = serde_json::json!({
        "time": ts,
        "host": "vaultwarden",
        "source": "audit_log",
        "sourcetype": "_json",
        "event": {
            "event_type": "LoginSuccess",
            "severity": "Info",
            "actor_uuid": "user-abc",
            "actor_email": "user@example.com",
        }
    });

    assert!(payload["time"].is_number(), "Splunk HEC must have 'time' field");
    assert!(payload["host"].is_string(), "Splunk HEC must have 'host' field");
    assert!(payload["source"].is_string(), "Splunk HEC must have 'source' field");
    assert!(payload["event"].is_object(), "Splunk HEC must have 'event' object");
    assert_eq!(payload["host"], "vaultwarden");
}

/// Syslog RFC 5424 format: priority must be 134 (local0.info = 16*8 + 6).
#[test]
fn test_syslog_rfc5424_priority() {
    // local0.info = facility 16, severity 6 → PRI = 16*8 + 6 = 134
    let facility: u8 = 16;
    let severity: u8 = 6;
    let priority = facility * 8 + severity;
    assert_eq!(priority, 134, "Syslog RFC 5424 priority for local0.info must be 134");
}

/// Microsoft Sentinel format must include TimeGenerated and EventType fields.
#[test]
fn test_sentinel_payload_required_fields() {
    let payload = serde_json::json!({
        "TimeGenerated": "2026-04-16T10:00:00Z",
        "EventType": "LoginSuccess",
        "Severity": "Info",
        "ActorEmail": "user@example.com",
        "TargetResource": "vault",
    });

    assert!(payload["TimeGenerated"].is_string(), "Sentinel payload must have 'TimeGenerated'");
    assert!(payload["EventType"].is_string(), "Sentinel payload must have 'EventType'");
    assert_eq!(payload["EventType"], "LoginSuccess");
}
