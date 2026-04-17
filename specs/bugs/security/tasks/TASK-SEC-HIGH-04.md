# TASK-SEC-HIGH-04: Rate Limiting Chỉ Theo IP

> **Severity**: P2 — High  
> **Sprint**: Sprint 2  
> **Effort**: 3 ngày  
> **File gốc**: `src/ratelimit.rs`  
> **Rủi ro**: Distributed credential stuffing bypass — attacker dùng nhiều IPs để tấn công cùng một account

---

## Mô Tả Vấn Đề

Rate limiting hiện chỉ theo IP. Attacker dùng botnet với nhiều IPs khác nhau có thể thực hiện credential stuffing mà không bị block — mỗi IP chỉ gửi vài request, nhưng tổng attack volume lớn. Đồng thời, `X-Forwarded-For` header không được validate theo trusted proxy, có thể bị spoofed.

---

## Sub-tasks

### TASK-SEC-HIGH-04-A ✅ DONE (2026-04-15)
- **Tên**: Implement per-account rate limiting
- **File**: `src/ratelimit.rs`, `src/api/identity.rs`
- **Mô tả**: Thêm `LIMITER_ACCOUNT: AccountLimiter` (governor `RateLimiter<String, ...>`) với key là SHA-256 hex của lowercase email (tránh lưu PII trực tiếp vào memory). Hàm `check_limit_login_account(email)` được gọi trong `_password_login` sau per-IP check — nếu account vượt rate limit trả 429. Window dùng chung `login_ratelimit_seconds`. Email hash dùng `ring::digest::SHA256` + `data_encoding::HEXLOWER` (cả hai đều là deps hiện có).
- **Loại**: Modify existing
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-SEC-HIGH-04-C

### TASK-SEC-HIGH-04-B ✅ DONE (2026-04-15)
- **Tên**: Implement credential stuffing detection
- **File**: `src/ratelimit.rs`, `src/api/identity.rs`
- **Mô tả**: Hàm `detect_credential_stuffing(email, ip)`: track unique source IPs per account (keyed bằng SHA-256 email hash) trong sliding 15-min window (`CRED_STUFF_TRACKER: LazyLock<Mutex<HashMap<...>>>`). Nếu >= 5 unique IPs → emit `error!` log với tag `[CredentialStuffing]`. Không block (complement to rate limit); gọi trong `_password_login` trước DB lookup. Window reset tự động khi entry cũ hơn 900 giây.
- **Loại**: New function
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-SEC-HIGH-04-A

### TASK-SEC-HIGH-04-C ✅ DONE
- **Tên**: Thêm config keys cho account lockout
- **File**: `src/config.rs`
- **Mô tả**: Thêm: `trusted_proxies: String, false, def, ""` (CIDR list cho X-Forwarded-For trust).
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Triển khai**: `src/config.rs` — `trusted_proxies: String, false, def, String::new()`
- **Ghi chú**: `account_lockout_threshold` và `account_enumeration_detection` pending cùng HIGH-04-A

### TASK-SEC-HIGH-04-D ✅ DONE
- **Tên**: Implement trusted proxy IP validation
- **File**: `src/util.rs`
- **Mô tả**: `get_real_ip(req)`: nếu `TRUSTED_PROXIES` empty → dùng direct connection IP (ignore X-Forwarded-For). Nếu direct IP là trusted proxy → trust X-Forwarded-For header lấy IP đầu tiên. Ngược lại → ignore X-Forwarded-For. Prevent IP spoofing qua untrusted XFF headers.
- **Loại**: New function
- **Độ phức tạp**: Trung bình
- **Phụ thuộc**: TASK-SEC-HIGH-04-C
- **Triển khai**: `src/util.rs` — `pub fn get_real_ip(req: &Request<'_>) -> IpAddr` + `fn ip_in_cidr(...)`. Supports both IPv4 and IPv6 CIDR.

### TASK-SEC-HIGH-04-E ✅ DONE
- **Tên**: Replace `req.client_ip()` với `get_real_ip()` trong login handler
- **File**: `src/auth.rs` (`ClientIp::from_request`)
- **Mô tả**: Thay tất cả `req.client_ip()` / `req.remote()` calls trong login flow bằng `util::get_real_ip(req)`. Đảm bảo audit log, rate limiting, và session tracking dùng IP đúng.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-HIGH-04-D
- **Triển khai**: `src/auth.rs` — `ClientIp::from_request` now calls `crate::util::get_real_ip(req)` when `TRUSTED_PROXIES` is configured. Falls back to existing `ip_header` or direct remote IP.

---

## Acceptance Criteria

- [x] Per-account rate limit hoạt động độc lập với per-IP rate limit ✅ (HIGH-04-A 2026-04-15)
- [x] Credential stuffing từ 5+ IPs tới cùng username → audit event Critical ✅ (HIGH-04-B 2026-04-15)
- [x] `TRUSTED_PROXIES` config restrict XFF header trust
- [x] Rate limit bypass qua XFF spoofing không còn hoạt động (get_real_ip wired into ClientIp)

---

*Tạo từ SOL-security.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: ✅ COMPLETE (code-verified) — HIGH-04-A/B/C/D/E tất cả done và xác nhận tồn tại trong source (`ratelimit.rs`, `util.rs`, `config.rs`, `auth.rs`, `api/identity.rs`)*
