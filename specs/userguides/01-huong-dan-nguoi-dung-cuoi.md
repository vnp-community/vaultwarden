# Hướng Dẫn Sử Dụng Vaultwarden — Dành Cho Người Dùng Cuối

> **Đối tượng**: Người dùng cá nhân sử dụng kho mật khẩu  
> **Phiên bản**: 1.0 | **Ngày**: 2026-04-10  
> **Ứng dụng khách**: Bitwarden (Web Vault, Desktop, Di động, Tiện ích mở rộng trình duyệt)

---

## Mục Lục

1. [Bắt đầu — Tạo tài khoản & Đăng nhập](#1-bắt-đầu--tạo-tài-khoản--đăng-nhập)
2. [Quản lý kho mật khẩu](#2-quản-lý-kho-mật-khẩu)
3. [Tổ chức mật khẩu bằng Thư mục](#3-tổ-chức-mật-khẩu-bằng-thư-mục)
4. [Tệp đính kèm](#4-tệp-đính-kèm)
5. [Thùng rác & Khôi phục](#5-thùng-rác--khôi-phục)
6. [Bảo mật tài khoản](#6-bảo-mật-tài-khoản)
7. [Xác thực hai yếu tố (2FA)](#7-xác-thực-hai-yếu-tố-2fa)
8. [Đăng nhập không mật khẩu (Passkey / AuthRequest)](#8-đăng-nhập-không-mật-khẩu-passkey--authrequest)
9. [Chia sẻ bảo mật với Bitwarden Send](#9-chia-sẻ-bảo-mật-với-bitwarden-send)
10. [Truy cập khẩn cấp](#10-truy-cập-khẩn-cấp)
11. [Cài đặt tài khoản](#11-cài-đặt-tài-khoản)
12. [Câu hỏi thường gặp](#12-câu-hỏi-thường-gặp)

---

## 1. Bắt Đầu — Tạo Tài Khoản & Đăng Nhập

### 1.1 Cài đặt ứng dụng khách

Vaultwarden hoạt động với **tất cả ứng dụng Bitwarden chính thức**. Tải xuống ứng dụng phù hợp:

| Loại ứng dụng | Nơi tải |
|--------------|---------|
| **Tiện ích mở rộng trình duyệt** | Chrome Web Store / Firefox Add-ons |
| **Ứng dụng Desktop** | [bitwarden.com/download](https://bitwarden.com/download/) |
| **Ứng dụng Di động (iOS)** | App Store |
| **Ứng dụng Di động (Android)** | Google Play Store |
| **Web Vault** | Truy cập trực tiếp qua URL máy chủ |

### 1.2 Kết nối đến máy chủ Vaultwarden

> ⚠️ **Quan trọng**: Bạn phải trỏ ứng dụng Bitwarden đến URL của máy chủ Vaultwarden — **không phải** `bitwarden.com`.

**Các bước thiết lập:**

1. Mở ứng dụng Bitwarden.
2. Nhấp vào biểu tượng **⚙️** hoặc menu **Cài đặt** (góc trên màn hình đăng nhập).
3. Chọn **Self-hosted** (Tự lưu trữ).
4. Nhập URL máy chủ Vaultwarden, ví dụ: `https://vault.cty-ban.com`
5. Nhấp **Lưu**.

### 1.3 Tạo tài khoản

1. Trên màn hình đăng nhập, nhấp **Tạo tài khoản**.
2. Nhập:
   - **Email**: Địa chỉ email của bạn.
   - **Tên hiển thị** (tùy chọn).
   - **Mật khẩu chính**: Mật khẩu bạn sẽ dùng để mở kho. **Hãy ghi nhớ — nếu quên, không thể khôi phục dữ liệu!**
   - **Xác nhận mật khẩu chính**.
   - **Gợi ý mật khẩu** (tùy chọn, không nên ghi rõ mật khẩu).
3. Nhấp **Tạo tài khoản**.
4. Nếu máy chủ yêu cầu xác minh email, hãy kiểm tra hộp thư và nhấp vào liên kết xác minh.

> 💡 **Lưu ý bảo mật**: Mật khẩu chính **không bao giờ** được gửi đến máy chủ. Máy chủ chỉ lưu trữ dữ liệu đã được mã hóa. Không ai — kể cả quản trị viên — có thể đọc dữ liệu kho của bạn.

### 1.4 Đăng nhập

1. Nhập địa chỉ email và mật khẩu chính.
2. Nhấp **Đăng nhập**.
3. Nếu đã bật 2FA, hoàn thành bước xác minh thứ hai.

**Thời gian phiên đăng nhập:**
- **Trình duyệt & Desktop**: Token làm mới có hiệu lực 30 ngày.
- **Di động**: Token làm mới có hiệu lực 90 ngày.

---

## 2. Quản Lý Kho Mật Khẩu

### 2.1 Các loại mục trong kho

| Loại | Dùng để lưu |
|------|------------|
| 🔑 **Đăng nhập** | Tên người dùng, mật khẩu, URL website |
| 📝 **Ghi chú bảo mật** | Văn bản bí mật tự do (mã PIN, khóa API, thông tin quan trọng) |
| 💳 **Thẻ tín dụng** | Số thẻ, ngày hết hạn, CVV |
| 🪪 **Danh tính** | Họ tên, ngày sinh, địa chỉ, hộ chiếu |
| 🔐 **Khóa SSH** | Khóa SSH công khai/riêng tư |

### 2.2 Thêm mục mới

1. Nhấp nút **➕ Mục mới** (hoặc dấu **+** trên di động).
2. Chọn loại mục.
3. Điền thông tin:
   - **Tên**: Tên mô tả (ví dụ: "Gmail cá nhân").
   - **Tên người dùng / Email**.
   - **Mật khẩu**: Nhập thủ công hoặc nhấp 🎲 để tạo mật khẩu mạnh tự động.
   - **URL**: Địa chỉ trang web (dùng cho tự điền).
   - **Ghi chú**: Thông tin thêm.
   - **Trường tùy chỉnh**: Thêm trường riêng (text, ẩn, boolean, liên kết).
4. Nhấp **Lưu**.

### 2.3 Sửa và xem mục

- Nhấp vào tên mục để xem chi tiết.
- Nhấp biểu tượng **✏️ Chỉnh sửa** để sửa.
- Dùng biểu tượng **📋 Sao chép** để sao chép mật khẩu vào clipboard.

### 2.4 Tìm kiếm mục

- Dùng **ô tìm kiếm** phía trên danh sách.
- Tìm kiếm theo: tên mục, tên người dùng, URL, ghi chú.

### 2.5 Lịch sử mật khẩu

Mỗi khi bạn thay đổi mật khẩu của một mục, phiên bản cũ được lưu lại:

1. Mở chi tiết mục.
2. Nhấp **Lịch sử mật khẩu** (hoặc biểu tượng đồng hồ 🕐).
3. Xem các mật khẩu trước đó (tối đa 5 phiên bản).

### 2.6 Mục yêu thích

- Mở chi tiết mục → nhấp biểu tượng ⭐ để đánh dấu yêu thích.
- Các mục yêu thích xuất hiện trong danh mục **Yêu thích** riêng.

### 2.7 Tự điền mật khẩu

Khi dùng **tiện ích mở rộng trình duyệt**:
1. Truy cập trang web.
2. Nhấp vào biểu tượng Bitwarden trên thanh công cụ.
3. Mục phù hợp với URL sẽ hiện ra — nhấp để tự điền.

---

## 3. Tổ Chức Mật Khẩu Bằng Thư Mục

Thư mục là cách **cá nhân** để nhóm các mục kho. Chúng không được chia sẻ với tổ chức.

### 3.1 Tạo thư mục

1. Trong thanh bên trái, nhấp **Thư mục** → **➕ Thư mục mới**.
2. Nhập tên thư mục.
3. Nhấp **Lưu**.

### 3.2 Gán mục vào thư mục

1. Mở hoặc chỉnh sửa một mục kho.
2. Tìm trường **Thư mục** và chọn thư mục phù hợp.
3. Nhấp **Lưu**.

### 3.3 Đổi tên / Xóa thư mục

- Nhấp chuột phải (hoặc nhấn giữ trên di động) vào tên thư mục.
- Chọn **Đổi tên** hoặc **Xóa**.

> 💡 Xóa thư mục **không xóa** các mục bên trong — chúng chỉ không còn thuộc thư mục đó nữa.

---

## 4. Tệp Đính Kèm

Bạn có thể đính kèm tệp vào bất kỳ mục kho nào (tài liệu, hình ảnh, chứng chỉ...). Tệp được **mã hóa** trước khi tải lên máy chủ.

### 4.1 Tải lên tệp đính kèm

1. Mở chi tiết mục kho.
2. Nhấp **Chỉnh sửa** → tìm phần **Tệp đính kèm**.
3. Nhấp **Chọn tệp** và chọn tệp cần đính kèm.
4. Nhấp **Lưu**.

> ⚠️ **Giới hạn**: Mỗi tệp tối đa **500 MB**. Quản trị viên máy chủ có thể giới hạn thêm.

### 4.2 Tải xuống tệp đính kèm

1. Mở chi tiết mục.
2. Trong phần **Tệp đính kèm**, nhấp tên tệp để tải xuống.

---

## 5. Thùng Rác & Khôi Phục

### 5.1 Xóa mục (vào thùng rác)

- Nhấp chuột phải vào mục → **Xóa**.
- Mục được chuyển vào **Thùng rác** — chưa bị xóa vĩnh viễn.

### 5.2 Xem thùng rác

- Trong thanh bên trái, cuộn xuống và nhấp **Thùng rác**.

### 5.3 Khôi phục mục

1. Vào **Thùng rác**.
2. Nhấp chuột phải vào mục → **Khôi phục**.
3. Mục quay về danh sách kho.

### 5.4 Xóa vĩnh viễn

1. Vào **Thùng rác**.
2. Nhấp chuột phải → **Xóa vĩnh viễn**.
3. Xác nhận — **không thể hoàn tác**.

> ⏲️ **Tự động xóa**: Quản trị viên có thể đặt thời gian tự động xóa mục trong thùng rác (ví dụ: sau 30 ngày). Hãy hỏi quản trị viên của bạn về chính sách này.

---

## 6. Bảo Mật Tài Khoản

### 6.1 Thay đổi mật khẩu chính

1. Vào **Cài đặt tài khoản** → **Bảo mật** → **Mật khẩu chính**.
2. Nhập mật khẩu chính **hiện tại**.
3. Nhập và xác nhận mật khẩu chính **mới**.
4. Nhấp **Thay đổi mật khẩu chính**.

> ⚠️ Thay đổi mật khẩu chính sẽ **đăng xuất tất cả các thiết bị khác**. Hãy cập nhật ứng dụng khách trên các thiết bị sau khi thay đổi.

### 6.2 Thay đổi địa chỉ email

1. Vào **Cài đặt tài khoản** → **Thông tin tài khoản**.
2. Nhập địa chỉ email mới.
3. Xác nhận bằng mật khẩu chính.
4. Kiểm tra hộp thư mới và xác nhận thay đổi.

### 6.3 Hành động được bảo vệ

Một số hành động yêu cầu **xác thực lại** bằng mật khẩu chính hoặc OTP email:

- Tắt/thay đổi 2FA.
- Xuất toàn bộ kho mật khẩu.
- Thay đổi mật khẩu chính.

Đây là lớp bảo vệ ngăn kẻ xấu thực hiện thao tác nguy hiểm nếu họ có quyền truy cập vào thiết bị đang đăng nhập của bạn.

### 6.4 Yêu cầu xác nhận lại trên mục nhạy cảm

Để bảo vệ những mục quan trọng:

1. Mở mục kho → **Chỉnh sửa**.
2. Tìm tùy chọn **Yêu cầu xác nhận lại mật khẩu chính**.
3. Bật tùy chọn này.
4. **Lưu**.

Khi bật, mỗi lần xem hoặc sao chép thông tin của mục đó, bạn phải nhập lại mật khẩu chính.

### 6.5 Xóa tài khoản

> ⚠️ **Cẩn thận**: Không thể hoàn tác sau khi xóa!

1. Vào **Cài đặt tài khoản** → **Tài khoản** → cuộn xuống **Xóa tài khoản**.
2. Nhấp **Xóa tài khoản**.
3. Một email xác nhận được gửi đến địa chỉ của bạn.
4. Nhấp liên kết trong email để hoàn tất.

---

## 7. Xác Thực Hai Yếu Tố (2FA)

2FA thêm một lớp bảo mật bổ sung. Ngay cả khi ai đó biết mật khẩu chính của bạn, họ vẫn không thể đăng nhập nếu không có yếu tố thứ hai.

### 7.1 Các phương pháp 2FA được hỗ trợ

| Phương pháp | Mô tả | Mức bảo mật |
|------------|-------|:-----------:|
| **Ứng dụng xác thực (TOTP)** | Google Authenticator, Authy, v.v. | ⭐⭐⭐ |
| **Email OTP** | Mã gửi qua email | ⭐⭐ |
| **Khóa phần cứng (FIDO2/WebAuthn)** | YubiKey, Windows Hello, Touch ID | ⭐⭐⭐⭐⭐ |
| **YubiKey OTP** | YubiKey ở chế độ OTP | ⭐⭐⭐⭐ |
| **Duo Security** | Ứng dụng Duo Mobile (doanh nghiệp) | ⭐⭐⭐⭐ |

### 7.2 Bật 2FA bằng ứng dụng xác thực (TOTP)

1. Vào **Cài đặt tài khoản** → **Bảo mật** → **Xác thực hai bước**.
2. Chọn **Ứng dụng xác thực** → nhấp **Quản lý**.
3. Nhập mật khẩu chính để xác nhận.
4. **Quét mã QR** bằng ứng dụng xác thực (Google Authenticator, Authy...) hoặc nhập thủ công khóa bí mật.
5. Nhập mã 6 chữ số từ ứng dụng xác thực để xác nhận.
6. Nhấp **Bật**.
7. **Lưu mã khôi phục** (được hiển thị một lần duy nhất) vào nơi an toàn!

> 💡 Khuyến nghị: Dùng **Authy** hoặc **1Password** làm ứng dụng xác thực vì chúng hỗ trợ sao lưu đám mây, giúp bạn không bị mất mã khi thay điện thoại.

### 7.3 Bật 2FA bằng khóa phần cứng (FIDO2/WebAuthn)

1. Vào **Cài đặt** → **Xác thực hai bước** → **Khóa bảo mật FIDO2 WebAuthn**.
2. Nhấp **Quản lý** → nhập mật khẩu chính.
3. Cắm YubiKey vào máy tính (hoặc chạm NFC trên di động).
4. Nhấp **Đọc khóa** và làm theo hướng dẫn.
5. Đặt tên cho khóa → nhấp **Lưu**.

### 7.4 Mã khôi phục

> ⚠️ **Bắt buộc lưu lại!** Nếu mất thiết bị 2FA mà không có mã khôi phục, bạn sẽ bị khóa tài khoản vĩnh viễn.

- Sau khi bật 2FA, bạn sẽ nhận được một mã khôi phục dài.
- **In ra giấy** hoặc lưu vào nơi an toàn (không phải trong Vaultwarden 😄).
- Để xem lại: Cài đặt → Xác thực hai bước → **Xem mã khôi phục**.

### 7.5 Thiết bị tin cậy

Để không phải nhập 2FA mỗi lần đăng nhập trên máy bạn thường dùng:

1. Sau khi nhập 2FA thành công, tích chọn **"Ghi nhớ thiết bị này"**.
2. Thiết bị sẽ không hỏi 2FA trong lần đăng nhập tiếp theo.

> 💡 Chỉ dùng tính năng này trên **thiết bị cá nhân** của bạn, không dùng trên máy tính công cộng.

---

## 8. Đăng Nhập Không Mật Khẩu (Passkey / AuthRequest)

Vaultwarden hỗ trợ đăng nhập từ thiết bị mới bằng cách **phê duyệt từ thiết bị đang tin cậy**.

### 8.1 Phê duyệt đăng nhập từ thiết bị mới

1. Trên thiết bị mới, nhập email → nhấp **Đăng nhập**.
2. Chọn **Đăng nhập bằng thiết bị khác**.
3. Một yêu cầu được gửi đến ứng dụng Bitwarden đang mở trên thiết bị tin cậy.
4. Mở ứng dụng trên thiết bị tin cậy → xem thông báo yêu cầu.
5. Nhấp **Phê duyệt** nếu đây là yêu cầu của bạn.
6. Thiết bị mới sẽ được đăng nhập.

> ⚠️ Nếu bạn nhận được yêu cầu không do bạn khởi tạo, hãy nhấp **Từ chối** ngay lập tức — có thể tài khoản đang bị xâm nhập.

---

## 9. Chia Sẻ Bảo Mật Với Bitwarden Send

**Bitwarden Send** cho phép bạn chia sẻ thông tin nhạy cảm với **bất kỳ ai** thông qua liên kết bảo mật — người nhận **không cần tài khoản** Vaultwarden.

### 9.1 Cách hoạt động của Send

- Nội dung được **mã hóa trên thiết bị của bạn** bằng khóa ngẫu nhiên.
- Khóa mã hóa được nhúng vào **phân đoạn URL** (`#...`) và **không bao giờ** gửi đến máy chủ.
- Máy chủ chỉ thấy dữ liệu đã mã hóa — không thể đọc nội dung.

### 9.2 Tạo Send văn bản

1. Trong thanh bên, nhấp **Send** → **➕ Tạo Send**.
2. Chọn loại **Văn bản**.
3. Nhập **Tên** (chỉ để nhận dạng của bạn).
4. Nhập **Nội dung văn bản** cần chia sẻ.
5. Cài đặt tùy chọn:
   - **Ngày hết hạn**: Sau ngày này, Send tự động bị vô hiệu hóa.
   - **Ngày xóa**: Send bị xóa hoàn toàn sau ngày này.
   - **Số lần xem tối đa**: Vô hiệu hóa sau N lần truy cập.
   - **Mật khẩu**: Người nhận cần nhập mật khẩu để xem.
   - **Ẩn email**: Ẩn địa chỉ email của bạn với người nhận.
6. Nhấp **Lưu**.
7. **Sao chép liên kết** và gửi cho người nhận.

### 9.3 Tạo Send tệp

1. Chọn loại **Tệp** khi tạo Send.
2. Nhấp **Chọn tệp** để tải tệp lên (tối đa 500 MB).
3. Cài đặt tương tự Send văn bản.
4. Nhấp **Lưu** và chia sẻ liên kết.

### 9.4 Quản lý Send đã tạo

- Trong mục **Send**, bạn thấy danh sách tất cả Send đã tạo.
- Nhấp vào Send để chỉnh sửa hoặc sao chép liên kết.
- Nhấp **🚫 Vô hiệu hóa** để thu hồi quyền truy cập ngay lập tức mà không xóa Send.
- Nhấp **🗑️ Xóa** để xóa hoàn toàn.

---

## 10. Truy Cập Khẩn Cấp

Tính năng này cho phép bạn chỉ định người tin cậy có thể truy cập kho mật khẩu của bạn trong trường hợp khẩn cấp.

### 10.1 Ủy quyền truy cập khẩn cấp

1. Vào **Cài đặt tài khoản** → **Truy cập khẩn cấp**.
2. Nhấp **➕ Thêm người liên hệ khẩn cấp**.
3. Nhập email của người được ủy quyền (họ phải có tài khoản Vaultwarden).
4. Chọn **Loại truy cập**:
   - **Xem**: Người được ủy quyền chỉ có thể đọc các mục kho.
   - **Tiếp quản**: Người được ủy quyền có thể đặt lại mật khẩu và chiếm quyền toàn bộ tài khoản.
5. Đặt **Thời gian chờ** (ngày): Thời gian chờ trước khi yêu cầu tự động được phê duyệt (ví dụ: 7 ngày).
6. Nhấp **Lưu** — người được ủy quyền sẽ nhận email mời.

### 10.2 Duyệt/từ chối yêu cầu khẩn cấp

Khi người được ủy quyền gửi yêu cầu, bạn nhận được email thông báo.

Để **từ chối**:
1. Vào **Cài đặt** → **Truy cập khẩn cấp**.
2. Nhấp **Từ chối** trên yêu cầu đang chờ.

Nếu không phản hồi trong thời gian chờ, yêu cầu **tự động được phê duyệt**.

---

## 11. Cài Đặt Tài Khoản

### 11.1 Tạo khóa API cá nhân (CLI/Automation)

1. Vào **Cài đặt tài khoản** → **Khóa API**.
2. Nhấp **Xem khóa API** → xác nhận mật khẩu.
3. Sao chép `client_id` và `client_secret` để dùng với Bitwarden CLI.

### 11.2 Gợi ý mật khẩu chính

Nếu quên mật khẩu chính và đã đặt gợi ý khi đăng ký:

1. Trên màn hình đăng nhập, nhấp **Quên mật khẩu?** → **Nhận gợi ý**.
2. Gợi ý sẽ được gửi qua email.

### 11.3 Tắt tài khoản khẩn cấp (khi bị đánh cắp thiết bị)

1. Đăng nhập từ thiết bị khác ngay lập tức.
2. Vào **Cài đặt** → **Phiên đang hoạt động** → **Đăng xuất tất cả thiết bị**.
3. Đổi mật khẩu chính ngay.

---

## 12. Câu Hỏi Thường Gặp

### ❓ Tôi quên mật khẩu chính — có thể khôi phục không?

**Không.** Mật khẩu chính không bao giờ được gửi đến máy chủ. Máy chủ không thể khôi phục mật khẩu chính. Tuy nhiên:
- Nếu bạn thuộc một **tổ chức** và quản trị viên đã bật **Khôi phục quản trị**, họ có thể giúp đặt lại.
- Nếu bạn đã thiết lập **Truy cập khẩn cấp**, người được ủy quyền loại "Tiếp quản" có thể giúp.

### ❓ Dữ liệu của tôi có an toàn không khi máy chủ bị tấn công?

**Có.** Tất cả dữ liệu kho được mã hóa **trước khi rời thiết bị của bạn**. Ngay cả khi máy chủ bị xâm phạm, kẻ tấn công chỉ thấy các khối mã hóa vô nghĩa.

### ❓ Tôi có thể dùng Vaultwarden với tài khoản Bitwarden.com không?

**Không.** Vaultwarden là máy chủ riêng biệt. Bạn phải chọn một trong hai — không thể kết hợp.

### ❓ Đồng bộ không tự động — phải làm gì?

1. Kiểm tra kết nối internet.
2. Đăng xuất và đăng nhập lại.
3. Hỏi quản trị viên xem WebSocket (`ENABLE_WEBSOCKET`) đã được bật chưa. Nếu chưa, đồng bộ sẽ xảy ra theo chu kỳ thay vì thời gian thực.

### ❓ Tôi có thể xem mật khẩu đã xóa không?

Có — trong **Thùng rác**. Sau khi xóa vĩnh viễn, không thể khôi phục.

---

*Cần hỗ trợ thêm? Liên hệ quản trị viên máy chủ Vaultwarden của tổ chức bạn.*
