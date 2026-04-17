# Vaultwarden — Tài Liệu Yêu Cầu Người Dùng (URD)

> **Phiên bản tài liệu**: 1.0  
> **Ngày**: 2026-04-10  
> **Trạng thái**: Bản nháp  
> **Tài liệu tham chiếu**:  
> - Đặc tả Yêu cầu Phần mềm: `specs/srs.md`  
> - Tài liệu Thiết kế Kỹ thuật: `specs/technical-design.md`  
> **Dự án nguồn**: `dani-garcia/vaultwarden` — Máy chủ quản lý mật khẩu tự lưu trữ tương thích Bitwarden

---

## Mục Lục

1. [Giới thiệu](#1-giới-thiệu)
2. [Hồ sơ người dùng & Mục tiêu](#2-hồ-sơ-người-dùng--mục-tiêu)
3. [Tổng quan use case](#3-tổng-quan-use-case)
4. [Yêu cầu người dùng theo vai trò](#4-yêu-cầu-người-dùng-theo-vai-trò)
   - 4.1 [Người dùng cuối — Quản lý kho mật khẩu cá nhân](#41-người-dùng-cuối--quản-lý-kho-mật-khẩu-cá-nhân)
   - 4.2 [Người dùng cuối — Cài đặt tài khoản & Bảo mật](#42-người-dùng-cuối--cài-đặt-tài-khoản--bảo-mật)
   - 4.3 [Người dùng cuối — Chia sẻ bảo mật (Send)](#43-người-dùng-cuối--chia-sẻ-bảo-mật-send)
   - 4.4 [Người dùng cuối — Truy cập khẩn cấp](#44-người-dùng-cuối--truy-cập-khẩn-cấp)
   - 4.5 [Chủ/Quản trị tổ chức — Quản lý nhóm](#45-chủquản-trị-tổ-chức--quản-lý-nhóm)
   - 4.6 [Chủ/Quản trị tổ chức — Kiểm soát truy cập & Chính sách](#46-chủquản-trị-tổ-chức--kiểm-soát-truy-cập--chính-sách)
   - 4.7 [Chủ/Quản trị tổ chức — Kiểm toán & Tuân thủ](#47-chủquản-trị-tổ-chức--kiểm-toán--tuân-thủ)
   - 4.8 [Quản trị viên máy chủ — Quản lý phiên bản](#48-quản-trị-viên-máy-chủ--quản-lý-phiên-bản)
   - 4.9 [Quản trị viên máy chủ — Cấu hình & Tích hợp](#49-quản-trị-viên-máy-chủ--cấu-hình--tích-hợp)
5. [Nhu cầu người dùng xuyên suốt](#5-nhu-cầu-người-dùng-xuyên-suốt)
   - 5.1 [Bảo mật & Quyền riêng tư](#51-bảo-mật--quyền-riêng-tư)
   - 5.2 [Đa thiết bị & Đồng bộ thời gian thực](#52-đa-thiết-bị--đồng-bộ-thời-gian-thực)
   - 5.3 [Xác thực hai yếu tố](#53-xác-thực-hai-yếu-tố)
   - 5.4 [Khả năng sử dụng & Tương thích ứng dụng khách](#54-khả-năng-sử-dụng--tương-thích-ứng-dụng-khách)
6. [Ràng buộc & Kỳ vọng người dùng](#6-ràng-buộc--kỳ-vọng-người-dùng)
7. [Tóm tắt tiêu chí chấp nhận](#7-tóm-tắt-tiêu-chí-chấp-nhận)
8. [Bảng thuật ngữ](#8-bảng-thuật-ngữ)

---

## 1. Giới Thiệu

### 1.1 Mục đích

Tài liệu Yêu cầu Người dùng (URD) này mô tả nhu cầu, mục tiêu và kỳ vọng của tất cả người dùng và các bên liên quan của **Vaultwarden** — một máy chủ quản lý mật khẩu tự lưu trữ, mã nguồn mở, hoàn toàn tương thích với hệ sinh thái ứng dụng khách Bitwarden.

Khác với SRS (xác định *hệ thống phải làm gì* theo thuật ngữ kỹ thuật), URD này mô tả *người dùng muốn có thể làm gì* và *tại sao*, sử dụng ngôn ngữ về mục tiêu người dùng, kịch bản và tiêu chí chấp nhận.

### 1.2 Tổng quan dự án

Vaultwarden cho phép các cá nhân, gia đình và nhóm nhỏ tự lưu trữ một máy chủ tương thích Bitwarden — cho họ toàn quyền kiểm soát dữ liệu kho mật khẩu của mình mà không phụ thuộc vào dịch vụ đám mây của bên thứ ba. Người dùng tương tác với Vaultwarden độc quyền thông qua các ứng dụng khách Bitwarden chính thức (web, desktop, di động, tiện ích mở rộng trình duyệt); máy chủ không hiển thị với người dùng cuối.

### 1.3 Phạm vi

Tài liệu này bao gồm yêu cầu người dùng cho:

- **Người dùng cuối** quản lý kho mật khẩu cá nhân
- **Chủ tổ chức và quản trị viên** quản lý kho mật khẩu nhóm chia sẻ
- **Quản trị viên máy chủ** triển khai và vận hành phiên bản Vaultwarden

**Ngoài phạm vi:**
- Thiết kế giao diện ứng dụng khách Bitwarden (ủy quyền cho Bitwarden)
- Quản lý thanh toán hoặc đăng ký
- Các tính năng doanh nghiệp độc quyền trên đám mây Bitwarden chính thức (ví dụ: Secrets Manager)

### 1.4 Quy ước tài liệu

Yêu cầu người dùng được viết theo định dạng:

> **UR-[VAI_TRÒ]-[SỐ]**: Là một **[vai trò]**, tôi muốn **[hành động]** để **[mục tiêu/lợi ích]**.

Tiêu chí chấp nhận đi kèm với mỗi yêu cầu khi cần thiết.

---

## 2. Hồ Sơ Người Dùng & Mục Tiêu

### 2.1 Người dùng cuối (Cá nhân)

| Thuộc tính | Mô tả |
|-----------|-------|
| **Là ai** | Cá nhân lưu trữ mật khẩu, thẻ tín dụng, danh tính và ghi chú bảo mật |
| **Mục tiêu chính** | Lưu trữ và truy cập thông tin đăng nhập an toàn trên tất cả các thiết bị |
| **Mối quan tâm chính** | Quyền riêng tư — không có dịch vụ đám mây bên thứ ba nào được phép xem dữ liệu của họ |
| **Trình độ kỹ thuật** | Thấp đến trung bình — sử dụng ứng dụng khách Bitwarden; không biết về nội bộ máy chủ |
| **Thiết bị sử dụng** | Trình duyệt web, ứng dụng desktop, di động (iOS/Android), tiện ích mở rộng trình duyệt |

### 2.2 Chủ tổ chức / Quản trị viên

| Thuộc tính | Mô tả |
|-----------|-------|
| **Là ai** | Trưởng nhóm, quản trị CNTT hoặc chủ doanh nghiệp quản lý thông tin đăng nhập chung |
| **Mục tiêu chính** | Chia sẻ thông tin đăng nhập an toàn với thành viên nhóm và thực thi chính sách truy cập |
| **Mối quan tâm chính** | Kiểm soát truy cập chi tiết, khả năng kiểm toán và quản lý thành viên |
| **Trình độ kỹ thuật** | Trung bình — hiểu vai trò, quyền và bộ sưu tập |
| **Thiết bị sử dụng** | Chủ yếu là web vault |

### 2.3 Quản trị viên máy chủ

| Thuộc tính | Mô tả |
|-----------|-------|
| **Là ai** | Kỹ sư DevOps, sysadmin hoặc người dùng kỹ thuật triển khai và bảo trì máy chủ |
| **Mục tiêu chính** | Chạy phiên bản Vaultwarden ổn định, bảo mật với chi phí tối thiểu |
| **Mối quan tâm chính** | Độ tin cậy, khả năng nâng cấp, linh hoạt cấu hình và sao lưu dữ liệu |
| **Trình độ kỹ thuật** | Cao — thành thạo Docker, biến môi trường và cơ sở dữ liệu |
| **Thiết bị sử dụng** | CLI, bảng quản trị web, SSH |

### 2.4 Người được ủy quyền khẩn cấp

| Thuộc tính | Mô tả |
|-----------|-------|
| **Là ai** | Người tin cậy (bạn bè, thành viên gia đình, đồng nghiệp) được chủ kho mật khẩu chỉ định |
| **Mục tiêu chính** | Truy cập kho mật khẩu của người tin cậy trong tình huống khẩn cấp |
| **Mối quan tâm chính** | Quy trình rõ ràng, có giới hạn thời gian với sự đồng ý và biện pháp bảo vệ |
| **Trình độ kỹ thuật** | Thấp — sử dụng ứng dụng khách Bitwarden tiêu chuẩn |

---

## 3. Tổng Quan Use Case

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Hệ thống Vaultwarden                         │
│                                                                     │
│  ┌─────────────────┐   ┌──────────────────┐   ┌─────────────────┐  │
│  │  Kho cá nhân    │   │  Kho Nhóm / Tổ  │   │  Bảng quản trị  │  │
│  │                 │   │  chức            │   │                 │  │
│  │ • Quản lý mục  │   │ • Chia sẻ MK    │   │ • Quản lý ND    │  │
│  │ • Dùng 2FA     │   │ • Kiểm soát     │   │ • Cấu hình      │  │
│  │ • Chia sẻ Send │   │   truy cập      │   │ • Sao lưu       │  │
│  │ • Truy cập     │   │ • Kiểm toán     │   │ • Giám sát      │  │
│  │   khẩn cấp     │   │ • Chính sách    │   │                 │  │
│  └────────┬────────┘   └────────┬─────────┘   └────────┬────────┘  │
│           │                     │                       │           │
│    [Người dùng cuối]    [Chủ/Quản trị TS]       [Quản trị viên]    │
└─────────────────────────────────────────────────────────────────────┘
```

| Nhóm use case | Diễn viên chính | Tóm tắt |
|--------------|----------------|---------|
| UC-01: Thiết lập tài khoản | Người dùng cuối | Đăng ký, xác minh email, cấu hình 2FA |
| UC-02: Sử dụng kho hằng ngày | Người dùng cuối | Thêm, xem, sao chép, tự điền thông tin đăng nhập |
| UC-03: Chia sẻ bảo mật | Người dùng cuối | Tạo liên kết Send cho văn bản hoặc tệp |
| UC-04: Truy cập khẩn cấp | Người dùng cuối / Người được ủy quyền | Ủy quyền hoặc kích hoạt truy cập kho khẩn cấp |
| UC-05: Onboarding nhóm | Chủ tổ chức | Tạo tổ chức, mời thành viên, gán bộ sưu tập |
| UC-06: Quản lý truy cập | Quản trị tổ chức | Quản lý vai trò thành viên và quyền bộ sưu tập |
| UC-07: Thực thi chính sách | Chủ tổ chức | Đặt yêu cầu 2FA, chính sách mật khẩu |
| UC-08: Xem xét kiểm toán | Quản trị tổ chức | Xem xét nhật ký sự kiện để tuân thủ |
| UC-09: Thiết lập máy chủ | Quản trị viên máy chủ | Triển khai, cấu hình và bảo trì máy chủ |
| UC-10: Tích hợp SSO | Quản trị viên máy chủ | Kết nối Nhà cung cấp danh tính doanh nghiệp |

---

## 4. Yêu Cầu Người Dùng Theo Vai Trò

### 4.1 Người Dùng Cuối — Quản Lý Kho Mật Khẩu Cá Nhân

---

**UR-USER-001**: Là một **người dùng cuối**, tôi muốn **tạo một tài khoản** để tôi có thể **bắt đầu lưu trữ thông tin đăng nhập của mình một cách bảo mật**.

*Tiêu chí chấp nhận:*
- Tôi có thể đăng ký bằng địa chỉ email và mật khẩu chính.
- Tôi nhận được email xác minh trước khi tài khoản được kích hoạt (nếu quản trị viên bật).
- Tôi cũng có thể được tổ chức mời trước khi nhận tài khoản.

---

**UR-USER-002**: Là một **người dùng cuối**, tôi muốn **đăng nhập từ bất kỳ ứng dụng khách Bitwarden chính thức nào** (web, desktop, di động, tiện ích mở rộng trình duyệt) để tôi có thể **truy cập kho mật khẩu trên bất kỳ thiết bị nào**.

*Tiêu chí chấp nhận:*
- Đăng nhập hoạt động trên tất cả ứng dụng khách Bitwarden chính thức mà không cần cấu hình thêm.
- Phiên của tôi vẫn hoạt động trong thời gian hợp lý (desktop/web: 30 ngày, di động: 90 ngày).
- Tôi tự động đăng xuất khi token truy cập hết hạn.

---

**UR-USER-003**: Là một **người dùng cuối**, tôi muốn **thêm, chỉnh sửa và xóa các mục kho mật khẩu** để tôi có thể **giữ thông tin đăng nhập của mình được tổ chức và cập nhật**.

*Tiêu chí chấp nhận:*
- Tôi có thể tạo các mục thuộc các loại sau: Đăng nhập, Ghi chú bảo mật, Thẻ tín dụng, Danh tính, Khóa SSH.
- Tôi có thể thêm trường tùy chỉnh, ghi chú và URL vào mỗi mục.
- Tôi có thể chuyển mục vào thùng rác và khôi phục hoặc xóa vĩnh viễn chúng.
- Tôi có thể xem lịch sử mật khẩu của bất kỳ mục nào.

---

**UR-USER-004**: Là một **người dùng cuối**, tôi muốn **tổ chức các mục kho mật khẩu thành các thư mục** để tôi có thể **tìm thấy chúng dễ dàng**.

*Tiêu chí chấp nhận:*
- Tôi có thể tạo, đổi tên và xóa thư mục.
- Tôi có thể gán các mục kho vào một hoặc nhiều thư mục.
- Cấu trúc thư mục là cá nhân và không hiển thị với thành viên tổ chức trừ khi mục được chia sẻ.

---

**UR-USER-005**: Là một **người dùng cuối**, tôi muốn **đánh dấu các mục kho mật khẩu là mục yêu thích** để tôi có thể **nhanh chóng truy cập những mục tôi sử dụng thường xuyên nhất**.

*Tiêu chí chấp nhận:*
- Tôi có thể bật/tắt trạng thái yêu thích trên bất kỳ mục kho nào.
- Mục yêu thích xuất hiện ở đầu hoặc trong phần dành riêng trong ứng dụng khách.

---

**UR-USER-006**: Là một **người dùng cuối**, tôi muốn **kho mật khẩu của mình tự động đồng bộ trên tất cả các thiết bị** để **bất kỳ thay đổi nào tôi thực hiện được phản chiếu ngay lập tức ở mọi nơi**.

*Tiêu chí chấp nhận:*
- Khi tôi thêm, cập nhật hoặc xóa một mục trên một thiết bị, các thiết bị đã đăng nhập khác của tôi nhận thay đổi theo thời gian thực (hoặc khi đồng bộ tiếp theo).
- Tôi không cần kích hoạt đồng bộ thủ công.

---

**UR-USER-007**: Là một **người dùng cuối**, tôi muốn **đính kèm tệp vào các mục kho mật khẩu** để tôi có thể **lưu trữ các tài liệu liên quan cùng với thông tin đăng nhập**.

*Tiêu chí chấp nhận:*
- Tôi có thể tải lên tệp (tối đa 500 MB) và đính kèm vào mục kho mật khẩu.
- Tôi có thể tải xuống tệp đính kèm từ bất kỳ thiết bị nào.
- Tệp đính kèm được mã hóa phía máy khách trước khi gửi đến máy chủ.

---

**UR-USER-008**: Là một **người dùng cuối**, tôi muốn **đặt yêu cầu xác nhận lại trên các mục kho nhạy cảm** để **bất kỳ ai có quyền truy cập vào ứng dụng khách đã mở của tôi phải nhập lại mật khẩu chính trước khi xem những mục đó**.

*Tiêu chí chấp nhận:*
- Tôi có thể bật "Yêu cầu xác nhận lại mật khẩu chính" trên bất kỳ mục kho riêng lẻ nào.
- Ứng dụng khách nhắc mật khẩu chính khi tôi cố gắng xem, sao chép hoặc sử dụng mục đó.

---

### 4.2 Người Dùng Cuối — Cài Đặt Tài Khoản & Bảo Mật

---

**UR-USER-010**: Là một **người dùng cuối**, tôi muốn **thay đổi mật khẩu chính** để tôi có thể **duy trì bảo mật tài khoản mạnh mẽ theo thời gian**.

*Tiêu chí chấp nhận:*
- Tôi có thể thay đổi mật khẩu chính từ cài đặt tài khoản web vault.
- Thay đổi mật khẩu tự động vô hiệu hóa tất cả các phiên đang hoạt động khác.
- Dữ liệu kho mật khẩu của tôi vẫn có thể truy cập đầy đủ sau khi thay đổi mật khẩu.

---

**UR-USER-011**: Là một **người dùng cuối**, tôi muốn **thay đổi địa chỉ email đã đăng ký** để tôi có thể **giữ thông tin tài khoản luôn cập nhật**.

*Tiêu chí chấp nhận:*
- Tôi phải xác minh quyền sở hữu địa chỉ email mới trước khi thay đổi có hiệu lực.
- Tôi nhận được thông báo xác nhận ở cả địa chỉ email cũ và mới.

---

**UR-USER-012**: Là một **người dùng cuối**, tôi muốn **bật xác thực hai yếu tố (2FA)** để tài khoản của tôi **vẫn được bảo vệ ngay cả khi mật khẩu chính bị xâm phạm**.

*Tiêu chí chấp nhận:*
- Tôi có thể thiết lập ít nhất một trong các phương pháp 2FA: ứng dụng xác thực (TOTP), OTP email, khóa phần cứng (YubiKey hoặc FIDO2/WebAuthn) hoặc Duo.
- Tôi được cung cấp mã khôi phục trong trường hợp tôi mất quyền truy cập vào thiết bị 2FA.
- Tôi có thể đánh dấu thiết bị tin cậy để bỏ qua 2FA cho thuận tiện.

---

**UR-USER-013**: Là một **người dùng cuối**, tôi muốn **đăng nhập mà không cần mật khẩu bằng cách phê duyệt thiết bị** để tôi có thể **xác thực an toàn từ thiết bị mới**.

*Tiêu chí chấp nhận:*
- Tôi có thể bắt đầu đăng nhập trên thiết bị mới và phê duyệt từ thiết bị tin cậy.
- Yêu cầu hết hạn nếu không được phê duyệt trong thời gian hợp lý.

---

**UR-USER-014**: Là một **người dùng cuối**, tôi muốn **xóa tài khoản** để tôi có thể **xóa vĩnh viễn tất cả dữ liệu của mình khỏi máy chủ**.

*Tiêu chí chấp nhận:*
- Xóa tài khoản yêu cầu xác nhận qua liên kết gửi đến email đã đăng ký.
- Sau khi xóa, tất cả dữ liệu kho mật khẩu, tệp đính kèm và thông tin hồ sơ của tôi bị xóa vĩnh viễn.

---

**UR-USER-015**: Là một **người dùng cuối**, tôi muốn **tạo khóa API cá nhân** để tôi có thể **sử dụng Bitwarden CLI hoặc tự động hóa để truy cập kho mật khẩu theo phương thức lập trình**.

*Tiêu chí chấp nhận:*
- Tôi có thể tạo và thu hồi khóa API cá nhân từ cài đặt tài khoản.
- Khóa API cấp quyền truy cập tương tự như phiên người dùng của tôi.

---

### 4.3 Người Dùng Cuối — Chia Sẻ Bảo Mật (Send)

---

**UR-SEND-001**: Là một **người dùng cuối**, tôi muốn **chia sẻ một đoạn văn bản hoặc tệp với bất kỳ ai** bằng liên kết bảo mật, có giới hạn thời gian để **tôi không cần sử dụng các kênh không an toàn như email hay chat**.

*Tiêu chí chấp nhận:*
- Tôi có thể tạo Send chứa văn bản hoặc tệp (tối đa 500 MB).
- Người nhận có thể truy cập Send qua URL duy nhất mà không cần tài khoản Vaultwarden.
- Nội dung được giải mã trong trình duyệt của người nhận bằng khóa nhúng trong phân đoạn URL (không bao giờ gửi đến máy chủ).

---

**UR-SEND-002**: Là một **người dùng cuối**, tôi muốn **thêm mật khẩu vào Send của mình** để **chỉ người nhận dự kiến mới có thể mở nó**.

*Tiêu chí chấp nhận:*
- Tôi có thể tùy chọn yêu cầu mật khẩu trước khi Send có thể được truy cập.
- Mật khẩu được xác minh ở phía máy chủ mà không lưu trữ mật khẩu dạng văn bản thuần túy.

---

**UR-SEND-003**: Là một **người dùng cuối**, tôi muốn **đặt ngày hết hạn và giới hạn truy cập trên Send** để nó **tự động không thể truy cập sau một khoảng thời gian đặt sẵn hoặc số lần xem**.

*Tiêu chí chấp nhận:*
- Tôi có thể đặt số lần truy cập tối đa, ngày hết hạn và ngày xóa một cách độc lập.
- Send tự động bị vô hiệu hóa khi đạt bất kỳ giới hạn nào trong số này.

---

**UR-SEND-004**: Là một **người dùng cuối**, tôi muốn **ẩn địa chỉ email của mình với người nhận Send** để tôi có thể **chia sẻ nội dung ẩn danh**.

*Tiêu chí chấp nhận:*
- Tôi có thể bật/tắt "Ẩn email của tôi" khi tạo hoặc chỉnh sửa Send.
- Nếu bị ẩn, email của tôi không được hiển thị trên trang truy cập Send.

---

**UR-SEND-005**: Là một **người dùng cuối**, tôi muốn **thủ công vô hiệu hóa hoặc xóa Send bất kỳ lúc nào** để tôi **có thể thu hồi quyền truy cập ngay lập tức nếu cần**.

*Tiêu chí chấp nhận:*
- Tôi có thể vô hiệu hóa Send (làm cho liên kết không thể truy cập) mà không xóa nó.
- Tôi có thể xóa vĩnh viễn Send từ ứng dụng khách.

---

### 4.4 Người Dùng Cuối — Truy Cập Khẩn Cấp

---

**UR-EMRG-001**: Là một **người dùng cuối**, tôi muốn **chỉ định một người liên hệ tin cậy là người được ủy quyền truy cập khẩn cấp** để **họ có thể truy cập kho mật khẩu của tôi nếu tôi mất khả năng hoạt động hoặc không thể liên lạc**.

*Tiêu chí chấp nhận:*
- Tôi có thể mời bất kỳ người dùng Vaultwarden nào làm người liên hệ khẩn cấp qua email.
- Tôi đặt xem người được ủy quyền chỉ có thể **xem** kho mật khẩu hay **tiếp quản** hoàn toàn tài khoản của tôi.
- Tôi xác định thời gian chờ (ví dụ: 7 ngày) trước khi yêu cầu của người được ủy quyền tự động được phê duyệt.

---

**UR-EMRG-002**: Là một **người dùng cuối (người ủy quyền)**, tôi muốn **xem xét và từ chối yêu cầu truy cập khẩn cấp** trong thời gian chờ để tôi có thể **ngăn truy cập trái phép nếu tôi có thể phản hồi**.

*Tiêu chí chấp nhận:*
- Tôi nhận được thông báo email khi người được ủy quyền bắt đầu yêu cầu truy cập khẩn cấp.
- Tôi có toàn bộ thời gian chờ để phê duyệt hoặc từ chối yêu cầu trực tiếp.
- Nếu tôi từ chối yêu cầu, quyền truy cập của người được ủy quyền bị từ chối.

---

**UR-EMRG-003**: Là một **người được ủy quyền khẩn cấp**, tôi muốn **bắt đầu yêu cầu truy cập khẩn cấp** để tôi có thể **truy cập kho mật khẩu của người liên hệ tin cậy khi cần thiết**.

*Tiêu chí chấp nhận:*
- Tôi có thể gửi yêu cầu từ ứng dụng khách Bitwarden.
- Tôi nhận được thông báo khi thời gian chờ kết thúc và quyền truy cập được cấp.
- Nếu loại truy cập là "Xem", tôi có thể đọc các mục kho nhưng không thể thực hiện thay đổi.
- Nếu loại truy cập là "Tiếp quản", tôi có thể đặt lại tài khoản và giành toàn quyền sở hữu.

---

### 4.5 Chủ/Quản Trị Tổ Chức — Quản Lý Nhóm

---

**UR-ORG-001**: Là một **chủ tổ chức**, tôi muốn **tạo một tổ chức** để nhóm của tôi có thể **chia sẻ thông tin đăng nhập và làm việc cộng tác**.

*Tiêu chí chấp nhận:*
- Tôi có thể tạo tổ chức mới với tên và email thanh toán.
- Tổ chức có kho chia sẻ riêng, tách biệt với kho cá nhân.
- Tôi tự động được gán vai trò Chủ sở hữu.

---

**UR-ORG-002**: Là một **chủ hoặc quản trị viên tổ chức**, tôi muốn **mời thành viên nhóm qua email** để họ có thể **tham gia kho chia sẻ**.

*Tiêu chí chấp nhận:*
- Tôi có thể gửi lời mời đến một hoặc nhiều địa chỉ email.
- Người được mời nhận email với liên kết để chấp nhận lời mời.
- Lời mời hết hạn nếu không được chấp nhận trong thời gian có thể cấu hình.

---

**UR-ORG-003**: Là một **chủ tổ chức**, tôi muốn **gán vai trò cho thành viên** để tôi có thể **kiểm soát những gì mỗi người có thể quản lý**.

*Tiêu chí chấp nhận:*
- Các vai trò có sẵn: Chủ sở hữu, Quản trị viên, Quản lý, Người dùng, Tùy chỉnh.
- Chủ sở hữu và Quản trị viên có thể quản lý tất cả bộ sưu tập và thành viên.
- Quản lý có thể quản lý các bộ sưu tập được gán của họ.
- Người dùng chỉ có thể truy cập các mục họ được cấp quyền.

---

**UR-ORG-004**: Là một **chủ hoặc quản trị viên tổ chức**, tôi muốn **tạo bộ sưu tập** để tôi có thể **nhóm logic các mục kho chia sẻ theo dự án, phòng ban hoặc cấp độ truy cập**.

*Tiêu chí chấp nhận:*
- Tôi có thể tạo, đổi tên và xóa bộ sưu tập.
- Tôi có thể gán người dùng hoặc nhóm cụ thể cho từng bộ sưu tập với quyền chỉ đọc hoặc toàn quyền.
- Một mục kho có thể thuộc một hoặc nhiều bộ sưu tập.

---

**UR-ORG-005**: Là một **chủ hoặc quản trị viên tổ chức**, tôi muốn **tạo các nhóm người dùng** để tôi có thể **quản lý quyền truy cập bộ sưu tập cho nhiều thành viên cùng một lúc**.

*Tiêu chí chấp nhận:*
- Tôi có thể tạo nhóm và thêm/xóa thành viên.
- Tôi có thể gán quyền truy cập bộ sưu tập cho nhóm thay vì từng thành viên riêng lẻ.
- Thay đổi thành viên trong nhóm tự động được phản chiếu trong quyền truy cập bộ sưu tập.

---

**UR-ORG-006**: Là một **chủ hoặc quản trị viên tổ chức**, tôi muốn **thu hồi quyền truy cập của thành viên** để **nhân viên hoặc nhà thầu rời đi ngay lập tức mất quyền truy cập**.

*Tiêu chí chấp nhận:*
- Thu hồi trạng thái thành viên ngay lập tức ngăn họ truy cập các bộ sưu tập tổ chức.
- Trạng thái của thành viên bị thu hồi có thể được khôi phục mà không mất gán vai trò/bộ sưu tập.

---

**UR-ORG-007**: Là một **chủ hoặc quản trị viên tổ chức**, tôi muốn **khôi phục tài khoản của thành viên** (đặt lại mật khẩu) để **nhân viên quên mật khẩu chính không bị khóa vĩnh viễn**.

*Tiêu chí chấp nhận:*
- Tôi có thể bắt đầu đặt lại mật khẩu khôi phục quản trị trên tài khoản của thành viên nếu họ đã đồng ý.
- Thành viên phải xác nhận lại mật khẩu mới khi đăng nhập tiếp theo.

---

### 4.6 Chủ/Quản Trị Tổ Chức — Kiểm Soát Truy Cập & Chính Sách

---

**UR-POLICY-001**: Là một **chủ tổ chức**, tôi muốn **yêu cầu tất cả thành viên sử dụng xác thực hai yếu tố** để **kho chia sẻ của tổ chức đáp ứng các yêu cầu tuân thủ bảo mật**.

*Tiêu chí chấp nhận:*
- Tôi có thể bật chính sách "Yêu cầu 2FA cho tất cả thành viên".
- Thành viên chưa cấu hình 2FA nhận cảnh báo và có thể bị hạn chế cho đến khi tuân thủ.

---

**UR-POLICY-002**: Là một **chủ tổ chức**, tôi muốn **thực thi độ mạnh mật khẩu chính tối thiểu** để **các thành viên sử dụng mật khẩu đáp ứng tiêu chuẩn bảo mật của chúng tôi**.

*Tiêu chí chấp nhận:*
- Tôi có thể đặt điểm phức tạp mật khẩu tối thiểu.
- Thành viên được nhắc cập nhật mật khẩu nếu không đáp ứng yêu cầu.

---

**UR-POLICY-003**: Là một **chủ tổ chức**, tôi muốn **hạn chế thành viên chỉ thuộc về tổ chức này** để **thông tin đăng nhập nhạy cảm không bị trộn với dữ liệu của các tổ chức khác**.

*Tiêu chí chấp nhận:*
- Tôi có thể bật chính sách "Một tổ chức duy nhất".
- Thành viên thuộc nhiều tổ chức được yêu cầu rời khỏi những tổ chức khác trước khi tham gia.

---

**UR-POLICY-004**: Là một **chủ tổ chức**, tôi muốn **tạo khóa API tổ chức** để **các quy trình tự động (đường ống CI/CD, tập lệnh triển khai) có thể truy cập các mục kho chia sẻ một cách bảo mật**.

*Tiêu chí chấp nhận:*
- Tôi có thể tạo và thu hồi khóa API tổ chức từ cài đặt quản trị.
- Khóa API cung cấp quyền truy cập trong phạm vi kho mật khẩu của tổ chức.

---

### 4.7 Chủ/Quản Trị Tổ Chức — Kiểm Toán & Tuân Thủ

---

**UR-AUDIT-001**: Là một **chủ hoặc quản trị viên tổ chức**, tôi muốn **xem nhật ký tất cả các hoạt động được thực hiện trong tổ chức** để tôi có thể **giám sát hoạt động cho mục đích bảo mật và tuân thủ**.

*Tiêu chí chấp nhận:*
- Nhật ký sự kiện ghi lại: ai đã thực hiện hành động, hành động nào được thực hiện, mục nào bị ảnh hưởng, khi nào nó xảy ra và từ IP/thiết bị nào.
- Sự kiện bao gồm: đăng nhập thành viên, tạo/cập nhật/xóa mục kho, thay đổi bộ sưu tập, mời thành viên, thay đổi vai trò và sửa đổi chính sách.

---

**UR-AUDIT-002**: Là một **chủ tổ chức**, tôi muốn **lưu trữ nhật ký kiểm toán trong khoảng thời gian có thể cấu hình** để tôi có thể **đáp ứng các yêu cầu quy định hoặc tuân thủ nội bộ**.

*Tiêu chí chấp nhận:*
- Quản trị viên có thể cấu hình thời gian lưu giữ nhật ký sự kiện trước khi tự động dọn dẹp.

---

**UR-AUDIT-003**: Là một **chủ hoặc quản trị viên tổ chức**, tôi muốn **xuất nhật ký sự kiện** để tôi có thể **lưu trữ hoặc phân tích sự kiện trong các công cụ bên ngoài**.

*Tiêu chí chấp nhận:*
- Dữ liệu sự kiện có thể truy cập qua điểm cuối API sự kiện của tổ chức.
- Dữ liệu xuất bao gồm tất cả các trường đã thu thập (người dùng, hành động, dấu thời gian, IP).

---

### 4.8 Quản Trị Viên Máy Chủ — Quản Lý Phiên Bản

---

**UR-ADMIN-001**: Là một **quản trị viên máy chủ**, tôi muốn **triển khai Vaultwarden như một container Docker** để tôi có thể **nhanh chóng thiết lập một phiên bản bảo mật, cô lập mà không cần cài đặt phức tạp**.

*Tiêu chí chấp nhận:*
- Một lệnh `docker run` đơn với tên miền và volume dữ liệu được cấu hình là đủ để khởi động máy chủ.
- Máy chủ có thể truy cập từ các ứng dụng khách Bitwarden chính thức ngay sau khi khởi động.
- Di chuyển cơ sở dữ liệu được áp dụng tự động khi chạy lần đầu.

---

**UR-ADMIN-002**: Là một **quản trị viên máy chủ**, tôi muốn **truy cập bảng quản trị dựa trên web** để tôi có thể **quản lý người dùng, xem trạng thái máy chủ và thay đổi cài đặt mà không cần dùng dòng lệnh**.

*Tiêu chí chấp nhận:*
- Bảng quản trị có thể truy cập tại `/admin`.
- Truy cập yêu cầu token quản trị bảo mật (được băm Argon2id).
- Tôi có thể xem tất cả người dùng đã đăng ký, mời người dùng và xem chẩn đoán máy chủ.

---

**UR-ADMIN-003**: Là một **quản trị viên máy chủ**, tôi muốn **cấu hình hoàn toàn máy chủ qua biến môi trường** để tôi có thể **tích hợp Vaultwarden vào tự động hóa cơ sở hạ tầng hiện có (Docker Compose, Kubernetes, Ansible)**.

*Tiêu chí chấp nhận:*
- Tất cả tùy chọn cấu hình (URL cơ sở dữ liệu, SMTP, tên miền, tính năng) đều có sẵn dưới dạng biến môi trường.
- Thay đổi cấu hình thực hiện qua bảng quản trị được lưu trong `config.json`.
- Biến môi trường luôn ưu tiên hơn giá trị `config.json`.

---

**UR-ADMIN-004**: Là một **quản trị viên máy chủ**, tôi muốn **sao lưu cơ sở dữ liệu** để tôi có thể **khôi phục dữ liệu trong trường hợp máy chủ gặp sự cố**.

*Tiêu chí chấp nhận:*
- Tôi có thể kích hoạt sao lưu SQLite qua lệnh CLI (`vaultwarden backup`), tín hiệu Unix (`SIGUSR1`) hoặc lịch cron tự động.
- Sao lưu tạo ảnh chụp nhất quán của cơ sở dữ liệu.
- *(Triển khai PostgreSQL/MySQL phụ thuộc vào công cụ sao lưu DB bên ngoài.)*

---

**UR-ADMIN-005**: Là một **quản trị viên máy chủ**, tôi muốn **quản lý đăng ký người dùng** để tôi có thể **kiểm soát ai có thể tạo tài khoản trên máy chủ của mình**.

*Tiêu chí chấp nhận:*
- Tôi có thể hạn chế đăng ký chỉ theo chế độ lời mời.
- Tôi có thể hạn chế đăng ký với các tên miền email cụ thể.
- Tôi có thể yêu cầu xác minh email trước khi tài khoản hoạt động.
- Tôi có thể mời người dùng thủ công từ bảng quản trị.

---

**UR-ADMIN-006**: Là một **quản trị viên máy chủ**, tôi muốn **vô hiệu hóa tài khoản người dùng** để tôi có thể **ngay lập tức ngăn truy cập của người dùng bị đình chỉ mà không xóa dữ liệu của họ**.

*Tiêu chí chấp nhận:*
- Tôi có thể bật hoặc tắt bất kỳ tài khoản người dùng nào từ bảng quản trị.
- Người dùng bị tắt không thể đăng nhập cho đến khi tài khoản được bật lại.

---

**UR-ADMIN-007**: Là một **quản trị viên máy chủ**, tôi muốn **chọn giữa SQLite, PostgreSQL và MySQL/MariaDB** để tôi có thể **sử dụng cơ sở dữ liệu phù hợp nhất với cơ sở hạ tầng của mình**.

*Tiêu chí chấp nhận:*
- Tôi có thể cấu hình cơ sở dữ liệu qua biến môi trường `DATABASE_URL`.
- SQLite là mặc định cho triển khai đơn giản một người dùng hoặc homelab.
- PostgreSQL có sẵn cho triển khai sản xuất nhiều người dùng hoặc có tính khả dụng cao.

---

**UR-ADMIN-008**: Là một **quản trị viên máy chủ**, tôi muốn **lưu trữ tệp đính kèm và tệp Send trên lưu trữ đối tượng tương thích S3** để tôi có thể **mở rộng lưu trữ tệp độc lập với máy chủ**.

*Tiêu chí chấp nhận:*
- Tôi có thể cấu hình backend tương thích S3 qua biến môi trường.
- Các thao tác đọc/ghi tệp được định tuyến trong suốt đến backend lưu trữ đã cấu hình.
- Lưu trữ hệ thống tệp cục bộ được sử dụng mặc định nếu S3 chưa được cấu hình.

---

### 4.9 Quản Trị Viên Máy Chủ — Cấu Hình & Tích Hợp

---

**UR-ADMIN-010**: Là một **quản trị viên máy chủ**, tôi muốn **cấu hình giao hàng email SMTP** để người dùng của tôi có thể **nhận email xác minh, cảnh báo 2FA và thông báo**.

*Tiêu chí chấp nhận:*
- Tôi có thể đặt máy chủ SMTP, cổng, thông tin đăng nhập và địa chỉ người gửi qua biến môi trường.
- Máy chủ gửi email cho: lời mời tài khoản, xác minh email, cảnh báo 2FA, truy cập khẩn cấp và xóa tài khoản.
- STARTTLS và TLS đều được hỗ trợ.

---

**UR-ADMIN-011**: Là một **quản trị viên máy chủ**, tôi muốn **bật SSO qua Nhà cung cấp danh tính bên ngoài** để người dùng của tôi có thể **đăng nhập bằng thông tin đăng nhập công ty (ví dụ: Okta, Azure AD, Google Workspace)**.

*Tiêu chí chấp nhận:*
- Tôi có thể cấu hình nhà cung cấp OIDC qua `SSO_AUTHORITY`, `SSO_CLIENT_ID` và `SSO_CLIENT_SECRET`.
- Đăng nhập SSO tuân theo luồng mã ủy quyền OIDC tiêu chuẩn với PKCE.
- Người dùng mới có thể được tự động tạo khi đăng nhập SSO lần đầu.
- SSO có thể cùng tồn tại với đăng nhập tên người dùng/mật khẩu tiêu chuẩn.

---

**UR-ADMIN-012**: Là một **quản trị viên máy chủ**, tôi muốn **bật thông báo push di động** để ứng dụng di động của người dùng **đồng bộ theo thời gian thực mà không cần làm mới thủ công**.

*Tiêu chí chấp nhận:*
- Tôi có thể cấu hình URI relay push để xử lý phân phối APNs và FCM.
- Sự kiện push được kích hoạt cho cùng loại thay đổi kho như thông báo WebSocket.
- Push có thể bị tắt nếu không cần thiết.

---

**UR-ADMIN-013**: Là một **quản trị viên máy chủ**, tôi muốn **bật đồng bộ thời gian thực qua WebSocket** để **ứng dụng khách Bitwarden của người dùng nhận cập nhật kho tức thì trên tất cả các phiên đang mở**.

*Tiêu chí chấp nhận:*
- Tôi có thể bật hỗ trợ WebSocket qua `ENABLE_WEBSOCKET=true`.
- WebSocket bị tắt mặc định và phải được bật rõ ràng.
- Nhiều thiết bị mỗi người dùng được hỗ trợ đồng thời.

---

**UR-ADMIN-014**: Là một **quản trị viên máy chủ**, tôi muốn **cấu hình ghi nhật ký có cấu trúc** để tôi có thể **khắc phục sự cố và tích hợp với hệ thống tổng hợp nhật ký**.

*Tiêu chí chấp nhận:*
- Tôi có thể đặt cấp độ nhật ký, đường dẫn tệp nhật ký, định dạng dấu thời gian và bật ghi nhật ký mở rộng.
- Các giá trị nhạy cảm (mật khẩu, token) bị che giấu trong tất cả đầu ra nhật ký.
- Ghi nhật ký query SQL có thể được bật để gỡ lỗi tương tác cơ sở dữ liệu.

---

**UR-ADMIN-015**: Là một **quản trị viên máy chủ**, tôi muốn **vô hiệu hóa các tính năng không cần thiết** (ví dụ: Send, web vault, WebSocket) để tôi có thể **giảm bề mặt tấn công của triển khai**.

*Tiêu chí chấp nhận:*
- Tôi có thể vô hiệu hóa Bitwarden Send qua `SENDS_ALLOWED=false`.
- Tôi có thể vô hiệu hóa web vault qua `WEB_VAULT_ENABLED=false`.
- Tôi có thể vô hiệu hóa kiểm tra token bảng quản trị qua `DISABLE_ADMIN_TOKEN` (cho môi trường sử dụng xác thực bên ngoài).
- Tôi có thể vô hiệu hóa proxy favicon một cách độc lập.

---

## 5. Nhu Cầu Người Dùng Xuyên Suốt

### 5.1 Bảo Mật & Quyền Riêng Tư

**UR-SEC-001**: Là **bất kỳ người dùng nào**, tôi muốn dữ liệu kho mật khẩu của mình được **mã hóa đầu cuối** để **người vận hành máy chủ không bao giờ có thể đọc mật khẩu hay thông tin cá nhân của tôi**.

> *Máy chủ chỉ lưu trữ các khối đã mã hóa. Mã hóa và giải mã xảy ra hoàn toàn ở phía máy khách bằng khóa dẫn xuất từ mật khẩu chính. Máy chủ không có quyền truy cập vào dữ liệu dạng văn bản thuần túy bất kỳ lúc nào.*

**UR-SEC-002**: Là **bất kỳ người dùng nào**, tôi muốn **các thao tác nhạy cảm yêu cầu xác nhận lại** để **một phiên đăng nhập để mở không thể sử dụng để xuất dữ liệu hay tắt 2FA của tôi**.

> *Xác thực lại bắt buộc (mật khẩu chính hoặc OTP email) trước: tắt 2FA, xuất kho hoặc thực hiện các thao tác rủi ro cao khác.*

**UR-SEC-003**: Là **bất kỳ người dùng nào**, tôi muốn được **bảo vệ chống lại các cuộc tấn công brute-force** để **đoán tự động mật khẩu chính của tôi không khả thi**.

> *Các điểm cuối đăng nhập, 2FA và đăng ký bị giới hạn tốc độ theo địa chỉ IP.*

**UR-SEC-004**: Là **bất kỳ người dùng nào**, tôi muốn tin tưởng rằng **phần mềm máy chủ không có đường dẫn mã unsafe có chủ đích** để tôi có thể **triển khai nó với sự tự tin**.

> *Codebase Rust bắt buộc `#![forbid(unsafe_code)]` ở cấp độ trình biên dịch.*

---

### 5.2 Đa Thiết Bị & Đồng Bộ Thời Gian Thực

**UR-SYNC-001**: Là **bất kỳ người dùng nào**, tôi muốn **thay đổi kho mật khẩu xuất hiện trên tất cả các thiết bị của mình tự động** để tôi **không bao giờ làm việc với dữ liệu cũ**.

> *Đồng bộ thời gian thực được cung cấp qua WebSocket (khi bật) và relay push di động. Tất cả ứng dụng khách Bitwarden hỗ trợ đồng bộ nền tự động.*

**UR-SYNC-002**: Là **bất kỳ người dùng nào**, tôi muốn **sử dụng nhiều thiết bị đồng thời** mà không xung đột đồng bộ để kho mật khẩu của tôi **vẫn nhất quán**.

> *Mỗi thiết bị duy trì phiên xác thực riêng. Máy chủ áp dụng thay đổi tuần tự và truyền cập nhật đến tất cả các phiên kết nối.*

---

### 5.3 Xác Thực Hai Yếu Tố

**UR-MFA-001**: Là **bất kỳ người dùng nào**, tôi muốn **nhiều tùy chọn 2FA** để tôi có thể **chọn phương pháp phù hợp nhất với thế trận bảo mật và phần cứng sẵn có của mình**.

| Phương pháp | Khi nào sử dụng |
|------------|----------------|
| TOTP (Ứng dụng xác thực) | Cân bằng tốt nhất giữa bảo mật và thuận tiện |
| Email OTP | Dự phòng khi không có ứng dụng xác thực |
| FIDO2 / WebAuthn | Bảo mật cao nhất (khóa phần cứng chống lừa đảo) |
| YubiKey OTP | Khóa phần cứng có hỗ trợ OTP |
| Duo Security | 2FA doanh nghiệp với phê duyệt push |
| Mã khôi phục | Dự phòng khẩn cấp khi mất thiết bị 2FA chính |

**UR-MFA-002**: Là **bất kỳ người dùng nào**, tôi muốn **ghi nhớ thiết bị tin cậy** để tôi **không phải nhập 2FA mỗi lần đăng nhập trên những thiết bị mình sở hữu**.

**UR-MFA-003**: Là **bất kỳ người dùng nào**, tôi muốn có **mã khôi phục** để tôi có thể **lấy lại quyền truy cập vào tài khoản nếu mất thiết bị 2FA**.

---

### 5.4 Khả Năng Sử Dụng & Tương Thích Ứng Dụng Khách

**UR-COMPAT-001**: Là **bất kỳ người dùng nào**, tôi muốn **sử dụng các ứng dụng khách Bitwarden tiêu chuẩn mà không cần cấu hình đặc biệt** để tôi có thể **hưởng lợi từ Vaultwarden với thiết lập tối thiểu**.

> *Vaultwarden hoàn toàn tương thích API với các ứng dụng khách Bitwarden chính thức. Người dùng chỉ cần hướng ứng dụng khách đến URL máy chủ Vaultwarden.*

**UR-COMPAT-002**: Là **bất kỳ người dùng nào**, tôi muốn máy chủ **vẫn tương thích với các bản cập nhật ứng dụng khách Bitwarden trong tương lai** để tôi **không gặp sự cố khi ứng dụng khách tự động cập nhật**.

---

## 6. Ràng Buộc & Kỳ Vọng Người Dùng

| # | Ràng buộc | Tác động với người dùng |
|---|----------|------------------------|
| UC-01 | Máy chủ phải được đặt sau reverse proxy (nginx/Caddy) xử lý HTTPS. | Người dùng phải truy cập Vaultwarden qua HTTPS; HTTP thuần túy không được hỗ trợ. |
| UC-02 | Tất cả mã hóa là phía máy khách; máy chủ là kho lưu trữ mã hóa mù. | Nếu người dùng quên mật khẩu chính và không có cơ chế khôi phục, dữ liệu kho không thể phục hồi. |
| UC-03 | Kích thước tải lên tệp bị giới hạn ở mức 525 MB mỗi lần tải lên. | Các tệp đính kèm rất lớn cần được nén hoặc phân chia trước khi tải lên. |
| UC-04 | Thông báo WebSocket phải được quản trị viên máy chủ bật rõ ràng. | Nếu không bật, máy khách sẽ đồng bộ theo khoảng thời gian thăm dò thay vì thời gian thực. |
| UC-05 | Thông báo push yêu cầu máy chủ relay bên ngoài. | Quản trị viên máy chủ phải cấu hình relay; đồng bộ di động có thể bị trễ nếu không có nó. |
| UC-06 | SSO yêu cầu Nhà cung cấp danh tính bên ngoài được cấu hình bởi quản trị viên máy chủ. | SSO không phải tự phục vụ cho người dùng cuối; phải được thiết lập ở cấp độ cơ sở hạ tầng. |
| UC-07 | Lưu trữ tệp S3 yêu cầu tính năng biên dịch. | Image Docker phải bao gồm hỗ trợ S3 nếu muốn dùng lưu trữ đối tượng. |
| UC-08 | Máy chủ được cấp phép theo AGPL-3.0. | Các tổ chức sửa đổi và triển khai Vaultwarden phải công bố các sửa đổi của họ. |

---

## 7. Tóm Tắt Tiêu Chí Chấp Nhận

Bảng sau tóm tắt các tiêu chí chấp nhận ở mức cao để xác nhận yêu cầu người dùng:

| Mã user story | Tính năng | Tín hiệu chấp nhận |
|--------------|---------|-------------------|
| UR-USER-001 | Đăng ký | Tài khoản mới được kích hoạt; email được xác minh nếu cần |
| UR-USER-002 | Đăng nhập đa ứng dụng khách | Tất cả ứng dụng khách Bitwarden chính thức xác thực thành công |
| UR-USER-003 | CRUD mục kho | Mục được tạo/chỉnh sửa/xóa và phản chiếu trong đồng bộ |
| UR-USER-012 | Thiết lập 2FA | 2FA được bật; đăng nhập yêu cầu yếu tố thứ hai |
| UR-USER-013 | Đăng nhập không mật khẩu | Luồng phê duyệt thiết bị hoàn thành; phiên được thiết lập |
| UR-SEND-001 | Tạo Send | Người nhận truy cập Send qua URL mà không cần tài khoản |
| UR-SEND-002 | Send có bảo vệ mật khẩu | Truy cập bị từ chối mà không có mật khẩu đúng |
| UR-EMRG-001 | Ủy quyền truy cập khẩn cấp | Người được ủy quyền có thể yêu cầu và nhận quyền truy cập sau thời gian chờ |
| UR-ORG-001 | Tạo tổ chức | Tổ chức được tạo; chủ sở hữu có thể mời thành viên |
| UR-ORG-004 | Bộ sưu tập | Mục được gán vào bộ sưu tập; quyền truy cập được thực thi theo vai trò |
| UR-POLICY-001 | Chính sách bắt buộc 2FA | Thành viên không có 2FA nhận cảnh báo/hạn chế |
| UR-AUDIT-001 | Nhật ký sự kiện | Tất cả hành động tổ chức được ghi với metadata đúng |
| UR-ADMIN-001 | Triển khai Docker | Máy chủ khởi động; ứng dụng khách kết nối; di chuyển được áp dụng |
| UR-ADMIN-002 | Bảng quản trị | Quản trị viên có thể quản lý người dùng và xem chẩn đoán |
| UR-ADMIN-010 | Email SMTP | Email xác minh/thông báo được giao thành công |
| UR-ADMIN-011 | SSO/OIDC | Người dùng có thể đăng nhập qua IdP đã cấu hình |

---

## 8. Bảng Thuật Ngữ

| Thuật ngữ | Định nghĩa thông thường |
|----------|------------------------|
| **2FA / MFA** | Bước xác minh thứ hai bắt buộc khi đăng nhập ngoài mật khẩu của bạn |
| **Bảng quản trị** | Giao diện web chỉ quản trị viên máy chủ có thể truy cập, tại `/admin` |
| **AES-256** | Thuật toán mã hóa đối xứng tiêu chuẩn công nghiệp để bảo vệ dữ liệu kho |
| **Argon2id** | Thuật toán băm mật khẩu bảo mật dùng để bảo vệ token quản trị |
| **Cipher (mục kho)** | Một mục duy nhất trong kho mật khẩu (đăng nhập, thẻ, ghi chú, danh tính hoặc khóa SSH) |
| **Bộ sưu tập** | Thư mục thuộc về tổ chức và có thể chia sẻ với nhiều thành viên |
| **Truy cập khẩn cấp** | Tính năng cho phép bạn chỉ định người tin cậy để truy cập kho trong trường hợp khẩn cấp |
| **Mã hóa đầu cuối (E2EE)** | Mã hóa chỉ người dùng (không phải máy chủ) mới có thể giải mã dữ liệu |
| **FIDO2 / WebAuthn** | Tiêu chuẩn cho khóa bảo mật phần cứng chống lừa đảo (ví dụ: YubiKey) |
| **Người được ủy quyền** | Người được chỉ định để nhận quyền truy cập khẩn cấp vào kho |
| **Người ủy quyền** | Chủ kho cấp quyền truy cập khẩn cấp cho người được ủy quyền |
| **Nhóm** | Tập hợp thành viên tổ chức có tên để gán quyền truy cập bộ sưu tập hàng loạt |
| **Nhà cung cấp danh tính (IdP)** | Dịch vụ bên ngoài xác thực người dùng cho SSO (ví dụ: Okta, Azure AD) |
| **Mật khẩu chính** | Mật khẩu chính dùng để dẫn xuất khóa mã hóa — không bao giờ được gửi đến hoặc lưu trữ bởi máy chủ |
| **OIDC** | OpenID Connect — giao thức dùng cho tích hợp Đăng nhập một lần |
| **OpenDAL** | Lớp trừu tượng lưu trữ tệp dùng bởi Vaultwarden (hỗ trợ đĩa cục bộ và S3) |
| **Tổ chức** | Không gian làm việc chia sẻ trên Vaultwarden để nhóm cộng tác trên các mục kho |
| **PKCE** | Phần mở rộng bảo mật cho OAuth/OIDC ngăn chặn chặn mã ủy quyền |
| **Thông báo push** | Thông báo thời gian thực gửi đến ứng dụng di động để kích hoạt đồng bộ kho |
| **Giới hạn tốc độ** | Hạn chế tự động các lần thử đăng nhập thất bại để ngăn tấn công brute-force |
| **Mã khôi phục** | Mã sử dụng một lần để truy cập tài khoản khi phương pháp 2FA chính không khả dụng |
| **Reverse proxy** | Máy chủ (nginx, Caddy) đứng trước Vaultwarden và xử lý HTTPS |
| **Vai trò** | Cấp độ quyền được xác định trong tổ chức (Chủ sở hữu, Quản trị viên, Quản lý, Người dùng, Tùy chỉnh) |
| **S3** | Amazon S3 hoặc bất kỳ dịch vụ lưu trữ đối tượng tương thích S3 nào (ví dụ: MinIO) |
| **Send** | Tính năng chia sẻ văn bản hoặc tệp được mã hóa qua liên kết bảo mật một lần |
| **Phiên** | Đăng nhập xác thực trên một thiết bị; phiên hết hạn sau một khoảng thời gian đặt sẵn |
| **Đăng nhập một lần (SSO)** | Đăng nhập bằng tài khoản danh tính công ty thay vì mật khẩu Vaultwarden trực tiếp |
| **SQLite / PostgreSQL / MySQL** | Các backend cơ sở dữ liệu được hỗ trợ bởi Vaultwarden |
| **TOTP** | Mật khẩu một lần dựa trên thời gian — mã 6 chữ số được tạo bởi ứng dụng xác thực |
| **Kho mật khẩu** | Bộ sưu tập mật khẩu và các mục nhạy cảm khác của bạn, được mã hóa |
| **WebSocket** | Công nghệ cho phép giao tiếp hai chiều thời gian thực giữa máy chủ và máy khách để đồng bộ tức thì |
| **YubiKey** | Token bảo mật phần cứng vật lý dùng để xác thực 2FA |

---

*Hết tài liệu*
