# Tài Liệu Thiết Kế Kỹ Thuật (TDD)
## Vaultwarden — Máy Chủ Quản Lý Mật Khẩu Tự Lưu Trữ

> **Phiên bản**: 1.0  
> **Nguồn**: Phân tích trực tiếp từ `src/` (Rust source code)  
> **Ngày**: 2026-04-10  
> **Trạng thái**: Hiện hành

---

## Mục Lục

1. [Tổng Quan Kiến Trúc](#1-tổng-quan-kiến-trúc)
2. [Cấu Trúc Module](#2-cấu-trúc-module)
3. [Lớp Web Framework & Server](#3-lớp-web-framework--server)
4. [Hệ Thống Xác Thực (Authentication)](#4-hệ-thống-xác-thực-authentication)
5. [Hệ Thống Phân Quyền (Authorization)](#5-hệ-thống-phân-quyền-authorization)
6. [Lớp Cơ Sở Dữ Liệu (Database Layer)](#6-lớp-cơ-sở-dữ-liệu-database-layer)
7. [Mô Hình Dữ Liệu (Data Models)](#7-mô-hình-dữ-liệu-data-models)
8. [API Endpoints](#8-api-endpoints)
9. [Hệ Thống Thông Báo Thời Gian Thực](#9-hệ-thống-thông-báo-thời-gian-thực)
10. [Xác Thực Hai Yếu Tố (2FA)](#10-xác-thực-hai-yếu-tố-2fa)
11. [Single Sign-On (SSO / OIDC)](#11-single-sign-on-sso--oidc)
12. [Hệ Thống Email](#12-hệ-thống-email)
13. [Rate Limiting](#13-rate-limiting)
14. [Tác Vụ Định Kỳ (Scheduled Jobs)](#14-tác-vụ-định-kỳ-scheduled-jobs)
15. [Hệ Thống Cấu Hình](#15-hệ-thống-cấu-hình)
16. [Bảo Mật](#16-bảo-mật)
17. [Lưu Trữ File & Attachment](#17-lưu-trữ-file--attachment)
18. [Phụ Lục: Schema Cơ Sở Dữ Liệu](#18-phụ-lục-schema-cơ-sở-dữ-liệu)

---

## 1. Tổng Quan Kiến Trúc

### 1.1 Ngôn Ngữ & Framework

| Thành phần | Công nghệ |
|-----------|-----------|
| **Runtime** | Rust (async, Tokio runtime) |
| **Web Framework** | Rocket 0.5 (async) |
| **ORM** | Diesel (với diesel_migrations nhúng) |
| **Serialization** | Serde / Serde JSON |
| **JWT** | `jsonwebtoken` crate, thuật toán RS256 |
| **Mã hóa** | OpenSSL (RSA 2048-bit), Argon2id |
| **Cấp phát bộ nhớ** | MiMalloc (tùy chọn, feature `enable_mimalloc`) |
| **Message format (WS)** | MessagePack (`rmpv`) |

### 1.2 Luồng Khởi Động (Startup Flow)

```
main()
 ├─ parse_args()           → CLI: --help, --version, hash, backup
 ├─ launch_info()          → In banner banner ra stdout
 ├─ init_logging()         → Fern-based multi-target logger
 ├─ check_data_folder()    → Kiểm tra /data persistent volume
 ├─ auth::initialize_keys()→ Load hoặc tạo RSA-2048 private key
 ├─ check_web_vault()      → Kiểm tra web-vault index.html
 ├─ create_dir(tmp_folder) → Tạo thư mục tmp
 ├─ create_db_pool()       → Migration + R2D2 connection pool
 ├─ schedule_jobs()        → Thread riêng cho cron jobs
 ├─ TwoFactor::migrate_*() → Migration U2F → WebAuthn / Passkey
 └─ launch_rocket()        → Mount routes, Rocket ignite + launch
```

### 1.3 Giới Hạn Request

Được đặt khi khởi tạo Rocket:

```
json      → 20 MB   (dành cho import lớn ~5000+ vault entries)
data-form → 525 MB  (upload file Send)
file      → 525 MB  (upload attachment)
```

---

## 2. Cấu Trúc Module

```
src/
├── main.rs              ← Entry point, server setup, job scheduler
├── auth.rs              ← JWT, RSA keys, request guards (Headers, OrgHeaders, AdminHeaders...)
├── config.rs            ← Macro-based config system (make_config!)
├── crypto.rs            ← Tiện ích mã hóa (random bytes, base32/64, ct_eq)
├── error.rs             ← Error type, err!/err_code!/err_json! macros
├── http_client.rs       ← Reqwest HTTP client wrapper
├── mail.rs              ← Lettre SMTP email sender
├── ratelimit.rs         ← Governor-based IP rate limiter (2 limiters)
├── sso.rs               ← SSO/OIDC flow: nonce, token exchange, cache
├── sso_client.rs        ← openidconnect OIDC client (Client struct, cached)
├── util.rs              ← Utilities: AppHeaders, Cors, BetterLogging, format helpers
├── api/
│   ├── mod.rs           ← Route exports, ApiResult type aliases, MasterPasswordPolicy
│   ├── admin.rs         ← Admin panel (/admin), Argon2id token verify
│   ├── core/            ← Core API (/api) - accounts, ciphers, organizations...
│   │   ├── mod.rs
│   │   ├── accounts.rs
│   │   ├── ciphers.rs
│   │   ├── collections.rs
│   │   ├── emergency_access.rs
│   │   ├── events.rs
│   │   ├── folders.rs
│   │   ├── organizations.rs
│   │   ├── sends.rs
│   │   └── two_factor/
│   │       ├── mod.rs           ← 2FA dispatch, enforce_2fa_policy
│   │       ├── authenticator.rs ← TOTP (RFC 6238)
│   │       ├── duo.rs           ← Duo iframe (legacy)
│   │       ├── duo_oidc.rs      ← Duo OIDC (current)
│   │       ├── email.rs         ← Email 2FA
│   │       ├── protected_actions.rs ← OTP for sensitive ops
│   │       ├── webauthn.rs      ← FIDO2/WebAuthn/Passkey
│   │       └── yubikey.rs       ← YubiKey OTP
│   ├── icons.rs         ← Favicon proxy (/icons)
│   ├── identity.rs      ← OAuth2 token endpoint (/identity)
│   ├── notifications.rs ← WebSocket hub (/notifications)
│   ├── push.rs          ← Bitwarden push relay integration
│   └── web.rs           ← Static files, web-vault serve (/), catchers
└── db/
    ├── mod.rs           ← DbPool, DbConn, migrations, db_run! macro
    ├── schema.rs        ← Diesel table! macro definitions (22 tables)
    ├── query_logger.rs  ← Slow query logger
    └── models/
        ├── attachment.rs
        ├── auth_request.rs
        ├── cipher.rs
        ├── collection.rs
        ├── device.rs
        ├── emergency_access.rs
        ├── event.rs
        ├── favorite.rs
        ├── folder.rs
        ├── group.rs
        ├── org_policy.rs
        ├── organization.rs
        ├── send.rs
        ├── sso_nonce.rs
        ├── two_factor.rs
        ├── two_factor_duo_context.rs
        ├── two_factor_incomplete.rs
        └── user.rs
```

---

## 3. Lớp Web Framework & Server

### 3.1 Route Mounting (launch_rocket)

```
{basepath}/              → api::web_routes()         (Static files, web-vault)
{basepath}/api           → api::core_routes()        (Bitwarden API)
{basepath}/admin         → api::admin_routes()       (Admin panel)
{basepath}/events        → api::core_events_routes() (Org events)
{basepath}/identity      → api::identity_routes()    (OAuth2 token)
{basepath}/icons         → api::icons_routes()       (Favicon proxy)
{basepath}/notifications → api::notifications_routes() (WebSocket)
```

### 3.2 Managed State

```rust
// Được Rocket manage() để inject vào handlers:
pool               → DbPool (R2D2 connection pool)
WS_USERS           → Arc<WebSocketUsers>         (authenticated WS connections)
WS_ANONYMOUS_SUBSCRIPTIONS → Arc<AnonymousWebSocketSubscriptions>
```

### 3.3 Fairings (Middleware)

| Fairing | Chức năng |
|---------|----------|
| `AppHeaders` | Thêm các header bảo mật (X-Content-Type-Options, CSP...) |
| `Cors` | Xử lý CORS preflight |
| `BetterLogging` | Request logging với extra_debug mode |

### 3.4 Cơ Chế Graceful Shutdown

- Xử lý `Ctrl+C` → gọi `CONFIG.shutdown()` → kích hoạt Rocket shutdown handle
- Unix: xử lý `SIGUSR1` để trigger SQLite backup

---

## 4. Hệ Thống Xác Thực (Authentication)

### 4.1 RSA Key Management (`auth.rs`)

```
Khởi động:
  auth::initialize_keys()
    → đọc private_rsa_key từ opendal operator (filesystem hoặc S3)
    → nếu không có → tạo mới RSA-2048, lưu lại
    → nạp vào PRIVATE_RSA_KEY (OnceLock<EncodingKey>)
              PUBLIC_RSA_KEY  (OnceLock<DecodingKey>)

Tất cả JWT được ký bằng RS256 dùng cặp khóa này.
```

### 4.2 JWT Token Types

| Loại JWT | Issuer Pattern | TTL | Struct |
|---------|----------------|-----|--------|
| Login (Access) | `{domain}\|login` | 2 giờ | `LoginJwtClaims` |
| Refresh | `{domain}\|login` | 30 ngày (90 ngày mobile) | `RefreshJwtClaims` |
| Invite | `{domain}\|invite` | `INVITATION_EXPIRATION_HOURS` | `InviteJwtClaims` |
| Emergency Access Invite | `{domain}\|emergencyaccessinvite` | config giờ | `EmergencyAccessInviteJwtClaims` |
| Delete | `{domain}\|delete` | config giờ | `BasicJwtClaims` |
| Verify Email | `{domain}\|verifyemail` | config giờ | `BasicJwtClaims` |
| Admin | `{domain}\|admin` | `ADMIN_SESSION_LIFETIME` phút | `BasicJwtClaims` |
| Send | `{domain}\|send` | 2 phút | `BasicJwtClaims` |
| Org API Key | `{domain}\|api.organization` | 1 giờ | `OrgApiKeyLoginJwtClaims` |
| File Download | `{domain}\|file_download` | 5 phút | `FileDownloadClaims` |
| Register Verify | `{domain}\|register_verify` | 30 phút | `RegisterVerifyClaims` |
| SSO | `{domain}\|sso` | 2 phút (nonce), 5 phút (code) | `SsoTokenJwtClaims` |

### 4.3 Login Flow (`identity.rs`)

**Endpoint:** `POST /identity/connect/token`

Hỗ trợ các `grant_type`:

```
1. "password"              → _password_login()
   - Nếu SSO_ONLY=true → từ chối
   - Kiểm tra user, enabled, password hash
   - Xử lý auth_request (passwordless login)
   - KDF upgrade nếu cần
   - 2FA enforcement
   - Trả về access_token + refresh_token

2. "refresh_token"         → _refresh_login()
   - Decode refresh JWT
   - Kiểm tra device còn tồn tại không
   - Cấp access_token mới

3. "client_credentials"    → _api_key_login()
   - scope "api"           → _user_api_key_login() (bỏ qua 2FA)
   - scope "api.organization" → _organization_api_key_login()

4. "authorization_code"
   - nếu SSO_ENABLED=true  → _sso_login()
   - ngược lại             → lỗi
```

### 4.4 Security Stamp

Mỗi User có `security_stamp` (UUID4). Mỗi lần đăng nhập, `sstamp` được nhúng vào JWT.

Khi xác minh request (`Headers::from_request`):
1. Decode JWT → lấy `sstamp` và `device_id`
2. Tìm device theo `device_id`
3. So sánh `user.security_stamp == claims.sstamp`
4. Nếu không khớp → kiểm tra `stamp_exception` (UserStampException)
   - Có thời hạn (`expire`)
   - Chỉ áp dụng cho các route cụ thể (`routes`)

### 4.5 AuthTokens Structure

```rust
pub struct AuthTokens {
    refresh_claims: RefreshJwtClaims,
    access_claims:  LoginJwtClaims,
}
// Các method:
// .access_token()  → encode JWT của access_claims
// .refresh_token() → encode JWT của refresh_claims (bao gồm device.refresh_token)
// .expires_in()    → số giây còn lại
// .scope()         → Vec<String> scope của token
```

---

## 5. Hệ Thống Phân Quyền (Authorization)

### 5.1 Request Guards (Rocket)

Rocket extract các guard từ request headers:

| Guard | Yêu cầu | Sử dụng khi |
|-------|---------|-------------|
| `Host` | Header Referer / X-Forwarded-Host | Mọi request cần host |
| `ClientIp` | X-Forwarded-For hoặc remote_addr | Rate limiting, logging |
| `ClientHeaders` | device-type header + IP | Login endpoints |
| `Headers` | Bearer token hợp lệ + device + security_stamp | Mọi authenticated API |
| `OrgHeaders` | Headers + membership trong org | Org API endpoints |
| `AdminHeaders` | OrgHeaders + MembershipType >= Admin | Org admin operations |
| `OwnerHeaders` | OrgHeaders + MembershipType == Owner | Owner-only operations |
| `ManagerHeaders` | OrgHeaders + MembershipType >= Manager | Manager operations |
| `CollectionHeaders` | OrgHeaders + Collection access | Collection-scoped ops |

### 5.2 MembershipType (RBAC)

```
MembershipType:
  Owner   = 0  ← Quyền cao nhất, không bị ràng buộc chính sách
  Admin   = 1  ← Quản trị, không bị ràng buộc chính sách 2FA
  Manager = 2  ← Quản lý collection được giao
  User    = 3  ← Thành viên thông thường
```

### 5.3 MembershipStatus (Trạng Thái Thành Viên)

```
Invited   = 0  ← Đã mời, chưa chấp nhận
Accepted  = 1  ← Đã chấp nhận, chờ confirm
Confirmed = 2  ← Đã confirm (có quyền truy cập)
Revoked   = -1 ← Bị thu hồi quyền
```

### 5.4 Quyền Truy Cập Collection

Bảng `users_collections` và `collections_groups`:

```
read_only      = true  → chỉ đọc
hide_passwords = true  → ẩn username/password
manage         = true  → có quyền quản lý collection
```

---

## 6. Lớp Cơ Sở Dữ Liệu (Database Layer)

### 6.1 Hỗ Trợ Đa Cơ Sở Dữ Liệu

Được build với Cargo feature flags:

```
sqlite     → diesel::sqlite::SqliteConnection      (mặc định)
postgresql → diesel::pg::PgConnection
mysql      → diesel::mysql::MysqlConnection
```

Xác định loại DB từ URL:

```
mysql://...       → MySQL
postgresql://...  → PostgreSQL
postgres://...    → PostgreSQL
<khác>            → SQLite
```

### 6.2 Connection Pool (R2D2)

```rust
DbPool {
    pool:      Option<Pool<DbConnManager>>   // R2D2 pool
    semaphore: Arc<Semaphore>                // Giới hạn concurrent connections
}

// Cấu hình (CONFIG):
database_max_conns      (pool size)
database_min_conns      (min idle)
database_idle_timeout   (seconds)
database_timeout        (connection timeout)
```

### 6.3 db_run! Macro

```rust
// Sử dụng trong models để chạy DB queries:
db_run! { conn: { diesel_query... } }

// Hoặc phân nhánh theo DB type:
db_run! { conn:
    sqlite { /* sqlite-specific */ }
    postgresql,mysql { /* pg + mysql */ }
}
```

### 6.4 SQLite Tối Ưu Hóa

```sql
PRAGMA busy_timeout = 5000;   -- Chờ 5 giây nếu DB bị lock
PRAGMA synchronous = NORMAL;  -- Cân bằng an toàn/hiệu năng
-- Tuỳ chọn (nếu ENABLE_DB_WAL=true):
PRAGMA journal_mode = wal;    -- Write-Ahead Logging
```

### 6.5 Migration

Các migration SQL được nhúng vào binary lúc compile:
```
migrations/sqlite/      ← SQLite migrations
migrations/postgresql/  ← PostgreSQL migrations  
migrations/mysql/       ← MySQL migrations
```

---

## 7. Mô Hình Dữ Liệu (Data Models)

### 7.1 Bảng Chính và Quan Hệ

```
users (uuid PK)
  ├── devices (uuid PK, user_uuid FK)
  ├── folders (uuid PK, user_uuid FK)
  ├── ciphers (uuid PK, user_uuid nullable FK)    ← personal
  ├── sends (uuid PK, user_uuid nullable FK)
  ├── twofactor (uuid PK, user_uuid FK)
  ├── emergency_access (uuid PK, grantor_uuid FK)
  ├── sso_users (user_uuid PK FK)
  └── users_organizations (uuid PK)
        ├── org_uuid FK → organizations
        └── (nhiều FK tới collection, group...)

organizations (uuid PK)
  ├── ciphers (nhiều, organization_uuid nullable FK) ← org ciphers
  ├── collections (uuid PK, org_uuid FK)
  │     ├── users_collections (user_uuid, collection_uuid PK)
  │     └── collections_groups (groups_uuid, collections_uuid PK)
  ├── groups (uuid PK, organizations_uuid FK)
  │     └── groups_users (groups_uuid, users_organizations_uuid PK)
  ├── org_policies (uuid PK, org_uuid FK)
  ├── organization_api_key (uuid PK, org_uuid FK)
  └── event (uuid PK)
```

### 7.2 Model: User

Các trường quan trọng (`users` table):

| Trường | Kiểu | Mô tả |
|--------|------|-------|
| `uuid` | Text PK | UUID người dùng |
| `email` | Text | Email (dùng để đăng nhập) |
| `password_hash` | Binary | Bcrypt/Argon2 hash của master password |
| `salt` | Binary | Salt cho hash |
| `password_iterations` | Integer | KDF iterations |
| `akey` | Text | Symmetric key (mã hóa E2E) |
| `private_key` | Text nullable | RSA private key của user (mã hóa bằng symmetric key) |
| `public_key` | Text nullable | RSA public key của user |
| `security_stamp` | Text | UUID đổi khi có thay đổi bảo mật |
| `stamp_exception` | Text nullable | JSON: tạm thời bỏ qua security stamp check |
| `client_kdf_type` | Integer | 0=PBKDF2, 1=Argon2id |
| `client_kdf_iter` | Integer | Số iterations |
| `client_kdf_memory` | Integer nullable | Argon2: bộ nhớ (KB) |
| `client_kdf_parallelism` | Integer nullable | Argon2: parallelism |
| `totp_recover` | Text nullable | Recovery code BASE32 (20 bytes) |
| `api_key` | Text nullable | API key của user |
| `verified_at` | Timestamp nullable | Thời điểm email được verify |
| `enabled` | Bool | Tài khoản có bị vô hiệu không |

### 7.3 Model: Cipher (Vault Item)

```
ciphers:
  uuid, created_at, updated_at
  user_uuid    nullable  ← personal cipher
  org_uuid     nullable  ← org cipher
  atype:
    1 = Login
    2 = SecureNote
    3 = Card
    4 = Identity
  name         (mã hóa E2E client-side)
  notes        nullable (mã hóa)
  fields       nullable (mã hóa JSON)
  data         TEXT     (mã hóa JSON payload chính)
  password_history nullable (mã hóa JSON)
  deleted_at   nullable (soft delete → trash)
  reprompt     nullable (0=None, 1=MasterPassword)
```

### 7.4 Model: Device

```
devices:
  uuid, user_uuid, name, atype (DeviceType: iOS, Android, Chrome...)
  push_uuid        ← ID đăng ký với Bitwarden push relay
  push_token       ← FCM/APNs token
  refresh_token    ← Opaque string, dùng trong JWT RefreshJwtClaims
  twofactor_remember ← Token "remember 2FA" (30 ngày)
```

### 7.5 Model: Emergency Access

```
emergency_access:
  grantor_uuid, grantee_uuid nullable, email nullable
  atype:
    0 = View       ← Chỉ xem vault
    1 = Takeover   ← Chiếm quyền tài khoản
  status:
    0 = Invited
    1 = Accepted
    2 = Confirmed
    3 = RecoveryInitiated
    4 = RecoveryApproved
  wait_time_days    ← Số ngày chờ trước khi auto-approve
  key_encrypted     ← Khóa vault người dùng được mã hóa bằng public key của grantee
```

### 7.6 Model: Send

```
sends:
  uuid, user_uuid nullable, organization_uuid nullable
  atype: 0=Text, 1=File
  data         (mã hóa E2E)
  akey         (send-specific symmetric key, mã hóa)
  password_hash, password_salt, password_iter  ← bảo vệ Send bằng password
  max_access_count nullable, access_count
  expiration_date nullable, deletion_date
  disabled     ← tắt send thủ công
  hide_email   ← ẩn email người tạo
```

### 7.7 Bảng Event (Audit Log)

```
event:
  uuid, event_type (i32), event_date
  user_uuid, org_uuid, cipher_uuid, collection_uuid
  group_uuid, org_user_uuid, act_user_uuid
  device_type, ip_address
  policy_uuid, provider_uuid, provider_user_uuid, provider_org_uuid
```

---

## 8. API Endpoints

### 8.1 Identity API (`/identity`)

| Method | Path | Chức năng |
|--------|------|----------|
| POST | `/connect/token` | Đăng nhập / refresh token / SSO |
| POST | `/accounts/prelogin` | Lấy KDF params của user |
| POST | `/accounts/register` | Đăng ký tài khoản (bước 1) |
| POST | `/accounts/register/send-verification-email` | Gửi email xác minh |
| POST | `/accounts/register/finish` | Hoàn tất đăng ký (bước 2) |
| GET | `/accounts/prevalidate` | Kiểm tra email đăng ký |
| GET | `/connect/oidc-signin` | Bắt đầu SSO flow |
| GET/POST | `/oidcsignin` | SSO callback |

### 8.2 Core API (`/api`)

**Accounts:**

| Method | Path | Chức năng |
|--------|------|----------|
| GET | `/accounts/profile` | Lấy profile |
| PUT | `/accounts/profile` | Cập nhật profile |
| PUT | `/accounts/password` | Đổi master password |
| PUT | `/accounts/kdf` | Cập nhật KDF settings |
| PUT | `/accounts/security-stamp` | Quay vòng security stamp |
| POST | `/accounts/verify-password` | Xác nhận mật khẩu |
| POST | `/accounts/verify-otp` | Xác nhận OTP |
| POST | `/accounts/email-token` | Gửi token đổi email |
| PUT | `/accounts/email` | Đổi email |
| DELETE | `/accounts` | Xoá tài khoản |
| GET | `/accounts/api-key` | Lấy API key |
| POST | `/accounts/rotate-api-key` | Tạo API key mới |
| POST | `/accounts/set-password` | Set password (SSO user) |
| POST | `/accounts/request-otp` | Yêu cầu OTP cho protected action |

**Ciphers:**

| Method | Path | Chức năng |
|--------|------|----------|
| GET | `/ciphers` | Danh sách tất cả cipher của user |
| GET | `/ciphers/{id}` | Chi tiết cipher |
| POST | `/ciphers` | Tạo cipher |
| PUT | `/ciphers/{id}` | Cập nhật cipher |
| DELETE | `/ciphers/{id}` | Xoá cipher (soft delete) |
| DELETE | `/ciphers` | Bulk delete |
| PUT | `/ciphers/{id}/restore` | Khôi phục từ trash |
| PUT | `/ciphers/restore` | Bulk restore |
| POST | `/ciphers/purge` | Xoá toàn bộ vault |
| GET | `/ciphers/{id}/attachment/{attachment_id}` | Tải attachment |
| POST | `/ciphers/{id}/attachment` | Upload attachment |
| DELETE | `/ciphers/{id}/attachment/{attachment_id}` | Xoá attachment |
| PUT | `/ciphers/{id}/share` | Chia sẻ cipher vào collection |
| PUT | `/ciphers/share` | Bulk share |
| PUT | `/ciphers/{id}/collections` | Cập nhật collections của cipher |
| GET | `/ciphers/organization-details` | Cipher của tổ chức |
| GET | `/ciphers/{id}/admin-details` | Admin view cipher |
| POST | `/ciphers/import` | Import vault |

**Folders:**

| Method | Path | Chức năng |
|--------|------|----------|
| GET | `/folders` | Danh sách folder |
| POST | `/folders` | Tạo folder |
| PUT | `/folders/{id}` | Cập nhật folder |
| DELETE | `/folders/{id}` | Xoá folder |

**Organizations:**

| Method | Path | Chức năng |
|--------|------|----------|
| POST | `/organizations` | Tạo tổ chức |
| GET | `/organizations/{id}` | Chi tiết tổ chức |
| PUT | `/organizations/{id}` | Cập nhật tổ chức |
| DELETE | `/organizations/{id}` | Xoá tổ chức |
| GET | `/organizations/{id}/users` | Danh sách thành viên |
| POST | `/organizations/{id}/users/invite` | Mời thành viên |
| PUT | `/organizations/{id}/users/{uid}/confirm` | Xác nhận thành viên |
| PUT | `/organizations/{id}/users/{uid}` | Cập nhật vai trò |
| DELETE | `/organizations/{id}/users/{uid}` | Xoá thành viên |
| POST | `/organizations/{id}/users/{uid}/revoke` | Thu hồi quyền |
| POST | `/organizations/{id}/users/{uid}/restore` | Khôi phục quyền |
| GET | `/organizations/{id}/collections` | Danh sách collection |
| POST | `/organizations/{id}/collections` | Tạo collection |
| PUT | `/organizations/{id}/collections/{cid}` | Cập nhật collection |
| DELETE | `/organizations/{id}/collections/{cid}` | Xoá collection |
| GET | `/organizations/{id}/policies` | Danh sách chính sách |
| PUT | `/organizations/{id}/policies/{type}` | Cập nhật chính sách |
| GET | `/organizations/{id}/groups` | Danh sách nhóm |
| POST | `/organizations/{id}/groups` | Tạo nhóm |
| PUT | `/organizations/{id}/groups/{gid}` | Cập nhật nhóm |
| DELETE | `/organizations/{id}/groups/{gid}` | Xoá nhóm |
| GET/POST | `/organizations/{id}/api-key` | Quản lý API key tổ chức |

**Two-Factor:**

| Method | Path | Chức năng |
|--------|------|----------|
| GET | `/two-factor` | Danh sách 2FA hiện có |
| POST | `/two-factor/get-recover` | Lấy recovery code |
| POST | `/two-factor/recover` | Recovery 2FA |
| POST/PUT | `/two-factor/disable` | Tắt 2FA |
| GET/POST | `/two-factor/get-authenticator` | TOTP setup |
| GET/POST | `/two-factor/get-yubikey` | YubiKey setup |
| GET/POST | `/two-factor/get-email` | Email 2FA setup |
| GET/POST | `/two-factor/webauthn` | WebAuthn setup |
| GET/POST | `/two-factor/get-duo` | Duo setup |

**Sends:**

| Method | Path | Chức năng |
|--------|------|----------|
| GET | `/sends` | Danh sách sends |
| POST | `/sends` | Tạo text send |
| POST | `/sends/file` | Tạo file send |
| PUT | `/sends/{id}` | Cập nhật send |
| DELETE | `/sends/{id}` | Xoá send |
| GET | `/sends/{id}/file/{file_id}` | Download file send |

**Emergency Access:**

| Method | Path | Chức năng |
|--------|------|----------|
| GET | `/emergency-access/granted` | Danh sách được cấp |
| GET | `/emergency-access/trusted` | Danh sách đã cấp |
| POST | `/emergency-access/invite` | Mời người tín nhiệm |
| PUT | `/emergency-access/{id}/accept` | Chấp nhận lời mời |
| PUT | `/emergency-access/{id}/confirm` | Xác nhận (với key) |
| POST | `/emergency-access/{id}/initiate` | Khởi động phục hồi |
| POST | `/emergency-access/{id}/approve` | Chấp thuận phục hồi |
| POST | `/emergency-access/{id}/reject` | Từ chối phục hồi |
| GET | `/emergency-access/{id}/view` | Xem vault của người được truy cập |
| POST | `/emergency-access/{id}/takeover` | Chiếm quyền tài khoản |

### 8.3 Events API (`/events`)

| Method | Path | Chức năng |
|--------|------|----------|
| GET | `/organizations/{id}/events` | Lấy events của tổ chức |
| GET | `/ciphers/{id}/events` | Events của cipher |
| GET | `/organizations/{id}/users/{uid}/events` | Events của thành viên |
| POST | `/organizations/{id}/events` | Bulk create events (API key) |

### 8.4 Admin Panel (`/admin`)

- Xác thực: `ADMIN_TOKEN` (Argon2id PHC string)
- Session: JWT với issuer `{domain}|admin`, TTL = `ADMIN_SESSION_LIFETIME` phút
- Rate limit: IP-based, `ADMIN_RATELIMIT_SECONDS`, `ADMIN_RATELIMIT_MAX_BURST`

Chức năng: Quản lý users toàn hệ thống, cấu hình server, xem thống kê, gửi email test, mời users, xem tổ chức...

---

## 9. Hệ Thống Thông Báo Thời Gian Thực

### 9.1 WebSocket (`/notifications/hub`)

```
Authenticated: GET /notifications/hub?access_token=<token>
               hoặc header Sec-WebSocket-Protocol: access_token
Anonymous:     GET /notifications/anonymous-hub?<token>
```

**Cấu trúc nội bộ:**

```rust
WebSocketUsers {
    map: Arc<DashMap<String /* user_uuid */, Vec<(Uuid, Sender<Message>)>>>
}
```

- Mỗi user có thể có nhiều kết nối WS đồng thời (nhiều thiết bị)
- `WSEntryMapGuard` tự động xoá entry khi kết nối đóng (Drop trait)
- Ping interval: 15 giây
- Message format: MessagePack (protocol: "messagepack", version: 1)

**Message Structure (SignalR-compatible MessagePack):**

```
[
  1,              // MessageType.Invocation
  {},             // Headers
  null,           // InvocationId
  "ReceiveMessage", // Target
  [{
    "ContextId": acting_device_id | Nil,
    "Type": UpdateType as i32,
    "Payload": { ... }
  }]
]
```

### 9.2 UpdateType Enum

```
SyncCipherUpdate = 0    SyncCipherCreate = 1    SyncLoginDelete = 2
SyncFolderDelete = 3    SyncCiphers = 4         SyncVault = 5
SyncOrgKeys = 6         SyncFolderCreate = 7    SyncFolderUpdate = 8
SyncSettings = 10       LogOut = 11
SyncSendCreate = 12     SyncSendUpdate = 13     SyncSendDelete = 14
AuthRequest = 15        AuthRequestResponse = 16
None = 100
```

### 9.3 Push Notifications (`api/push.rs`)

Khi WS không khả dụng (mobile apps), dùng Bitwarden Push Relay:

```
Flow:
  1. Lấy auth token từ {PUSH_IDENTITY_URI}/connect/token
     (client_credentials, scope=api.push, installation_id+key)
  2. Token được cache, hết hạn sau expires_in/2 giây
  3. Gửi notification JSON tới {PUSH_RELAY_URI}/push/send
     Authorization: Bearer <auth_token>
```

**Notification payload:**
```json
{
  "userId": "...",
  "organizationId": null,
  "deviceId": "<push_uuid>",
  "identifier": "<device_uuid>",
  "type": 0,
  "payload": { "id": "...", "userId": "...", "revisionDate": "..." },
  "installationId": null
}
```

Push chỉ gửi cho cipher của personal vault (không gửi cho org ciphers).

---

## 10. Xác Thực Hai Yếu Tố (2FA)

### 10.1 Các Phương Thức Hỗ Trợ

| ID | Loại | Module |
|----|------|--------|
| 0 | TOTP Authenticator (RFC 6238) | `authenticator.rs` |
| 1 | Email | `email.rs` |
| 2 | Duo (legacy iframe) | `duo.rs` |
| 3 | YubiKey OTP | `yubikey.rs` |
| 4 | WebAuthn / Passkey / FIDO2 | `webauthn.rs` |
| 5 | Duo OIDC (hiện tại) | `duo_oidc.rs` |
| 7 | Recovery Code | (xử lý trong `identity.rs`) |
| 9 | Remember Device (30 ngày) | (xử lý trong `identity.rs`) |

### 10.2 2FA Flow trong Login

```
twofactor_auth()
  → TwoFactor::find_by_user() → lấy danh sách 2FA methods đã bật
  → Nếu rỗng → enforce_2fa_policy() → OK (không cần 2FA)
  → TwoFactorIncomplete::mark_incomplete() ← ghi log
  → Nếu không có two_factor_token trong request → trả lỗi với TwoFactorProviders2
  → Validate dựa trên selected_id:
      TOTP        → HOTP (time-based, TOTP_STEP=30s window±1)
      WebAuthn    → challenge/response flow
      YubiKey     → API call tới yubico servers
      Duo         → legacy iframe hoặc OIDC flow
      Email       → mã 6 số, ttl config
      Remember    → ct_eq với device.twofactor_remember token
      RecoveryCode→ xoá tất cả 2FA sau khi dùng
  → TwoFactorIncomplete::mark_complete()
  → Nếu remember=1 → device.refresh_twofactor_remember() → lưu token 30 ngày
```

### 10.3 Chính Sách 2FA Tổ Chức

```rust
enforce_2fa_policy(user, act_user_id, device_type, ip, conn)
  → Với mỗi Membership của user có OrgPolicyType::TwoFactorAuthentication
  → Chỉ áp dụng cho MembershipType < Admin
  → Nếu vi phạm: revoke membership + gửi email thông báo + log event
```

### 10.4 Protected Actions OTP

Với các hành động nhạy cảm (đổi password, xem API key...):
```
POST /accounts/request-otp   ← Gửi email OTP
POST /accounts/verify-otp    ← Xác minh OTP
```
OTP được lưu trong `TwoFactor` với type `OrganizationDuo` (type 6) làm placeholder.

---

## 11. Single Sign-On (SSO / OIDC)

### 11.1 Kiến Trúc SSO

```
Modules:
  sso.rs        ← SSO flow logic (authorize URL, exchange code, redeem)
  sso_client.rs ← openidconnect OIDC Client (cached, lazy init)

Database:
  sso_nonce     ← PKCE verifier, nonce, redirect_uri (TTL 10 phút)
  sso_users     ← Mapping user_uuid ↔ OIDC identifier (issuer/subject)
```

### 11.2 SSO Authorization Flow

```
1. Client → GET /identity/connect/oidc-signin → authorize_url()
   → Tạo OIDCState (random UUID)
   → Client::authorize_url(state, redirect_uri) → PKCE code verifier + nonce
   → Lưu SsoNonce vào DB
   → Redirect client đến IdP/Provider

2. IdP → redirect về /oidcsignin?code=...&state=...
   → Decode state (base64) → OIDCState
   → Tùy client_id: sso-connector.html (web), bitwarden://sso-callback (mobile), localhost (CLI)

3. Client → POST /identity/connect/token (grant_type=authorization_code)
   → _sso_login()
   → decode_code_claims() → giải mã JWT chứa OIDCCodeWrapper
   → exchange_code():
       → Client::exchange_code(code, nonce) → token_response + id_claims
       → Client::user_info(access_token)
       → Lấy email, email_verified, identifier (issuer/subject)
       → Cache AuthenticatedUser vào AC_CACHE (TTL 10 phút)
   → SsoUser::find_by_identifier → tìm user hiện có
   → Nếu mới: tạo User mới (nếu email domain được phép)
   → 2FA flow nếu cần
   → redeem(state): xoá nonce, lấy auth user từ cache
   → create_auth_tokens(): tạo JWT access/refresh từ SSO tokens
```

### 11.3 Identifier

```rust
OIDCIdentifier = "{issuer}/{subject}"
// Dùng để map SSO user ↔ Vaultwarden user
// Stored in sso_users.identifier
```

### 11.4 Cấu Hình SSO

| Biến | Mô tả |
|-----|-------|
| `SSO_ENABLED` | Bật SSO |
| `SSO_ONLY` | Chỉ cho phép SSO (chặn password login) |
| `SSO_AUTHORITY` | URL của OIDC authority (ví dụ: Keycloak realm URL) |
| `SSO_CLIENT_ID` | Client ID |
| `SSO_CLIENT_SECRET` | Client Secret |
| `SSO_SCOPES` | Scopes bổ sung (mặc định: openid, email, profile, offline_access) |
| `SSO_SIGNUPS_MATCH_EMAIL` | Cho phép link SSO user với Vaultwarden user qua email |
| `SSO_ALLOW_UNKNOWN_EMAIL_VERIFICATION` | Chấp nhận khi IdP không gửi email_verified |
| `SSO_AUTH_ONLY_NOT_SESSION` | Dùng SSO chỉ để xác thực, không cho phép SSO quản lý session |

---

## 12. Hệ Thống Email

### 12.1 Thư Viện

- **`lettre`**: SMTP transport
- **Templates**: Handlebars (`.hbs` trong `src/static/templates/email/`)

### 12.2 Các Email Được Gửi

| Sự kiện | Hàm trong mail.rs |
|---------|-------------------|
| Xác minh email đăng ký | `send_verify_email()` |
| Mời vào tổ chức | `send_invite_email()` |
| Xác nhận đã tham gia tổ chức | `send_org_confirmed_email()` |
| Đăng nhập từ thiết bị mới | `send_new_device_logged_in()` |
| 2FA chưa hoàn thành | `send_incomplete_2fa_login()` |
| 2FA bị xoá khỏi tổ chức | `send_2fa_removed_from_org()` |
| Emergency access mời | `send_emergency_access_invite()` |
| Emergency access nhắc nhở | `send_emergency_access_reminder()` |
| Emergency access thông báo hết hạn | `send_emergency_access_recovery_*()` |
| Đổi mật khẩu | `send_change_email()` |
| Xoá tài khoản | `send_delete_account()` |
| Email 2FA OTP | `send_token()` |
| Protected action OTP | `send_otp()` |
| SSO email thay đổi | `send_sso_change_email()` |
| Admin test email | `send_test()` |

### 12.3 Cấu Hình SMTP

```
SMTP_HOST, SMTP_PORT, SMTP_SECURITY (starttls/force_tls/off)
SMTP_USERNAME, SMTP_PASSWORD
SMTP_FROM, SMTP_FROM_NAME
SMTP_AUTH_MECHANISM (plain/login/xoauth2)
SMTP_TIMEOUT (seconds)
SMTP_DEBUG (bật lettre debug logging - chứa thông tin nhạy cảm!)
```

---

## 13. Rate Limiting

Sử dụng `governor` crate, IP-based, DashMap state store:

| Limiter | Áp dụng cho | Cấu hình |
|---------|------------|---------|
| `LIMITER_LOGIN` | `POST /identity/connect/token` và SSO | `LOGIN_RATELIMIT_SECONDS`, `LOGIN_RATELIMIT_MAX_BURST` |
| `LIMITER_ADMIN` | `GET/POST /admin/*` | `ADMIN_RATELIMIT_SECONDS`, `ADMIN_RATELIMIT_MAX_BURST` |

**Mặc định (từ config.rs):**
```
LOGIN_RATELIMIT_SECONDS   = 60    (1 request / IP / phút window)
LOGIN_RATELIMIT_MAX_BURST = 10    (cho phép burst 10 requests)
ADMIN_RATELIMIT_SECONDS   = 300   (5 phút)
ADMIN_RATELIMIT_MAX_BURST = 3
```

Trả về HTTP 429 khi vượt giới hạn.

---

## 14. Tác Vụ Định Kỳ (Scheduled Jobs)

Thread riêng (`job-scheduler`) sử dụng `job-scheduler-ng` crate:

| Tác vụ | Biến cấu hình | Chức năng |
|--------|--------------|----------|
| Purge Sends | `SEND_PURGE_SCHEDULE` | Xoá Sends đã qua `deletion_date` |
| Purge Trash | `TRASH_PURGE_SCHEDULE` | Xoá cipher trong trash đã cũ |
| Incomplete 2FA | `INCOMPLETE_2FA_SCHEDULE` | Gửi email cảnh báo 2FA chưa hoàn thành |
| Emergency Timeout | `EMERGENCY_REQUEST_TIMEOUT_SCHEDULE` | Auto-approve emergency access |
| Emergency Reminder | `EMERGENCY_NOTIFICATION_REMINDER_SCHEDULE` | Gửi nhắc nhở emergency access |
| Auth Request Purge | `AUTH_REQUEST_PURGE_SCHEDULE` | Xoá auth request hết hạn |
| Duo Context Purge | `DUO_CONTEXT_PURGE_SCHEDULE` | Xoá Duo OIDC contexts hết hạn |
| Event Cleanup | `EVENT_CLEANUP_SCHEDULE` | Xoá events cũ hơn `EVENTS_DAYS_RETAIN` ngày |
| SSO Nonce Purge | `PURGE_INCOMPLETE_SSO_NONCE` | Xoá SSO nonce chưa hoàn thành |

**Poll interval**: `JOB_POLL_INTERVAL_MS` (mặc định 30000ms = 30 giây)

---

## 15. Hệ Thống Cấu Hình

### 15.1 Macro make_config!

Hệ thống cấu hình được định nghĩa qua macro `make_config!` trong `config.rs`. Mỗi entry gồm:

```rust
name: type {
    env_var: ENV_VAR_NAME,
    default: default_value,
    description: "...",
    # optional: editable/readonly/hidden qua admin panel
}
```

### 15.2 Nguồn Cấu Hình (Ưu Tiên Cao → Thấp)

1. Biến môi trường (ENV)
2. File `.env` trong working directory
3. Giá trị mặc định trong code

### 15.3 Admin Panel Override

Một số cấu hình có thể thay đổi qua admin panel (lưu vào `DATA_FOLDER/config.json`):
- Chúng override ENV vars cho phiên hiện tại
- Cần restart để áp dụng các cấu hình read-only (như DATABASE_URL)

### 15.4 Nhóm Cấu Hình Chính

| Nhóm | Tiền tố | Ví dụ |
|------|---------|-------|
| Database | `DATABASE_*` | URL, pool size, timeout |
| Folders | `DATA_FOLDER`, `WEB_VAULT_FOLDER`, `ATTACHMENTS_FOLDER`, `SENDS_FOLDER`, `TMP_FOLDER`, `ICON_CACHE_FOLDER` |
| Domain | `DOMAIN` | Public URL |
| Signup | `SIGNUPS_ALLOWED`, `SIGNUPS_VERIFY`, `SIGNUPS_DOMAINS_WHITELIST` |
| Invitations | `INVITATIONS_ALLOWED`, `INVITATION_EXPIRATION_HOURS` |
| Password Hints | `SHOW_PASSWORD_HINT` |
| Email | `SMTP_*`, `REQUIRE_DEVICE_EMAIL` |
| 2FA | `TWO_FACTOR_*`, `INCOMPLETE_2FA_*` |
| SSO | `SSO_*` |
| Org | `ORG_CREATION_USERS`, `ORG_GROUPS_ENABLED`, `ORG_EVENTS_ENABLED`, `EVENTS_DAYS_RETAIN` |
| Emergency Access | `EMERGENCY_ACCESS_ALLOWED` |
| Send | `SENDS_ALLOWED`, `SEND_PURGE_SCHEDULE` |
| Push | `PUSH_ENABLED`, `PUSH_INSTALLATION_ID`, `PUSH_INSTALLATION_KEY`, `PUSH_RELAY_URI`, `PUSH_IDENTITY_URI` |
| Admin | `ADMIN_TOKEN`, `ADMIN_SESSION_LIFETIME`, `ADMIN_RATELIMIT_*` |
| Logging | `LOG_LEVEL`, `LOG_FILE`, `USE_SYSLOG`, `EXTENDED_LOGGING` |
| Rate Limit | `LOGIN_RATELIMIT_*`, `ADMIN_RATELIMIT_*` |
| JObs | `JOB_POLL_INTERVAL_MS`, `*_SCHEDULE` |
| Storage | `DATA_FOLDER` (S3 prefix `s3://` hỗ trợ via `opendal`) |

---

## 16. Bảo Mật

### 16.1 Mã Hóa Mật Khẩu

```
Master Password → Client-side:
  KDF type 0 (PBKDF2-SHA256): iterations mặc định 600000
  KDF type 1 (Argon2id): memory=64MB, iterations=3, parallelism=4

Server lưu: password_hash = BCrypt/Argon2 hash của
  PBKDF2(password, email, iterations=200000) hoặc tương đương
(Server KHÔNG biết master password gốc)
```

### 16.2 Mã Hóa E2E Vault

```
Mọi cipher data (name, notes, fields, data) được mã hóa client-side
trước khi gửi lên server. Server chỉ lưu ciphertext.

Symmetric key của user (akey) được mã hóa bằng:
  - Master password hash (personal)
  - Public key RSA của organization (khi share vào org)
```

### 16.3 Admin Token

```bash
# Tạo Argon2id PHC hash:
vaultwarden hash --preset bitwarden
# Output: $argon2id$v=19$m=65540,t=3,p=4$...$...
# Lưu vào ADMIN_TOKEN trong .env

# Presets:
# bitwarden: m=65540KB, t=3, p=4 (mặc định)
# owasp:     m=19456KB, t=2, p=1
```

Verify khi login admin: `argon2::Argon2::verify_password(input, stored_hash)`

### 16.4 Security Headers (util.rs AppHeaders fairing)

```
X-Content-Type-Options: nosniff
X-Frame-Options: SAMEORIGIN
X-XSS-Protection: 1; mode=block
Content-Security-Policy: ...
Referrer-Policy: same-origin
Permissions-Policy: ...
```

### 16.5 TLS

Vaultwarden không terminate TLS trực tiếp (không có built-in HTTPS).
Yêu cầu reverse proxy (Nginx/Caddy) để cung cấp HTTPS — **bắt buộc** cho Bitwarden clients.

---

## 17. Lưu Trữ File & Attachment

### 17.1 OpenDAL Integration

Cấu hình `DATA_FOLDER`:
- Filesystem: đường dẫn thông thường (mặc định `./data`)
- S3: `s3://bucket-name/prefix`

OpenDAL operator được tạo từ `CONFIG.opendal_operator_for_path_type(PathType)`.

`PathType` enum:
- `Data` → `DATA_FOLDER`
- `Attachments` → `ATTACHMENTS_FOLDER`  
- `Sends` → `SENDS_FOLDER`
- `IconCache` → `ICON_CACHE_FOLDER`
- `RsaKey` → RSA key file location

### 17.2 Attachment Flow

```
Upload:
  POST /api/ciphers/{id}/attachment
  → Multipart form, giới hạn 525 MB
  → Lưu file tới {ATTACHMENTS_FOLDER}/{cipher_uuid}/{attachment_id}
  → Ghi record vào bảng attachments

Download:
  GET /api/ciphers/{id}/attachment/{attachment_id}
  → Tạo FileDownloadClaims JWT (TTL 5 phút)
  → Client dùng JWT để download từ endpoint khác
```

---

## 18. Phụ Lục: Schema Cơ Sở Dữ Liệu

### 18.1 Danh Sách 22 Bảng

| Bảng | PK | Mô tả |
|------|-----|-------|
| `users` | `uuid` | Thông tin người dùng + KDF + keys |
| `devices` | `(uuid, user_uuid)` | Thiết bị đăng nhập, push token, refresh token |
| `ciphers` | `uuid` | Vault items (Login, Card, Note, Identity) |
| `attachments` | `id` | File đính kèm của cipher |
| `folders` | `uuid` | Thư mục tổ chức personal vault |
| `favorites` | `(user_uuid, cipher_uuid)` | Cipher yêu thích |
| `folders_ciphers` | `(cipher_uuid, folder_uuid)` | Cipher trong folder |
| `sends` | `uuid` | Bitwarden Send (text/file chia sẻ bảo mật) |
| `organizations` | `uuid` | Tổ chức với keypair |
| `users_organizations` | `uuid` | Membership + role + status |
| `collections` | `uuid` | Collection thuộc org |
| `ciphers_collections` | `(cipher_uuid, collection_uuid)` | Cipher trong collection |
| `users_collections` | `(user_uuid, collection_uuid)` | Quyền user với collection |
| `groups` | `uuid` | Nhóm người dùng trong org |
| `groups_users` | `(groups_uuid, users_organizations_uuid)` | Thành viên group |
| `collections_groups` | `(collections_uuid, groups_uuid)` | Quyền group với collection |
| `org_policies` | `uuid` | Chính sách bảo mật của org |
| `organization_api_key` | `(uuid, org_uuid)` | API key của org |
| `twofactor` | `uuid` | Cấu hình 2FA của user |
| `twofactor_incomplete` | `(user_uuid, device_uuid)` | 2FA chưa hoàn thành |
| `twofactor_duo_ctx` | `state` | Duo OIDC context tạm thời |
| `emergency_access` | `uuid` | Emergency access grants |
| `event` | `uuid` | Audit log (org events) |
| `auth_requests` | `uuid` | Passwordless login requests |
| `sso_nonce` | `state` | PKCE nonce + verifier cho SSO |
| `sso_users` | `user_uuid` | Mapping user ↔ OIDC identifier |
| `invitations` | `email` | Email đã được mời (pre-registration) |

### 18.2 Sơ Đồ Quan Hệ Chính (ERD Simplified)

```
users ─────────────────────────────────────────────────────────┐
  │ 1:N devices                                                 │
  │ 1:N folders                                                 │
  │ 1:N ciphers (personal)                                      │
  │ 1:N sends                                                   │
  │ 1:N twofactor                                               │
  │ 1:1 sso_users                                               │
  │ N:M organizations (via users_organizations)                 │
  │   users_organizations ────────────────────────────────────  │
  │     │ N:M collections (via users_collections)          │   │
  │     │ N:M groups (via groups_users)                         │
  │                                                             │
organizations                                                   │
  │ 1:N collections                                             │
  │   collections ──────────────────────────────────────────   │
  │     │ N:M ciphers (via ciphers_collections)                 │
  │     │ N:M groups (via collections_groups)                   │
  │ 1:N groups                                                  │
  │ 1:N org_policies                                            │
  │ 1:1 organization_api_key                                    │
  │ 1:N event (audit log)                                       │
  │ 1:N ciphers (org)                                           │
```

---

## Tài Liệu Tham Khảo Mã Nguồn

| File | Dòng quan trọng |
|------|----------------|
| `src/main.rs` | L71-94: main() startup; L557-622: launch_rocket(); L624-724: schedule_jobs() |
| `src/auth.rs` | L25-48: JWT constants; L163-282: LoginJwtClaims; L510-1248: Request Guards |
| `src/api/identity.rs` | L38-50: routes(); L52-128: login handler; L166-317: _sso_login() |
| `src/api/notifications.rs` | L22-33: WS static maps; L107-253: WS handlers; L618-652: UpdateType enum |
| `src/api/push.rs` | L36-85: push auth token; L87-302: push notification functions |
| `src/db/mod.rs` | L47-55: DbConnInner; L114-123: DbConnType; L185-234: DbPool::from_config() |
| `src/db/schema.rs` | Toàn bộ: 22 bảng, quan hệ joinable!, allow_tables_to_appear_in_same_query! |
| `src/sso.rs` | L23-24: AC_CACHE; L183-199: authorize_url(); L278-340: exchange_code() |
| `src/ratelimit.rs` | L9-37: 2 rate limiters (login + admin) |
| `src/api/core/two_factor/mod.rs` | L180-248: enforce_2fa_policy() |
