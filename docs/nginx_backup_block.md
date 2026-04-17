# Nginx Backup Folder Blocking Template

## TASK-SEC-LOW-04-C — Protect the Backup Folder from Web Access

By default Vaultwarden writes database backups to `{DATA_FOLDER}/backups/`.  
If Nginx proxies any path under `DATA_FOLDER` (e.g., `/data/`), these backup files
**must** be explicitly blocked to prevent accidental exposure.

---

## Quick Setup

### Option A — Block by URL path prefix (recommended)

If your Nginx `location` block exposes a path that overlaps with the backup folder,
add the following **before** the main `location /` block:

```nginx
# Block direct web access to Vaultwarden backup files.
# Adjust the path prefix to match your DATA_FOLDER location.
location ~ ^/data/backups/ {
    deny all;
    return 403;
}

# If you configured a custom BACKUP_FOLDER outside DATA_FOLDER, block that too.
# Example for BACKUP_FOLDER=/mnt/secure-backups:
# location ~ ^/mnt/secure-backups/ {
#     deny all;
#     return 403;
# }
```

### Option B — Block at the filesystem level (defense-in-depth)

Ensure Nginx's worker process does not have read access to the backup directory:

```bash
# Set backup folder ownership to vaultwarden service user only
chown -R vaultwarden:vaultwarden /opt/vaultwarden/data/backups
chmod 700 /opt/vaultwarden/data/backups

# Verify Nginx worker (www-data) cannot read backup files
sudo -u www-data ls /opt/vaultwarden/data/backups  # should fail with "Permission denied"
```

---

## Full Example Nginx Configuration with Backup Block

```nginx
server {
    listen 443 ssl http2;
    server_name vault.example.com;

    ssl_certificate     /etc/letsencrypt/live/vault.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/vault.example.com/privkey.pem;
    ssl_protocols       TLSv1.2 TLSv1.3;
    ssl_ciphers         HIGH:!aNULL:!MD5;

    client_max_body_size 525M;

    # ─── SECURITY: Block backup folder access ──────────────────────────────────
    # Place BEFORE the main proxy location.
    location ~ ^/data/backups/ {
        deny all;
        return 403;
    }

    # Block any .sqlite3, .db, .bak files at the root level too.
    location ~* \.(sqlite3|db|bak|sql|dump)$ {
        deny all;
        return 403;
    }
    # ────────────────────────────────────────────────────────────────────────────

    # WebSocket notifications
    location /notifications/hub {
        proxy_pass http://127.0.0.1:3012;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Main Vaultwarden API
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;
    }
}

# Redirect HTTP → HTTPS
server {
    listen 80;
    server_name vault.example.com;
    return 301 https://$host$request_uri;
}
```

---

## Environment Variable Reference

| Variable | Default | Description |
|---|---|---|
| `BACKUP_FOLDER` | `{DATA_FOLDER}/backups` | Where backup `.sqlite3` files are written |
| `DATA_FOLDER` | `data` | Root data directory for all Vaultwarden files |

---

## Verification Checklist

- [ ] `curl -I https://vault.example.com/data/backups/` returns `403 Forbidden`
- [ ] `curl -I https://vault.example.com/data/db.sqlite3` returns `403 Forbidden`
- [ ] Nginx error log shows `access forbidden` for any backup path probe
- [ ] Backup files are present in `BACKUP_FOLDER` and are owned by the Vaultwarden service user
- [ ] Nginx worker process (`www-data`) cannot `stat` files in `BACKUP_FOLDER`

---

> **Security note:** Even with a `deny all` location block, defense-in-depth recommends
> also setting filesystem permissions (Option B) because a misconfigured Nginx include
> or future location block could silently override the deny rule.
