# Hướng Dẫn Sử Dụng Vaultwarden — Dành Cho Quản Trị Viên Tổ Chức

> **Đối tượng**: Chủ tổ chức (Owner), Quản trị viên (Admin), Quản lý (Manager)  
> **Phiên bản**: 1.0 | **Ngày**: 2026-04-10  
> **Điều kiện tiên quyết**: Đã có tài khoản, máy chủ Vaultwarden đang hoạt động

---

## Mục Lục

1. [Tổng quan về tổ chức](#1-tổng-quan-về-tổ-chức)
2. [Tạo và cấu hình tổ chức](#2-tạo-và-cấu-hình-tổ-chức)
3. [Quản lý thành viên](#3-quản-lý-thành-viên)
4. [Vai trò và phân quyền](#4-vai-trò-và-phân-quyền)
5. [Quản lý bộ sưu tập (Collections)](#5-quản-lý-bộ-sưu-tập-collections)
6. [Quản lý nhóm người dùng (Groups)](#6-quản-lý-nhóm-người-dùng-groups)
7. [Chia sẻ mật khẩu trong tổ chức](#7-chia-sẻ-mật-khẩu-trong-tổ-chức)
8. [Chính sách bảo mật tổ chức](#8-chính-sách-bảo-mật-tổ-chức)
9. [Nhật ký sự kiện và kiểm toán](#9-nhật-ký-sự-kiện-và-kiểm-toán)
10. [Khôi phục tài khoản thành viên](#10-khôi-phục-tài-khoản-thành-viên)
11. [Khóa API tổ chức](#11-khóa-api-tổ-chức)
12. [Các câu hỏi thường gặp](#12-các-câu-hỏi-thường-gặp)

---

## 1. Tổng Quan Về Tổ Chức

### 1.1 Tổ chức là gì?

**Tổ chức** trong Vaultwarden là một không gian làm việc chia sẻ, cho phép các thành viên nhóm chia sẻ mật khẩu và thông tin nhạy cảm một cách **bảo mật, có kiểm soát** và **có thể kiểm toán**.

**Kiến trúc phân cấp:**

```
Tổ chức
  ├── Chủ sở hữu (Owner)
  ├── Quản trị viên (Admin)
  ├── Bộ sưu tập (Collection)
  │     ├── Mục kho (Cipher)
  │     └── Quyền truy cập gán cho Thành viên / Nhóm
  └── Nhóm (Group)
        └── Thành viên
```

### 1.2 Điểm khác biệt: Kho cá nhân vs. Kho tổ chức

| Đặc điểm | Kho cá nhân | Kho tổ chức |
|---------|------------|------------|
| **Chủ sở hữu** | Một người dùng | Tổ chức |
| **Chia sẻ** | Chỉ qua Send | Qua bộ sưu tập |
| **Kiểm soát truy cập** | Không | Theo vai trò và bộ sưu tập |
| **Kiểm toán** | Không | Có (nhật ký sự kiện) |
| **Khi rời tổ chức** | Giữ lại dữ liệu | Mất quyền truy cập kho tổ chức |

---

## 2. Tạo Và Cấu Hình Tổ Chức

### 2.1 Tạo tổ chức mới

> ⚠️ Quản trị viên máy chủ có thể giới hạn ai được phép tạo tổ chức.

1. Đăng nhập vào **Web Vault** (`https://your-vault.example.com`).
2. Nhấp vào **➕ Tổ chức mới** trong thanh bên trái.
3. Nhập:
   - **Tên tổ chức**: Tên hiển thị với tất cả thành viên.
   - **Email thanh toán**: Email liên lạc của tổ chức.
4. Nhấp **Tạo tổ chức**.

Bạn tự động được gán vai trò **Chủ sở hữu**.

### 2.2 Cài đặt tổ chức

Vào **Tổ chức** → **Cài đặt** (chỉ Chủ sở hữu và Quản trị viên):

| Mục cài đặt | Mô tả |
|------------|-------|
| **Tên tổ chức** | Thay đổi tên hiển thị |
| **Xóa tổ chức** | Xóa toàn bộ tổ chức và dữ liệu (không thể hoàn tác) |
| **Xuất kho** | Xuất toàn bộ kho tổ chức ra file |

---

## 3. Quản Lý Thành Viên

### 3.1 Mời thành viên mới

1. Vào **Tổ chức** → **Thành viên** → nhấp **➕ Mời thành viên**.
2. Nhập địa chỉ email (có thể mời nhiều người cùng lúc, mỗi email một dòng).
3. Chọn **Vai trò** (xem bảng vai trò ở §4).
4. Tùy chọn: Gán ngay vào **Bộ sưu tập** cụ thể.
5. Nhấp **Lưu** — email mời được gửi đến người được mời.

**Trạng thái vòng đời thành viên:**

```
Được mời
    ↓ (Người được mời nhấp chấp nhận trong email)
Đã chấp nhận
    ↓ (Chủ sở hữu/Quản trị viên xác nhận)
Đã xác nhận  ← Thành viên đang hoạt động
    ↓ (Nếu cần)
Bị thu hồi  ← Quyền truy cập bị đình chỉ (dữ liệu được giữ lại)
```

### 3.2 Xác nhận thành viên

Sau khi người được mời chấp nhận lời mời:

1. Vào **Thành viên** → tìm người có trạng thái **"Đã chấp nhận"**.
2. Tích chọn ô bên cạnh tên → nhấp **Xác nhận**.
3. Người dùng chuyển sang trạng thái **"Đã xác nhận"** và có thể truy cập kho.

> 💡 Bước xác nhận là bước bảo mật quan trọng: đây là lúc khóa mã hóa tổ chức được chia sẻ bảo mật với thành viên mới.

### 3.3 Thu hồi quyền truy cập

Khi thành viên rời nhóm hoặc cần tạm đình chỉ:

1. Vào **Thành viên** → nhấp vào tên thành viên.
2. Nhấp **Thu hồi quyền truy cập** (hoặc **Xóa thành viên**).

| Hành động | Hiệu quả |
|----------|---------|
| **Thu hồi** | Ngay lập tức mất quyền truy cập; có thể khôi phục sau |
| **Xóa** | Xóa khỏi tổ chức; phải mời lại từ đầu |

> ⚠️ Quyền truy cập bị thu hồi **ngay lập tức** — phiên hiện tại của thành viên sẽ thất bại khi đồng bộ tiếp theo.

### 3.4 Khôi phục thành viên bị thu hồi

1. Vào **Thành viên** → tìm thành viên trạng thái **"Bị thu hồi"**.
2. Nhấp **Khôi phục thành viên**.
3. Trạng thái trở về **"Đã xác nhận"** với đầy đủ vai trò và quyền truy cập bộ sưu tập trước đó.

---

## 4. Vai Trò Và Phân Quyền

### 4.1 Bảng so sánh vai trò

| Quyền hạn | Chủ sở hữu | Quản trị viên | Quản lý | Người dùng | Tùy chỉnh |
|----------|:----------:|:------------:|:-------:|:----------:|:----------:|
| Quản lý thành viên | ✅ | ✅ | ❌ | ❌ | ⚙️ |
| Xóa tổ chức | ✅ | ❌ | ❌ | ❌ | ❌ |
| Quản lý tất cả bộ sưu tập | ✅ | ✅ | ❌ | ❌ | ⚙️ |
| Quản lý bộ sưu tập được gán | ✅ | ✅ | ✅ | ❌ | ⚙️ |
| Truy cập mục kho | ✅ | ✅ | ✅ | ✅ (được gán) | ⚙️ |
| Cài đặt chính sách | ✅ | ✅ | ❌ | ❌ | ⚙️ |
| Xem nhật ký sự kiện | ✅ | ✅ | ❌ | ❌ | ⚙️ |
| Xuất kho tổ chức | ✅ | ✅ | ❌ | ❌ | ⚙️ |

> ⚙️ = Cấu hình riêng trong tùy chọn "Tùy chỉnh"

### 4.2 Hướng dẫn chọn vai trò

| Trường hợp | Vai trò phù hợp |
|-----------|----------------|
| Người quản lý IT, cần toàn quyền | **Quản trị viên** |
| Trưởng nhóm, quản lý mật khẩu của nhóm họ | **Quản lý** |
| Nhân viên chỉ cần truy cập mật khẩu được gán | **Người dùng** |
| Trường hợp đặc biệt cần quyền tùy chỉnh | **Tùy chỉnh** |

### 4.3 Thay đổi vai trò thành viên

1. Vào **Thành viên** → nhấp vào tên thành viên.
2. Tìm trường **Vai trò** → chọn vai trò mới.
3. Nhấp **Lưu**.

---

## 5. Quản Lý Bộ Sưu Tập (Collections)

**Bộ sưu tập** là thư mục chia sẻ của tổ chức — đây là đơn vị kiểm soát truy cập cơ bản.

### 5.1 Tạo bộ sưu tập

1. Vào **Tổ chức** → **Bộ sưu tập** → nhấp **➕ Bộ sưu tập mới**.
2. Nhập **Tên bộ sưu tập** (ví dụ: "Hạ tầng - AWS", "Marketing", "Kế toán").
3. Gán quyền truy cập cho thành viên hoặc nhóm:
   - Chọn thành viên/nhóm từ danh sách.
   - Chọn quyền: **Có thể xem**, **Có thể chỉnh sửa**.
   - Tùy chọn: **Quản lý bộ sưu tập** (cho Quản lý).
4. Nhấp **Lưu**.

### 5.2 Thực tiễn tốt nhất về đặt tên bộ sưu tập

| Ví dụ cấu trúc | Mô tả |
|---------------|-------|
| `Hạ tầng - Cloud` | Theo lĩnh vực |
| `Dự án XYZ` | Theo dự án |
| `Dev - Production` | Theo môi trường |
| `Đội ngũ - Backend` | Theo phòng ban |

### 5.3 Gán / Thu hồi quyền truy cập bộ sưu tập

**Theo thành viên:**
1. Vào **Thành viên** → nhấp tên thành viên.
2. Trong phần **Bộ sưu tập**, thêm hoặc xóa bộ sưu tập.

**Theo bộ sưu tập:**
1. Vào **Bộ sưu tập** → nhấp vào bộ sưu tập.
2. Trong tab **Truy cập**, thêm hoặc xóa thành viên/nhóm.

### 5.4 Di chuyển mục kho vào bộ sưu tập

**Từ kho cá nhân sang tổ chức:**
1. Trong Web Vault, chọn mục kho cá nhân.
2. Nhấp **Chỉnh sửa** → phần **Bộ sưu tập**.
3. Chọn tổ chức và bộ sưu tập.
4. Nhấp **Lưu** — mục sẽ được chuyển vào tổ chức.

> ⚠️ Sau khi chuyển vào tổ chức, mục không còn thuộc kho cá nhân nữa. Chủ sở hữu của mục là **tổ chức**.

**Thêm mục trực tiếp vào bộ sưu tập:**
1. Nhấp **➕ Mục mới**.
2. Nhập thông tin mục.
3. Trong trường **Chủ sở hữu**, chọn **tổ chức**.
4. Trong **Bộ sưu tập**, chọn bộ sưu tập phù hợp.
5. Nhấp **Lưu**.

---

## 6. Quản Lý Nhóm Người Dùng (Groups)

> 💡 Tính năng Nhóm phải được quản trị viên máy chủ bật qua `ORG_GROUPS_ENABLED=true`.

**Nhóm** cho phép bạn gán quyền truy cập bộ sưu tập cho nhiều người cùng lúc thay vì gán từng người một.

### 6.1 Tạo nhóm

1. Vào **Tổ chức** → **Nhóm** → nhấp **➕ Nhóm mới**.
2. Nhập tên nhóm (ví dụ: "Lập trình viên Backend", "Nhóm Sales").
3. Gán **Thành viên** vào nhóm.
4. Gán **Bộ sưu tập** với quyền truy cập tương ứng.
5. Nhấp **Lưu**.

### 6.2 Thêm thành viên vào nhóm

1. Vào **Nhóm** → nhấp tên nhóm.
2. Trong tab **Thành viên**, tìm và thêm người dùng.
3. Nhấp **Lưu**.

> Thành viên **tự động nhận** tất cả quyền truy cập bộ sưu tập của nhóm khi được thêm vào.

### 6.3 Kịch bản sử dụng điển hình

**Ví dụ: Công ty 20 người — Nhóm Kỹ thuật vs. Nhóm Kinh doanh**

```
Nhóm "Kỹ thuật" (10 người)
  → Bộ sưu tập "Hạ tầng Cloud" (Có thể chỉnh sửa)
  → Bộ sưu tập "Staging Credentials" (Có thể chỉnh sửa)
  → Bộ sưu tập "Production Read-only" (Chỉ xem)

Nhóm "Kinh doanh" (8 người)
  → Bộ sưu tập "CRM & Marketing Tools" (Có thể chỉnh sửa)
  → Bộ sưu tập "Social Media" (Chỉ xem)
```

Khi nhân viên mới vào nhóm Kỹ thuật, chỉ cần thêm họ vào nhóm — họ tự động có đủ quyền truy cập.

---

## 7. Chia Sẻ Mật Khẩu Trong Tổ Chức

### 7.1 Nguyên tắc chia sẻ

- Mọi mục trong tổ chức phải thuộc **ít nhất một bộ sưu tập**.
- Người dùng chỉ thấy mục trong bộ sưu tập **họ được phép truy cập**.
- Dữ liệu mục vẫn được **mã hóa đầu cuối** — máy chủ không thể đọc nội dung.

### 7.2 Kiểm soát quyền chỉnh sửa

Khi gán thành viên vào bộ sưu tập, chọn mức quyền:

| Quyền | Xem | Tạo mới | Chỉnh sửa | Xóa |
|-------|:---:|:-------:|:----------:|:---:|
| **Chỉ xem** | ✅ | ❌ | ❌ | ❌ |
| **Có thể chỉnh sửa** | ✅ | ✅ | ✅ | ✅ |
| **Ngoại trừ mật khẩu** | ✅ (ẩn MK) | ✅ | ✅ | ✅ |
| **Quản lý bộ sưu tập** | ✅ | ✅ | ✅ | ✅ + Quản lý quyền BC |

### 7.3 Xuất kho tổ chức

1. Vào **Công cụ** → **Xuất kho**.
2. Chọn **Định dạng**: JSON (đề xuất), CSV.
3. Chọn **Tổ chức** cần xuất.
4. Nhập mật khẩu chính để xác nhận.
5. Nhấp **Xuất kho**.

> ⚠️ File xuất ra **không được mã hóa** (trừ định dạng `.json` mã hóa). Hãy bảo quản file này cẩn thận.

---

## 8. Chính Sách Bảo Mật Tổ Chức

**Chính sách** cho phép Chủ sở hữu và Quản trị viên **bắt buộc** các tiêu chuẩn bảo mật cho tất cả thành viên.

Vào: **Tổ chức** → **Chính sách**

### 8.1 Danh sách chính sách

#### 🔒 Yêu cầu xác thực hai yếu tố (2FA)

**Bật chính sách này nếu**: Tổ chức cần đảm bảo tất cả thành viên dùng 2FA.

- Khi bật: Thành viên chưa cấu hình 2FA nhận cảnh báo và **bị thu hồi quyền truy cập** cho đến khi tuân thủ.
- Thành viên bị loại ra khỏi chính sách sẽ nhận email `send_2fa_removed_from_org` thông báo.

**Bật:**
1. Vào **Chính sách** → **Yêu cầu xác thực hai yếu tố**.
2. Nhấp **Bật**.

#### 🔒 Độ mạnh mật khẩu chính tối thiểu

**Bật chính sách này nếu**: Muốn đảm bảo mật khẩu chính của thành viên đủ mạnh.

- Đặt điểm phức tạp tối thiểu (0–4 tương ứng với mức Yếu → Rất mạnh).

#### 🔒 Tổ chức duy nhất

**Bật chính sách này nếu**: Muốn thành viên chỉ thuộc tổ chức này, không phải tổ chức nào khác trên cùng máy chủ.

- Hữu ích khi cần đảm bảo dữ liệu không bị "trộn" với tổ chức khác.
- Thành viên đang thuộc nhiều tổ chức sẽ được yêu cầu rời khỏi những tổ chức còn lại.

#### 🔒 Chỉ chấp nhận thiết bị tin cậy

**Bật chính sách này nếu**: Muốn thành viên chỉ đăng nhập từ thiết bị đã được phê duyệt.

#### 🔒 Vô hiệu hóa Send

**Bật chính sách này nếu**: Không muốn thành viên dùng tính năng Bitwarden Send cho dữ liệu tổ chức.

#### 🔒 Cấm nhập mật khẩu bị vi phạm (HIBP)

**Bật chính sách này nếu**: Muốn ngăn thành viên lưu mật khẩu đã xuất hiện trong các vụ rò rỉ dữ liệu.

> Yêu cầu quản trị viên máy chủ đã cấu hình `HIBP_API_KEY`.

#### 🔒 Chính sách Trình tạo mật khẩu

Thiết lập độ dài tối thiểu, yêu cầu ký tự đặc biệt, số... khi thành viên tạo mật khẩu mới trong tổ chức.

### 8.2 Checklist chính sách cho SMB

Khuyến nghị cho công ty 10–50 người:

- [ ] ✅ Bật **Yêu cầu 2FA** bắt buộc
- [ ] ✅ Bật **Độ mạnh mật khẩu** tối thiểu mức 3 (Mạnh)
- [ ] ✅ Cân nhắc **Tổ chức duy nhất** nếu cần kiểm soát chặt
- [ ] ✅ Cân nhắc **HIBP** nếu bảo mật là ưu tiên hàng đầu

---

## 9. Nhật Ký Sự Kiện Và Kiểm Toán

> 💡 Tính năng này phải được quản trị viên máy chủ bật qua `ORG_EVENTS_ENABLED=true`.

### 9.1 Xem nhật ký sự kiện

1. Vào **Tổ chức** → **Nhật ký sự kiện** (hoặc **Events**).
2. Xem danh sách sự kiện với thông tin:
   - **Thời gian** xảy ra sự kiện.
   - **Tên người dùng** thực hiện hành động.
   - **Loại sự kiện** (ví dụ: Đăng nhập, Thêm mục, Xóa thành viên...).
   - **Mục bị ảnh hưởng** (nếu có).
   - **Địa chỉ IP**.

### 9.2 Các sự kiện được ghi lại

| Danh mục | Sự kiện ghi lại |
|---------|----------------|
| **Xác thực** | Đăng nhập thành công/thất bại, đăng xuất |
| **Kho mật khẩu** | Tạo, chỉnh sửa, xóa, xem mục |
| **Thành viên** | Mời, xác nhận, xóa, thu hồi, thay đổi vai trò |
| **Bộ sưu tập** | Tạo, chỉnh sửa, xóa bộ sưu tập |
| **Chính sách** | Thay đổi chính sách |
| **Tổ chức** | Thay đổi cài đặt |

### 9.3 Lọc và tìm kiếm sự kiện

- Lọc theo **khoảng thời gian**.
- Lọc theo **thành viên cụ thể**.
- Xuất dữ liệu nhật ký qua API cho phân tích bên ngoài.

### 9.4 Tuân thủ và lưu giữ

- Quản trị viên máy chủ có thể cấu hình số ngày lưu trữ nhật ký (`EVENTS_DAYS_RETAIN`).
- Nếu cần lưu trữ dài hạn, hãy yêu cầu quản trị viên **xuất nhật ký định kỳ**.

---

## 10. Khôi Phục Tài Khoản Thành Viên

Tính năng **Khôi phục quản trị** (Admin Password Reset) cho phép Chủ sở hữu/Quản trị viên đặt lại mật khẩu chính của thành viên khi cần thiết.

### 10.1 Điều kiện tiên quyết

Thành viên phải **tự nguyện bật** tùy chọn cho phép khôi phục:
1. Thành viên vào **Cài đặt tài khoản** → **Tổ chức**.
2. Bật **Cho phép quản trị viên đặt lại mật khẩu chính của tôi**.

### 10.2 Đặt lại mật khẩu thành viên (Quản trị viên thực hiện)

1. Vào **Thành viên** → nhấp tên thành viên bị khóa.
2. Nhấp **Đặt lại mật khẩu chính**.
3. Nhập mật khẩu chính **mới tạm thời**.
4. Nhấp **Xác nhận**.
5. Thông báo thành viên về mật khẩu tạm thời qua kênh bảo mật (không qua Vaultwarden).

> ⚠️ Sau khi đặt lại, thành viên **phải đổi mật khẩu chính** ngay khi đăng nhập lần tiếp theo.

### 10.3 Lưu ý bảo mật

- Mọi lần đặt lại mật khẩu đều được **ghi vào nhật ký sự kiện**.
- Chỉ dùng tính năng này trong trường hợp thực sự cần thiết (ví dụ: nhân viên quên mật khẩu, không phải để kiểm soát bất hợp lệ).

---

## 11. Khóa API Tổ Chức

Dùng khi cần tích hợp tự động (CI/CD, script, công cụ provisioning).

### 11.1 Tạo khóa API tổ chức

1. Vào **Cài đặt tổ chức** → **Khóa API**.
2. Nhấp **Xem khóa API** → xác nhận bằng mật khẩu chính.
3. Lưu lại:
   - `client_id`: Định danh của tổ chức.
   - `client_secret`: Khóa bí mật.

### 11.2 Sử dụng với Bitwarden CLI

```bash
# Đăng nhập bằng khóa API tổ chức
bw login --apikey

# Xuất kho tổ chức
bw export --organizationid <org-id> --format json --output backup.json
```

### 11.3 Thu hồi khóa API

Nếu khóa bị lộ:
1. Vào **Cài đặt** → **Khóa API** → nhấp **Tạo lại khóa API**.
2. Cập nhật `client_secret` mới vào tất cả các script/pipeline.

---

## 12. Các Câu Hỏi Thường Gặp

### ❓ Thành viên không nhận được email mời?

1. Kiểm tra thư mục Spam/Junk.
2. Yêu cầu quản trị viên máy chủ kiểm tra cấu hình SMTP.
3. Quản trị viên có thể gửi lại lời mời từ **Thành viên** → nhấp thành viên → **Gửi lại email mời**.

### ❓ Bộ sưu tập không xuất hiện với thành viên?

Kiểm tra các nguyên nhân:
1. Thành viên có trạng thái **"Đã xác nhận"** chưa? (Không phải chỉ "Đã chấp nhận")
2. Thành viên đã được **gán vào bộ sưu tập** chưa?
3. Thành viên đã **đồng bộ lại** ứng dụng khách chưa?

### ❓ Làm thế nào để xem ai đã thay đổi một mật khẩu cụ thể?

Vào **Nhật ký sự kiện** và lọc theo:
- Loại sự kiện: `Cipher Updated` (Mục kho được cập nhật).
- UUID của mục kho (lấy từ URL khi xem mục).

### ❓ Thành viên có thể xem mật khẩu họ không được gán không?

**Không.** Phân quyền theo bộ sưu tập được áp dụng ở cả phía máy chủ và máy khách. Thành viên chỉ nhận được dữ liệu đã mã hóa của các mục trong bộ sưu tập họ được phép.

### ❓ Có thể thiết lập ngày hết hạn cho thành viên không?

Hiện tại Vaultwarden không hỗ trợ ngày hết hạn thành viên tự động. Bạn phải **thu hồi thủ công** khi thành viên rời nhóm/hợp đồng hết hạn.

**Giải pháp thực tế**: Tạo lịch nhắc nhở định kỳ để xem xét danh sách thành viên.

### ❓ Sự khác biệt giữa "Xóa thành viên" và "Thu hồi quyền"?

| | Thu hồi quyền | Xóa thành viên |
|-|:-------------:|:---------------:|
| Mất quyền truy cập ngay | ✅ | ✅ |
| Giữ lại hồ sơ trong tổ chức | ✅ | ❌ |
| Có thể khôi phục dễ dàng | ✅ | ❌ (phải mời lại) |
| Giữ lại vai trò/bộ sưu tập khi khôi phục | ✅ | ❌ |

**Khuyến nghị**: Dùng **Thu hồi** cho nhân viên tạm nghỉ; dùng **Xóa** cho nhân viên đã rời công ty.

---

*Cần hỗ trợ thêm? Liên hệ quản trị viên máy chủ Vaultwarden hoặc tham khảo tài liệu kỹ thuật tại `specs/`.*
