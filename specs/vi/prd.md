# Vaultwarden — Tài Liệu Yêu Cầu Sản Phẩm (PRD)

> **Phiên bản tài liệu**: 1.0  
> **Ngày**: 2026-04-10  
> **Trạng thái**: Bản nháp  
> **Tác giả**: Nhóm Sản phẩm  
> **Tài liệu tham chiếu**:
> - Tài liệu Yêu cầu Người dùng: `specs/urd.md`
> - Đặc tả Yêu cầu Phần mềm: `specs/srs.md`
> - Tài liệu Thiết kế Kỹ thuật: `specs/technical-design.md`

---

## Mục Lục

1. [Tóm tắt điều hành](#1-tóm-tắt-điều-hành)
2. [Tầm nhìn & Chiến lược sản phẩm](#2-tầm-nhìn--chiến-lược-sản-phẩm)
3. [Người dùng mục tiêu & Thị trường](#3-người-dùng-mục-tiêu--thị-trường)
4. [Phát biểu vấn đề](#4-phát-biểu-vấn-đề)
5. [Mục tiêu sản phẩm & Chỉ số thành công](#5-mục-tiêu-sản-phẩm--chỉ-số-thành-công)
6. [Danh mục tính năng](#6-danh-mục-tính-năng)
   - 6.1 [Quản lý kho mật khẩu cốt lõi](#61-quản-lý-kho-mật-khẩu-cốt-lõi)
   - 6.2 [Xác thực & Bảo mật](#62-xác-thực--bảo-mật)
   - 6.3 [Xác thực hai yếu tố (2FA)](#63-xác-thực-hai-yếu-tố-2fa)
   - 6.4 [Quản lý tổ chức & Cộng tác nhóm](#64-quản-lý-tổ-chức--cộng-tác-nhóm)
   - 6.5 [Chia sẻ bảo mật — Bitwarden Send](#65-chia-sẻ-bảo-mật--bitwarden-send)
   - 6.6 [Truy cập khẩn cấp](#66-truy-cập-khẩn-cấp)
   - 6.7 [Đồng bộ thời gian thực & Thông báo](#67-đồng-bộ-thời-gian-thực--thông-báo)
   - 6.8 [Đăng nhập một lần (SSO / OIDC)](#68-đăng-nhập-một-lần-sso--oidc)
   - 6.9 [Bảng quản trị & Quản lý máy chủ](#69-bảng-quản-trị--quản-lý-máy-chủ)
   - 6.10 [Nhật ký kiểm toán & Sự kiện](#610-nhật-ký-kiểm-toán--sự-kiện)
   - 6.11 [Thông báo qua email](#611-thông-báo-qua-email)
   - 6.12 [Lưu trữ tệp & Tệp đính kèm](#612-lưu-trữ-tệp--tệp-đính-kèm)
7. [Ưu tiên tính năng (MoSCoW)](#7-ưu-tiên-tính-năng-moscow)
8. [Luồng người dùng](#8-luồng-người-dùng)
9. [Yêu cầu sản phẩm phi chức năng](#9-yêu-cầu-sản-phẩm-phi-chức-năng)
10. [Chiến lược phát hành & Cột mốc](#10-chiến-lược-phát-hành--cột-mốc)
11. [Rủi ro & Biện pháp giảm thiểu](#11-rủi-ro--biện-pháp-giảm-thiểu)
12. [Câu hỏi mở & Quyết định](#12-câu-hỏi-mở--quyết-định)
13. [Phụ lục: Ma trận truy xuất nguồn gốc](#13-phụ-lục-ma-trận-truy-xuất-nguồn-gốc)

---

## 1. Tóm Tắt Điều Hành

**Vaultwarden** là một máy chủ quản lý mật khẩu mã nguồn mở, tự lưu trữ, hoàn toàn tương thích với hệ sinh thái ứng dụng khách chính thức của Bitwarden. Nó cho phép các cá nhân, người đam mê homelab và các nhóm nhỏ đến vừa chạy máy chủ tương thích Bitwarden trên cơ sở hạ tầng của riêng mình — loại bỏ sự phụ thuộc vào dịch vụ đám mây chính thức của Bitwarden trong khi vẫn duy trì đầy đủ chức năng.

Được viết bằng Rust và cấp phép theo AGPL-3.0, Vaultwarden được thiết kế để:
- **Tiết kiệm tài nguyên**: chạy trên phần cứng yếu (Raspberry Pi, VPS với 256 MB RAM)
- **Tương thích hoàn toàn**: hoạt động với tất cả ứng dụng khách Bitwarden chính thức (web, desktop, di động, tiện ích mở rộng trình duyệt) mà không cần sửa đổi ứng dụng khách
- **Đầy đủ tính năng**: triển khai tất cả tính năng miễn phí và hầu hết tính năng premium của Bitwarden
- **Thân thiện với người vận hành**: triển khai Docker đơn giản, ít phụ thuộc, cấu hình qua biến môi trường

Tài liệu PRD này xác định phạm vi sản phẩm hoàn chỉnh, bộ tính năng, thứ tự ưu tiên, chỉ số thành công và kế hoạch phát hành cho Vaultwarden.

---

## 2. Tầm Nhìn & Chiến Lược Sản Phẩm

### 2.1 Tuyên bố Tầm nhìn

> **Trao quyền cho mọi người làm chủ mật khẩu của mình.**
>
> Vaultwarden cho phép mọi cá nhân và nhóm khả năng chạy trình quản lý mật khẩu bảo mật, đầy đủ tính năng trên cơ sở hạ tầng của riêng họ — không phí thuê bao, không bị khóa vào nhà cung cấp, và không đánh đổi quyền riêng tư.

### 2.2 Định vị chiến lược

| Tiêu chí | Vaultwarden | Bitwarden chính thức | 1Password / LastPass |
|----------|------------|--------------------|--------------------|
| **Lưu trữ** | Tự lưu trữ | Đám mây + Tự lưu trữ | Chỉ đám mây |
| **Chi phí** | Miễn phí (AGPL) | Gói miễn phí + trả phí | Cần thuê bao |
| **Kiểm soát dữ liệu** | Hoàn toàn (người vận hành sở hữu dữ liệu) | Bitwarden kiểm soát đám mây | Nhà cung cấp kiểm soát |
| **Sử dụng tài nguyên** | Rất thấp (~50 MB RAM) | Cao (Java stack) | N/A |
| **Tương thích ứng dụng khách** | Tất cả ứng dụng Bitwarden | Tất cả ứng dụng Bitwarden | Chỉ ứng dụng 1Password |
| **Đối tượng mục tiêu** | Người quan tâm quyền riêng tư, homelabbers, SMB | Cá nhân đến doanh nghiệp | Cá nhân đến doanh nghiệp |

### 2.3 Nguyên tắc thiết kế

1. **Quyền riêng tư theo kiến trúc** — Máy chủ tuyệt đối không được đọc dữ liệu kho mật khẩu của người dùng. Mã hóa đầu cuối là yêu cầu không thể thương lượng.
2. **Ưu tiên tương thích ứng dụng khách** — Không thay đổi hành vi ứng dụng khách. Vaultwarden phải hoạt động trong suốt với các ứng dụng khách Bitwarden chính thức.
3. **Đơn giản cho người vận hành** — Việc triển khai và cấu hình phải thực hiện được trong vòng 10 phút qua Docker.
4. **Bảo mật không thỏa hiệp** — Rust an toàn bộ nhớ, không có mã unsafe, Argon2id cho dữ liệu bí mật, giới hạn tốc độ cho tất cả các điểm cuối nhạy cảm.
5. **Ngang bằng tính năng** — Duy trì tương thích với tất cả tính năng Bitwarden dành cho tài khoản cá nhân miễn phí và premium.

---

## 3. Người Dùng Mục Tiêu & Thị Trường

### 3.1 Chân dung người dùng chính

#### Chân dung 1 — Alex, Cá nhân quan tâm quyền riêng tư
- **Thông tin**: Lập trình viên phần mềm, 28 tuổi, vận hành homelab cá nhân.
- **Nhu cầu**: Muốn có máy chủ tương thích Bitwarden mà họ kiểm soát hoàn toàn. Không tin tưởng lưu trữ đám mây của bên thứ ba cho mật khẩu.
- **Trình độ kỹ thuật**: Cao — thành thạo Docker và Linux.
- **Tính năng chính**: Quản lý kho mật khẩu, 2FA, kiểm soát tự lưu trữ.

#### Chân dung 2 — Maya, Quản trị viên CNTT SMB
- **Thông tin**: Quản trị CNTT tại công ty 20 người. Nhóm sử dụng mật khẩu chung cho các dịch vụ và cơ sở hạ tầng.
- **Nhu cầu**: Chia sẻ thông tin đăng nhập an toàn trong nhóm với kiểm soát truy cập dựa trên vai trò và nhật ký kiểm toán.
- **Trình độ kỹ thuật**: Trung bình-cao.
- **Tính năng chính**: Tổ chức, bộ sưu tập, nhóm, nhật ký kiểm toán, SSO, bảng quản trị.

#### Chân dung 3 — Jordan, Quản trị Homelab Gia đình
- **Thông tin**: Người đam mê công nghệ vận hành dịch vụ cho các thành viên gia đình (4–6 người dùng).
- **Nhu cầu**: Kho mật khẩu tự lưu trữ đơn giản mà gia đình có thể sử dụng trên điện thoại và laptop mà không bị cản trở.
- **Trình độ kỹ thuật**: Trung bình.
- **Tính năng chính**: Thiết lập ứng dụng khách dễ dàng, thông báo email, 2FA, truy cập khẩn cấp.

#### Chân dung 4 — Sam, Trưởng nhóm chú trọng bảo mật
- **Thông tin**: Dẫn đầu nhóm kỹ thuật 10 người. Yêu cầu bắt buộc 2FA và chính sách bảo mật tổ chức.
- **Nhu cầu**: Bắt buộc MFA trong nhóm, xem xét sự kiện truy cập, và tích hợp với SSO của công ty.
- **Trình độ kỹ thuật**: Cao.
- **Tính năng chính**: Chính sách tổ chức, nhật ký sự kiện, SSO, tích hợp Duo.

### 3.2 Cơ hội thị trường

- Nhu cầu ngày càng tăng về công cụ tự lưu trữ do quy định quyền riêng tư (GDPR, CCPA) và các vụ vi phạm đám mây nổi tiếng.
- Bitwarden tự lưu trữ chính thức yêu cầu tài nguyên đáng kể (Java + SQL Server hoặc PostgreSQL), khiến Vaultwarden là lựa chọn thay thế nhẹ duy nhất.
- Cộng đồng mã nguồn mở tích cực với hàng chục nghìn triển khai sản xuất.

---

## 4. Phát Biểu Vấn Đề

### 4.1 Các vấn đề cốt lõi được giải quyết

| Vấn đề | Điểm đau hiện tại | Giải pháp Vaultwarden |
|--------|------------------|----------------------|
| **Tin tưởng đám mây** | Phải tin tưởng dịch vụ bên thứ ba với toàn bộ mật khẩu | Tự lưu trữ hoàn toàn; người vận hành sở hữu dữ liệu |
| **Chi phí tự lưu trữ chính thức** | Máy chủ Bitwarden chính thức yêu cầu ~2 GB RAM, Java, SQL Server | Vaultwarden chạy trong ~50–100 MB RAM với SQLite |
| **Manh sát phí thuê bao** | Một số tính năng premium Bitwarden yêu cầu gói trả phí | Vaultwarden cung cấp các tính năng tương đương premium miễn phí |
| **Chia sẻ mật khẩu nhóm** | Không có cách chia sẻ thông tin đăng nhập an toàn, có kiểm toán mà không cần đăng ký dịch vụ | Tổ chức, bộ sưu tập, nhật ký sự kiện |
| **Chia sẻ không an toàn** | Nhóm dùng email/chat để chia sẻ thông tin đăng nhập một lần | Bitwarden Send cung cấp chia sẻ tạm thời được mã hóa |
| **Khôi phục tài khoản** | Quên mật khẩu chính = bị khóa vĩnh viễn | Ủy quyền truy cập khẩn cấp |

### 4.2 Giới hạn vấn đề

Vaultwarden **không** nhằm mục đích giải quyết:
- Quản lý bí mật cho máy móc/dịch vụ (đó là lĩnh vực của Bitwarden Secrets Manager).
- Quản lý thanh toán, cung cấp và đăng ký doanh nghiệp.
- Tùy chỉnh giao diện ứng dụng khách.

---

## 5. Mục Tiêu Sản Phẩm & Chỉ Số Thành Công

### 5.1 Mục tiêu sản phẩm

| Mã mục tiêu | Mục tiêu | Danh mục |
|------------|---------|---------|
| G-01 | Tương thích 100% với tất cả ứng dụng khách Bitwarden chính thức | Tương thích |
| G-02 | Có thể triển khai trong vòng 10 phút qua một lệnh Docker duy nhất | Trải nghiệm người vận hành |
| G-03 | Dữ liệu kho mật khẩu được chứng minh không thể truy cập bởi máy chủ (E2EE) | Bảo mật |
| G-04 | Hỗ trợ tổ chức lên đến 100 người dùng mà không giảm hiệu suất | Hiệu suất |
| G-05 | Không có lỗ hổng bảo mật nghiêm trọng trong các đường dẫn xác thực và mã hóa cốt lõi | Bảo mật |
| G-06 | Tất cả cấu hình qua biến môi trường; không cần thay đổi mã nguồn | Khả năng vận hành |

### 5.2 Chỉ số thành công chính (KPI)

| Chỉ số | Mục tiêu | Phương pháp đo lường |
|--------|---------|---------------------|
| **Tương thích ứng dụng khách** | 100% điểm cuối API ứng dụng khách Bitwarden được hỗ trợ | Bộ kiểm tra tích hợp ứng dụng khách |
| **Thời gian triển khai** | < 10 phút từ đầu đến máy chủ hoạt động | Đánh giá chuẩn người dùng mới |
| **Lượng bộ nhớ sử dụng** | < 150 MB RAM dưới tải thông thường (10–50 người dùng) | Kiểm tra tải với phân tích bộ nhớ |
| **Độ trễ đăng nhập** | < 300 ms p95 cho `/identity/connect/token` | Giám sát độ trễ API |
| **Độ trễ đồng bộ kho** | < 500 ms p95 cho phản hồi `/api/sync` đầy đủ | Giám sát độ trễ API |
| **Phân phối WebSocket** | < 2 giây cho việc truyền thay đổi đến các máy khách kết nối | Kiểm tra độ trễ đồng bộ E2E |
| **Thời gian hoạt động** | 99,9% tính khả dụng (triển khai một nút) | Giám sát kiểm tra sức khỏe |
| **Kích thước bản dựng** | < 20 MB cho nhị phân `release-micro` | Kiểm tra kích thước nhị phân CI |

---

## 6. Danh Mục Tính Năng

Mỗi tính năng được mô tả với: **Chức năng**, **Tầm quan trọng**, **Hành vi chính**, và **Chủ thể/Diễn viên**.

---

### 6.1 Quản Lý Kho Mật Khẩu Cốt Lõi

**Mã tính năng**: F-VAULT  
**Ưu tiên**: 🔴 Bắt buộc có  
**Chủ thể**: Người dùng cuối

#### Chức năng
Cung cấp các thao tác CRUD cốt lõi để quản lý các mục kho được mã hóa (gọi là *cipher*) qua năm loại: Đăng nhập, Ghi chú bảo mật, Thẻ tín dụng, Danh tính và Khóa SSH.

#### Tầm quan trọng
Đây là đề xuất giá trị chính của sản phẩm. Nếu không có kho mật khẩu hoạt động, tất cả các tính năng khác đều vô nghĩa.

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Loại mục** | Đăng nhập (1), Ghi chú bảo mật (2), Thẻ (3), Danh tính (4), Khóa SSH (5) |
| **CRUD** | Tạo, Đọc, Cập nhật, Xóa (xóa mềm vào thùng rác, xóa vĩnh viễn theo lịch) |
| **Thao tác hàng loạt** | Di chuyển, xóa, chia sẻ nhiều mục cùng lúc |
| **Đồng bộ kho** | `/api/sync` trả về tất cả cipher, thư mục, bộ sưu tập và cài đặt |
| **Lịch sử mật khẩu** | Lịch sử mật khẩu mỗi mục được lưu trữ mã hóa |
| **Nhắc lại** | Cờ "yêu cầu mật khẩu chính để xem" mỗi mục |
| **Thư mục** | Nhóm cá nhân, riêng tư của các mục |
| **Mục yêu thích** | Cờ yêu thích riêng cho từng người dùng trên mỗi mục |
| **Mã hóa** | Tất cả dữ liệu mục được mã hóa phía máy khách trước khi rời thiết bị |

#### Tiêu chí chấp nhận
- [ ] Tất cả 5 loại mục có thể được tạo, chỉnh sửa và xóa từ bất kỳ ứng dụng khách Bitwarden nào.
- [ ] Các mục đã xóa xuất hiện trong thùng rác và được xóa sạch sau lịch trình đã cấu hình.
- [ ] Đồng bộ kho trả về trạng thái nhất quán trên tất cả các thiết bị kết nối.
- [ ] Máy chủ không lưu trữ dữ liệu kho dạng văn bản thuần túy (có thể xác minh qua kiểm tra cơ sở dữ liệu).

---

### 6.2 Xác Thực & Bảo Mật

**Mã tính năng**: F-AUTH  
**Ưu tiên**: 🔴 Bắt buộc có  
**Chủ thể**: Người dùng cuối, Quản trị viên máy chủ

#### Chức năng
Quản lý toàn bộ vòng đời xác thực: đăng ký, đăng nhập, cấp phát và làm mới token, đăng ký thiết bị, giới hạn tốc độ và xác thực lại cho các hành động được bảo vệ.

#### Tầm quan trọng
Xác thực là cổng vào kho mật khẩu. Bất kỳ điểm yếu nào ở đây đều trực tiếp ảnh hưởng đến bảo mật dữ liệu người dùng.

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Ký token** | RS256 với cặp khóa RSA 2048-bit tự động tạo |
| **TTL token truy cập** | 2 giờ |
| **TTL token làm mới** | 30 ngày (desktop/web), 90 ngày (di động) |
| **Giới hạn tốc độ** | Theo IP, trên các điểm cuối đăng nhập, 2FA và đăng ký |
| **Hành động được bảo vệ** | Yêu cầu xác thực lại (mật khẩu chính hoặc OTP email) cho: tắt 2FA, xuất kho, thay đổi khóa |
| **Đăng nhập không mật khẩu** | Luồng phê duyệt yêu cầu từ thiết bị đến thiết bị (`AuthRequest`) |
| **Đăng ký thiết bị** | Mỗi thiết bị máy khách được đăng ký với UUID và token push |
| **Chính sách đăng ký** | Mở / chỉ theo lời mời / giới hạn tên miền / xác minh email |
| **Dấu bảo mật** | Thay đổi khi cập nhật tài khoản nhạy cảm; vô hiệu hóa tất cả phiên |

#### Tiêu chí chấp nhận
- [ ] Đăng nhập từ bất kỳ ứng dụng khách Bitwarden chính thức nào thành công.
- [ ] Đăng nhập thất bại sau khi vượt ngưỡng giới hạn tốc độ và thành công sau khi hạ nhiệt.
- [ ] Thay đổi mật khẩu chính vô hiệu hóa tất cả các phiên hiện hoạt khác.
- [ ] Các hành động được bảo vệ không thể hoàn thành mà không xác thực lại.
- [ ] Luồng phê duyệt thiết bị không dùng mật khẩu hoàn thành đăng nhập thành công.

---

### 6.3 Xác Thực Hai Yếu Tố (2FA)

**Mã tính năng**: F-MFA  
**Ưu tiên**: 🔴 Bắt buộc có  
**Chủ thể**: Người dùng cuối, Chủ/Quản trị tổ chức

#### Chức năng
Cung cấp xác minh yếu tố thứ hai tùy chọn (hoặc bắt buộc theo chính sách) khi đăng nhập, hỗ trợ sáu phương pháp riêng biệt.

#### Tầm quan trọng
2FA là nâng cấp bảo mật có tác động lớn nhất cho tài khoản người dùng. Việc thiết lập và sử dụng phải không có ma sát.

#### Các phương pháp được hỗ trợ

| Phương pháp | Trường hợp sử dụng | Mức bảo mật |
|------------|------------------|-------------|
| **TOTP** (RFC 6238) | Ứng dụng xác thực (Google Authenticator, Authy) | ⭐⭐⭐ |
| **Email OTP** | Dự phòng; không cần phần cứng | ⭐⭐ |
| **FIDO2 / WebAuthn** | Khóa phần cứng (YubiKey 5, Passkeys) | ⭐⭐⭐⭐⭐ |
| **YubiKey OTP** | YubiKey ở chế độ OTP | ⭐⭐⭐⭐ |
| **Duo Security** | Phê duyệt push doanh nghiệp; tích hợp với quản trị Duo | ⭐⭐⭐⭐ |
| **Mã khôi phục** | Truy cập khẩn cấp khi 2FA chính không khả dụng | N/A (khôi phục) |

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Thiết bị tin cậy** | 2FA có thể bỏ qua trên thiết bị người dùng đánh dấu là tin cậy |
| **Bắt buộc từ tổ chức** | Chính sách tổ chức có thể bắt buộc 2FA cho tất cả thành viên |
| **Cảnh báo 2FA chưa hoàn thành** | Công việc theo lịch phát hiện và gửi email cho người dùng đã qua bước mật khẩu nhưng chưa hoàn thành 2FA |
| **Duo OIDC** | Tích hợp Duo hiện đại ở cấp tổ chức qua OIDC |

#### Tiêu chí chấp nhận
- [ ] Người dùng có thể đăng ký và sử dụng mỗi trong sáu phương pháp 2FA.
- [ ] Đăng nhập bị chặn mà không có yếu tố thứ hai khi 2FA được bật.
- [ ] Mã khôi phục có thể sử dụng khi 2FA chính không khả dụng.
- [ ] Chính sách 2FA của tổ chức ngăn truy cập cho các thành viên không tuân thủ.

---

### 6.4 Quản Lý Tổ Chức & Cộng Tác Nhóm

**Mã tính năng**: F-ORG  
**Ưu tiên**: 🔴 Bắt buộc có  
**Chủ thể**: Chủ tổ chức, Quản trị viên, Quản lý, Người dùng

#### Chức năng
Cho phép nhóm chia sẻ các mục kho được mã hóa thông qua hệ thống phân cấp gồm tổ chức, bộ sưu tập, nhóm, vai trò và chính sách thành viên.

#### Tầm quan trọng
Đối với người dùng SMB và nhóm, quản lý kho chia sẻ là lý do chính để tự lưu trữ. Nếu không có tính năng tổ chức mạnh mẽ, Vaultwarden chỉ hữu ích cho cá nhân.

#### Hành vi chính

**Cấu trúc tổ chức:**
```
Tổ chức
  └── Bộ sưu tập (nhóm mục logic)
        ├── được gán cho Người dùng (trực tiếp)
        └── được gán cho Nhóm → Người dùng
```

**Vai trò & Quyền hạn:**

| Vai trò | Quản lý thành viên | Quản lý toàn bộ bộ sưu tập | Quản lý bộ sưu tập được gán | Truy cập mục |
|---------|:-----------------:|:-------------------------:|:---------------------------:|:------------:|
| Chủ sở hữu | ✅ | ✅ | ✅ | ✅ |
| Quản trị viên | ✅ | ✅ | ✅ | ✅ |
| Quản lý | ❌ | ❌ | ✅ | ✅ |
| Người dùng | ❌ | ❌ | ❌ | ✅ (chỉ được gán) |

**Vòng đời thành viên:**

```
Được mời → Đã chấp nhận → Đã xác nhận → [Thành viên đang hoạt động]
                                               ↓
                                           Bị thu hồi (quyền truy cập bị đình chỉ; dữ liệu được giữ lại)
```

**Hành vi bổ sung:**

| Hành vi | Chi tiết |
|---------|---------|
| **Nhóm** | Gán quyền truy cập bộ sưu tập cho một nhóm; thêm/xóa người dùng khỏi nhóm |
| **Khôi phục quản trị** | Chủ/Quản trị có thể đặt lại mật khẩu chính của thành viên (với sự đồng ý của người dùng) |
| **Khóa API tổ chức** | Truy cập tài khoản máy máy cho đường ống tự động hóa |
| **API công khai** | Tương thích Directory Connector |

#### Tiêu chí chấp nhận
- [ ] Chủ sở hữu có thể tạo tổ chức, mời thành viên, gán vai trò và tạo bộ sưu tập.
- [ ] Người dùng chỉ có thể truy cập các mục kho trong bộ sưu tập họ được gán.
- [ ] Thu hồi thành viên ngay lập tức ngăn truy cập của họ.
- [ ] Các nhóm truyền thay đổi quyền truy cập bộ sưu tập đến tất cả thành viên nhóm.
- [ ] Khôi phục mật khẩu chính quản trị cho phép chủ sở hữu lấy lại quyền truy cập thay mặt thành viên bị khóa.

---

### 6.5 Chia Sẻ Bảo Mật — Bitwarden Send

**Mã tính năng**: F-SEND  
**Ưu tiên**: 🟠 Nên có  
**Chủ thể**: Người dùng cuối

#### Chức năng
Cung cấp cơ chế chia sẻ tệp hoặc văn bản được mã hóa, bảo mật, tạm thời. Người nhận truy cập nội dung được chia sẻ mà không cần tài khoản Vaultwarden.

#### Tầm quan trọng
Loại bỏ nhu cầu chia sẻ mật khẩu qua email hoặc chat. Khóa mã hóa được nhúng trong phân đoạn URL và không bao giờ được gửi đến máy chủ, cung cấp bảo mật đầu cuối thực sự cho nội dung được chia sẻ.

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Loại** | Văn bản (loại 0), Tệp (loại 1, tối đa 500 MB) |
| **Kiểm soát truy cập** | Số lần truy cập tối đa, ngày hết hạn, ngày xóa |
| **Bảo vệ mật khẩu** | Tùy chọn; xác minh phía máy chủ qua Argon2id |
| **Quyền riêng tư email** | Người gửi có thể ẩn địa chỉ email với người nhận |
| **Bảo mật khóa** | Khóa giải mã trong phân đoạn URL — không bao giờ đến máy chủ |
| **Tự dọn dẹp** | Send hết hạn được xóa bởi bộ lập lịch nền |
| **Quản trị từ chối** | Quản trị viên có thể tắt tất cả Send qua `SENDS_ALLOWED=false` |

#### Tiêu chí chấp nhận
- [ ] Send có thể được tạo và người nhận truy cập mà không có tài khoản Vaultwarden.
- [ ] Send được bảo vệ mật khẩu từ chối mật khẩu không chính xác.
- [ ] Send tự động không thể truy cập sau ngày hết hạn hoặc số lần truy cập tối đa.
- [ ] Send tệp tối đa 500 MB tải lên và tải xuống thành công.
- [ ] Cơ sở dữ liệu máy chủ không chứa văn bản thuần túy có thể phục hồi của nội dung Send.

---

### 6.6 Truy Cập Khẩn Cấp

**Mã tính năng**: F-EMERGENCY  
**Ưu tiên**: 🟠 Nên có  
**Chủ thể**: Người dùng cuối (Người ủy quyền), Người được ủy quyền khẩn cấp

#### Chức năng
Cho phép người dùng chỉ định một người liên hệ tin cậy (người được ủy quyền) có thể yêu cầu quyền truy cập vào kho của họ trong tình huống khẩn cấp, tùy thuộc vào khoảng thời gian chờ có thể cấu hình và cơ chế đồng ý.

#### Tầm quan trọng
Ngăn mất dữ liệu vĩnh viễn khi chủ kho mật khẩu qua đời hoặc mất khả năng hoạt động. Mang lại sự an tâm cho gia đình và nhóm nhỏ.

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Loại truy cập** | Xem (chỉ đọc), Tiếp quản (đặt lại tài khoản đầy đủ) |
| **Thời gian chờ** | Có thể cấu hình mỗi lần ủy quyền (ví dụ: 7, 14, 30 ngày) |
| **Cửa sổ đồng ý** | Người ủy quyền có thể phê duyệt hoặc từ chối trong thời gian chờ |
| **Tự động phê duyệt** | Công việc nền cấp quyền truy cập sau thời gian chờ |
| **Email nhắc nhở** | Người ủy quyền được thông báo trước khi quyền truy cập được cấp |
| **Lời mời** | Người được ủy quyền được mời qua email; token sử dụng một lần |

#### Luồng Truy cập Khẩn cấp

```
Người được ủy quyền gửi yêu cầu
    ↓
Thời gian chờ bắt đầu
    ↓
Người ủy quyền nhận email thông báo (có thể phê duyệt/từ chối bất cứ lúc nào)
    ↓
[Nếu không có hành động] → Tự động phê duyệt sau thời gian chờ
    ↓
Người được ủy quyền nhận thông báo quyền truy cập được cấp
    ↓
Người được ủy quyền xem kho (chế độ Xem) hoặc đặt lại tài khoản (chế độ Tiếp quản)
```

#### Tiêu chí chấp nhận
- [ ] Người ủy quyền có thể mời người được ủy quyền và đặt thời gian chờ và loại truy cập.
- [ ] Người được ủy quyền có thể gửi yêu cầu và được quyền truy cập sau thời gian chờ.
- [ ] Người ủy quyền có thể từ chối yêu cầu trước khi tự động phê duyệt.
- [ ] Người được ủy quyền "Xem" có thể đọc các mục kho nhưng không sửa đổi.
- [ ] Người được ủy quyền "Tiếp quản" có thể đặt mật khẩu chính mới và tiếp quản toàn quyền.

---

### 6.7 Đồng Bộ Thời Gian Thực & Thông Báo

**Mã tính năng**: F-SYNC  
**Ưu tiên**: 🟠 Nên có  
**Chủ thể**: Người dùng cuối

#### Chức năng
Đẩy các sự kiện thay đổi kho đến tất cả các máy khách Bitwarden đã kết nối theo thời gian thực, loại bỏ nhu cầu đồng bộ thủ công hoặc thăm dò.

#### Tầm quan trọng
Đồng bộ thời gian thực cải thiện đáng kể trải nghiệm người dùng cho người dùng nhiều thiết bị. Nếu không có nó, những thay đổi được thực hiện trên một thiết bị có thể không xuất hiện trên thiết bị khác trong nhiều phút.

#### Hành vi chính

| Kênh | Công nghệ | Mặc định |
|------|---------|---------|
| **WebSocket** | MessagePack qua WSS tại `/notifications/hub` | Tắt (yêu cầu `ENABLE_WEBSOCKET=true`) |
| **Push di động** | Relay bên ngoài → APNs / FCM | Tắt (yêu cầu cấu hình relay) |

**Loại sự kiện được truyền:**
`SyncCipherCreate`, `SyncCipherUpdate`, `SyncCipherDelete`, `SyncFolderCreate`, `SyncFolderUpdate`, `SyncFolderDelete`, `SyncVault`, `SyncOrgKeys`, `SyncSendCreate`, `SyncSendUpdate`, `SyncSendDelete`, `SyncSettings`, `LogOut`, `AuthRequest`, `AuthRequestResponse`

| Hành vi | Chi tiết |
|---------|---------|
| **Nhiều thiết bị** | Một người dùng → nhiều phiên đồng thời, tất cả được thông báo |
| **Xác thực** | Bearer token trong tham số query (`?access_token=`) hoặc header |
| **Phiên đồng thời** | DashMap (không khóa) để tra cứu O(1) theo người dùng |

#### Tiêu chí chấp nhận
- [ ] Khi WebSocket được bật, thay đổi kho trên thiết bị A xuất hiện trên thiết bị B trong vòng 2 giây.
- [ ] Nhiều thiết bị đăng nhập với cùng một người dùng đều nhận cùng một sự kiện.
- [ ] Thiết bị di động nhận thông báo push khi có thay đổi kho.

---

### 6.8 Đăng Nhập Một Lần (SSO / OIDC)

**Mã tính năng**: F-SSO  
**Ưu tiên**: 🟡 Tốt nếu có (bắt buộc cho triển khai doanh nghiệp)  
**Chủ thể**: Quản trị viên máy chủ, Người dùng cuối

#### Chức năng
Tích hợp với bất kỳ Nhà cung cấp danh tính (IdP) tương thích OpenID Connect nào (Okta, Azure AD, Google Workspace, Keycloak, v.v.) để cho phép đăng nhập SSO của công ty.

#### Tầm quan trọng
Đối với các tổ chức đã sử dụng IdP, SSO cung cấp onboarding liền mạch, thu hồi quyền truy cập tập trung và loại bỏ quản lý mật khẩu mỗi người dùng ở phía Vaultwarden.

#### Luồng đăng nhập SSO

```
1. Người dùng nhấp "Đăng nhập với SSO" trong ứng dụng khách Bitwarden
2. Máy khách truy cập /identity/connect/auth?sso=1
3. Máy chủ tạo PKCE code_challenge + nonce → lưu vào DB
4. Người dùng được chuyển hướng đến trang đăng nhập IdP
5. IdP xác thực người dùng → chuyển hướng lại với auth code
6. Máy chủ trao đổi code lấy token (qua sso_client.rs)
7. Người dùng được tra cứu hoặc tự động tạo trong Vaultwarden
8. JWT Vaultwarden được cấp phát → trả về cho máy khách
```

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Giao thức** | Mã ủy quyền OIDC + PKCE |
| **Tự động tạo** | Người dùng mới được tạo khi đăng nhập SSO lần đầu |
| **Cache trạng thái** | TTL 10 phút, tối đa 1.000 trạng thái đồng thời |
| **Cấu hình** | `SSO_AUTHORITY`, `SSO_CLIENT_ID`, `SSO_CLIENT_SECRET` |
| **Đồng tồn tại** | Đăng nhập tên người dùng/mật khẩu vẫn khả dụng cùng với SSO |
| **Dọn dẹp nonce** | Nonce hết hạn được xóa hằng ngày |

#### Tiêu chí chấp nhận
- [ ] Người dùng có thể đăng nhập qua nhà cung cấp OIDC đã cấu hình mà không cần mật khẩu Vaultwarden.
- [ ] Người dùng mới được tự động tạo khi đăng nhập SSO lần đầu.
- [ ] Đăng nhập SSO thất bại an toàn khi IdP không thể truy cập.
- [ ] Tắt SSO không ảnh hưởng đến đăng nhập tên người dùng/mật khẩu thông thường.

---

### 6.9 Bảng Quản Trị & Quản Lý Máy Chủ

**Mã tính năng**: F-ADMIN  
**Ưu tiên**: 🔴 Bắt buộc có  
**Chủ thể**: Quản trị viên máy chủ

#### Chức năng
Cung cấp giao diện quản trị dựa trên web tại `/admin` để quản lý người dùng, tổ chức, cấu hình và sức khỏe máy chủ mà không cần truy cập trực tiếp vào cơ sở dữ liệu hoặc CLI.

#### Tầm quan trọng
Hầu hết quản trị viên máy chủ không thoải mái khi chỉnh sửa cơ sở dữ liệu trực tiếp. Bảng quản trị là giao diện vận hành chính cho những người chạy Vaultwarden.

#### Hành vi chính

| Khả năng | Chi tiết |
|---------|---------|
| **Kiểm soát truy cập** | Token được băm Argon2id (`ADMIN_TOKEN`); tạo qua CLI `vaultwarden hash` |
| **Quản lý người dùng** | Liệt kê, mời, kích hoạt, vô hiệu hóa, xóa người dùng |
| **Quản lý tổ chức** | Liệt kê tổ chức và các thành viên của họ |
| **Cấu hình** | Chỉnh sửa tất cả cài đặt; lưu vào `config.json` |
| **Chẩn đoán** | Thông tin máy chủ, phiên bản, trạng thái DB |
| **Sao lưu SQLite** | Kích hoạt sao lưu từ bảng điều khiển |
| **Kiểm soát phiên** | Thời gian sống phiên có thể cấu hình (`ADMIN_SESSION_LIFETIME`) |
| **Chế độ không token** | `DISABLE_ADMIN_TOKEN` cho môi trường có xác thực bên ngoài |

**Cấu hình sẵn có của Argon2id để tạo token:**

| Cấu hình sẵn | Bộ nhớ | Số lần lặp | Luồng | Khuyến nghị cho |
|-------------|--------|----------|-------|----------------|
| `bitwarden` (mặc định) | 64 MiB | 3 | 4 | Triển khai tiêu chuẩn |
| `owasp` | 19 MiB | 2 | 1 | Máy chủ tài nguyên thấp |

#### Tiêu chí chấp nhận
- [ ] Bảng quản trị có thể truy cập tại `/admin` với token Argon2id hợp lệ.
- [ ] Quản trị viên có thể mời người dùng nhận được email mời thành công.
- [ ] Quản trị viên có thể thay đổi cài đặt cấu hình; thay đổi tồn tại sau khi khởi động lại máy chủ.
- [ ] Token văn bản thuần túy hoặc bcrypt bị từ chối.

---

### 6.10 Nhật Ký Kiểm Toán & Sự Kiện

**Mã tính năng**: F-AUDIT  
**Ưu tiên**: 🟠 Nên có  
**Chủ thể**: Chủ tổ chức, Quản trị viên

#### Chức năng
Ghi lại tất cả các hành động quan trọng được thực hiện trong một tổ chức vào nhật ký kiểm toán bất biến có dấu thời gian, có thể truy cập qua API.

#### Tầm quan trọng
Nhật ký kiểm toán thường là yêu cầu tuân thủ (SOC 2, ISO 27001, GDPR) và rất quan trọng để điều tra sự cố ("ai đã thay đổi mật khẩu này và khi nào?").

#### Các trường sự kiện được ghi lại

| Trường | Ví dụ |
|-------|-------|
| Loại sự kiện | `CipherUpdated`, `MemberRemoved` |
| UUID người dùng thực hiện | `uuid` |
| UUID cipher mục tiêu | `uuid` |
| UUID tổ chức | `uuid` |
| UUID thiết bị | `uuid` |
| Địa chỉ IP | `192.168.1.10` |
| Dấu thời gian | `2026-04-10T09:00:00Z` |

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Kích hoạt** | Yêu cầu `ORG_EVENTS_ENABLED=true` |
| **Truy cập API** | Điểm cuối `/events` |
| **Lưu giữ** | Lịch dọn dẹp có thể cấu hình (`EVENT_CLEANUP_SCHEDULE`) |
| **Tự dọn dẹp** | Công việc nền xóa các mục cũ |

#### Tiêu chí chấp nhận
- [ ] Mỗi hành động cấp tổ chức tạo một mục nhật ký sự kiện tương ứng.
- [ ] Các sự kiện bao gồm tất cả các trường bắt buộc (nhân vật, loại, mục tiêu, IP, dấu thời gian).
- [ ] Nhật ký tồn tại qua các lần khởi động lại máy chủ.
- [ ] Các sự kiện cũ được dọn sạch theo lịch đã cấu hình.

---

### 6.11 Thông Báo Qua Email

**Mã tính năng**: F-EMAIL  
**Ưu tiên**: 🔴 Bắt buộc có  
**Chủ thể**: Tất cả người dùng (người nhận), Quản trị viên máy chủ (cấu hình)

#### Chức năng
Gửi email giao dịch cho các sự kiện vòng đời tài khoản, cảnh báo bảo mật và quy trình lời mời.

#### Tầm quan trọng
Email là kênh giao tiếp ngoài băng tần chính cho các hành động và cảnh báo tài khoản. Nếu không có email, nhiều luồng quan trọng (xác minh email, lời mời, truy cập khẩn cấp) bị chặn.

#### Email được kích hoạt

| Sự kiện | Người nhận |
|---------|-----------|
| Lời mời tài khoản | Người được mời |
| Xác minh địa chỉ email | Người dùng mới |
| Cảnh báo đăng nhập 2FA chưa hoàn thành | Chủ tài khoản |
| Lời mời tổ chức | Người được mời |
| Lời mời truy cập khẩn cấp | Người được ủy quyền |
| Yêu cầu truy cập khẩn cấp được khởi tạo | Người ủy quyền |
| Truy cập khẩn cấp được cấp | Người được ủy quyền |
| Truy cập khẩn cấp bị từ chối | Người được ủy quyền |
| Nhắc nhở truy cập khẩn cấp | Người ủy quyền |
| Xác nhận xóa tài khoản | Chủ tài khoản |

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Phương thức vận chuyển** | SMTP (STARTTLS / TLS) hoặc Sendmail |
| **Mẫu** | Tệp Handlebars `.hbs`; hoàn toàn có thể tùy chỉnh |
| **TLS** | `rustls` với chứng chỉ gốc gốc |
| **Chế độ debug** | `SMTP_DEBUG=true` để ghi log SMTP chi tiết |

#### Tiêu chí chấp nhận
- [ ] Tất cả email được kích hoạt được gửi khi SMTP được cấu hình đúng.
- [ ] Email hiển thị đúng trong các ứng dụng email chính.
- [ ] Email không thể gửi được ghi vào log với thông báo lỗi đầy đủ thông tin.

---

### 6.12 Lưu Trữ Tệp & Tệp Đính Kèm

**Mã tính năng**: F-STORAGE  
**Ưu tiên**: 🔴 Bắt buộc có  
**Chủ thể**: Người dùng cuối, Quản trị viên máy chủ

#### Chức năng
Quản lý lưu trữ tệp cho tệp đính kèm mục kho và tệp Bitwarden Send thông qua lớp trừu tượng thống nhất (OpenDAL) hỗ trợ cả hệ thống tệp cục bộ và lưu trữ đối tượng tương thích S3.

#### Tầm quan trọng
Tệp đính kèm là tính năng premium quan trọng. Hỗ trợ S3 cho phép người vận hành mở rộng lưu trữ độc lập với tiến trình máy chủ.

#### Hành vi chính

| Hành vi | Chi tiết |
|---------|---------|
| **Trừu tượng hóa lưu trữ** | Apache OpenDAL — cùng API cho cục bộ và S3 |
| **Đường dẫn mặc định** | `data/attachments/`, `data/sends/`, `data/rsa_key.pem` |
| **Hỗ trợ S3** | Được bật qua tính năng Cargo `s3`; cấu hình qua biến môi trường |
| **Tải lên tối đa** | 525 MB mỗi tệp |
| **Xác thực** | Tải xuống tệp yêu cầu JWT token tải xuống tệp sử dụng một lần |
| **Khóa RSA** | Khóa ký JWT của máy chủ được lưu trữ qua OpenDAL |

**Bố cục đường dẫn lưu trữ:**
```
data/
├── attachments/    ← Tệp đính kèm cipher kho mật khẩu
├── sends/          ← Tệp Bitwarden Send
├── rsa_key.pem     ← Khóa ký JWT
└── config.json     ← Cấu hình runtime
```

#### Tiêu chí chấp nhận
- [ ] Tệp đính kèm tối đa 525 MB có thể tải lên và tải xuống.
- [ ] Tệp được lưu trữ trong cấu trúc thư mục đúng.
- [ ] Các triển khai được cấu hình S3 lưu trữ và truy xuất tệp từ bucket S3.
- [ ] Liên kết tải xuống tệp hết hạn sau lần sử dụng đầu tiên.

---

## 7. Ưu Tiên Tính Năng (MoSCoW)

### 7.1 Định nghĩa mức độ ưu tiên

| Ưu tiên | Nhãn | Định nghĩa |
|---------|------|-----------|
| 🔴 | **Bắt buộc có** | Chức năng cốt lõi; sản phẩm không khả thi nếu thiếu |
| 🟠 | **Nên có** | Quan trọng với hầu hết người dùng; nên có trong v1 |
| 🟡 | **Tốt nếu có** | Có giá trị cho phân khúc cụ thể; nên có nếu thời gian cho phép |
| ⚪ | **Chưa có (tạm thời)** | Hoãn rõ ràng sang phiên bản tương lai |

### 7.2 Bảng ưu tiên tính năng

| Tính năng | Mã | Ưu tiên | Lý do |
|-----------|-----|:-------:|-------|
| Kho cốt lõi (CRUD, đồng bộ, thư mục) | F-VAULT | 🔴 Bắt buộc | Giá trị sản phẩm chính |
| Xác thực & Quản lý JWT | F-AUTH | 🔴 Bắt buộc | Cổng vào tất cả tính năng |
| Thông báo Email (SMTP) | F-EMAIL | 🔴 Bắt buộc | Cần thiết cho lời mời, xác minh |
| Lưu trữ Tệp & Đính kèm | F-STORAGE | 🔴 Bắt buộc | Ngang bằng premium Bitwarden |
| Bảng quản trị | F-ADMIN | 🔴 Bắt buộc | Giao diện vận hành chính |
| Xác thực hai yếu tố (tất cả loại) | F-MFA | 🔴 Bắt buộc | Yêu cầu bảo mật cốt lõi |
| Quản lý tổ chức & Nhóm | F-ORG | 🔴 Bắt buộc | Chủ chốt cho nhóm nhỏ |
| Bitwarden Send | F-SEND | 🟠 Nên có | Trường hợp dùng phổ biến; chia sẻ ưu tiên quyền riêng tư |
| Truy cập khẩn cấp | F-EMERGENCY | 🟠 Nên có | Quan trọng cho tin cậy và khả năng phục hồi tài khoản |
| Đồng bộ thời gian thực (WebSocket) | F-SYNC | 🟠 Nên có | Cải thiện đáng kể trải nghiệm người dùng |
| Thông báo push di động | F-SYNC | 🟠 Nên có | Cần thiết cho UX di động |
| Nhật ký kiểm toán & Sự kiện | F-AUDIT | 🟠 Nên có | Cần thiết cho tổ chức chú trọng tuân thủ |
| Đăng nhập một lần (OIDC) | F-SSO | 🟡 Tốt nếu có | Cần cho doanh nghiệp; phức tạp để cấu hình |
| Lưu trữ tệp S3 | F-STORAGE | 🟡 Tốt nếu có | Tính năng người vận hành nâng cao |
| Tích hợp Duo Security | F-MFA | 🟡 Tốt nếu có | Yêu cầu dành riêng cho doanh nghiệp |
| API Directory Connector | F-ORG | 🟡 Tốt nếu có | Tính năng doanh nghiệp nâng cao |
| Bộ cấp phát MiMalloc | (NFR) | ⚪ Chưa có | Tối ưu hóa; ưu tiên thấp |

---

## 8. Luồng Người Dùng

### 8.1 Đăng ký người dùng mới & Đăng nhập lần đầu

```
Người dùng mở ứng dụng khách Bitwarden
    → Đặt URL máy chủ thành phiên bản Vaultwarden
    → Nhấp "Tạo Tài khoản"
    → Nhập email + mật khẩu chính
    → Máy khách dẫn xuất khóa mã hóa cục bộ (PBKDF2/Argon2id)
    → POST /identity/accounts/register
        ← Máy chủ tạo tài khoản người dùng (lưu hash mật khẩu, khóa đã mã hóa)
    → [Nếu SIGNUPS_VERIFY=true]
        Máy chủ gửi email xác minh
        Người dùng nhấp liên kết → GET /api/accounts/verify-email?token=…
    → Người dùng đăng nhập:
        POST /identity/connect/token
        ← Máy chủ xác thực thông tin đăng nhập, trả về token truy cập + làm mới
    → Kho được mở khóa → người dùng có thể thêm mục
```

### 8.2 Onboarding thành viên tổ chức

```
Chủ tổ chức mở web vault
    → Điều hướng đến Tổ chức → Thành viên
    → Nhấp "Mời" → nhập email thành viên
    → Máy chủ tạo JWT Invitation → gửi email

Thành viên nhận email → nhấp "Chấp nhận Lời mời"
    → Đăng ký hoặc đăng nhập (nếu tài khoản tồn tại)
    → Trạng thái: Được mời → Đã chấp nhận

Chủ tổ chức quay lại Thành viên
    → Xác nhận thành viên (gán akey)
    → Trạng thái: Đã chấp nhận → Đã xác nhận

Thành viên có thể truy cập bộ sưu tập họ được gán
```

### 8.3 Bitwarden Send — Chia sẻ văn bản bảo mật

```
Người dùng mở ứng dụng khách Bitwarden
    → Đến Send → Tạo Send mới
    → Gõ tin nhắn (hoặc tải lên tệp)
    → Tùy chọn: đặt ngày hết hạn, số lượng xem tối đa, mật khẩu
    → [Phía máy khách] tạo khóa AES-256 ngẫu nhiên
    → Mã hóa nội dung bằng khóa
    → POST /api/sends {dữ_liệu_đã_mã_hóa, kiểm_soát_truy_cập}
        ← Máy chủ lưu blob đã mã hóa + trả về URL send

Người dùng chia sẻ URL với người nhận (URL chứa #phân_đoạn_khóa)

Người nhận mở URL trên bất kỳ trình duyệt nào
    → JS máy khách trích xuất khóa từ phân đoạn URL (không bao giờ gửi đến máy chủ)
    → GET /api/sends/{id}/access
        ← Máy chủ trả về blob đã mã hóa
    → Máy khách giải mã bằng khóa từ URL → hiển thị nội dung
```

### 8.4 Luồng đăng nhập SSO

```
Người dùng nhấp "Đăng nhập với SSO"
    → Máy khách truy cập GET /identity/connect/auth?sso=1
    → Máy chủ tạo PKCE challenge + nonce → lưu vào SsoNonce
        ← Trả về redirect_uri đến IdP

Trình duyệt người dùng được chuyển hướng đến trang đăng nhập IdP
    → Người dùng xác thực với thông tin đăng nhập công ty
    → IdP chuyển hướng lại: /identity/connect/oidc-signin?code=…

Máy chủ nhận callback
    → Xác thực PKCE + state
    → Trao đổi code lấy token với IdP
    → Tra cứu hoặc tự động tạo người dùng Vaultwarden
    → Cấp JWT Vaultwarden
        ← Trả về token truy cập cho máy khách

Người dùng hiện đã đăng nhập vào kho Vaultwarden qua SSO
```

---

## 9. Yêu Cầu Sản Phẩm Phi Chức Năng

### 9.1 Yêu cầu bảo mật

| Yêu cầu | Lý do sản phẩm |
|---------|--------------|
| Mã hóa đầu cuối (AES-256-GCM/CBC) | Cam kết tin cậy cốt lõi: máy chủ là kho mù |
| Không lưu trữ văn bản thuần túy của bí mật | Yêu cầu quy định và tin cậy |
| Argon2id cho token quản trị | Chống lại các cuộc tấn công bẻ khóa bằng GPU |
| Giới hạn tốc độ trên các điểm cuối xác thực | Phòng thủ chống nhồi thông tin đăng nhập |
| `#![forbid(unsafe_code)]` | Đảm bảo an toàn bộ nhớ từ Rust |
| PKCE cho luồng SSO | Ngăn chặn chặn mã ủy quyền |
| Chỉ HTTPS qua reverse proxy | Không có thông tin đăng nhập ở dạng văn bản thuần túy qua mạng |

### 9.2 Yêu cầu hiệu suất

| Kịch bản | Mục tiêu |
|---------|---------|
| Đăng nhập (`/identity/connect/token`) | < 300ms p95 |
| Đồng bộ kho (`/api/sync`) | < 500ms p95 cho kho lên đến 500 mục |
| Phân phối sự kiện WebSocket | < 2 giây từ đầu đến cuối |
| Tải lên tệp (10 MB) | < 5 giây |
| Bộ nhớ máy chủ (nhàn rỗi, 10 người dùng) | < 50 MB RAM |
| Bộ nhớ máy chủ (đang hoạt động, 50 người dùng đồng thời) | < 150 MB RAM |

### 9.3 Yêu cầu độ tin cậy

| Yêu cầu | Mục tiêu |
|---------|---------|
| Di chuyển cơ sở dữ liệu | Tự động áp dụng khi khởi động; không cần bước thủ công |
| Khả năng phục hồi kết nối DB | Thử lại `DB_CONNECTION_RETRIES` lần trước khi thất bại |
| Sao lưu SQLite | Theo yêu cầu và theo lịch; ảnh chụp nhất quán |
| Công việc nền | Luồng OS riêng biệt; không chặn các trình xử lý HTTP |

### 9.4 Yêu cầu tương thích

| Yêu cầu | Mục tiêu |
|---------|---------|
| Tương thích ứng dụng khách Bitwarden | 100% — tất cả điểm cuối hoạt động |
| Hỗ trợ cơ sở dữ liệu | SQLite (mặc định), PostgreSQL, MySQL/MariaDB |
| Hỗ trợ container | Docker (amd64, arm64), Podman |
| Mục tiêu xây dựng | Linux (glibc, musl), macOS |
| Rust MSRV | 1.89.0 |

### 9.5 Yêu cầu khả năng vận hành

| Yêu cầu | Mục tiêu |
|---------|---------|
| Cấu hình | 100% qua biến môi trường |
| Triển khai | Lệnh `docker run` đơn |
| Nhật ký | Có cấu trúc; mức độ có thể cấu hình; giá trị nhạy cảm bị che giấu |
| UX quản trị | Không cần CLI cho các thao tác hằng ngày |

---

## 10. Chiến Lược Phát Hành & Cột Mốc

### 10.1 Phương pháp phiên bản

Vaultwarden theo mô hình phân phối liên tục với **phát hành theo lịch** phù hợp với các cột mốc tương thích API Bitwarden.

### 10.2 Kế hoạch cột mốc

#### Cột mốc 1 — Kho cốt lõi (v1.0 Cơ sở)
**Mục tiêu**: Kho mật khẩu tự lưu trữ tối thiểu khả thi.

| Tính năng | Trạng thái |
|---------|---------|
| Đăng ký & đăng nhập người dùng | ✅ Đã triển khai |
| CRUD mục kho (tất cả 5 loại) | ✅ Đã triển khai |
| Thư mục & mục yêu thích | ✅ Đã triển khai |
| Tệp đính kèm (lưu trữ cục bộ) | ✅ Đã triển khai |
| Email SMTP | ✅ Đã triển khai |
| Bảng quản trị | ✅ Đã triển khai |
| Cơ sở dữ liệu SQLite | ✅ Đã triển khai |
| Triển khai container Docker | ✅ Đã triển khai |

#### Cột mốc 2 — Bảo mật & MFA (v1.1)
**Mục tiêu**: Thế trận bảo mật cấp sản xuất.

| Tính năng | Trạng thái |
|---------|---------|
| TOTP 2FA | ✅ Đã triển khai |
| Email OTP | ✅ Đã triển khai |
| WebAuthn / FIDO2 | ✅ Đã triển khai |
| YubiKey OTP | ✅ Đã triển khai |
| Duo Security | ✅ Đã triển khai |
| Giới hạn tốc độ | ✅ Đã triển khai |
| Xác thực lại hành động được bảo vệ | ✅ Đã triển khai |
| Không mật khẩu (AuthRequest) | ✅ Đã triển khai |

#### Cột mốc 3 — Nhóm & Cộng tác (v1.2)
**Mục tiêu**: Kích hoạt các trường hợp sử dụng nhóm.

| Tính năng | Trạng thái |
|---------|---------|
| Tổ chức & thành viên | ✅ Đã triển khai |
| Bộ sưu tập | ✅ Đã triển khai |
| Nhóm | ✅ Đã triển khai |
| Chính sách tổ chức | ✅ Đã triển khai |
| Nhật ký sự kiện / kiểm toán | ✅ Đã triển khai |
| Khôi phục mật khẩu quản trị | ✅ Đã triển khai |

#### Cột mốc 4 — Tính năng nâng cao (v1.3)
**Mục tiêu**: Ngang bằng premium và tích hợp nâng cao.

| Tính năng | Trạng thái |
|---------|---------|
| Bitwarden Send | ✅ Đã triển khai |
| Truy cập khẩn cấp | ✅ Đã triển khai |
| Đồng bộ thời gian thực WebSocket | ✅ Đã triển khai |
| Thông báo push di động | ✅ Đã triển khai |
| SSO / OIDC | ✅ Đã triển khai |
| Lưu trữ đối tượng S3 | ✅ Đã triển khai |

#### Cột mốc 5 — Gia cố & Vận hành (v1.4+)
**Mục tiêu**: Gia cố sản xuất, quan sát tính và công cụ vận hành.

| Tính năng | Trạng thái |
|---------|---------|
| Hỗ trợ PostgreSQL | ✅ Đã triển khai |
| Hỗ trợ MySQL/MariaDB | ✅ Đã triển khai |
| Duo OIDC (luồng hiện đại) | ✅ Đã triển khai |
| Sao lưu SQLite (`SIGUSR1`) | ✅ Đã triển khai |
| Công việc nền có thể cấu hình | ✅ Đã triển khai |
| Bộ cấp phát MiMalloc (bản dựng musl) | ✅ Đã triển khai |
| Ghi log mở rộng & ghi log query | ✅ Đã triển khai |

---

## 11. Rủi Ro & Biện Pháp Giảm Thiểu

| Mã rủi ro | Rủi ro | Khả năng xảy ra | Tác động | Biện pháp giảm thiểu |
|----------|-------|:--------------:|:--------:|---------------------|
| R-01 | Thay đổi API ứng dụng khách Bitwarden phá vỡ tương thích | Trung bình | Cao | Theo dõi changelog API Bitwarden; duy trì bộ kiểm tra tích hợp với các máy khách mới nhất |
| R-02 | Lỗ hổng bảo mật trong xác thực hoặc mã hóa cốt lõi | Thấp | Nghiêm trọng | Sử dụng thư viện đã được xác lập (ring, jsonwebtoken); cấm mã unsafe; kiểm tra phụ thuộc thường xuyên |
| R-03 | Hỏng dữ liệu SQLite dưới các ghi đồng thời | Trung bình | Cao | Sử dụng chế độ WAL; khuyến nghị PostgreSQL cho triển khai nhiều người dùng; cung cấp công cụ sao lưu |
| R-04 | Cấu hình SMTP sai chặn email xác minh | Cao | Trung bình | Cung cấp điểm cuối kiểm tra SMTP trong bảng quản trị; thông báo lỗi rõ ràng; chế độ debug |
| R-05 | Thời gian chết IdP SSO ngăn tất cả đăng nhập | Trung bình | Cao | Cho phép đăng nhập tên người dùng/mật khẩu cùng tồn tại với SSO; tài liệu người vận hành |
| R-06 | Lộ thông tin đăng nhập S3 qua cấu hình sai | Thấp | Cao | Che giấu tất cả thông tin đăng nhập trong nhật ký và đầu ra API; tài liệu hướng dẫn về chính sách IAM ít đặc quyền nhất |
| R-07 | Tấn công brute-force token quản trị | Thấp | Nghiêm trọng | Bắt buộc Argon2id; giới hạn tốc độ đăng nhập quản trị; từ chối token văn bản thuần túy |
| R-08 | Tấn công chuỗi cung ứng phụ thuộc | Thấp | Cao | Sử dụng `cargo audit`; ghim phiên bản phụ thuộc; xem xét tư vấn RUSTSEC |
| R-09 | Người vận hành sử dụng HTTP thuần túy (không có TLS reverse proxy) | Trung bình | Cao | Cảnh báo tài liệu; khuyến nghị bắt buộc HTTPS |
| R-10 | Vi phạm tuân thủ AGPL của người vận hành | Trung bình | Trung bình | Tài liệu cấp phép rõ ràng; nhận thức cộng đồng |

---

## 12. Câu Hỏi Mở & Quyết Định

| # | Câu hỏi | Chủ sở hữu | Trạng thái | Quyết định |
|---|---------|-----------|---------|-----------|
| CH-01 | Vaultwarden có nên hỗ trợ gốc API secrets manager không? | Sản phẩm | 🔴 Mở | Có thể ngoài phạm vi — dự án riêng biệt |
| CH-02 | Có nên thêm điểm cuối metrics Prometheus chính thức không? | Kỹ thuật | 🟡 Đang thảo luận | Dùng metrics dựa trên log tạm thời |
| CH-03 | Có nên có giới hạn tốc độ cho lần thử đăng nhập bảng quản trị không? | Bảo mật | 🟡 Đang thảo luận | TBD — Argon2id cung cấp chi phí thời gian theo mặc định |
| CH-04 | WebSocket có nên được bật mặc định không? | Sản phẩm | ✅ Đã quyết định | Tắt mặc định; người vận hành phải bật thủ công |
| CH-05 | Xác minh email có nên bắt buộc mặc định không? | Sản phẩm | ✅ Đã quyết định | Tùy chọn (có thể cấu hình): `SIGNUPS_VERIFY` |
| CH-06 | Số người dùng tối đa được hỗ trợ cho triển khai SQLite đơn? | Kỹ thuật | 🟡 Đang thảo luận | Khuyến nghị: < 100 người dùng với SQLite; PostgreSQL cho số lượng lớn hơn |

---

## 13. Phụ Lục: Ma Trận Truy Xuất Nguồn Gốc

Ma trận này liên kết Yêu cầu Sản phẩm (PRD) với Yêu cầu Người dùng (URD), Yêu cầu Phần mềm (SRS) và Thiết kế Kỹ thuật (TDD).

| Tính năng PRD | Tham chiếu URD | Tham chiếu SRS | Phần TDD |
|--------------|--------------|--------------|---------|
| F-VAULT (Kho cốt lõi CRUD) | UR-USER-003, UR-USER-004, UR-USER-005, UR-USER-007, UR-USER-008 | FR-CIPHER-001 đến FR-CIPHER-010 | §6.2 Mô hình Cipher, §4 HTTP Routes |
| F-AUTH (Xác thực) | UR-USER-001, UR-USER-002, UR-USER-010, UR-USER-013 | FR-AUTH-001 đến FR-AUTH-008 | §5 Xác thực & Ủy quyền |
| F-MFA (Xác thực hai yếu tố) | UR-USER-012, UR-MFA-001 đến UR-MFA-003, UR-POLICY-001 | FR-2FA-001 đến FR-2FA-009 | §6.4 Mô hình TwoFactor |
| F-ORG (Tổ chức) | UR-ORG-001 đến UR-ORG-007, UR-POLICY-001 đến UR-POLICY-004 | FR-ORG-001 đến FR-ORG-010 | §6.3 Mô hình Org & Membership |
| F-SEND (Bitwarden Send) | UR-SEND-001 đến UR-SEND-005 | FR-SEND-001 đến FR-SEND-005 | §6.5 Mô hình Send |
| F-EMERGENCY (Truy cập khẩn cấp) | UR-EMRG-001 đến UR-EMRG-003 | FR-EMRG-001 đến FR-EMRG-006 | §6.6 Các mô hình khác |
| F-SYNC (Đồng bộ thời gian thực) | UR-SYNC-001, UR-SYNC-002, UR-ADMIN-012, UR-ADMIN-013 | FR-NOTIF-001 đến FR-PUSH-004 | §9 Hệ thống thông báo |
| F-SSO (Đăng nhập một lần) | UR-ADMIN-011 | FR-SSO-001 đến FR-SSO-006 | §10 Tích hợp OIDC/SSO |
| F-ADMIN (Bảng quản trị) | UR-ADMIN-001 đến UR-ADMIN-008, UR-ADMIN-015 | FR-ADMIN-001 đến FR-ADMIN-006 | §5.2 Xác thực Admin, §12 Cấu hình |
| F-AUDIT (Nhật ký sự kiện) | UR-AUDIT-001 đến UR-AUDIT-003 | FR-EVENT-001 đến FR-EVENT-004 | §6.6 Mô hình Event |
| F-EMAIL (Hệ thống email) | UR-ADMIN-010 | FR-EMAIL-001 đến FR-EMAIL-004 | §13 Hệ thống Email |
| F-STORAGE (Lưu trữ tệp) | UR-USER-007, UR-ADMIN-008 | FR-ATTACH-001 đến FR-ATTACH-004 | §8 Lưu trữ Tệp (OpenDAL) |

---

*Hết tài liệu*
