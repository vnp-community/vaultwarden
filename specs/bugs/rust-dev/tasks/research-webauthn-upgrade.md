# Research: webauthn-rs 0.6.x Upgrade Feasibility

**Task**: TASK-RUSTDEV-LOW-04-B  
**Date**: 2026-04-15  
**Status**: DONE — **DEFER** (0.6.x not yet stable)

---

## Current State

```toml
# Cargo.toml
webauthn-rs = { version = "0.5.0-dev.10", ... }
```

Vaultwarden currently pins `webauthn-rs` at `0.5.0-dev.10` (pre-release).

---

## 0.6.x Status

As of 2026-04-15, checking `crates.io`:

| Version | Status | Date |
|---------|--------|------|
| `0.5.0-dev.10` | Used (yanked series) | 2023 |
| `0.5.0` | Stable GA (first stable) | 2024 |
| `0.6.x` | **Pre-release / not on crates.io** | N/A |

**webauthn-rs 0.6.x has not been published to crates.io.** The upstream [kanidm/webauthn-rs](https://github.com/kanidm/webauthn-rs) repository has a `0.6.x` development branch but it has not reached GA.

---

## 0.5.0 (Stable) Upgrade Path

Instead of waiting for `0.6.x`, the more actionable step is **migrating from `0.5.0-dev.10` → `0.5.0` (stable)**:

### Breaking Changes (`dev.10` → `0.5.0` stable)

1. `WebauthnBuilder::new()` signature updated — `rp_id` is now validated more strictly.
2. `PasskeyAuthentication` and `SecurityKeyAuthentication` types reorganized into feature flags.
3. `AuthenticationResult` now exposes `counter_updated: bool` (was different before).
4. The `attested_resident_key` flow has minor API changes.

### Effort Estimate (0.5.0 upgrade)

| Area | Files affected | Effort |
|------|----------------|--------|
| `Cargo.toml` version bump | 1 | 5 min |
| `src/api/core/two_factor/webauthn.rs` | 1 | 1–2 days |
| Integration test validation | — | 0.5 day |
| **Total** | 2 | ~2–3 days |

---

## Recommendation

| Action | Priority | Notes |
|--------|----------|-------|
| Upgrade `dev.10` → `0.5.0` stable | ✅ **DO** in Sprint 5 | Eliminates pre-release dep risk |
| Upgrade `0.5.x` → `0.6.x` | ⏸ **WAIT** | 0.6.x not published; revisit in 6 months |

The `dev.10` pre-release pin is a dependency health risk:
- Pre-release versions may be yanked at any time.
- Security fixes in `0.5.0` stable are not backported to `dev.10`.

**Sprint 5 action**: bump `webauthn-rs` from `"0.5.0-dev.10"` to `"0.5"` and resolve
any compile errors in `src/api/core/two_factor/webauthn.rs`.

---

## References

- crates.io: <https://crates.io/crates/webauthn-rs>
- GitHub: <https://github.com/kanidm/webauthn-rs/blob/master/CHANGELOG.md>
- Vaultwarden FIDO2 handler: `src/api/core/two_factor/webauthn.rs`

---

*Research: 2026-04-15 | Verdict: 0.6.x not available; upgrade dev.10→0.5.0 in Sprint 5*
