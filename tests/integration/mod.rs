//! TASK-RUSTDEV-LOW-02-C: Integration test entry point.
//!
//! The real integration tests live in `src/tests.rs` (part of the binary crate)
//! where they have direct access to all internal modules including `build_rocket`,
//! `db::DbPool`, and `app_state::test_utils`.
//!
//! Run them with:
//!   cargo test --features sqlite
//!
//! Filter to integration suite only:
//!   cargo test --features sqlite integration::
//!
//! Future extension: when a lib target is added (or CONFIG moves to AppState DI),
//! add PostgreSQL container tests here using `testcontainers` (LOW-02-D).
