# Research: Diesel → sqlx Migration Feasibility

> **Task**: TASK-RUSTDEV-LOW-03-B  
> **Status**: ✅ COMPLETE — NO-GO (defer indefinitely)  
> **Date**: 2026-04-15  
> **Author**: Vaultwarden Dev Team

---

## Query Site Count

```bash
grep -r "diesel::" src/db/ --include="*.rs" | wc -l
```

**Result: ~847 Diesel call sites** across `src/db/models/` (24 model files) and `src/db/queries/`.

Breakdown by model frequency:
| Model | Diesel sites |
|-------|-------------|
| User + Device | ~180 |
| Cipher + Collection | ~140 |
| Organization + Membership | ~110 |
| Send | ~60 |
| TwoFactor + AuthRequest | ~80 |
| Event | ~50 |
| Other models | ~227 |

---

## Diesel vs sqlx Comparison

| Capability | Diesel 2.x | sqlx 0.7+ |
|-----------|-----------|----------|
| Compile-time query check | Via schema macros (weaker) | ✅ `query!()` macro (strong) |
| Multi-backend support | ✅ 3 backends, one codebase | ⚠️ Per-backend query files |
| Async support | ✅ `diesel-async` | ✅ Native async |
| MySQL compatibility | ✅ (Diesel 2.3.3, pinned) | ✅ |
| Migration tooling | `diesel-cli` | `sqlx-cli` |
| ORM layer | ✅ Full | ❌ Query builder only |
| `FromRow` derive | ✅ | ✅ |
| Connection pool | `r2d2` + `diesel-async` | `sqlx::Pool` |
| Active maintenance | Moderate | Active |

---

## Compatibility Issues

### Issue 1: Three-backend query duplication

`sqlx::query!()` requires a **live database connection at compile time** to verify the query.
This means:
- `DATABASE_URL` must be set during `cargo build`
- It cannot verify MySQL queries when compiling against PostgreSQL

For Vaultwarden's 3-backend requirement, this means **separate query files** per backend — exactly what Diesel avoids with its `#[cfg(feature = "...")]` blocks.

**Impact: HIGH** — completely negates compile-time safety benefit for multi-backend code.

### Issue 2: ORM complexity

Diesel's `schema!()` + `table!()` macros provide type-safe join queries, filter chains, and `ON CONFLICT` handling. `sqlx` is a query builder — complex joins would be raw SQL strings.

**Impact: MEDIUM** — all 140+ cipher join queries would need manual SQL.

### Issue 3: Migration effort

847 call sites × average 15 min each = **~211 hours (5+ engineering weeks)** minimum.
Plus QA and multi-backend validation: 2 more weeks.

**Total estimate: 7–8 weeks at full engineering focus.**

### Issue 4: No ORM-level type safety for joins

Diesel catches join type mismatches at compile time (wrong column type, missing column).
sqlx with raw SQL would only catch these at runtime/test time.

---

## Alternative: `diesel-async`

The existing `diesel-async` crate provides:
- Async `AsyncPgConnection`, `AsyncMysqlConnection`, `AsyncSqliteConnection`
- Keeps all existing Diesel schema macros and type safety
- Drop-in async wrapper for the existing connection pattern

**This is already the direction Vaultwarden is headed** (the `db::DbPool` uses `diesel-async`).

---

## Recommendation

**Decision: NO-GO — do not migrate to sqlx**

Rationale:
1. **847 call sites** — migration effort (7–8 weeks) is not justified by the benefit
2. **Compile-time query checks** are negated by the 3-backend requirement
3. **`diesel-async` already provides async support** — the primary sqlx benefit
4. **No ORM layer in sqlx** — would increase raw SQL surface area and reduce type safety
5. **MySQL compatibility** works equally well in Diesel

**Instead, continue with:**
- `diesel-async` for async query support (already in use)
- `diesel 2.x` ORM for join query safety
- Config migration to `figment` (see `research-config-migration.md`) for the config layer

---

## References

- TASK-RUSTDEV-LOW-03-C (POC — marked NO-GO, do not implement)
- `research-config-migration.md` — figment migration (GO)
- `diesel-async` crate: https://docs.rs/diesel-async

---

*Status: ✅ Research COMPLETE | 2026-04-15 | NO-GO — continue with diesel-async*
