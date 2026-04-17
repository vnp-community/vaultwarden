# Vaultwarden Configuration System — Developer Guide

> **Task**: TASK-RUSTDEV-LOW-01-A  
> This file explains the `make_config!` DSL in `src/config.rs`.  
> For a step-by-step guide to adding a new config key, see [`../CONTRIBUTING.md`](../CONTRIBUTING.md).

---

## Overview

Vaultwarden configuration is driven by the `make_config!` macro in `src/config.rs`.
The macro generates:
- A `Config` struct with accessor methods (e.g., `CONFIG.smtp_host()`)
- A `ConfigBuilder` struct for deserializing from `config.json` and environment variables
- Admin UI form data (`GroupData` / `ElementData`) for the `/admin` panel

Configuration values are loaded in priority order:
1. **Environment variables** (highest priority) — read at startup via `get_env()`
2. **config.json** — persisted by the admin UI, merged on top of env defaults
3. **Compiled defaults** — fallback values in the macro

---

## `make_config!` DSL Syntax

```
make_config! {
    /// Optional group-level doc (shown in admin UI section header)
    group_name : optional_toggle_field {
        /// Field description |> Longer description shown in admin UI tooltip
        field_name : TypeName, is_editable, kind [, default_expr] ;
    },
}
```

### Field Breakdown

| Column        | Type      | Meaning |
|---------------|-----------|---------|
| `field_name`  | ident     | Rust identifier. Env var = `FIELD_NAME` (upper-snake). Accessor = `CONFIG.field_name()` |
| `TypeName`    | type      | Rust type — see **Type Reference** below |
| `is_editable` | bool lit  | `true` → field appears in the admin UI and can be changed at runtime |
| `kind`        | ident     | How the default is resolved — see **Kind Reference** below |
| `default_expr`| expr      | Required for `def`, `auto`, `generated`. The default value or closure |

### Type Reference

| Type     | Notes |
|----------|-------|
| `String` | UTF-8 string, env variable is taken verbatim |
| `bool`   | Parsed with `get_env_bool()` — accepts `true/false/1/0/yes/no` |
| `u32`, `u64`, `i32`, `i64` | Parsed from string |
| `Pass`   | Alias for `String`. Values are masked as `***` in the admin UI, support JSON, and are **never serialized to `config.json`** |
| `Option<T>` | Produced by the `option` kind — field may be absent |

### Kind Reference

| Kind        | `default_expr` | Behaviour |
|-------------|----------------|-----------|
| `def`       | Rust expression | If the env/config.json value is absent, use this literal default |
| `option`    | _(none)_        | Value is `Option<T>` — `None` when absent. Used for truly optional fields |
| `auto`      | `\|c\| expr`    | Closure receiving `&ConfigItems`. If field is absent, the closure derives a value from other fields (e.g., paths under `data_folder`) |
| `generated` | `\|c\| expr`    | Field is **always** computed from the closure, ignoring any env/JSON value. Used for private cached/derived values (prefixed `_`) |

### Group Toggles

A group can have an optional toggle:

```rust
smtp : smtp_enabled {
    // fields only shown in admin UI when smtp_enabled() is true
}
```

The `: smtp_enabled` references another field in the macro. That field controls whether this group is shown in the admin panel.

---

## Examples

### Simple default
```rust
/// Allow Sends |> Controls whether users can create Bitwarden Sends.
sends_allowed: bool, true, def, true;
```
- Env: `SENDS_ALLOWED=false`
- Admin UI: editable checkbox, default `true`

### Optional field
```rust
/// HIBP Api Key |> HaveIBeenPwned API Key
hibp_api_key: Pass, true, option;
```
- Env: `HIBP_API_KEY=secret` → `Some("secret")`, absent → `None`
- Admin UI: editable, masked as `***`
- Accessor returns `Option<String>`

### Auto-derived path
```rust
/// Icon cache folder
icon_cache_folder: String, false, auto, |c| format!("{}/icon_cache", c.data_folder);
```
- Env: not set → `data/icon_cache` (derived from `data_folder`)  
- Env: `ICON_CACHE_FOLDER=/mnt/cache` → overrides the auto value

### Generated (always computed)
```rust
/// Internal IP header property
_ip_header_enabled: bool, false, generated, |c| &c.ip_header.trim().to_lowercase() != "none";
```
- Always computed, user cannot override
- Name starts with `_` by convention (private/internal)

### Secret (Pass type)
```rust
/// Admin token/Argon2 PHC
admin_token: Pass, true, option;
```
- Never written to `config.json`
- Admin UI shows `***`
- Accessor returns `Option<String>`

---

## Adding a New Config Key — Quick Reference

See full step-by-step in `CONTRIBUTING.md`. Summary:

1. Pick the right **group** (or add a new one)
2. Add a doc comment starting with `/// Friendly Name |> tooltip description`
3. Choose **type** and **kind** from the tables above
4. Set `is_editable` to `true` only if operators should change it via the admin panel without restart
5. Use `Pass` for secrets — they will never appear in `config.json`
6. Add to `.env.template` with a commented-out example
7. Add validation in `Config::load()` if needed (see `validate_config()`)

---

## Internal Notes

- `CONFIG` is a `static LazyLock<Config>` initialized once at first access  
- **Thread safety**: all reads go through `CONFIG.inner.read().unwrap()` (RwLock)  
- **Hot reload**: the admin UI calls `Config::update_config()` which acquires write lock, not restart-free loading  
- **Env variable names**: always `UPPER_SNAKE_CASE` of the field name  
- **Non-editable fields** (`false`): changes require server restart; cannot be saved via admin UI

---

*TASK-RUSTDEV-LOW-01-A | Created: 2026-04-15 | See also: `CONTRIBUTING.md` §Config, `research-config-migration.md`*
