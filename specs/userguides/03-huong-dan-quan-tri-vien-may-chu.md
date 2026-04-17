# Hướng Dẫn Vận Hành Vaultwarden — Dành Cho Quản Trị Viên Máy Chủ

> **Đối tượng**: Quản trị viên hệ thống, DevOps, người vận hành máy chủ  
> **Phiên bản**: 1.0 | **Ngày**: 2026-04-10  
> **Điều kiện tiên quyết**: Docker, kiến thức Linux cơ bản, quyền truy cập máy chủ

---

## Mục Lục

1. [Triển khai Vaultwarden với Docker](#1-triển-khai-vaultwarden-với-docker)
2. [Cấu hình biến môi trường](#2-cấu-hình-biến-môi-trường)
3. [Cấu hình cơ sở dữ liệu](#3-cấu-hình-cơ-sở-dữ-liệu)
4. [Reverse proxy và HTTPS](#4-reverse-proxy-và-https)
5. [Bảo mật bảng quản trị](#5-bảo-mật-bảng-quản-trị)
6. [Sử dụng bảng quản trị](#6-sử-dụng-bảng-quản-trị)
7. [Cấu hình email SMTP](#7-cấu-hình-email-smtp)
8. [Cấu hình SSO (Đăng nhập một lần)](#8-cấu-hình-sso-đăng-nhập-một-lần)
9. [Thông báo push và WebSocket](#9-thông-báo-push-và-websocket)
10. [Sao lưu và phục hồi](#10-sao-lưu-và-phục-hồi)
11. [Quản lý người dùng](#11-quản-lý-người-dùng)
12. [Giám sát và nhật ký](#12-giám-sát-và-nhật-ký)
13. [Nâng cấp Vaultwarden](#13-nâng-cấp-vaultwarden)
14. [Xử lý sự cố thường gặp](#14-xử-lý-sự-cố-thường-gặp)
15. [Tham chiếu biến môi trường đầy đủ](#15-tham-chiếu-biến-môi-trường-đầy-đủ)

---

## 1. Triển Khai Vaultwarden Với Docker

### 1.1 Triển khai tối thiểu (1 lệnh)

```bash
docker run -d \
  --name vaultwarden \
  -v /vw-data/:/data/ \
  -p 80:80 \
  vaultwarden/server:latest
```

> ⚠️ Đây là cấu hình tối thiểu — **không đủ cho production**. Phải thêm HTTPS và cấu hình đúng URL domain.

### 1.2 Triển khai với Docker Compose (Khuyến nghị)

Tạo file `/opt/vaultwarden/docker-compose.yml`:

```yaml
version: "3.8"

services:
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: vaultwarden
    restart: unless-stopped
    volumes:
      - ./data:/data
    environment:
      DOMAIN: "https://vault.example.com"
      ADMIN_TOKEN: "${ADMIN_TOKEN}"
      SMTP_HOST: "smtp.example.com"
      SMTP_FROM: "no-reply@example.com"
      SMTP_PORT: "587"
      SMTP_SECURITY: "starttls"
      SMTP_USERNAME: "${SMTP_USERNAME}"
      SMTP_PASSWORD: "${SMTP_PASSWORD}"
      SIGNUPS_ALLOWED: "false"
      WEBSOCKET_ENABLED: "true"
    ports:
      - "127.0.0.1:8080:80"   # HTTP (Nginx proxy đến đây)
      - "127.0.0.1:3012:3012" # WebSocket
```

Tạo file `/opt/vaultwarden/.env`:

```env
ADMIN_TOKEN=<generated_argon2_hash>
SMTP_USERNAME=your-smtp-user@example.com
SMTP_PASSWORD=your-smtp-password
```

```bash
# Khởi động
docker compose up -d

# Xem logs
docker compose logs -f vaultwarden
```

### 1.3 Cấu trúc thư mục data

```
/opt/vaultwarden/data/
├── db.sqlite3          ← Cơ sở dữ liệu SQLite chính
├── db.sqlite3-wal      ← Write-Ahead Log (đồng bộ với db.sqlite3)
├── db.sqlite3-shm      ← Shared memory file
├── rsa_key.pem         ← Khóa ký JWT (tự tạo khi khởi động)
├── rsa_key.pub.pem     ← Khóa công khai JWT
├── config.json         ← Cấu hình lưu từ bảng quản trị
├── attachments/        ← Tệp đính kèm của người dùng (đã mã hóa)
└── sends/              ← Tệp của Bitwarden Send (đã mã hóa)
```

> ⚠️ **Sao lưu toàn bộ thư mục `data/`** định kỳ. Đây là tất cả dữ liệu của bạn.

---

## 2. Cấu Hình Biến Môi Trường

### 2.1 Nguyên tắc cấu hình

- **Ưu tiên**: Biến môi trường > `config.json` (bảng quản trị).
- Config trong bảng quản trị được lưu vào `data/config.json`.
- Nếu cùng một biến được đặt cả ở môi trường và config.json, log sẽ cảnh báo **override**.

### 2.2 Biến môi trường cốt lõi

| Biến môi trường | Bắt buộc | Mô tả | Ví dụ |
|----------------|:--------:|-------|-------|
| `DOMAIN` | ✅ | URL công khai của server (bao gồm `https://`) | `https://vault.example.com` |
| `ADMIN_TOKEN` | ✅ | Mã bảo mật bảng quản trị (nên dùng Argon2id) | Xem §5 |
| `DATABASE_URL` | ❌ | URL cơ sở dữ liệu (mặc định: SQLite) | `postgresql://user:pw@host/db` |
| `SMTP_HOST` | ❌ | Máy chủ gửi email | `smtp.gmail.com` |
| `SIGNUPS_ALLOWED` | ❌ | Cho phép đăng ký tự do (mặc định: `true`) | `false` |
| `PUSH_ENABLED` | ❌ | Bật thông báo push cho di động | `true` |
| `ENABLE_WEBSOCKET` | ❌ | Bật đồng bộ realtime qua WebSocket | `true` |

---

## 3. Cấu Hình Cơ Sở Dữ Liệu

### 3.1 SQLite (Mặc định — Phù hợp cho homelab và <50 người dùng)

Không cần cấu hình thêm. Dữ liệu lưu tại `data/db.sqlite3`.

**Khuyến nghị:**
- Bật WAL mode (mặc định đã bật): giúp hiệu suất ghi đồng thời.
- **Không nên** dùng cho >100 người dùng hoặc tải cao.

### 3.2 PostgreSQL (Khuyến nghị cho production)

```bash
# Cài đặt PostgreSQL
docker run -d \
  --name postgres \
  -e POSTGRES_DB=vaultwarden \
  -e POSTGRES_USER=vw \
  -e POSTGRES_PASSWORD=strongpassword \
  -v pgdata:/var/lib/postgresql/data \
  postgres:16-alpine
```

Cấu hình Vaultwarden:
```env
DATABASE_URL=postgresql://vw:strongpassword@postgres:5432/vaultwarden
```

### 3.3 MySQL / MariaDB

```env
DATABASE_URL=mysql://vw:strongpassword@mysql:3306/vaultwarden
```

> 💡 Di chuyển cơ sở dữ liệu (schema migration) **tự động** được áp dụng khi khởi động — không cần bước thủ công.

### 3.4 Cấu hình nâng cao database

```env
DATABASE_MAX_CONNS=10          # Số kết nối tối đa trong pool
DATABASE_MIN_CONNS=2           # Số kết nối tối thiểu
DATABASE_TIMEOUT=30            # Timeout khi lấy kết nối (giây)
DATABASE_IDLE_TIMEOUT=600      # Timeout đóng kết nối nhàn rỗi (giây)
DB_CONNECTION_RETRIES=15       # Số lần thử kết nối khi khởi động
```

---

## 4. Reverse Proxy Và HTTPS

> ⚠️ **Bắt buộc**: Vaultwarden phải chạy sau reverse proxy có HTTPS. Không nên expose Vaultwarden trực tiếp trên port 80!

### 4.1 Cấu hình Nginx

Cài đặt Nginx và Certbot (Let's Encrypt):

```bash
sudo apt install nginx certbot python3-certbot-nginx
sudo certbot --nginx -d vault.example.com
```

Cấu hình `/etc/nginx/sites-available/vaultwarden`:

```nginx
# Chuyển hướng HTTP → HTTPS
server {
    listen 80;
    server_name vault.example.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name vault.example.com;

    # SSL (Certbot tự cấu hình)
    ssl_certificate /etc/letsencrypt/live/vault.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/vault.example.com/privkey.pem;

    # Bảo mật headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "SAMEORIGIN" always;

    # Giới hạn upload (cho tệp đính kèm & Send)
    client_max_body_size 525M;

    # Proxy đến Vaultwarden
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket (nếu bật ENABLE_WEBSOCKET=true)
    location /notifications/hub {
        proxy_pass http://127.0.0.1:3012;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /notifications/hub/negotiate {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

```bash
sudo nginx -t && sudo systemctl reload nginx
```

### 4.2 Cấu hình Caddy (Tự động HTTPS — Đơn giản hơn)

Tạo file `Caddyfile`:

```caddyfile
vault.example.com {
    encode gzip

    # Chuyển hướng WebSocket
    reverse_proxy /notifications/hub localhost:3012

    # Proxy chính
    reverse_proxy localhost:8080
}
```

```bash
caddy start
```

> 💡 Caddy tự động lấy và gia hạn chứng chỉ Let's Encrypt — không cần cấu hình thêm.

---

## 5. Bảo Mật Bảng Quản Trị

### 5.1 Tạo ADMIN_TOKEN bảo mật với Argon2id

> ⚠️ Token văn bản thuần túy **không được chấp nhận** từ phiên bản mới. Phải dùng Argon2id.

**Phương pháp 1: Dùng lệnh vaultwarden hash (Khuyến nghị)**

```bash
# Dùng cấu hình sẵn bitwarden (64 MiB, 3 iterations, 4 threads)
docker run --rm -it vaultwarden/server:latest /vaultwarden hash --preset bitwarden

# Hoặc cấu hình sẵn owasp (nhẹ hơn, cho máy tài nguyên thấp)
docker run --rm -it vaultwarden/server:latest /vaultwarden hash --preset owasp
```

Nhập mật khẩu quản trị khi được hỏi → sao chép chuỗi PHC bắt đầu bằng `$argon2id$`.

**Phương pháp 2: Dùng công cụ argon2**

```bash
# Cài đặt
sudo apt install argon2

# Tạo hash
echo -n 'your-admin-password' | argon2 $(openssl rand -base64 32) -id -k 65540 -t 3 -p 4 -l 32 -e
```

### 5.2 Đặt ADMIN_TOKEN trong cấu hình

```env
# Trong .env hoặc docker-compose.yml
ADMIN_TOKEN=$argon2id$v=19$m=65540,t=3,p=4$...toàn_bộ_chuỗi_PHC...
```

> 💡 Để trống `ADMIN_TOKEN` sẽ **vô hiệu hóa bảng quản trị** — đây là lựa chọn bảo mật nếu bạn không cần dùng bảng quản trị.

### 5.3 Truy cập bảng quản trị

```
https://vault.example.com/admin
```

- Nhập mật khẩu quản trị (không phải hash, mật khẩu gốc).
- Phiên quản trị hết hạn sau **20 phút** mặc định (cấu hình bằng `ADMIN_SESSION_LIFETIME`).

### 5.4 Giới hạn tốc độ đăng nhập admin

```env
ADMIN_RATELIMIT_SECONDS=300    # Khoảng cách trung bình giữa các lần thử (giây)
ADMIN_RATELIMIT_MAX_BURST=3    # Số lần burst tối đa
```

---

## 6. Sử Dụng Bảng Quản Trị

Truy cập: `https://vault.example.com/admin`

### 6.1 Quản lý người dùng

Vào tab **Users**:

| Hành động | Mô tả |
|----------|-------|
| **Invite User** | Mời người dùng mới qua email |
| **Delete User** | Xóa tài khoản và toàn bộ dữ liệu |
| **Deactivate** | Tắt tài khoản (người dùng không thể đăng nhập) |
| **Enable** | Bật lại tài khoản đã tắt |
| **Reset 2FA** | Xóa tất cả phương pháp 2FA của người dùng |

### 6.2 Quản lý tổ chức

Vào tab **Organizations**:
- Xem danh sách tất cả tổ chức.
- Xem thành viên và bộ sưu tập của từng tổ chức.

### 6.3 Thay đổi cấu hình

Vào tab **Settings**:
- Thay đổi hầu hết biến cấu hình mà không cần restart.
- Cài đặt được lưu vào `data/config.json`.
- **READ-ONLY CONFIG**: Các biến chỉ có thể đặt qua biến môi trường (thường là các biến liên quan đến database, đường dẫn, network).

**Thử nghiệm SMTP:**
1. Vào **Settings** → phần **SMTP**.
2. Nhập email thử nghiệm → nhấp **Send test email**.

### 6.4 Sao lưu cơ sở dữ liệu SQLite

Vào tab **Settings** → **Backup Database** → nhấp **Backup Database**.

File sao lưu được tạo tại `data/db-backup-YYYYMMDD-HHmmss.sqlite3`.

### 6.5 Chẩn đoán hệ thống

Vào tab **Diagnostics**:
- Xem phiên bản server.
- Kiểm tra kết nối cơ sở dữ liệu.
- Xem thông tin môi trường.
- Kiểm tra trạng thái các tính năng.

---

## 7. Cấu Hình Email SMTP

Email là **bắt buộc** cho nhiều tính năng: xác minh tài khoản, lời mời, 2FA, truy cập khẩn cấp.

### 7.1 Cấu hình cơ bản

```env
SMTP_HOST=smtp.gmail.com
SMTP_FROM=no-reply@example.com
SMTP_FROM_NAME=Vaultwarden
SMTP_PORT=587
SMTP_SECURITY=starttls      # Chọn: starttls, force_tls, off
SMTP_USERNAME=your@gmail.com
SMTP_PASSWORD=your-app-password
```

### 7.2 Lựa chọn giao thức

| Giá trị `SMTP_SECURITY` | Cổng thường dùng | Mô tả |
|------------------------|:---------------:|-------|
| `starttls` | 587 | STARTTLS — Khởi động không mã hóa rồi nâng cấp lên TLS (Khuyến nghị) |
| `force_tls` | 465 | TLS từ đầu (SSL Implicit) |
| `off` | 25 | Không mã hóa (Chỉ dùng trong internal network) |

### 7.3 Cấu hình với Gmail

1. Bật **2-Step Verification** cho Google Account.
2. Tạo **App Password**: myaccount.google.com → Security → App passwords.
3. Dùng App Password thay vì mật khẩu Google:

```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_SECURITY=starttls
SMTP_USERNAME=your@gmail.com
SMTP_PASSWORD=xxxx-xxxx-xxxx-xxxx  # App Password
```

### 7.4 Debug SMTP

```env
SMTP_DEBUG=true    # Ghi log chi tiết giao tiếp SMTP
LOG_LEVEL=debug    # Tăng mức log để xem thêm thông tin
```

### 7.5 Sử dụng Sendmail cục bộ

```env
USE_SENDMAIL=true
SENDMAIL_COMMAND=/usr/sbin/sendmail
```

---

## 8. Cấu Hình SSO (Đăng Nhập Một Lần)

SSO cho phép người dùng đăng nhập qua Nhà cung cấp danh tính (IdP) như Okta, Azure AD, Google Workspace, Keycloak.

### 8.1 Điều kiện tiên quyết

1. Có tài khoản quản trị IdP.
2. Tạo một OIDC Application trong IdP.
3. Đặt Redirect URI: `https://vault.example.com/identity/connect/oidc-signin`

### 8.2 Cấu hình Vaultwarden

```env
SSO_ENABLED=true
SSO_AUTHORITY=https://your-idp.example.com         # OpenID Connect Discovery URL
SSO_CLIENT_ID=vaultwarden                           # Client ID từ IdP
SSO_CLIENT_SECRET=your-client-secret               # Client Secret từ IdP
SSO_CALLBACK_PATH=/identity/connect/oidc-signin    # Callback path (mặc định)
```

### 8.3 Ví dụ với Keycloak

```env
SSO_ENABLED=true
SSO_AUTHORITY=https://keycloak.example.com/realms/myrealm
SSO_CLIENT_ID=vaultwarden
SSO_CLIENT_SECRET=secret-from-keycloak
```

### 8.4 Ví dụ với Azure Active Directory

```env
SSO_ENABLED=true
SSO_AUTHORITY=https://login.microsoftonline.com/{tenant-id}/v2.0
SSO_CLIENT_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
SSO_CLIENT_SECRET=your-client-secret
```

### 8.5 Hành vi SSO

| Hành vi | Cấu hình |
|--------|---------|
| Tự động tạo tài khoản khi đăng nhập lần đầu | Bật theo mặc định |
| Đồng thời hỗ trợ đăng nhập password thông thường | Có — SSO và password cùng tồn tại |
| TTL cache trạng thái SSO | 10 phút, tối đa 1000 phiên đồng thời |

---

## 9. Thông Báo Push Và WebSocket

### 9.1 WebSocket (Đồng bộ thời gian thực qua Web/Desktop)

```env
ENABLE_WEBSOCKET=true   # Bật WebSocket
```

Đảm bảo Nginx đã cấu hình proxy WebSocket (xem §4.1).

**Kiểm tra hoạt động:**
1. Mở Web Vault trên 2 browser tab.
2. Thêm mục mới trong tab 1.
3. Tab 2 phải tự động cập nhật mà không cần refresh.

### 9.2 Push Notification (Đồng bộ cho ứng dụng di động)

Push notification yêu cầu relay từ Bitwarden.com:

1. Đăng ký tại [bitwarden.com/host](https://bitwarden.com/host) để lấy Installation ID và Key.
2. Cấu hình:

```env
PUSH_ENABLED=true
PUSH_INSTALLATION_ID=your-installation-id
PUSH_INSTALLATION_KEY=your-installation-key
PUSH_RELAY_URI=https://push.bitwarden.com      # Mặc định
PUSH_IDENTITY_URI=https://identity.bitwarden.com  # Mặc định
```

> 💡 Nếu không có relay push, ứng dụng di động vẫn hoạt động nhưng đồng bộ theo chu kỳ thay vì thời gian thực.

---

## 10. Sao Lưu Và Phục Hồi

### 10.1 Chiến lược sao lưu

**Bắt buộc sao lưu:**
1. `data/db.sqlite3` (hoặc dump DB nếu dùng PostgreSQL/MySQL)
2. `data/attachments/` (tệp đính kèm)
3. `data/sends/` (tệp Send)
4. `data/rsa_key.pem` + `data/rsa_key.pub.pem` (khóa JWT)
5. `data/config.json` (cấu hình)

### 10.2 Sao lưu SQLite

**Phương pháp 1: Script tự động với SQLite backup API**

```bash
#!/bin/bash
# /opt/vaultwarden/backup.sh

BACKUP_DIR="/backup/vaultwarden/$(date +%Y-%m-%d)"
DATA_DIR="/opt/vaultwarden/data"

mkdir -p "$BACKUP_DIR"

# Sao lưu SQLite (nhất quán, không khóa DB)
sqlite3 "$DATA_DIR/db.sqlite3" ".backup '$BACKUP_DIR/db.sqlite3'"

# Sao lưu tệp
cp -a "$DATA_DIR/attachments/" "$BACKUP_DIR/"
cp -a "$DATA_DIR/sends/" "$BACKUP_DIR/"
cp "$DATA_DIR/rsa_key.pem" "$BACKUP_DIR/"
cp "$DATA_DIR/config.json" "$BACKUP_DIR/" 2>/dev/null || true

# Nén
tar -czf "$BACKUP_DIR.tar.gz" "$BACKUP_DIR"
rm -rf "$BACKUP_DIR"

echo "Backup completed: $BACKUP_DIR.tar.gz"
```

```bash
chmod +x /opt/vaultwarden/backup.sh

# Đặt lịch cron sao lưu hằng ngày lúc 2:00 AM
echo "0 2 * * * /opt/vaultwarden/backup.sh >> /var/log/vaultwarden-backup.log 2>&1" | crontab -
```

**Phương pháp 2: Sao lưu qua tín hiệu SIGUSR1**

```bash
# Gửi tín hiệu SIGUSR1 để kích hoạt sao lưu nội bộ của Vaultwarden
docker kill --signal=SIGUSR1 vaultwarden

# File backup được tạo tại data/db-backup-YYYYMMDD-HHmmss.sqlite3
```

**Phương pháp 3: Sao lưu từ bảng quản trị**

Vào `/admin` → Settings → **Backup Database** → nhấp **Backup Database**.

### 10.3 Sao lưu PostgreSQL

```bash
#!/bin/bash
BACKUP_DIR="/backup/vaultwarden/$(date +%Y-%m-%d)"
mkdir -p "$BACKUP_DIR"

# Dump PostgreSQL
docker exec postgres pg_dump -U vw vaultwarden | gzip > "$BACKUP_DIR/db.sql.gz"

# Sao lưu tệp đính kèm
cp -a /opt/vaultwarden/data/attachments/ "$BACKUP_DIR/"
cp -a /opt/vaultwarden/data/sends/ "$BACKUP_DIR/"
cp /opt/vaultwarden/data/rsa_key.pem "$BACKUP_DIR/"
```

### 10.4 Phục hồi từ sao lưu

**Phục hồi SQLite:**

```bash
# 1. Dừng Vaultwarden
docker compose down

# 2. Giải nén backup
tar -xzf backup-2026-04-10.tar.gz -C /tmp/

# 3. Phục hồi dữ liệu
cp /tmp/backup/db.sqlite3 /opt/vaultwarden/data/
cp -a /tmp/backup/attachments/ /opt/vaultwarden/data/
cp -a /tmp/backup/sends/ /opt/vaultwarden/data/
cp /tmp/backup/rsa_key.pem /opt/vaultwarden/data/

# 4. Kiểm tra quyền
chown -R 1000:1000 /opt/vaultwarden/data/

# 5. Khởi động lại
docker compose up -d
```

---

## 11. Quản Lý Người Dùng

### 11.1 Kiểm soát đăng ký

```env
# Tắt đăng ký tự do (Khuyến nghị cho môi trường nội bộ)
SIGNUPS_ALLOWED=false

# Chỉ cho phép các tên miền email cụ thể
SIGNUPS_DOMAINS_WHITELIST=company.com,partner.com

# Yêu cầu xác minh email trước khi dùng tài khoản
SIGNUPS_VERIFY=true
SIGNUPS_VERIFY_RESEND_TIME=3600   # Giây giữa các lần gửi lại
SIGNUPS_VERIFY_RESEND_LIMIT=6     # Số lần gửi lại tối đa
```

### 11.2 Mời người dùng qua bảng quản trị

Ngay cả khi `SIGNUPS_ALLOWED=false`, bạn vẫn có thể mời người dùng thủ công:

1. Vào `/admin` → **Users** → nhấp **Invite User**.
2. Nhập địa chỉ email.
3. Nhấp **Send Invite**.
4. Người dùng nhận email với liên kết đăng ký.

### 11.3 Kiểm soát tạo tổ chức

```env
# Chỉ cho phép người dùng cụ thể tạo tổ chức
ORG_CREATION_USERS=admin@company.com,manager@company.com

# Không ai được phép tạo tổ chức (chỉ quản trị viên máy chủ)
ORG_CREATION_USERS=none
```

### 11.4 Cấu hình truy cập khẩn cấp

```env
# Tắt tính năng truy cập khẩn cấp toàn hệ thống
EMERGENCY_ACCESS_ALLOWED=false
```

### 11.5 Thời gian phiên đăng nhập

```env
# Thời gian hết hạn token JWT (giây)
# Mặc định: 7200 (2 giờ) cho access token
# Token làm mới: 2.592.000 (30 ngày) cho desktop/web
# Token làm mới: 7.776.000 (90 ngày) cho mobile
```

---

## 12. Giám Sát Và Nhật Ký

### 12.1 Cấu hình log

```env
LOG_LEVEL=info              # trace, debug, info, warn, error, off
LOG_FILE=/data/vaultwarden.log  # Lưu log vào file
EXTENDED_LOGGING=true       # Bật log chi tiết hơn
LOG_TIMESTAMP_FORMAT="%Y-%m-%d %H:%M:%S%.3f"  # Định dạng timestamp
USE_SYSLOG=false            # Gửi log đến syslog
```

### 12.2 Xem log với Docker

```bash
# Xem log gần nhất
docker compose logs --tail=100 vaultwarden

# Theo dõi log realtime
docker compose logs -f vaultwarden

# Tìm kiếm lỗi
docker compose logs vaultwarden | grep -i error
```

### 12.3 Kiểm tra sức khỏe máy chủ

```bash
# Kiểm tra endpoint alive
curl -s https://vault.example.com/alive

# Kiểm tra phiên bản
curl -s https://vault.example.com/vw_static/scripts/admin.js | head -1

# Kiểm tra kết nối database từ container
docker exec vaultwarden /bin/sh -c "sqlite3 /data/db.sqlite3 'PRAGMA integrity_check;'"
```

### 12.4 Theo dõi nhật ký sự kiện tổ chức

Bật nhật ký sự kiện:
```env
ORG_EVENTS_ENABLED=true
EVENTS_DAYS_RETAIN=90        # Giữ lại log trong 90 ngày
```

---

## 13. Nâng Cấp Vaultwarden

### 13.1 Quy trình nâng cấp

```bash
# 1. Sao lưu trước khi nâng cấp
/opt/vaultwarden/backup.sh

# 2. Pull image mới
docker compose pull

# 3. Dừng và khởi động lại
docker compose down
docker compose up -d

# 4. Kiểm tra log sau nâng cấp
docker compose logs -f vaultwarden
```

> 💡 Schema migration tự động chạy khi khởi động — không cần thao tác thủ công.

### 13.2 Rollback khi gặp sự cố

```bash
# Dừng container
docker compose down

# Khôi phục dữ liệu từ backup
cp /backup/db.sqlite3 /opt/vaultwarden/data/

# Chạy lại image cũ
docker run -d --name vaultwarden \
  -v /opt/vaultwarden/data:/data \
  vaultwarden/server:1.30.0  # Phiên bản cũ
```

### 13.3 Kiểm tra phiên bản hiện tại

```bash
docker exec vaultwarden /vaultwarden --version
```

---

## 14. Xử Lý Sự Cố Thường Gặp

### ❌ Không kết nối được từ ứng dụng khách

**Kiểm tra:**
```bash
# URL có đúng HTTPS và domain chính xác chưa?
curl -I https://vault.example.com/alive

# Nginx có proxy đúng chưa?
docker compose logs vaultwarden | tail -50
sudo nginx -t
```

**Giải pháp:**
- Đảm bảo biến `DOMAIN` khớp với URL ứng dụng khách sử dụng.
- Kiểm tra chứng chỉ SSL còn hạn: `certbot renew --dry-run`.

---

### ❌ Không gửi được email

**Kiểm tra:**
1. Vào `/admin` → Settings → SMTP → **Send test email**.
2. Xem log: `docker compose logs vaultwarden | grep -i smtp`.

**Bật SMTP debug:**
```env
SMTP_DEBUG=true
LOG_LEVEL=debug
```

**Vấn đề thường gặp:**
- Gmail: Cần App Password, không dùng mật khẩu thông thường.
- Port 587 bị chặn bởi ISP: Thử port 465 với `SMTP_SECURITY=force_tls`.

---

### ❌ Bảng quản trị từ chối token

**Vấn đề**: Đang dùng token văn bản thuần túy thay vì Argon2id.

**Giải pháp:**
```bash
# Xem cảnh báo trong log
docker compose logs vaultwarden | grep -i "admin_token"

# Tạo Argon2id hash (xem §5.1)
docker run --rm -it vaultwarden/server:latest /vaultwarden hash --preset bitwarden
```

---

### ❌ WebSocket không hoạt động

**Kiểm tra:**
- Biến `ENABLE_WEBSOCKET=true` đã được đặt.
- Nginx đã cấu hình proxy WebSocket (xem §4.1).

**Test WebSocket:**
```bash
# Dùng wscat
npm install -g wscat
wscat -c "wss://vault.example.com/notifications/hub?access_token=TOKEN"
```

---

### ❌ Tải lên tệp thất bại

**Kiểm tra:**
- Giới hạn size trong Nginx: `client_max_body_size 525M;`
- Giới hạn Vaultwarden: `USER_ATTACHMENT_LIMIT` và `ORG_ATTACHMENT_LIMIT`
- Quyền ghi vào `data/attachments/`

---

### ❌ Database bị lỗi khi restart

**Cho SQLite:**
```bash
# Kiểm tra tính toàn vẹn
docker exec vaultwarden sqlite3 /data/db.sqlite3 "PRAGMA integrity_check;"

# Nếu bị hỏng, khôi phục từ backup WAL
docker exec vaultwarden sqlite3 /data/db.sqlite3 ".recover" > /tmp/recovered.sql
```

---

## 15. Tham Chiếu Biến Môi Trường Đầy Đủ

### 15.1 Thư mục và lưu trữ

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `DATA_FOLDER` | `data` | Thư mục dữ liệu gốc |
| `DATABASE_URL` | `data/db.sqlite3` | URL kết nối cơ sở dữ liệu |
| `WEB_VAULT_FOLDER` | `web-vault/` | Thư mục chứa web vault |

### 15.2 Domain và mạng

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `DOMAIN` | `http://localhost` | URL công khai (bắt buộc đặt) |
| `IP_HEADER` | `X-Real-IP` | Header để lấy IP thực của client |
| `WEB_VAULT_ENABLED` | `true` | Bật/tắt web vault |

### 15.3 Đăng ký và người dùng

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `SIGNUPS_ALLOWED` | `true` | Cho phép đăng ký tự do |
| `SIGNUPS_VERIFY` | `false` | Yêu cầu xác minh email |
| `SIGNUPS_DOMAINS_WHITELIST` | (trống) | Danh sách domain email được phép |
| `INVITATIONS_ALLOWED` | `true` | Cho phép mời qua org admin |
| `INVITATION_EXPIRATION_HOURS` | `120` | Thời gian hết hạn token mời (giờ) |
| `EMAIL_CHANGE_ALLOWED` | `true` | Cho phép người dùng đổi email |
| `PASSWORD_HINTS_ALLOWED` | `true` | Cho phép gợi ý mật khẩu |
| `PASSWORD_ITERATIONS` | `600000` | Số vòng lặp PBKDF2 |
| `ORG_CREATION_USERS` | (trống = tất cả) | Ai được tạo tổ chức |

### 15.4 Bảo mật

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `ADMIN_TOKEN` | (trống) | Token quản trị (Argon2id PHC) |
| `DISABLE_ADMIN_TOKEN` | `false` | Tắt kiểm tra token (dùng auth ngoài) |
| `ADMIN_SESSION_LIFETIME` | `20` | Phiên quản trị (phút) |
| `LOGIN_RATELIMIT_SECONDS` | `60` | Khoảng cách trung bình giữa login (giây) |
| `LOGIN_RATELIMIT_MAX_BURST` | `10` | Số lần burst tối đa |
| `ADMIN_RATELIMIT_SECONDS` | `300` | Khoảng cách login admin (giây) |
| `ADMIN_RATELIMIT_MAX_BURST` | `3` | Burst login admin tối đa |
| `DISABLE_2FA_REMEMBER` | `false` | Bắt buộc 2FA mỗi lần login |
| `EMERGENCY_ACCESS_ALLOWED` | `true` | Bật tính năng truy cập khẩn cấp |

### 15.5 SMTP Email

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `SMTP_HOST` | (bắt buộc) | Địa chỉ máy chủ SMTP |
| `SMTP_FROM` | (bắt buộc) | Địa chỉ email người gửi |
| `SMTP_FROM_NAME` | `Vaultwarden` | Tên người gửi |
| `SMTP_PORT` | `587` | Cổng SMTP |
| `SMTP_SECURITY` | `starttls` | Bảo mật: `starttls`, `force_tls`, `off` |
| `SMTP_USERNAME` | (bắt buộc) | Tên đăng nhập SMTP |
| `SMTP_PASSWORD` | (bắt buộc) | Mật khẩu SMTP |
| `SMTP_DEBUG` | `false` | Ghi log debug SMTP |

### 15.6 Lưu trữ tệp (S3)

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `USER_ATTACHMENT_LIMIT` | (không giới hạn) | Giới hạn dung lượng mỗi người dùng (KB) |
| `ORG_ATTACHMENT_LIMIT` | (không giới hạn) | Giới hạn dung lượng mỗi tổ chức (KB) |
| `USER_SEND_LIMIT` | (không giới hạn) | Giới hạn dung lượng Send (KB) |

> 💡 S3 được cấu hình qua tính năng Cargo `s3` — cần build image riêng.

### 15.7 WebSocket và Push

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `ENABLE_WEBSOCKET` | `true` | Bật WebSocket |
| `PUSH_ENABLED` | `false` | Bật thông báo push di động |
| `PUSH_RELAY_URI` | `https://push.bitwarden.com` | URI relay push |
| `PUSH_INSTALLATION_ID` | (trống) | Installation ID từ bitwarden.com |
| `PUSH_INSTALLATION_KEY` | (trống) | Installation Key từ bitwarden.com |

### 15.8 SSO

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `SSO_ENABLED` | `false` | Bật SSO/OIDC |
| `SSO_AUTHORITY` | (bắt buộc) | URL discovery OIDC của IdP |
| `SSO_CLIENT_ID` | (bắt buộc) | Client ID từ IdP |
| `SSO_CLIENT_SECRET` | (bắt buộc) | Client Secret từ IdP |
| `SSO_CALLBACK_PATH` | `/identity/connect/oidc-signin` | Callback path |

### 15.9 Lịch công việc nền

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `SEND_PURGE_SCHEDULE` | `0 5 * * * *` | Xóa Send hết hạn (hằng giờ) |
| `TRASH_PURGE_SCHEDULE` | `0 5 0 * * *` | Xóa thùng rác (hằng ngày) |
| `INCOMPLETE_2FA_SCHEDULE` | `30 * * * * *` | Kiểm tra 2FA chưa hoàn thành |
| `EMERGENCY_NOTIFICATION_REMINDER_SCHEDULE` | `0 3 * * * *` | Nhắc nhở truy cập khẩn cấp |
| `EMERGENCY_REQUEST_TIMEOUT_SCHEDULE` | `0 7 * * * *` | Xử lý yêu cầu khẩn cấp hết hạn |
| `EVENT_CLEANUP_SCHEDULE` | `0 10 0 * * *` | Xóa sự kiện cũ (hằng ngày) |
| `TRASH_AUTO_DELETE_DAYS` | (không tự xóa) | Số ngày trước khi tự xóa mục trong thùng rác |
| `EVENTS_DAYS_RETAIN` | (vô thời hạn) | Số ngày lưu nhật ký sự kiện |

### 15.10 Nhật ký và chẩn đoán

| Biến | Mặc định | Mô tả |
|-----|---------|-------|
| `LOG_LEVEL` | `info` | Mức log: trace, debug, info, warn, error, off |
| `LOG_FILE` | (stdout) | Đường dẫn file log |
| `EXTENDED_LOGGING` | `true` | Log chi tiết request |
| `LOG_TIMESTAMP_FORMAT` | `%Y-%m-%d %H:%M:%S%.3f` | Định dạng timestamp |
| `USE_SYSLOG` | `false` | Gửi log đến syslog |

---

## Checklist Triển Khai Production

Trước khi đưa vào sử dụng, hãy xác nhận:

- [ ] ✅ `DOMAIN` được đặt đúng với URL HTTPS.
- [ ] ✅ `ADMIN_TOKEN` sử dụng Argon2id PHC (không phải văn bản thuần túy).
- [ ] ✅ HTTPS được cấu hình qua reverse proxy.
- [ ] ✅ SMTP hoạt động (gửi thử nghiệm từ bảng quản trị).
- [ ] ✅ `SIGNUPS_ALLOWED=false` (hoặc giới hạn domain).
- [ ] ✅ Cron job sao lưu tự động đã chạy.
- [ ] ✅ Volume `data/` được mount ra ngoài container.
- [ ] ✅ `client_max_body_size 525M` trong Nginx.
- [ ] ✅ WebSocket proxy được cấu hình (nếu dùng).
- [ ] ✅ Firewall chỉ mở port 80 và 443, không mở 8080 và 3012 ra ngoài.

---

*Tài liệu tham chiếu thêm: [Vaultwarden Wiki](https://github.com/dani-garcia/vaultwarden/wiki) | `specs/technical-design.md` | `specs/srs.md`*
