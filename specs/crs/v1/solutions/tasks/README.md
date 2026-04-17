# Tasks — CRS v1 Solutions

> Mỗi file task tương ứng với một giải pháp trong `../`. Mỗi task có: mô tả, file cần tạo/sửa, loại thay đổi, độ phức tạp, và phụ thuộc.

---

## Danh Sách Tasks

| File | CR | Giải pháp | Tasks | Sprints | Độ phức tạp tổng |
|------|----|-----------|-------|---------|-----------------|
| [TASKS-SOL-001.md](TASKS-SOL-001.md) | CR-001 | Enterprise Compliance Framework | 16 | 5 | Thấp–Trung bình |
| [TASKS-SOL-002.md](TASKS-SOL-002.md) | CR-002 | Audit Log & SIEM Integration | 18 | 6 | Trung bình–Cao |
| [TASKS-SOL-003.md](TASKS-SOL-003.md) | CR-003 | AD/LDAP & SCIM 2.0 | 20 | 10 | Trung bình–Cao |
| [TASKS-SOL-004.md](TASKS-SOL-004.md) | CR-004 | Granular RBAC & Access Control | 22 | 11 | Trung bình–Cao |
| [TASKS-SOL-005.md](TASKS-SOL-005.md) | CR-005 | High Availability & Scaling | 17 | 4 phases | Trung bình–Cao |
| [TASKS-SOL-006.md](TASKS-SOL-006.md) | CR-006 | Disaster Recovery | 15 | 9 | Trung bình–Cao |
| [TASKS-SOL-007.md](TASKS-SOL-007.md) | CR-007 | Privileged Access Management | 17 | 10 | Cao |
| [TASKS-SOL-008.md](TASKS-SOL-008.md) | CR-008 | Enterprise API Management | 18 | 9 | Trung bình–Cao |
| [TASKS-SOL-009.md](TASKS-SOL-009.md) | CR-009 | MDM & Certificate Auth | 15 | 8 | Trung bình–Cao |
| [TASKS-SOL-010.md](TASKS-SOL-010.md) | CR-010 | Observability & Alerting | 17 | 8 | Thấp–Trung bình |
| [TASKS-SOL-011.md](TASKS-SOL-011.md) | CR-011 | Multi-Tenancy & Isolation | 21 | 10 | Rất Cao ⚠️ |

**Tổng**: 196 tasks

---

## Cấu Trúc Mỗi Task

```
### TASK-{SOL}-{NUM}
- **Tên**: Tên ngắn gọn
- **File**: File cần tạo hoặc sửa
- **Mô tả**: Chi tiết implementation
- **Loại**: New file | New code | Modify existing | New migration | Testing | Documentation
- **Độ phức tạp**: Thấp | Trung bình | Cao
- **Phụ thuộc**: TASK-xxx-yyy (task phải hoàn thành trước)
- **Dependency mới**: Crate mới cần thêm vào Cargo.toml (nếu có)
```

---

## Thứ Tự Triển Khai Đề Xuất

### Giai Đoạn 1 — Foundation (không có phụ thuộc chéo)
1. **SOL-010** (Observability) — additive, không thay đổi core, nên làm trước để có metrics
2. **SOL-001** (Compliance) — security headers, GDPR pipeline
3. **SOL-002** (Audit Log) — hash chain infrastructure, các SOL khác depend vào đây

### Giai Đoạn 2 — Core Features (depend vào SOL-002)
4. **SOL-005** (HA) — Redis layer, health endpoints
5. **SOL-006** (DR) — backup pipeline
6. **SOL-004** (RBAC) — permission model

### Giai Đoạn 3 — Identity & Integration (depend vào SOL-004)
7. **SOL-003** (LDAP/SCIM) — user provisioning
8. **SOL-009** (MDM) — device trust
9. **SOL-008** (API) — enterprise API keys

### Giai Đoạn 4 — Advanced Features (depend vào nhiều SOL khác)
10. **SOL-007** (PAM) — depend vào SOL-002, SOL-004
11. **SOL-011** (Multi-Tenancy) — thay đổi lớn nhất, làm sau cùng

---

## Dependencies Mới (Cargo.toml)

| Crate | Version | SOL | Feature Flag |
|-------|---------|-----|--------------|
| `sha2` | 0.10 | SOL-002 | default |
| `csv` | 1.x | SOL-001 | default |
| `ldap3` | 0.11 | SOL-003 | default |
| `chrono-tz` | 0.x | SOL-004 | default |
| `deadpool-redis` | 0.18 | SOL-005 | `redis` |
| `redis` | 0.27 | SOL-005 | `redis` |
| `prometheus-client` | 0.22 | SOL-010 | default |
| `tracing` | 0.1 | SOL-010 | default |
| `tracing-subscriber` | 0.3 | SOL-010 | default |
| `opentelemetry` | 0.27 | SOL-010 | `otel` |
| `opentelemetry-otlp` | 0.27 | SOL-010 | `otel` |
| `tracing-opentelemetry` | 0.28 | SOL-010 | `otel` |

---

*Tạo: 2026-04-13 | Trạng thái: Draft*
