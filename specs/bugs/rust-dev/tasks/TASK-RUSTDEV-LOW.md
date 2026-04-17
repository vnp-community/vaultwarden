# TASK-RUSTDEV-LOW: P4–P5 — Sprint 4+ / Dài Hạn

> **Severity**: P4–P5 — Low / Technical Debt  
> **Sprint**: Sprint 4+ (tuần 7+) và Dài hạn (3–6 tháng)  
> **Nguồn**: [SOL-rust-dev.md](../SOL-rust-dev.md)

---

## §2.1: Config Macro Hell [Sprint 4 — 2 tuần]

**File**: `src/config.rs`  
**Rủi ro**: `make_config!` macro 100+ dòng — khó debug, không có IDE support, khó onboard contributor mới

### TASK-RUSTDEV-LOW-01-A ✅ DONE (2026-04-15)
- **Tên**: Document `make_config!` DSL với inline comments
- **File**: `src/config.rs`, `src/config_guide.md` (mới)
- **Mô tả**: Đã tạo `src/config_guide.md` (145 dòng) giải thích đầy đủ DSL syntax: field kinds (def/option/auto/generated), type reference (String, bool, Pass, integers, Option<T>), bảng so sánh, ví dụ annotated cho mỗi kind. Đã thêm comment block 15 dòng trên macro call trong `config.rs` (thay comment cũ 8 dòng) với link đến guide, quick column reference. Admin UI editability và Pass type semantics được giải thích rõ.
- **Loại**: Documentation
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-RUSTDEV-LOW-01-B ✅ DONE (2026-04-15)
- **Tên**: Thêm `CONTRIBUTING.md` section về adding config keys
- **File**: `CONTRIBUTING.md` (mới, root của project)
- **Mô tả**: Đã tạo `CONTRIBUTING.md` đầy đủ với 3 sections: §1 Adding a Config Key (7-step guide với diff example), §2 Database Backend Guidelines (compatibility matrix SQLite/PostgreSQL/MySQL, migration rules, Diesel patterns), §3 Code Style (format, lint, error macros, no-unwrap rule). Bao gồm bảng compat matrix cho `RETURNING`, `gen_random_uuid()`, `ILIKE`, BOOLEAN, autoincrement.
- **Loại**: Documentation
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-LOW-01-A

### TASK-RUSTDEV-LOW-01-C ✅ DONE (2026-04-15)
- **Tên**: Research migration sang `serde` + `validator`
- **File**: `specs/bugs/rust-dev/tasks/research-config-migration.md` (mới)
- **Mô tả**: Đã viết research doc đầy đủ: So sánh Diesel make_config! vs serde+figment+validator. Phân tích compatibility (config.json format 100% backward compat, Pass → `#[serde(skip_serializing)]`). Effort estimate: 17 ngày (5 phases). Recommendation: **GO — defer đến Sprint 5+** sau khi AppState migration (MED-04-A) hoàn tất. Không migrate khi CONFIG vẫn là LazyLock<Config> global.
- **Loại**: Research
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-RUSTDEV-LOW-01-A

### TASK-RUSTDEV-LOW-01-D (Dài hạn — deferred đến Sprint 5+)
- **Tên**: Migrate config sang `serde` struct
- **File**: `src/config.rs`
- **Mô tả**: Thay `make_config!` bằng `#[derive(Deserialize, Serialize)]` struct với serde attributes. Dùng `figment` crate cho layered config loading (env → config.json → defaults). `#[serde(skip_serializing)]` cho secrets (thay Pass marker). Research (LOW-01-C) đã xác nhận GO — gated on MED-04-A và AppState migration.
- **Loại**: Major refactor
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-RUSTDEV-LOW-01-C, MED-04-A (DEFERRED)

---

## §2.10: Test Coverage [Sprint 4+ và Dài hạn]

**Rủi ro**: Không có integration test suite — regressions không được phát hiện trước release

### TASK-RUSTDEV-LOW-02-A ✅ DONE (hoàn thành sớm cùng Sprint 2)
- **Tên**: Thêm unit test cho `auth.rs` — JWT roundtrip
- **File**: `src/auth.rs`
- **Mô tả**: Đã hoàn thành cùng TASK-RUSTDEV-CRIT-01-C trong Sprint 2. Đã có trong `src/auth.rs` mod tests: `test_encode_jwt_returns_ok()`, `test_encode_decode_roundtrip()`, `test_expired_jwt_rejected()`, `test_tampered_jwt_rejected()`. Xem TASK-RUSTDEV-CRIT.md — đã được mark DONE tại CRIT-01-C.
- **Loại**: New tests
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-CRIT-01-A (encode_jwt phải là Result) — đã done

### TASK-RUSTDEV-LOW-02-B ✅ DONE (hoàn thành sớm cùng Sprint 2/3)
- **Tên**: Thêm unit tests cho `error.rs` — ErrorKind mapping
- **File**: `src/error.rs`
- **Mô tả**: Đã implement đầy đủ trong `src/error.rs` `#[cfg(test)] mod tests`. 7 test functions: `test_not_found_category_and_code`, `test_unauthorized_category_and_code`, `test_forbidden_category_and_code`, `test_validation_category_and_code`, `test_internal_category_and_code`, `test_default_category_is_internal`, `test_with_category_builder`, `test_error_category_variants_distinct`. Tất cả assert category và error_code đúng cho mỗi `ErrorCategory` variant.
- **Loại**: New tests
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-MED-01-A — đã done

### TASK-RUSTDEV-LOW-02-C ✅ DONE (2026-04-15)
- **Tên**: Integration test skeleton với SQLite in-memory
- **File**: `src/tests.rs` (trực tiếp trong binary crate để có full access)
- **Mô tả**: Đã extracted `pub fn build_rocket(pool, state, extra_debug)` từ `launch_rocket()` trong `main.rs` (trả `Rocket<Build>` chưa ignite). Tests trong `src/tests.rs` dùng `build_rocket()` + `Client::tracked()` để boot Rocket. `setup_test_env()` set `SKIP_CONFIG_VALIDATION=true` (bypass cron validation) và các env vars cần thiết. 3 tests active (không có `#[ignore]`): `test_health_check_alive` (GET /alive → 200), `test_login_bad_credentials_returns_4xx` (login sai credentials → 4xx; verify NoopRateLimiter inject), `test_profile_without_auth_returns_401` (auth guard active). Tất cả 24 tests pass (`cargo test --features sqlite`).
- **Loại**: New test file + Modify main.rs
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-RUSTDEV-MED-03-A (AppState — done ✅)

### TASK-RUSTDEV-LOW-02-D (Dài hạn)
- **Tên**: Thêm `testcontainers` integration tests cho PostgreSQL
- **File**: `tests/integration/pg_test.rs` (mới)
- **Mô tả**: Thêm `[dev-dependencies]`: `testcontainers = "0.22"`, `testcontainers-modules = { version = "0.9", features = ["postgres"] }`. Test full login flow với real PostgreSQL container. Bao gồm: register → login → get vault → logout. Chỉ chạy khi `DOCKER_HOST` available (CI env).
- **Loại**: New test file
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-RUSTDEV-LOW-02-C (activate fully first)

---

## §2.4: Diesel → sqlx Migration [Dài Hạn — 3–6 tháng]

**Rủi ro**: MySQL pinned tại Diesel 2.3.3, 3 migration directories, code duplication giữa DB backends

**KẾT QUẢ RESEARCH (LOW-03-B): NO-GO** — migrate sang sqlx không khả thi. Tiếp tục với diesel-async.

### TASK-RUSTDEV-LOW-03-A ✅ DONE (2026-04-15)
- **Tên**: Document database backend guidelines
- **File**: `CONTRIBUTING.md` (§2 Database Backend Guidelines)
- **Mô tả**: Đã viết §2 trong `CONTRIBUTING.md`: (1) migration phải implement cho cả 3 backends trong `migrations/sqlite/`, `migrations/postgresql/`, `migrations/mysql/`, (2) Bảng compat matrix đầy đủ (RETURNING, gen_random_uuid(), ILIKE, BOOLEAN, TEXT, autoincrement per-backend), (3) Rules cho Diesel query code (no raw SQL, UUID qua `crate::util::get_uuid()`, timestamps qua NaiveDateTime), (4) Di chuyển test manual cho 3 backends.
- **Loại**: Documentation
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-RUSTDEV-LOW-03-B ✅ DONE (2026-04-15) — NO-GO
- **Tên**: Research `sqlx` migration feasibility
- **File**: `specs/bugs/rust-dev/tasks/research-sqlx-migration.md` (mới)
- **Mô tả**: Đã count 847 Diesel call sites (`grep -r 'diesel::' src/db/`). Đã so sánh đầy đủ sqlx vs Diesel: compile-time query check bị vô hiệu bởi 3-backend requirement (sqlx cần DATABASE_URL at compile time — không thể verify MySQL khi compiling against SQLite). `diesel-async` đã cung cấp async support. Tổng effort estimate: 7–8 engineering weeks. **Recommendation: NO-GO — tiếp tục với diesel-async + Diesel 2.x**.
- **Loại**: Research
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-RUSTDEV-LOW-03-A

### TASK-RUSTDEV-LOW-03-C ❌ CANCELLED (NO-GO từ LOW-03-B)
- **Tên**: Implement `sqlx` proof-of-concept trên 1 model
- **Mô tả**: Cancelled. Research (LOW-03-B) cho kết quả NO-GO. sqlx không phù hợp cho codebase 3-backend này.
- **Loại**: Experiment / POC
- **Phụ thuộc**: TASK-RUSTDEV-LOW-03-B (NO-GO)

---

## Dependency Risk Actions [Sprint 4]

### TASK-RUSTDEV-LOW-04-A ✅ DONE (2026-04-15)
- **Tên**: Migrate `job_scheduler_ng` → `tokio-cron-scheduler`
- **File**: `Cargo.toml`, `src/config.rs`, `src/main.rs`
- **Mô tả**: Đã xóa `job_scheduler_ng = "2.4.0"` khỏi Cargo.toml. Đã thêm `tokio-cron-scheduler = { version = "0.13", features = ["signal"] }` và `croner = "2.2.0"` (direct dep dùng cho cron validation trong config.rs). Đã migrate toàn bộ `schedule_jobs()` function từ `fn` (sync) sang `async fn`: (1) Xóa `Arc<Runtime>` + `thread::spawn`, (2) 9 jobs chuyển từ `Job::new(cron, FnMut)` → `Job::new_async(cron, |_uuid, _lock| Box::pin(async { ... }))`, (3) Mỗi job spawn tokio task, giữ `catch_unwind` wrapper, (4) `sched.start().await` thay vì loop + `rt.block_on(sleep)`. Call site trong `main()` dùng `tokio::spawn(schedule_jobs(...))`. Cron validation trong `config.rs` dùng `croner::Cron::new(s).parse()` thay vì `s.parse::<Schedule>()`. `cargo check --features sqlite` pass.
- **Loại**: Dependency migration
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-RUSTDEV-HIGH-03-B (research — đã done)

### TASK-RUSTDEV-LOW-04-B ✅ DONE (2026-04-15) — DEFER 0.6.x
- **Tên**: Evaluate `webauthn-rs` 0.6.x upgrade
- **File**: `specs/bugs/rust-dev/tasks/research-webauthn-upgrade.md` (mới)
- **Mô tả**: **webauthn-rs 0.6.x chưa được publish lên crates.io** (chỉ có development branch). Khuyến nghị: Upgrade từ `dev.10` → `0.5.0` stable trong Sprint 5 (loại bỏ pre-release dep risk, có security fixes). 0.6.x: revisit sau 6 tháng.
- **Loại**: Research
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: Không

---

## Acceptance Criteria

- [x] `src/config_guide.md` tồn tại và giải thích macro syntax rõ ràng ✅ (LOW-01-A 2026-04-15)
- [x] `CONTRIBUTING.md` có section về database backend guidelines ✅ (LOW-01-B + LOW-03-A 2026-04-15)
- [x] JWT unit tests pass: roundtrip, expired, tampered ✅ (hoàn thành cùng CRIT-01-C)
- [x] `error.rs` unit tests pass: ErrorCategory → HTTP status mapping ✅ (LOW-02-B done)
- [x] Research docs tồn tại: config migration ✅ (LOW-01-C 2026-04-15), sqlx migration ✅ (LOW-03-B 2026-04-15), scheduler migration ✅ (đã có)
- [x] `tokio-cron-scheduler` migration hoàn thành, `job_scheduler_ng` được xóa ✅ (LOW-04-A 2026-04-15)
- [x] Integration test skeleton chạy cấu trúc đúng với active (không `#[ignore]`) tests ✅ (LOW-02-C DONE 2026-04-15)
- [x] Integration tests thực sự chạy được: `GET /alive` → 200, login invalid → 4xx, unauthenticated profile → 401 ✅
- [x] `webauthn-rs` 0.6.x evaluation ✅ (LOW-04-B 2026-04-15 — 0.6.x not available; upgrade dev.10→0.5.0 in Sprint 5)
- [ ] Config migration sang serde+figment (LOW-01-D — Sprint 5+, gated on MED-04-A)

---

*Tạo: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: Sprint 4 ✅ COMPLETE — LOW-01-A/B/C ✅, LOW-02-A/B/C ✅, LOW-03-A/B ✅, LOW-04-A/B ✅. LOW-03-C ❌ CANCELLED (NO-GO). Còn lại: LOW-01-D (Sprint 5+), LOW-02-D (dài hạn)*
