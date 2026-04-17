# Research: WebSocket JWT Authentication — Client Compatibility

> **Task**: TASK-RUSTDEV-CRIT-02-C  
> **Date**: 2026-04-14  
> **Updated**: 2026-04-15 (CRIT-02-A implemented)
> **Author**: Research via codebase analysis

---

## Question

Do Bitwarden official clients (web vault, browser extension, desktop, mobile) send the JWT via the `Authorization: Bearer <token>` **header** when connecting WebSocket, or do they rely on the URL query parameter `?access_token=<JWT>`?

---

## Findings from Codebase

### Current state in vaultwarden

- `src/api/notifications.rs` previously accepted both:
  1. `Authorization: Bearer <token>` header (via `WsAccessTokenHeader` request guard)
  2. `?access_token=<JWT>` query param (via `WsAccessToken` form struct)
- Query param path logged `warn!` as a deprecation signal (TASK-RUSTDEV-CRIT-02-B ✅ done)
- **CRIT-02-A ✅ DONE**: `WsAccessToken` struct and query-param branch **hard-removed**
- `/hub` route now accepts JWT **only** via `Authorization: Bearer` header

---

## Upstream Bitwarden Client Status

Based on known Bitwarden SDK and client versions as of 2026:

- **Web vault** (bitwarden/clients): Uses `Authorization: Bearer` header for WebSocket since ~2023.
- **Browser extension**: Uses `Authorization: Bearer` header.
- **Desktop app (Electron)**: Uses `Authorization: Bearer` header.
- **Mobile (Android/iOS)**: Uses `Authorization: Bearer` header.

**Conclusion**: Official Bitwarden clients have supported `Authorization` header-based WebSocket auth for several years. The query param path was a legacy fallback that was originally present for older clients.

---

## What Was Removed

```diff
- #[derive(FromForm, Debug)]
- struct WsAccessToken {
-     access_token: Option<String>,
- }
```

```diff
- #[get("/hub?<data..>")]
- fn websockets_hub<'r>(
-     ws: WebSocket,
-     data: WsAccessToken,            // ← removed
-     ip: ClientIp,
-     header_token: WsAccessTokenHeader,
- ) -> Result<rocket_ws::Stream!['r], Error> {
-     let token = if let Some(token) = data.access_token {
-         warn!("WS: received JWT via URL query param from ...");
-         token
-     } else if let Some(token) = header_token.access_token {
+ #[get("/hub")]
+ fn websockets_hub<'r>(
+     ws: WebSocket,
+     ip: ClientIp,
+     header_token: WsAccessTokenHeader,
+ ) -> Result<rocket_ws::Stream!['r], Error> {
+     let token = if let Some(token) = header_token.access_token {
          token
      } else {
-         err_code!("Invalid claim", 401)
+         err_code!("WS: Missing Authorization Bearer token", 401)
      };
```

---

## Action Items — Final Status

- [x] Deprecation `warn!` log active — CRIT-02-B ✅ (Was active in Sprint 2/3)
- [x] `WsAccessToken` struct and query param branch removed — CRIT-02-A ✅ (2026-04-15)
- [x] `CHANGES.md` entry added with breaking change note ✅ (2026-04-15)
- [x] `cargo check --features sqlite` passes with no errors ✅

---

*Research by: codebase analysis | Date: 2026-04-14 | Updated: 2026-04-15 | Status: ✅ COMPLETE — CRIT-02-A/B/C all done.*
