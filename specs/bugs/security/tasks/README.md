# Tasks — Security Bug Fixes

> Phân tách từ [SOL-security.md](../SOL-security.md)  
> Tham chiếu phân tích: [security-analysis.md](../security-analysis.md)  
> Ngày: 2026-04-13 | Cập nhật: 2026-04-14

---

## Trạng Thái Tổng Thể

| Nhóm | Implemented | Pending |
|------|------------|---------|
| P1 Critical (CRIT-01/02) | A/B/C ✅, A/B/D ✅ | CRIT-01-D, CRIT-02-C/E |
| P2 High (HIGH-01) | B ✅ (deprecation), C ✅ (research) | A (removal Sprint 3), D (nginx docs) |
| P2 High (HIGH-02) | A/B/C ✅ | D/E/F/G (DB revocation Sprint 4+) |
| P2 High (HIGH-03) | **A ✅** (default true), C ✅ (blocklist) | B (DNS rebinding), D (IP literals) — Sprint 3 |
| P2 High (HIGH-04) | C/D/E ✅ (trusted proxies + get_real_ip) | A/B (per-account ratelimit Sprint 3) |
| P3 Medium (MED-04) | B/C ✅ (secrets audit + perms) | A (env_only macro), MED-01/02/03 Sprint 3+ |
| P3–P4 (MED-05, LOW) | LOW-02-B ✅ partial (anon WS cap) | All others pending Sprint 4+ |

---

## Danh Sách Task Files

| File | Issues | Severity | Sprint | Status |
|------|--------|----------|--------|--------|
| [TASK-SEC-CRIT-01.md](TASK-SEC-CRIT-01.md) | SEC-CRIT-01 | P1 Critical | Ngay | ✅ A/B/C Done |
| [TASK-SEC-CRIT-02.md](TASK-SEC-CRIT-02.md) | SEC-CRIT-02 | P1 Critical | Ngay | ✅ A/B/D Done |
| [TASK-SEC-HIGH-01.md](TASK-SEC-HIGH-01.md) | SEC-HIGH-01 | P2 High | Sprint 1 | ✅ B/C Done; A Sprint 3 |
| [TASK-SEC-HIGH-02.md](TASK-SEC-HIGH-02.md) | SEC-HIGH-02 | P2 High | Sprint 2 | ✅ A/B/C Done |
| [TASK-SEC-HIGH-03.md](TASK-SEC-HIGH-03.md) | SEC-HIGH-03 | P2 High | Sprint 1 | ✅ A/C Done; B/D Sprint 3 |
| [TASK-SEC-HIGH-04.md](TASK-SEC-HIGH-04.md) | SEC-HIGH-04 | P2 High | Sprint 2 | ✅ C/D/E Done |
| [TASK-SEC-MED-01-04.md](TASK-SEC-MED-01-04.md) | SEC-MED-01 → 04 | P3 Medium | Sprint 2–3 | ✅ MED-04-B/C Done |
| [TASK-SEC-MED-05-LOW.md](TASK-SEC-MED-05-LOW.md) | SEC-MED-05, SEC-LOW-01 → 05 | P3–P4 | Sprint 4–5 | ⏳ LOW-02-B partial |

---

## Thứ Tự Ưu Tiên

### Ngay (P1 — Critical) — Sprint 1 ✅ COMPLETE
| Task | Mô tả | Status |
|------|-------|--------|
| TASK-SEC-CRIT-01-A/B/C | Reject non-Argon2 admin token | ✅ DONE |
| TASK-SEC-CRIT-02-A/B/D | Double-confirm để disable admin token | ✅ DONE |

### Sprint 1 (P2 High — Tuần 1-2) — Mostly Complete
| Task | Mô tả | Status |
|------|-------|--------|
| TASK-SEC-HIGH-01-B | Deprecation log cho JWT query param | ✅ DONE |
| TASK-SEC-HIGH-01-C | Client compat research — safe to remove | ✅ DONE |
| TASK-SEC-HIGH-01-A | Xóa JWT query param từ WebSocket | ⏳ Sprint 3 |
| TASK-SEC-HIGH-03-A | Default block_non_global_ips=true | ✅ DONE (already was `true`) |
| TASK-SEC-HIGH-03-C | Domain blocklist cho icon proxy | ✅ DONE |
| TASK-SEC-HIGH-03-B | DNS rebinding prevention | ⏳ Sprint 3 |

### Sprint 2 (P2 High + P3 Medium — Tuần 3-4) — Partial
| Task | Mô tả | Status |
|------|-------|--------|
| TASK-SEC-HIGH-04-C/D/E | TRUSTED_PROXIES + get_real_ip + wiring | ✅ DONE |
| TASK-SEC-HIGH-04-A/B | Per-account rate limit + credential stuffing | ⏳ Sprint 3 |
| TASK-SEC-HIGH-02-A/B/C | JWT TTL giảm + logout-all | ✅ DONE |
| TASK-SEC-MED-04-B/C | Secrets audit + file permission check | ✅ DONE |

### Sprint 3 (P3 Medium — Tuần 5-6) — In Progress
| Task | Mô tả | Status |
|------|-------|--------|
| TASK-SEC-HIGH-01-A | Xóa `WsAccessToken` query param | ⏳ Pending (research done ✅) |
| TASK-SEC-HIGH-04-A/B | Per-account ratelimit + credential stuffing | ⏳ Pending |
| TASK-SEC-HIGH-03-B | DNS rebinding prevention | ⏳ Pending |
| TASK-SEC-MED-04-A | env_only macro cho sensitive config fields | ⏳ Pending |
| TASK-SEC-MED-02-A/B/C | SSO group whitelist | ⏳ Pending |
| TASK-SEC-MED-01-A | Hint chỉ sau auth | ⏳ Pending |
| TASK-SEC-MED-03-A/B/C | Emergency access multi-channel alert | ⏳ Pending |

### Sprint 4 (P3-P4 — Tuần 7-8)
| Task | Mô tả | Status |
|------|-------|--------|
| TASK-SEC-MED-05-A/B | Push relay data minimization | ⏳ Pending |
| TASK-SEC-LOW-03-A/B | KDF minimum enforcement | ⏳ Pending |
| TASK-SEC-LOW-02-A/B | Anon WebSocket rate limiting | ⏳ Pending |
| TASK-SEC-LOW-04-A/B/C | SQLite backup path warning | ⏳ Pending |
| TASK-SEC-HIGH-02-D/E/F/G | DB-backed token revocation (opt-in) | ⏳ Pending |

### Sprint 5 (P4 Low — Tuần 9-10)
| Task | Mô tả | Status |
|------|-------|--------|
| TASK-SEC-LOW-01-A/B | RSA key rotation | ⏳ Pending |
| TASK-SEC-LOW-01-C | ES256 research | ⏳ Pending |

---

## Cross-References

- **SEC-HIGH-01** (JWT in URL) → cùng vấn đề với `specs/bugs/rust-dev/SOL-rust-dev.md`
- **SEC-LOW-05** (CSP headers) → Implement qua SOL-001 (CRS): [TASKS-SOL-001.md](../../crs/v1/solutions/tasks/TASKS-SOL-001.md)
- **SEC-HIGH-04** (rate limiting) → Dùng Redis cache từ SOL-005 nếu `CLUSTER_MODE=true`

---

## Files Cần Thay Đổi (tổng hợp)

| File | Issues | Status |
|------|--------|--------|
| `src/api/admin.rs` | CRIT-01, CRIT-02 | ✅ Done |
| `src/config.rs` | CRIT-01, CRIT-02, HIGH-02, HIGH-04, MED-02, MED-04, LOW-02, LOW-03, LOW-04 | Partial ✅ |
| `src/main.rs` | CRIT-01, CRIT-02, MED-03 | ✅ Done (CRIT) |
| `src/api/notifications.rs` | HIGH-01, LOW-02 | Partial ✅ (deprecation log) |
| `src/auth.rs` | HIGH-02, HIGH-04-E, LOW-01 | ✅ Done |
| `src/ratelimit.rs` | HIGH-04 | ⏳ Pending |
| `src/http_client.rs` | HIGH-03 | ⏳ Pending (HIGH-03-B DNS rebinding) |
| `src/api/icons.rs` | HIGH-03 | ✅ Done (A: block_non_global default, C: domain blocklist) |
| `src/util.rs` | HIGH-04 | ✅ Done (get_real_ip) |
| `src/api/identity.rs` | HIGH-04, MED-01 | Partial ✅ |
| `src/sso.rs` | MED-02 | ⏳ Pending |
| `src/api/core/emergency_access.rs` | MED-03 | ⏳ Pending |
| `src/api/core/accounts.rs` | HIGH-02, LOW-03 | ✅ Done (HIGH-02-C) |
| `src/db/mod.rs` | LOW-04 | ⏳ Pending |

---

*Tạo: 2026-04-13 | Cập nhật: 2026-04-15 — Sprint 3 in progress: P1 CRIT-01/02 ✅, P2 HIGH-01-B/C ✅ HIGH-02/03-A/03-C/04-C/D/E ✅, P3 MED-04-B/C ✅, SEC-LOW-02-B ✅ partial | Pending Sprint 3: HIGH-01-A, HIGH-03-B/D, HIGH-04-A/B, MED-01–04-A*
