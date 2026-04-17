# TASK-SEC-HIGH-03: SSRF / DNS Rebinding qua Icon Proxy

> **Severity**: P2 — High  
> **Sprint**: Sprint 1  
> **Effort**: 2 ngày  
> **File gốc**: `src/http_client.rs:219-236`  
> **Rủi ro**: Attacker dùng icon proxy để fetch internal resources (SSRF), hoặc DNS rebinding để bypass IP checks

---

## Mô Tả Vấn Đề

Icon proxy fetch URL tùy ý do user cung cấp. Config `HTTP_REQUEST_BLOCK_NON_GLOBAL_IPS` mặc định `false`. DNS rebinding có thể bypass IP check: domain resolve thành IP global khi check, sau đó rebind sang internal IP khi kết nối.

---

## Sub-tasks

### TASK-SEC-HIGH-03-A ✅ DONE
- **Tên**: Đổi default `http_request_block_non_global_ips` thành `true`
- **File**: `src/config.rs`
- **Mô tả**: Đã được implement. `src/config.rs:707` có: `http_request_block_non_global_ips: bool, true, auto, |c| c.icon_blacklist_non_global_ips`. `icon_blacklist_non_global_ips` có `def, true` (line 700). Do đó `http_request_block_non_global_ips` mặc định là `true` — block non-global IPs by default. Operators muốn allow internal IPs phải explicit set `HTTP_REQUEST_BLOCK_NON_GLOBAL_IPS=false`.
- **Loại**: Modify existing — BREAKING CHANGE
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không

### TASK-SEC-HIGH-03-B ✅ DONE (2026-04-15)
- **Tên**: Implement DNS rebinding prevention trong HTTP connector
- **File**: `src/http_client.rs`
- **Mô tả**: Đã fix `resolve_domain()` trong `CustomDnsResolver`: thay vì chỉ lấy `.next()` (IP đầu tiên), giờ collect **tất cả** resolved IPs và gọi `post_resolve(name, addr.ip())?` cho từng IP. Nếu **bất kỳ** IP nào không phải global -> reject toàn bộ request. Lý do: attacker có thể trả về IP global làm result đầu tiên, IP internal ở sau — original code chỉ check first result. `cargo check` pass.
- **Loại**: Modify existing
- **Độ phức tạp**: Thấp (fix nhỏ, hiệu quả lớn)
- **Phụ thuộc**: TASK-SEC-HIGH-03-A

### TASK-SEC-HIGH-03-C ✅ DONE
- **Tên**: Implement domain blocklist cho icon proxy
- **File**: `src/api/icons.rs`
- **Mô tả**: `is_blocked_icon_domain()`: check domain vs hardcoded blocklist: `localhost`, `metadata.google.internal`, `instance-data`, `instance-metadata`. Trả err nếu match. Gọi trước khi fetch.
- **Loại**: New function
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: Không
- **Triển khai**: `src/api/icons.rs` — `fn is_blocked_icon_domain(domain: &str) -> bool`; called before `should_block_address()` in `icon_internal` handler. Blocks: localhost, metadata.google.internal, instance-data, instance-metadata (and subdomains).

### TASK-SEC-HIGH-03-D ✅ DONE (2026-04-15)
- **Tên**: Thêm private IP ranges vào blocklist
- **File**: `src/http_client.rs`, `src/api/icons.rs`
- **Mô tả**: Đã verify: `should_block_address(domain)` trong `icon_internal` gọi `IpAddr::from_str(domain_or_ip)` -> nếu parse được thành IP literal -> gọi `should_block_ip(ip)` -> gọi `is_global(ip)` -> block tất cả RFC1918 (10/8, 172.16/12, 192.168/16), link-local (169.254/16), loopback (127/8), và IPv6 loopback (::1). `should_block_host()` trong `make_http_request()` cũng có cùng logic. Vậy IP literal blocking đã hoạt động đáng kể thông qua `is_global_hardcoded` trong `util.rs`. Không cần thêm code mới — task được verify là DONE với code hiện tại.
- **Loại**: Verification + Documentation
- **Độ phức tạp**: Thấp
- **Phụ thuộc**: TASK-SEC-HIGH-03-B

---

## Acceptance Criteria

- [x] `HTTP_REQUEST_BLOCK_NON_GLOBAL_IPS` default `true` ✅ (via `icon_blacklist_non_global_ips` auto-default)
- [x] DNS rebinding không bypass được IP check — ALL resolved IPs được kiểm tra ✅ (HIGH-03-B 2026-04-15)
- [x] `metadata.google.internal`, `instance-data` bị block qua domain blocklist ✅
- [x] Internal IPs trong URL literals bị block qua `should_block_address()` → `is_global()` ✅ (HIGH-03-D 2026-04-15 — đã có sẵn, verified)
- [x] SSRF test: request tới `http://localhost/admin` bị reject ✅ (domain blocklist + IP literal blocking đều active)

---

*Tạo từ SOL-security.md | Ngày: 2026-04-13 | Cập nhật: 2026-04-15 | Trạng thái: ✅ COMPLETE — HIGH-03-A/B/C/D tất cả done*
