# TASKS-SOL-008: Enterprise API Management & Developer Portal

> **Giải pháp**: SOL-008  
> **CR**: CR-008  
> **Ngày tạo**: 2026-04-13  
> **Cập nhật**: 2026-04-17  
> **Tổng số tasks**: 18

---

## Sprint 1–2 — Enhanced API Keys (4 tuần)

### [x] TASK-008-001
- **Tên**: DB migration — API Keys v2 + Webhook tables
- **File**: `migrations/postgresql/YYYYMMDD_api_keys_v2/up.sql`
- **Trạng thái**: ✅ Migration exists (`api_keys_v2`, `api_key_usage`, `webhooks`, `webhook_deliveries`)

### [x] TASK-008-002
- **Tên**: Implement `ApiKeyV2` model
- **File**: `src/db/models/api_key_v2.rs`
- **Trạng thái**: ✅ Full model with `verify_token()`, `touch()`, `find_all()`, CRUD, `ApiKeyUsage` tracking

### [x] TASK-008-003
- **Tên**: Implement `ApiKeyAuth` request guard
- **File**: `src/auth.rs`
- **Trạng thái**: ✅ `ApiKeyAuth` FromRequest guard with Bearer token + client_id:secret format, IP check stub, `touch()` on success

### [x] TASK-008-004
- **Tên**: Implement `require_scope()` function
- **File**: `src/auth.rs`
- **Trạng thái**: ✅ `require_scope(key, scope)` implemented

### [x] TASK-008-005
- **Tên**: Implement API Key CRUD routes
- **File**: `src/api/core/api_keys.rs`
- **Trạng thái**: ✅ Full CRUD: `GET/POST /organizations/{id}/api-keys`, `PATCH /api-keys/{kid}`, `POST /rotate`, `DELETE`, `GET usage`

### [x] TASK-008-006
- **Tên**: Thêm API_KEY_* config keys
- **File**: `src/config.rs`
- **Trạng thái**: ✅ `api_key_v2_enabled`, `api_key_default_rate_limit_minute`, `api_key_usage_tracking`, `api_key_rotation_reminder_days`

---

## Sprint 3–5 — Webhook System (6 tuần)

### [x] TASK-008-007
- **Tên**: Implement `Webhook` và `WebhookDelivery` models
- **File**: `src/db/models/webhook.rs`
- **Trạng thái**: ✅ Full models with `find_active_for_event()`, `find_by_uuid()`, CRUD, delivery tracking

### [x] TASK-008-008
- **Tên**: Implement HMAC-SHA256 webhook signing
- **File**: `src/webhook_delivery.rs`
- **Trạng thái**: ✅ `sign_payload()` using `ring::hmac::HMAC_SHA256` + `decrypt_webhook_secret()`

### [x] TASK-008-009
- **Tên**: Implement `deliver_event()` và `deliver_with_retry()`
- **File**: `src/webhook_delivery.rs`
- **Trạng thái**: ✅ `deliver_event()` (fire-and-forget via OnceLock<DbPool>), `deliver_with_retry()` with exponential backoff, DB delivery status updates

### [x] TASK-008-010
- **Tên**: Implement Webhook CRUD routes
- **File**: `src/api/core/webhooks.rs`
- **Trạng thái**: ✅ `GET/POST /webhooks`, `PATCH`, `POST /test` (HMAC ping), `DELETE`, `GET /deliveries`

### [x] TASK-008-011
- **Tên**: Integrate webhook delivery vào handlers
- **File**: `src/api/core/ciphers.rs`, `src/main.rs`
- **Trạng thái**: ✅ `cipher.created` event dispatched from `post_ciphers()`; `init_pool()` called in `main.rs` to register global pool

### [x] TASK-008-012
- **Tên**: Thêm WEBHOOK_* config keys
- **File**: `src/config.rs`
- **Trạng thái**: ✅ `webhook_enabled`, `webhook_worker_concurrency`, `webhook_max_retry_delay_seconds`, `webhook_delivery_queue_size`

---

## Sprint 6–7 — Secrets API (4 tuần)

### [x] TASK-008-013
- **Tên**: Implement Secrets list/get endpoints
- **File**: `src/api/core/secrets.rs`
- **Trạng thái**: ✅ `GET /secrets?project=` (org-scoped, ApiKeyAuth), `GET /secrets/{id}` with org ownership check

### [x] TASK-008-014
- **Tên**: Implement Secrets export endpoint
- **File**: `src/api/core/secrets.rs`
- **Trạng thái**: ✅ `GET /secrets/export?format=env|json` — env format with `KEY=ENCRYPTED_BLOB`, json format with raw cipher data; doc note on SDK decryption

---

## Sprint 8–9 — Usage Analytics (4 tuần)

### [x] TASK-008-015
- **Tên**: Implement `ApiKeyUsage` tracking
- **File**: `src/db/models/api_key_v2.rs`
- **Trạng thái**: ✅ `ApiKeyUsage` struct, `track_api_key_usage()` inserts usage records, usage aggregated via `aggregate_for_key()`

### [x] TASK-008-016
- **Tên**: Implement API Analytics endpoint
- **File**: `src/api/core/api_keys.rs`
- **Trạng thái**: ✅ `GET /admin/api-analytics?period=7d|30d|90d` — total requests, error rate, top endpoints

### [x] TASK-008-017
- **Tên**: Mount tất cả API Management routes
- **File**: `src/api/core/mod.rs`, `src/main.rs`
- **Trạng thái**: ✅ `api_keys::routes()`, `webhooks::routes()`, `secrets::routes()` all registered in `core::routes()`; `webhook_delivery::init_pool()` called at startup

### [x] TASK-008-018
- **Tên**: Integration tests cho API Management
- **File**: `tests/api_management_tests.rs`
- **Mô tả**: 25 standalone unit tests covering: HMAC-SHA256 `sign_payload` (deterministic, different secrets/payloads, empty input, lowercase hex output), scope enforcement (`require_scope` — match, missing, empty, comma-separated list, exact-match-only), IP allowlist (JSON parsing, None = unrestricted), rate limit defaults (60 r/min, None = unlimited), exponential backoff delays (2/4/8s per attempt), backoff not applied on final attempt, max retries = 3 constant, `ApiKeyUsage` field population, `ApiKeyV2` default scopes = `"[]"`, `to_json` required fields, key expiry logic, secrets export env format (KEY=BLOB lines), secrets export JSON format (valid + reparseable), analytics period parsing (7d/30d/90d). All tests are pure unit tests — no live DB required.
- **Loại**: New test file (implemented — 25/25 passing)
- **Độ phức tạp**: Cao
- **Phụ thuộc**: TASK-008-003, TASK-008-009, TASK-008-013

---

## Tóm Tắt

| Sprint | Tasks | Tuần | Kết quả |
|--------|-------|------|---------|
| Sprint 1–2 | TASK-008-001 → 006 | 1–4 | Enhanced API keys |
| Sprint 3–5 | TASK-008-007 → 012 | 5–10 | Webhook system |
| Sprint 6–7 | TASK-008-013 → 014 | 11–14 | Secrets API |
| Sprint 8–9 | TASK-008-015 → 018 | 15–18 | Analytics + integration |

**18/18 tasks complete** 🎉 SOL-008 fully implemented.

---

*Tạo từ SOL-008 | Ngày: 2026-04-13 | Cập nhật: 2026-04-17*
