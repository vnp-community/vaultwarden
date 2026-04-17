# CR-001: Enterprise Compliance Framework

> **Change Request ID**: CR-001  
> **Title**: Enterprise Compliance Framework (PCI DSS, SOC 2, ISO 27001, GDPR/PDPA, MAS TRM, Basel III)  
> **Priority**: P1 — Critical  
> **Target Release**: v2.0  
> **Driven By**: [specs/crs/product-market-analysis.md §2.1]  
> **Affects**: PRD §9, URD §4.7, SRS §5.1

---

## 1. Problem Statement

Vaultwarden hiện tại không đáp ứng bất kỳ tiêu chuẩn tuân thủ nào bắt buộc với banking, tổ chức tài chính, và các doanh nghiệp được quản lý chặt. Điều này loại trừ hoàn toàn thị trường FSI (Financial Services Industry) và các doanh nghiệp quy mô >10,000 nhân sự.

---

## 2. Scope of Change

### 2.1 Compliance Targets

| Tiêu chuẩn | Yêu cầu chính | Thay đổi cần thiết |
|------------|--------------|-------------------|
| **PCI DSS v4.0** | Req 7 (Access control), Req 10 (Audit logs), Req 8 (Auth management) | Granular RBAC, tamper-evident logs, MFA enforcement |
| **SOC 2 Type II** | CC6 (Logical access), CC7 (System operations), CC9 (Risk mitigation) | Continuous access monitoring, anomaly detection, formal change management |
| **ISO 27001:2022** | A.5.15 (Access control), A.5.33 (Protection of records), A.8.16 (Monitoring activities) | Access review workflow, log retention policy, security monitoring |
| **GDPR / PDPA** | Art 17 (Right to erasure), Art 25 (Data by design), Art 32 (Security) | PII deletion pipeline, data residency controls, encryption-at-rest documentation |
| **MAS TRM (Singapore)** | §9 (Access control), §10 (Cryptography), §12 (Audit) | Key management procedures, audit trail completeness |
| **Basel III / BCBS 239** | Operational resilience, data integrity, RTO/RPO | HA clustering, automated backup verification |
| **Circular 09/2020/TT-NHNN (Vietnam)** | Information security for credit institutions | System-wide audit log, penetration test evidence, ISMS documentation |

### 2.2 New Capabilities Required

#### 2.2.1 Compliance Documentation Layer
- **Compliance Evidence API** (`GET /api/compliance/evidence`): xuất báo cáo bằng chứng tuân thủ theo tiêu chuẩn được chọn
- **Compliance Report Generator**: PDF/CSV report cho PCI DSS log review, SOC 2 access review, GDPR data inventory
- **Data Processing Register**: tự động tạo danh sách dữ liệu cá nhân được lưu trữ, mục đích, thời gian lưu giữ

#### 2.2.2 Data Residency Controls
```
NEW CONFIG:
DATA_RESIDENCY_REGION=VN|SG|US|EU     # Restrict which region data can reside
DATA_RESIDENCY_ENFORCE=true            # Reject S3 buckets in wrong regions
PII_ENCRYPTION_KEY_ID=<KMS key ARN>   # Separate key for PII fields
```

#### 2.2.3 Right to Erasure (GDPR Art 17)
- **Automated PII deletion pipeline**: khi user xóa tài khoản, tất cả PII (email, tên, IP logs) bị xóa sạch trong vòng 30 ngày
- **Erasure audit trail**: log đặc biệt ghi nhận khi nào dữ liệu bị xóa theo yêu cầu GDPR (không thể xóa log này)
- **Export my data**: user có thể export toàn bộ dữ liệu của mình (GDPR Art 20 — portability)

#### 2.2.4 Security Assessment Support
- **Penetration Test Mode**: read-only mode cho security assessors không làm gián đoạn production
- **Security Headers Enforcement**: HSTS, CSP, X-Frame-Options, X-Content-Type-Options tại application layer
- **Vulnerability Disclosure Contact**: `.well-known/security.txt` endpoint

---

## 3. Acceptance Criteria

- [ ] Compliance report generator produces valid PCI DSS Req 10 evidence report
- [ ] GDPR erasure pipeline completes within 30 days and produces audit receipt
- [ ] Data residency controls reject uploads to non-compliant regions when enforced
- [ ] Security headers present on all responses (CSP, HSTS, X-Frame-Options minimum)
- [ ] `GET /api/compliance/evidence?standard=pci_dss` returns structured evidence JSON
- [ ] All PII fields documented in automated data processing register

---

## 4. Dependencies

- CR-002 (Tamper-Evident Audit Log) — required for PCI DSS Req 10 and SOC 2 CC7
- CR-003 (SCIM / AD Integration) — required for SOC 2 CC6 access review
- CR-005 (High Availability) — required for Basel III operational resilience

---

## 5. Estimated Effort

| Area | Effort |
|------|--------|
| Compliance Evidence API | 3 sprints |
| GDPR erasure pipeline | 2 sprints |
| Data residency controls | 2 sprints |
| Security headers | 0.5 sprint |
| Compliance report generator | 3 sprints |

---

*Status: Draft | Author: Product Team | Date: 2026-04-12*
