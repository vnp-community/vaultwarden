# CR-002: System-Wide Tamper-Evident Audit Log & SIEM Integration

> **Change Request ID**: CR-002  
> **Title**: System-Wide Tamper-Evident Audit Log & SIEM Integration  
> **Priority**: P1 — Critical  
> **Target Release**: v2.0  
> **Driven By**: [specs/crs/product-market-analysis.md §2.1 Audit Log, §2.6 Monitoring]  
> **Affects**: PRD §6.10, URD §4.7, SRS §4.13

---

## 1. Problem Statement

Event logging hiện tại chỉ áp dụng ở **organization level** và không đủ cho banking compliance:
- Không log: failed login attempts, admin panel access, configuration changes, file downloads
- Logs có thể bị admin xóa (không tamper-evident)
- Không có SIEM integration (Splunk, IBM QRadar, Microsoft Sentinel)
- Không có log retention policy được enforce
- Log format là text, không phải structured JSON

---

## 2. Scope of Change

### 2.1 System-Wide Audit Log

**Thay đổi kiến trúc**: Tách biệt audit log khỏi application database thông thường.

```
┌─────────────────────────────────────────────────────┐
│                  Audit Log Architecture              │
│                                                     │
│  Application Events                                 │
│       ↓                                             │
│  AuditEventEmitter (async channel)                  │
│       ↓                                             │
│  ┌─────────────┐    ┌──────────────┐               │
│  │ DB Audit    │    │ SIEM Forward │               │
│  │ Table       │    │ (Syslog/HTTP)│               │
│  │ (append-    │    │              │               │
│  │  only)      │    └──────────────┘               │
│  └─────────────┘                                   │
│       ↓                                             │
│  Hash Chain (each entry includes hash of previous) │
└─────────────────────────────────────────────────────┘
```

### 2.2 Events Logged (Extended)

**Hiện tại** (chỉ org-level): cipher CRUD, member changes, policy changes

**Mới — System Level**:

| Category | Events |
|----------|--------|
| **Authentication** | Login success, Login failure (wrong password), Login failure (wrong 2FA), Account lockout, Token refresh, Logout, Passwordless auth attempt |
| **Admin Panel** | Admin login success, Admin login failure, Config change (field name, old value masked, new value masked), User management actions, Backup triggered |
| **Session** | Session created, Session expired, Session revoked, Concurrent session limit exceeded |
| **File Operations** | Attachment uploaded, Attachment downloaded, Send created, Send accessed, Send deleted |
| **Security Events** | Rate limit triggered, Suspicious IP detected, Emergency access requested, Emergency access granted |
| **System** | Server start, Server stop, Migration applied, Key rotation |

### 2.3 Tamper-Evident Mechanism

Mỗi audit log entry chứa:
```rust
AuditEntry {
    id: u64,                          // Monotonic, không thể skip
    timestamp: DateTime<Utc>,
    event_type: AuditEventType,
    actor_user_uuid: Option<UserId>,
    target_resource: Option<String>,
    ip_address: IpAddr,
    user_agent: String,
    org_uuid: Option<OrganizationId>,
    metadata: serde_json::Value,      // Event-specific data
    prev_hash: [u8; 32],              // SHA-256 của entry trước
    entry_hash: [u8; 32],             // SHA-256 của toàn bộ entry này
}
```

- **Hash chain**: mỗi entry hash bao gồm hash của entry trước → không thể xóa/sửa mà không phá vỡ chain
- **Append-only table**: không có DELETE quyền trên audit table; chỉ INSERT
- **Separate audit DB**: có thể cấu hình audit log vào DB/schema riêng

### 2.4 SIEM Integration

```
NEW CONFIG:
AUDIT_SIEM_ENABLED=true
AUDIT_SIEM_ENDPOINT=https://splunk.example.com:8088/services/collector
AUDIT_SIEM_TOKEN=<HEC token>
AUDIT_SIEM_FORMAT=splunk_hec|syslog_rfc5424|json_lines|microsoft_sentinel
AUDIT_SIEM_RETRY_COUNT=3
AUDIT_SIEM_BATCH_SIZE=100
AUDIT_SIEM_FLUSH_INTERVAL_MS=5000
```

**Supported formats**:
- Splunk HEC (HTTP Event Collector)
- Syslog RFC 5424 (TCP/TLS)
- JSON Lines (generic HTTP POST)
- Microsoft Sentinel (Azure Monitor / Data Collection API)
- IBM QRadar (Syslog)

### 2.5 Log Retention Policy

```
NEW CONFIG:
AUDIT_RETENTION_DAYS=2555          # 7 years (banking default)
AUDIT_RETENTION_ENFORCE=true       # Cannot be reduced below minimum
AUDIT_RETENTION_MINIMUM_DAYS=365   # Admin cannot set below this
```

### 2.6 Audit Log API

```
GET  /api/audit/events?from=&to=&type=&user=&org=&page=&limit=
GET  /api/audit/events/{id}
GET  /api/audit/verify-chain?from=&to=   # Verify hash chain integrity
GET  /api/audit/export?format=csv|json&from=&to=
```

---

## 3. Acceptance Criteria

- [ ] Failed login attempts logged with IP, timestamp, username (not password)
- [ ] Admin config changes logged with masked old/new values
- [ ] Hash chain passes `verify-chain` after 10,000 entries with no tampering
- [ ] Deleting an audit entry breaks subsequent hash verifications
- [ ] SIEM forward delivers events to Splunk HEC within 10 seconds
- [ ] Log retention enforced — entries older than retention period auto-archived, not deleted
- [ ] System-wide audit log captures events even when org event logging is disabled

---

## 4. Security Considerations

- Audit log tables require separate DB credentials with INSERT-only permission
- SIEM tokens must be masked in all log output (`***`)
- Audit log export requires admin-level authentication + protected action re-verification
- Hash chain verification endpoint is read-only; no mutation possible

---

## 5. Estimated Effort

| Area | Effort |
|------|--------|
| Event emitter refactor | 2 sprints |
| Hash chain implementation | 1 sprint |
| Extended event types | 2 sprints |
| SIEM integration (Splunk + Syslog) | 2 sprints |
| Additional SIEM formats | 1 sprint |
| Retention policy engine | 1 sprint |
| Audit export API | 1 sprint |

---

*Status: Draft | Author: Product Team | Date: 2026-04-12*
