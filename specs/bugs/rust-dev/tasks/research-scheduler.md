# Research: Job Scheduler Migration — job_scheduler_ng vs tokio-cron-scheduler

> **Task**: TASK-RUSTDEV-HIGH-03-B  
> **Date**: 2026-04-14  
> **Author**: Research via codebase analysis and crate comparison

---

## Question

Should we migrate from `job_scheduler_ng = "2.4.0"` to `tokio-cron-scheduler = "0.13"`?

---

## Current State

- **Crate used**: `job_scheduler_ng = "2.4.0"` (in `Cargo.toml:118`)
- **Usage**: `src/main.rs` — schedule_jobs() creates 9+ background jobs
- **Panic recovery**: Already wrapped with `catch_unwind` (TASK-RUSTDEV-HIGH-03-A ✅)

---

## Comparison

| Feature | `job_scheduler_ng` | `tokio-cron-scheduler` |
|---------|-------------------|------------------------|
| Version | 2.4.0 | 0.13 |
| Async-native | No (spawns threads per job) | Yes (tokio tasks) |
| API style | `Job::new(cron, FnMut)` | `Job::new_async(cron, async fn)` |
| Panic safety | Manual (we added catch_unwind) | Manual (same requirement) |
| Maintenance | Moderate activity | Active |
| Dependencies | Low | Higher (full tokio ecosystem) |
| Error handling | Limited | Better async Result support |
| Test support | Limited | Better (AsyncScheduler) |

### API surface comparison

**Current (job_scheduler_ng)**:
```rust
let mut sched = JobScheduler::new();
sched.add(Job::new("0 * * * * *", |_, _| {
    // sync fn runs in scheduler thread
    runtime.block_on(async_job());
})?);
sched.start();
```

**tokio-cron-scheduler**:
```rust
let sched = JobScheduler::new().await?;
sched.add(Job::new_async("0 * * * * *", |_, _| Box::pin(async {
    async_job().await;
}))?).await?;
sched.start().await?;
```

---

## Migration Effort Estimate

- ~9 jobs in `src/main.rs`
- Each job needs:
  - `Job::new` → `Job::new_async` + `Box::pin(async { ... })`
  - Remove explicit `runtime.block_on()` wrapping
  - Keep `catch_unwind` wrapper (still needed)
- Estimated effort: **0.5–1 day**
- Risk: LOW (well-isolated scheduler setup in main.rs)

---

## Recommendation

**MIGRATE** in Sprint 4 (after MED-03 AppState work):

- `tokio-cron-scheduler` is better aligned with the project's async-first direction
- Removes the need to pass `Arc<Runtime>` into every job closure
- Reduces thread spawning overhead (all jobs run as tokio tasks)
- Makes future job additions simpler

**Go decision**: YES — schedule for Sprint 4, replace `job_scheduler_ng`.

**Add to Cargo.toml** (Sprint 4):
```toml
tokio-cron-scheduler = { version = "0.13", features = ["signal"] }
```
**Remove**: `job_scheduler_ng = "2.4.0"`

---

*Research by: codebase analysis | Date: 2026-04-14 | Status: ✅ Research complete — Go decision: YES, migrate to tokio-cron-scheduler in Sprint 4 (TASK-RUSTDEV-LOW-04-A ⏳ READY).*
