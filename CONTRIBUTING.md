# Contributing to Vaultwarden

Thank you for contributing! Please read this guide before submitting PRs.

---

## Table of Contents

1. [Adding a Configuration Key](#1-adding-a-configuration-key)
2. [Database Backend Guidelines](#2-database-backend-guidelines)
3. [Code Style](#3-code-style)

---

## 1. Adding a Configuration Key

> **Reference**: See [`src/config_guide.md`](src/config_guide.md) for a complete DSL syntax reference.

Configuration keys are declared in the `make_config!` macro in `src/config.rs`.
Follow these steps to add a new key correctly:

### Step 1 — Find (or create) the right group

Keys are organized into groups in `config.rs`:
- `folders` — file/directory paths
- `ws` — WebSocket settings
- `push` — push notification settings
- `jobs` — cron job schedules
- `settings` — general user-facing settings
- `advanced` — server operator settings
- `smtp` — email settings
- `yubico` / `duo` / `sso` — authentication providers

Pick the most appropriate group. For new subsystems, add a new group block.

### Step 2 — Choose the correct type and kind

```rust
//   field_name: TypeName, is_editable, kind [, default_expr];
```

| You want...                    | Use type  | Use kind    |
|-------------------------------|-----------|-------------|
| Required with a default        | `String` / `bool` / `u32` etc. | `def, default_expr` |
| Optional (may be absent)       | `String` / `bool` etc. | `option` |
| Derived from another field     | any type  | `auto, \|c\| expr` |
| Always computed, ignores input | any type  | `generated, \|c\| expr` |
| A secret or password           | `Pass`    | Any of the above |

**Key rules:**
- Use `Pass` for **any value that should not appear in `config.json`** (tokens, secrets, API keys, encryption keys)
- Use `is_editable = false` for settings that require a server restart — they will not appear in the admin UI
- Use `is_editable = true` only for settings operators can safely change at runtime via the admin panel

### Step 3 — Write the doc comment

Doc comments are **required** — they appear in the admin UI and help output.
Format: `/// Short name |> Longer tooltip description`

```rust
/// My Feature Toggle |> Controls whether the new feature is enabled.
/// Set to false to disable it entirely.
my_feature_enabled: bool, true, def, true;
```

### Step 4 — Add an example diff

Example of a complete, correct addition in `config.rs`:

```diff
 advanced {
+    /// Strict JWT validation |> When true, rejects JWTs with non-standard algorithm headers.
+    /// Disable only for legacy client compatibility testing.
+    strict_jwt_validation: bool, false, def, true;
```

### Step 5 — Add to `.env.template`

Add a commented-out example entry to `.env.template`:

```env
## Strict JWT validation (default: true)
## Disable only for debugging legacy clients
# STRICT_JWT_VALIDATION=true
```

### Step 6 — Validate in `Config::load()` if needed

For cross-field validation (e.g., URL format, min/max range), add a check in `Config::load()`:

```rust
if config.my_timeout > 3600 {
    warn!("MY_TIMEOUT is very high ({}s), consider reducing it", config.my_timeout);
}
```

### Step 7 — Run `cargo check`

```bash
cargo check --features sqlite
```

---

## 2. Database Backend Guidelines

Vaultwarden supports **three database backends**: SQLite, PostgreSQL, and MySQL (via Diesel).
All new database migrations and queries **must work across all three backends**.

### Migration files

Every schema change requires **three migration files**:
```
migrations/
  sqlite/YYYY-MM-DD-HHMMSS_description/up.sql
  postgresql/YYYY-MM-DD-HHMMSS_description/up.sql
  mysql/YYYY-MM-DD-HHMMSS_description/up.sql
```

Create matching `down.sql` files for reversibility.

### Backend compatibility matrix

| SQL Feature                   | SQLite | PostgreSQL | MySQL |
|-------------------------------|--------|-----------|-------|
| `RETURNING` clause            | ✅ 3.35+ | ✅ | ❌ |
| `gen_random_uuid()`           | ❌      | ✅ | ❌ |
| `NOW() AT TIME ZONE 'UTC'`    | ❌      | ✅ | ❌ |
| `ILIKE` (case-insensitive)    | ❌      | ✅ | ❌ (use `LIKE` + collation) |
| `BOOLEAN` type                | ✅ (as INTEGER 0/1) | ✅ | ✅ (as TINYINT) |
| `TEXT` for long strings       | ✅ | ✅ | ✅ (use `LONGTEXT` if >65535 chars) |
| `INTEGER` primary key autoincrement | ✅ | `SERIAL` or `GENERATED` | `AUTO_INCREMENT` |

### Rules for Diesel query code

1. **No raw SQL** in Diesel builders unless inside `#[cfg(feature = "...")]` blocks
2. For UUID generation: use `crate::util::get_uuid()` (wraps `uuid` crate, backend-agnostic)
3. For timestamps: use `Utc::now().naive_utc()` (NaiveDateTime), not database functions
4. For bulk inserts: use `insert_or_ignore_into` / `on_conflict_do_nothing()` where supported
5. Always test migrations manually on all three backends before submitting:

```bash
# SQLite
cargo run --features sqlite -- migrate

# PostgreSQL
DATABASE_URL=postgres://... cargo run --features postgresql -- migrate

# MySQL
DATABASE_URL=mysql://... cargo run --features mysql -- migrate
```

### Adding new model fields

For new nullable columns, provide a default in the migration so existing rows are filled:
```sql
-- SQLite / PostgreSQL
ALTER TABLE users ADD COLUMN new_field TEXT DEFAULT NULL;

-- MySQL
ALTER TABLE users ADD COLUMN new_field LONGTEXT DEFAULT NULL;
```

In the Diesel model, use `Option<String>` for nullable columns.

---

## 3. Code Style

- **Edition**: Rust 2021
- **Format**: `cargo fmt` before committing
- **Lint**: `cargo clippy --features sqlite -- -D warnings`
- **Error handling**: Use `err!` / `err_unauthorized!` / `err_not_found!` macros from `src/error.rs`
- **Logging**: Use `info!` / `warn!` / `error!` / `debug!` from the `log` crate
- **No `unwrap()` in production paths** — use `?`, `map_res()`, or explicit error handling

---

*TASK-RUSTDEV-LOW-01-B + TASK-RUSTDEV-LOW-03-A | Created: 2026-04-15*
