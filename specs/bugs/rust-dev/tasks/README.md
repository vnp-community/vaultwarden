# Tasks — Rust Dev Bug Fixes

> Phân tách từ [SOL-rust-dev.md](../SOL-rust-dev.md)  
> Tham chiếu phân tích: [rust-dev-analysis.md](../../rust-dev-analysis.md)  
> Ngày: 2026-04-13

---

## Danh Sách Task Files

| File | Issues | Severity | Sprint | Effort tổng |
|------|--------|----------|--------|-------------|
| [TASK-RUSTDEV-CRIT.md](TASK-RUSTDEV-CRIT.md) | TD-06, SEC-HIGH-01 | P1 Critical | Ngay | 1.5 ngày |
| [TASK-RUSTDEV-HIGH.md](TASK-RUSTDEV-HIGH.md) | §2.9, §2.8, §2.7 | P2 High | Sprint 1 | 4 ngày |
| [TASK-RUSTDEV-MED.md](TASK-RUSTDEV-MED.md) | §2.5, §2.6, §2.2, §2.3 | P3 Medium | Sprint 2–3 | ~3.5 tuần |
| [TASK-RUSTDEV-LOW.md](TASK-RUSTDEV-LOW.md) | §2.1, §2.10, §2.4, Dependencies | P4–P5 | Sprint 4+ | Dài hạn |

---

## Thứ Tự Ưu Tiên

### Ngay (P1 — Critical)
| Task | Mô tả | Effort |
|------|-------|--------|
| TASK-RUSTDEV-CRIT-01-A/B/C | `encode_jwt` → Result, update callers, test | 0.5 ngày |
| TASK-RUSTDEV-CRIT-02-A/B/C | Xóa JWT query param từ WebSocket | 1 ngày |

### Sprint 1 (P2 High — Tuần 1–2)
| Task | Mô tả | Effort |
|------|-------|--------|
| TASK-RUSTDEV-HIGH-01-A/B | ArcSwap thay Mutex cho regex | 1 ngày |
| TASK-RUSTDEV-HIGH-02-A/B/C | WS cleanup task + anon connection limit | 2 ngày |
| TASK-RUSTDEV-HIGH-03-A/B | Job scheduler catch_unwind + research | 1 ngày |

### Sprint 2 (P3 Medium — Tuần 3–4)
| Task | Mô tả | Effort |
|------|-------|--------|
| TASK-RUSTDEV-MED-01-A/B/C/D | ErrorKind enum + typed macros + HTTP mapping | 1 tuần |
| TASK-RUSTDEV-MED-02-A/B/C/D | RSA key encryption at rest | 3 ngày |

### Sprint 3 (P3 Medium — Tuần 5–6)
| Task | Mô tả | Effort |
|------|-------|--------|
| TASK-RUSTDEV-MED-03-A/B/C/D | AppState DI pattern + RateLimiter trait | 2 tuần |
| TASK-RUSTDEV-MED-04-A | Xóa blocking spawn workaround | 0.5 ngày |

### Sprint 4 (P4 Low — Tuần 7–8)
| Task | Mô tả | Effort |
|------|-------|--------|
| TASK-RUSTDEV-LOW-01-A/B/C | Config macro documentation | 2 tuần |
| TASK-RUSTDEV-LOW-02-A/B/C | Unit tests + integration test skeleton | 1 tuần |
| TASK-RUSTDEV-LOW-04-A/B | Dependency migration research | 3 ngày |

### Dài Hạn (P5 — Tháng 3–6)
| Task | Mô tả | Effort |
|------|-------|--------|
| TASK-RUSTDEV-LOW-02-D | testcontainers PostgreSQL integration tests | 1 tuần |
| TASK-RUSTDEV-LOW-01-D | Config macro → serde migration | 2 tuần |
| TASK-RUSTDEV-LOW-03-A/B/C | Diesel → sqlx migration (POC) | 3–6 tháng |

---

## Cross-References

- **SEC-HIGH-01** (JWT in URL) → Trùng với `specs/bugs/security/tasks/TASK-SEC-HIGH-01.md` — coordinate để implement một lần
- **§2.8** (WS anonymous limit) → Trùng với `specs/bugs/security/tasks/TASK-SEC-LOW-02.md` — merge implementation
- **§2.2** (AppState/RateLimiter trait) → Là foundation cho `specs/bugs/security/tasks/TASK-SEC-HIGH-04.md` (per-account rate limiting)
- **§2.6** (RSA key encryption) → Liên quan đến `specs/bugs/security/tasks/TASK-SEC-MED-05-LOW.md` SEC-LOW-01 (JWT key rotation)
- **§2.7** (Job scheduler) → Jobs được thêm bởi `specs/crs/v1/solutions/tasks/TASKS-SOL-002.md` (audit retention), `TASKS-SOL-006.md` (backup scheduler)

---

## Files Cần Thay Đổi (tổng hợp)

| File | Issues |
|------|--------|
| `src/auth.rs` | TD-06 (encode_jwt Result), §2.6 (RSA encryption) |
| `src/api/identity.rs` | TD-06 (callers), SEC-HIGH-01 |
| `src/api/notifications.rs` | SEC-HIGH-01 (WsAccessToken), §2.8 (WS cleanup) |
| `src/config.rs` | §2.1 (documentation), §2.6 (RSA_KEY_ENCRYPTION_KEY), §2.3 (remove blocking) |
| `src/error.rs` | §2.5 (ErrorKind enum) |
| `src/http_client.rs` | §2.9 (ArcSwap regex) |
| `src/main.rs` | §2.7 (catch_unwind), §2.8 (start_ws_cleanup_task), §2.2 (AppState) |
| `src/ratelimit.rs` | §2.2 (RateLimiter trait) |
| `src/app_state.rs` (mới) | §2.2 (AppState struct) |
| `Cargo.toml` | `arc-swap`, `tokio-cron-scheduler` (sprint 4) |
| `CONTRIBUTING.md` | §2.1, §2.4 (database guidelines) |
| `src/config_guide.md` (mới) | §2.1 (macro documentation) |

---

## Dependencies Mới

| Crate | Version | Sprint | Lý do |
|-------|---------|--------|-------|
| `arc-swap` | `"1.7"` | Sprint 1 | Lock-free regex reads |
| `tokio-cron-scheduler` | `"0.13"` | Sprint 4 | Replace `job_scheduler_ng` |
| `validator` | `"0.18"` | Sprint 4+ | Config validation (nếu migrate) |
| `testcontainers` | `"0.22"` | Dài hạn | PostgreSQL integration tests |
| `testcontainers-modules` | `"0.9"` | Dài hạn | PostgreSQL image |

---

*Tạo: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: **Sprint 1–4 Complete** ✅ — P1 CRIT ✅, P2 HIGH ✅, P3 MED ✅, P4 LOW ✅ | Còn lại: Sprint 5+ (LOW-01-D config migration, LOW-02-D testcontainers, webauthn 0.5.0 upgrade)*
