# Vaultwarden — Phân Tích Bảo Mật (Security Analysis)

> **Tác giả**: Phân tích bởi chuyên gia bảo mật  
> **Ngày**: 2026-04-11  
> **Phiên bản**: 1.0  
> **Phạm vi**: Phân tích tĩnh mã nguồn, thiết kế hệ thống, và cấu hình triển khai  

---

## Tóm tắt điểm mạnh

Vaultwarden được xây dựng với nền tảng bảo mật tốt:

- `#![forbid(unsafe_code)]` — không có unsafe Rust trong codebase
- `#![forbid(non_ascii_idents)]` — ngăn chặn homograph attack
- Mã hóa đầu-đến-đầu (E2E): server không bao giờ thấy plaintext vault
- Argon2id cho admin token hashing
- PKCE cho SSO/OIDC flow
- Constant-time comparison (`ct_eq`) cho token validation
- Rate limiting per-IP bằng `governor`

---

## Điểm yếu & Giới hạn Bảo Mật

### SEV-1: NGHIÊM TRỌNG

---

#### SEC-CRIT-01: Admin Token Fallback về Plaintext

**File**: [src/api/admin.rs:245](src/api/admin.rs#L245)

```rust
Some(t) => crate::crypto::ct_eq(t.trim(), token.trim()),
```

**Vấn đề**: Nếu `ADMIN_TOKEN` không bắt đầu bằng `$argon2`, hệ thống **tự động fallback** sang so sánh plaintext. Operator có thể đặt token dạng cleartext mà không nhận cảnh báo lỗi — chỉ có cảnh báo UI trong trình duyệt sau 30 ngày. Token plaintext có entropy thấp sẽ dễ bị brute-force trong thời gian ngắn.

**Rủi ro**: Brute-force admin panel → kiểm soát toàn bộ server.

**Khuyến nghị**: Từ chối khởi động nếu `ADMIN_TOKEN` không phải Argon2 PHC string. Hoặc ít nhất ghi log WARN rõ ràng mỗi lần server khởi động.

---

#### SEC-CRIT-02: `DISABLE_ADMIN_TOKEN` — Bỏ qua hoàn toàn xác thực Admin

**File**: [src/config.rs:758](src/config.rs#L758)

```
disable_admin_token: bool, false, def, false;
```

**Vấn đề**: Khi `DISABLE_ADMIN_TOKEN=true`, admin panel hoàn toàn không yêu cầu xác thực. Tính năng này dành cho "external access controls" nhưng không có cơ chế nào đảm bảo external control thực sự được bật. Nếu môi trường container/orchestration vô tình set biến này, admin panel mở toàn bộ internet.

**Rủi ro**: Toàn quyền quản lý server (xóa user, thay đổi config, trigger backup) mà không cần xác thực.

**Khuyến nghị**: Yêu cầu thêm biến xác nhận thứ hai, hoặc ghi audit log rõ ràng khi option này được kích hoạt.

---

### SEV-2: CAO

---

#### SEC-HIGH-01: JWT Access Token lộ trong URL Query Parameter

**File**: [src/api/notifications.rs:51-53](src/api/notifications.rs#L51)

```rust
struct WsAccessToken {
    access_token: Option<String>,
}
```

**Vấn đề**: WebSocket authentication chấp nhận JWT qua `?access_token=<token>` trong URL. Token sẽ bị lưu trong:
- Server access logs (nginx, Caddy, v.v.)
- Reverse proxy logs
- Browser history
- HTTP `Referer` header khi redirect

JWT có hiệu lực 2 giờ, đủ thời gian khai thác nếu logs bị lộ.

**Rủi ro**: Session hijacking qua log exfiltration.

**Khuyến nghị**: Chỉ chấp nhận JWT qua `Authorization: Bearer` header. Loại bỏ hoàn toàn query param auth.

---

#### SEC-HIGH-02: Không có JWT Revocation (Token Blacklist)

**File**: [src/auth.rs:30-32](src/auth.rs#L30)

```rust
pub static DEFAULT_REFRESH_VALIDITY: LazyLock<TimeDelta> = LazyLock::new(|| TimeDelta::try_days(30).unwrap());
pub static MOBILE_REFRESH_VALIDITY: LazyLock<TimeDelta> = LazyLock::new(|| TimeDelta::try_days(90).unwrap());
pub static DEFAULT_ACCESS_VALIDITY: LazyLock<TimeDelta> = LazyLock::new(|| TimeDelta::try_hours(2).unwrap());
```

**Vấn đề**: JWT là stateless. Một khi token được phát, nó có hiệu lực cho đến hết `exp`. Cơ chế duy nhất để vô hiệu hóa token là thay đổi `security_stamp` của user (đổi password, v.v.), nhưng:
- Refresh token (90 ngày mobile) không bị vô hiệu trừ khi user đổi mật khẩu
- Không có "đăng xuất tất cả thiết bị" ngay lập tức đối với stolen tokens
- Không có token revocation list

**Rủi ro**: Stolen token có thể dùng tới 90 ngày sau khi bị đánh cắp.

**Khuyến nghị**: Triển khai database-backed token revocation list hoặc giảm thời hạn refresh token xuống đáng kể.

---

#### SEC-HIGH-03: SSRF qua Icon Proxy — DNS Rebinding

**File**: [src/http_client.rs:219-236](src/http_client.rs#L219), [src/api/icons.rs](src/api/icons.rs)

```rust
fn pre_resolve(name: &str) -> Result<(), CustomHttpClientError> {
    if should_block_address(name) { ... }
}
fn post_resolve(name: &str, ip: IpAddr) -> Result<(), CustomHttpClientError> {
    if should_block_ip(ip) { ... }
}
```

**Vấn đề**:
1. `http_request_block_non_global_ips` mặc định có thể bị tắt
2. **DNS Rebinding**: Có khoảng thời gian giữa `pre_resolve` và kết nối thực tế. Kẻ tấn công kiểm soát DNS server có thể trả về IP công khai trong lần resolve đầu, sau đó switch sang `127.0.0.1` khi kết nối thực sự xảy ra
3. Regex block (`http_request_block_regex`) là optional và không có giá trị mặc định

**Rủi ro**: Attacker có thể yêu cầu server fetch tài nguyên nội bộ (metadata service cloud, database admin interfaces).

**Khuyến nghị**: Enforce `http_request_block_non_global_ips=true` theo mặc định; sử dụng TCP-level IP binding verification sau DNS resolution.

---

#### SEC-HIGH-04: Rate Limiting chỉ theo IP — Dễ bypass

**File**: [src/ratelimit.rs](src/ratelimit.rs)

**Vấn đề**:
- Rate limiting chỉ theo IP address, không theo username/account
- Attacker dùng distributed botnet (nhiều IP) hoàn toàn bypass
- Người dùng sau NAT (nhiều user dùng chung IP) bị ảnh hưởng bởi một người vi phạm
- Không có account lockout: tấn công credential stuffing từ nhiều IP không bị phát hiện
- Header trust (`X-Forwarded-For`, `X-Real-IP`, `X-Client-IP`) có thể bị giả mạo nếu reverse proxy không được cấu hình đúng

**Rủi ro**: Credential stuffing attacks không bị giới hạn hiệu quả.

**Khuyến nghị**: Thêm per-account rate limiting; log cảnh báo khi cùng một username bị thử đăng nhập từ nhiều IP.

---

### SEV-3: TRUNG BÌNH

---

#### SEC-MED-01: Password Hint Lưu Plaintext trong Database

**File**: [src/db/models/user.rs:42](src/db/models/user.rs#L42)

```rust
pub password_hint: Option<String>,
```

**Vấn đề**: Password hint được lưu plaintext trong database. Nếu database bị leak (SQL injection, backup exposure, direct access), attacker có thể dùng hint để đoán master password của từng user.

**Rủi ro**: Trên quy mô lớn, hint có thể tiết lộ pattern mật khẩu.

**Khuyến nghị**: Mã hóa hint với một key riêng hoặc chỉ hiển thị hint sau khi xác minh email/2FA.

---

#### SEC-MED-02: SSO Auto-Provisioning Bypass SIGNUPS_ALLOWED

**File**: [src/api/identity.rs](src/api/identity.rs), [src/sso.rs](src/sso.rs)

**Vấn đề**: Khi `SSO_ENABLED=true`, bất kỳ user nào trong Identity Provider đều có thể tự động được tạo tài khoản trong Vaultwarden, ngay cả khi `SIGNUPS_ALLOWED=false`. Nếu tổ chức dùng IdP chung với nhiều service, tất cả nhân viên đều có thể truy cập Vaultwarden mà không cần được mời riêng.

**Rủi ro**: Unauthorized user provisioning.

**Khuyến nghị**: Cho phép whitelist groups/claims từ IdP để kiểm soát ai được auto-provision.

---

#### SEC-MED-03: Emergency Access — Grantor có thể không nhận Email

**File**: [src/api/core/emergency_access.rs](src/api/core/emergency_access.rs), [src/db/models/emergency_access.rs](src/db/models/emergency_access.rs)

**Vấn đề**: Emergency access tự động được phê duyệt bởi background job sau wait time mà không yêu cầu bất kỳ hành động nào từ grantor. Nếu:
- Email bị vào spam
- SMTP bị lỗi tạm thời
- Grantor không kiểm tra email trong thời gian wait

Vault của grantor có thể bị grantee access mà grantor không hay biết.

**Rủi ro**: Unauthorized vault access trong trường hợp email delivery failure.

**Khuyến nghị**: Yêu cầu grantor xác nhận qua in-app notification (WebSocket) thay vì chỉ email. Hoặc gửi nhắc nhở nhiều lần và trên nhiều kênh.

---

#### SEC-MED-04: Config.json có thể Chứa Secrets

**File**: [src/config.rs:20-22](src/config.rs#L20)

```rust
static CONFIG_FILE: LazyLock<String> = LazyLock::new(|| {
    let data_folder = get_env("DATA_FOLDER").unwrap_or_else(|| String::from("data"));
    get_env("CONFIG_FILE").unwrap_or_else(|| format!("{data_folder}/config.json"))
});
```

**Vấn đề**: `config.json` trong `data/` có thể chứa:
- `SMTP_PASSWORD`
- `SSO_CLIENT_SECRET`
- `PUSH_RELAY_URI` với credentials
- `ADMIN_TOKEN`

Nếu data directory bị expose qua misconfigured web server, file này lộ toàn bộ credentials.

**Rủi ro**: Credential exposure qua misconfigured file serving.

**Khuyến nghị**: Không lưu secrets nhạy cảm vào `config.json` — chỉ lưu non-secret settings. Secrets nên đến từ environment variables hoặc secret manager.

---

#### SEC-MED-05: Push Relay Leak Metadata

**File**: [src/api/push.rs](src/api/push.rs)

**Vấn đề**: Push notifications được relay qua external server (`PUSH_RELAY_URI`). Server relay biết được:
- Thời điểm user sync vault
- Số lượng event sync
- Device UUID của từng user

Đây là metadata leak về pattern sử dụng, có thể được khai thác để xác định khi nào user đang online và thiết bị nào họ dùng.

**Rủi ro**: Privacy leak về usage patterns tới bên thứ ba.

**Khuyến nghị**: Document rõ ràng về metadata được relay; cung cấp option self-hosted relay.

---

### SEV-4: THẤP / THIẾT KẾ

---

#### SEC-LOW-01: RSA-2048 cho JWT — Không Future-Proof

**File**: [src/auth.rs:74](src/auth.rs#L74)

```rust
let rsa_key = Rsa::generate(2048)?;
```

**Vấn đề**: RSA-2048 hiện tại vẫn an toàn nhưng:
- Chậm hơn ECDSA đáng kể (mỗi JWT sign/verify)
- NIST khuyến nghị chuyển sang post-quantum cryptography trước 2035
- Không có cơ chế key rotation — key được tạo một lần và dùng mãi mãi

**Khuyến nghị**: Hỗ trợ ES256 (ECDSA P-256) như alternative; triển khai key rotation schedule.

---

#### SEC-LOW-02: WebSocket Anonymous Endpoint

**File**: [src/api/notifications.rs](src/api/notifications.rs)

**Vấn đề**: `/notifications/anonymous` là unauthenticated endpoint. Mặc dù dùng cho AuthRequest flow, endpoint này có thể bị abuse để:
- Gây resource exhaustion (unlimited WebSocket connections)
- Information gathering về server status

**Khuyến nghị**: Áp dụng rate limiting cho anonymous WebSocket connections.

---

#### SEC-LOW-03: KDF Iterations Không Được Enforce Server-Side

**File**: [src/db/models/user.rs:128](src/db/models/user.rs#L128)

```rust
password_iterations: CONFIG.password_iterations(),
```

**Vấn đề**: Server chấp nhận và lưu `client_kdf_iter` do client gửi lên. Client có thể gửi `iterations=1` để làm yếu KDF cho account của mình. Server không enforce giá trị tối thiểu từ client-side KDF config.

**Khuyến nghị**: Validate `client_kdf_iter` đạt ngưỡng tối thiểu an toàn (ví dụ: PBKDF2 ≥ 600,000, Argon2id memory ≥ 64MB).

---

#### SEC-LOW-04: SQLite Backup File Exposure Risk

**File**: [src/db/mod.rs](src/db/mod.rs)

**Vấn đề**: SQLite backup được tạo trong data folder và có thể được trigger qua:
- SIGUSR1 signal
- Admin panel button
- Cron schedule

Nếu backup file (`*.sqlite3.bak`) vô tình nằm trong web-accessible directory, toàn bộ database (bao gồm encrypted vault data, password hints, email addresses) bị expose.

**Khuyến nghị**: Backup vào separate directory ngoài web root; notify operator về backup location.

---

#### SEC-LOW-05: Content Security Policy Không Được Enforce

**Vấn đề**: Không có CSP headers được set tại application layer. Toàn bộ trách nhiệm này được delegate cho reverse proxy operator. Một misconfigured proxy sẽ expose web vault đến XSS attacks.

**Khuyến nghị**: Set CSP headers mặc định tại application layer (có thể override bởi operator).

---

## Ma trận Rủi ro

| ID | Mô tả | Mức độ nghiêm trọng | Khả năng khai thác | Ưu tiên xử lý |
|----|-------|--------------------|--------------------|---------------|
| SEC-CRIT-01 | Admin token plaintext fallback | Nghiêm trọng | Cao | P1 |
| SEC-CRIT-02 | DISABLE_ADMIN_TOKEN không có safeguard | Nghiêm trọng | Trung bình | P1 |
| SEC-HIGH-01 | JWT trong URL query parameter | Cao | Cao | P2 |
| SEC-HIGH-02 | Không có token revocation | Cao | Trung bình | P2 |
| SEC-HIGH-03 | SSRF / DNS Rebinding qua icon proxy | Cao | Trung bình | P2 |
| SEC-HIGH-04 | Rate limiting chỉ theo IP | Cao | Cao | P2 |
| SEC-MED-01 | Password hint plaintext | Trung bình | Thấp | P3 |
| SEC-MED-02 | SSO bypass SIGNUPS_ALLOWED | Trung bình | Trung bình | P3 |
| SEC-MED-03 | Emergency access email failure | Trung bình | Thấp | P3 |
| SEC-MED-04 | config.json chứa secrets | Trung bình | Trung bình | P3 |
| SEC-MED-05 | Push relay metadata leak | Trung bình | Cao | P3 |
| SEC-LOW-01 | RSA-2048 không future-proof | Thấp | Rất thấp | P4 |
| SEC-LOW-02 | Anonymous WebSocket endpoint | Thấp | Thấp | P4 |
| SEC-LOW-03 | KDF iterations không enforce | Thấp | Thấp | P4 |
| SEC-LOW-04 | SQLite backup exposure | Thấp | Thấp | P4 |
| SEC-LOW-05 | CSP không được enforce | Thấp | Trung bình | P4 |

---

## Kiến trúc Bảo Mật Tổng Thể

### Điểm mạnh cốt lõi không thay đổi
- Mô hình E2E encryption là đúng đắn: server thực sự không thấy plaintext
- `#![forbid(unsafe_code)]` loại trừ một lớp lớn lỗ hổng memory safety
- Sử dụng thư viện crypto uy tín (`ring`, `argon2`, `webauthn-rs`)
- Security stamp mechanism cho phép invalidate sessions sau account changes

### Giới hạn kiến trúc
- **Single-instance design**: Không có multi-instance/cluster support → single point of failure
- **No audit log for failed auth**: Failed login attempts không được log theo cách có thể alert (chỉ error log)
- **Trust model for reverse proxy**: Hoàn toàn tin tưởng header IP từ reverse proxy mà không có validation
- **No secret management integration**: Không có native support cho Vault/AWS Secrets Manager/Kubernetes Secrets

---

*End of Document*
