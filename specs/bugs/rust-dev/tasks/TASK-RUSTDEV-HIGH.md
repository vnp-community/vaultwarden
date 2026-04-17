# TASK-RUSTDEV-HIGH: P2 High — Sprint 1

> **Severity**: P2 — High  
> **Sprint**: Sprint 1 (tuần 1–2)  
> **Effort tổng**: 4 ngày  
> **Nguồn**: [SOL-rust-dev.md](../SOL-rust-dev.md)

---

## §2.9: Regex Lock Contention [1 ngày]

**File**: `src/http_client.rs`  
**Rủi ro**: `Mutex<Option<Regex>>` trên hot path (icon proxy requests) → lock contention dưới tải cao

### TASK-RUSTDEV-HIGH-01-A ✅ DONE
- **Tên**: Thêm dependency `arc-swap`
- **File**: `Cargo.toml`
- **Mô tả**: Đã thêm `arc-swap = "1.7"` vào `[dependencies]` trong `Cargo.toml` với comment giải thích mục đích sử dụng.
- **Loại**: Dependency mới
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-RUSTDEV-HIGH-01-B ✅ DONE
- **Tên**: Thay `Mutex<Option<Regex>>` bằng `ArcSwap`
- **File**: `src/http_client.rs`
- **Mô tả**: Đã thay `static COMPILED_REGEX: Mutex<...>` bằng `LazyLock<ArcSwap<Option<(String, Regex)>>>`. Read path (hot) sự dụng `COMPILED_REGEX.load()` và `.as_ref().as_ref()` — lock-free. Write path (cold) dùng separate `WriteMutex<()>` để serialize writers, sau đó `COMPILED_REGEX.store(Arc::new(...))`. Double-check pattern sau lock acquisition để tránh redundant recompiles.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-HIGH-01-A

**Code tham khảo**:
```rust
use arc_swap::ArcSwap;

static COMPILED_REGEX: LazyLock<ArcSwap<Option<(String, Regex)>>> =
    LazyLock::new(|| ArcSwap::new(Arc::new(None)));

fn get_block_regex() -> Option<Regex> {
    COMPILED_REGEX.load()
        .as_ref()
        .as_ref()
        .map(|(_, re)| re.clone())
}

fn update_block_regex(pattern: &str) -> Result<(), Error> {
    let regex = Regex::new(pattern)
        .map_err(|e| Error::new(&format!("Invalid regex: {e}"), ""))?;
    COMPILED_REGEX.store(Arc::new(Some((pattern.to_string(), regex))));
    Ok(())
}
```

---

## §2.8: WebSocket Memory Leak [2 ngày]

**File**: `src/api/notifications.rs`  
**Rủi ro**: `DashMap` WS sessions tích lũy entries cho disconnected clients — memory leak dài hạn

### TASK-RUSTDEV-HIGH-02-A ✅ DONE
- **Tên**: Implement WS session cleanup task
- **File**: `src/api/notifications.rs`
- **Mô tả**: Đã thêm `start_ws_cleanup_task()`: spawn tokio task 60s interval. `WS_USERS.map.retain()` xóa entries với empty/closed senders. `WS_ANONYMOUS_SUBSCRIPTIONS.map.retain()` cũng được cleanup. Debug log hiển thị counts sau mỗi sweep.

### TASK-RUSTDEV-HIGH-02-B ✅ DONE
- **Tên**: Thêm anonymous WebSocket connection limit
- **File**: `src/api/notifications.rs`
- **Mô tả**: Đã thêm `static WS_ANON_ACTIVE: AtomicUsize` và `static WS_ANON_MAX: LazyLock<usize>` (default 100, configurable via `WS_ANON_MAX_CONNECTIONS` env). Check + reject với 429 khi vượt limit. `fetch_add` khi connect, `fetch_sub` trong `Drop` impl của `WSAnonymousEntryMapGuard`.
- **Loại**: New code
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-HIGH-02-A

### TASK-RUSTDEV-HIGH-02-C ✅ DONE
- **Tên**: Gọi cleanup task từ main.rs
- **File**: `src/main.rs`
- **Mô tả**: Đã gọi `api::start_ws_cleanup_task()` sau `schedule_jobs()` trong `main()`. Re-exported qua `api/mod.rs`.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-HIGH-02-A

---

## §2.7: Job Scheduler Panic Recovery [1 ngày]

**File**: `src/main.rs`  
**Rủi ro**: Scheduler thread crash → tất cả background jobs (cleanup, email, backup) dừng hoàn toàn

### TASK-RUSTDEV-HIGH-03-A ✅ DONE
- **Tên**: Wrap tất cả scheduled jobs với `catch_unwind`
- **File**: `src/main.rs`
- **Mô tả**: Đã wrap tất cả 9 job closures bằng `panic::catch_unwind(panic::AssertUnwindSafe(...))` pattern. Runtime được wrapped trong `Arc<Runtime>` để clone vào mỗi job. Trong mỗi `FnMut` job invocation, `Arc::clone` được thực hiện trước khi move vào `AssertUnwindSafe` closure. Nếu panic xảy ra: log `error!("Job '...' panicked: ...")` và tiếp tục loop.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

**Code tham khảo**:
```rust
fn create_job_with_recovery<F>(cron: &str, job_name: &str, f: F) -> Job
where F: Fn() + Send + Sync + 'static
{
    let name = job_name.to_string();
    Job::new(cron, move |_, _| {
        let result = std::panic::catch_unwind(
            std::panic::AssertUnwindSafe(|| f())
        );
        if let Err(e) = result {
            error!("Job '{}' panicked: {:?}", name, e);
        }
    }).expect("Invalid cron expression")
}
```

### TASK-RUSTDEV-HIGH-03-B ✅ DONE
- **Tên**: Research migration sang `tokio-cron-scheduler`
- **File**: `specs/bugs/rust-dev/tasks/research-scheduler.md`
- **Mô tả**: Research hoàn thành. Kết quả: `tokio-cron-scheduler 0.13` tốt hơn về async-native support (loại bỏ `Arc<Runtime>` pass vào mỗi job), giảm thread spawning overhead, maintenance tích cực hơn. API delta: `Job::new` → `Job::new_async` + `Box::pin(async { ... })`. Effort estimate: **0.5–1 ngày**, risk LOW. **Go decision: YES** — migrate trong Sprint 4.  `job_scheduler_ng` sẽ bị xóa. Dependency `tokio-cron-scheduler = { version = "0.13", features = ["signal"] }` được add và Sprint 4 (TASK-RUSTDEV-LOW-04-A).
- **Loại**: Research
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-RUSTDEV-HIGH-03-A

---

## Acceptance Criteria

- [x] Regex read path không acquire Mutex — dùng `ArcSwap::load()` ✅
- [x] WS_USERS DashMap không tăng vô hạn — cleanup task xóa stale entries mỗi 60s ✅
- [x] Anonymous WS connections bị cap tại 100 (configurable via env `WS_ANON_MAX_CONNECTIONS`) ✅
- [x] Nếu một job scheduler task panic, các jobs khác vẫn tiếp tục chạy ✅
- [x] `cargo check` pass với `arc-swap` dependency ✅
- [x] Research `tokio-cron-scheduler`: go/no-go decision documented → GO, migrate Sprint 4 ✅

---

*Tạo từ SOL-rust-dev.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: Sprint 1 ✅ — Tất cả HIGH tasks hoàn thành (HIGH-01 ✅, HIGH-02 ✅, HIGH-03-A ✅, HIGH-03-B ✅)*
