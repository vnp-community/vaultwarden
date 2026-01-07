# Vaultwarden ELT Documentation

This document describes the database schema, data relationships, and key considerations for performing **Extract, Load, Transform (ELT)** processes on the Vaultwarden database.

> [!IMPORTANT]
> **Encryption Warning**: Much of the meaningful data in Vaultwarden (passwords, notes, credit card numbers, etc.) is stored as **encrypted JSON blobs** or encrypted text strings. An ELT process running on the server side **cannot decrypt this data** without the user's encryption keys, which are never stored on the server. ELT pipelines should generally treat these fields as opaque binary/text blobs.

## 1. Database Overview

Vaultwarden uses an SQL database (SQLite, MySQL/MariaDB, or PostgreSQL) to store all persistent state. The schema is defined using **Diesel ORM**.

### Key Entities

*   **Users**: The system's principals.
*   **Ciphers**: The individual vault items (Logins, Cards, Identities, Notes).
*   **Organizations**: Shared workspaces for multiple users.
*   **Collections**: Logical groupings of Ciphers within an Organization.
*   **Sends**: The "Bitwarden Send" ephemeral data transfer objects.

## 2. Schema Reference

### Users & Authentication

| Table | Description | Key Fields | Extraction Notes |
| :--- | :--- | :--- | :--- |
| `users` | Registered users. | `uuid` (PK), `email`, `created_at`, `updated_at` | `email` is arguably PII. `password_hash` is sensitive. `akey` is the user's encrypted master key. |
| `devices` | Trusted client devices. | `uuid` (PK), `user_uuid` (FK), `push_token` | Useful for analyzing user activity/adoption. |
| `twofactor` | 2FA configurations. | `uuid`, `user_uuid` (FK), `atype` | `data` contains encrypted secrets for the 2FA method. |
| `invitations`| Pending invites. | `email` (PK) | |

### Core Vault Data

| Table | Description | Key Fields | Extraction Notes |
| :--- | :--- | :--- | :--- |
| `ciphers` | Vault items. | `uuid` (PK), `user_uuid` (FK), `organization_uuid` (FK), `data`, `updated_at` | **`data`** is the encrypted payload. `name` and `notes` are also often encrypted or contain metadata only. Supports **Soft Deletes** via `deleted_at`. |
| `folders` | Personal folders. | `uuid` (PK), `user_uuid` (FK), `name` | `name` is encrypted text. |
| `favorites` | User favorites. | `user_uuid`, `cipher_uuid` | Join table. |
| `sends` | Temporal shares. | `uuid` (PK), `data` | **`data`** is encrypted. Contains ephemeral data (check `deletion_date`). |
| `attachments`| File attachments. | `id` (PK), `cipher_uuid` (FK), `file_name` | The actual file content is stored on the filesystem/blob storage, not the DB. |

### Organization & Sharing

| Table | Description | Key Fields | Extraction Notes |
| :--- | :--- | :--- | :--- |
| `organizations` | Org definitions. | `uuid` (PK), `name` | `private_key` and `public_key` are used for org-wide encryption. |
| `users_organizations` | Org membership. | `uuid` (PK), `user_uuid`, `org_uuid` | Defines roles (`atype`) and access level (`status`). |
| `collections` | Org collections. | `uuid` (PK), `org_uuid` (FK) | Grouping mechanism. |
| `groups` | Org groups. | `uuid` (PK), `organizations_uuid` (FK) | Access control groups. |
| `users_collections` | User-Coll Access. | `user_uuid`, `collection_uuid`, `read_only` | Join table defining permissions. |
| `ciphers_collections` | Item-Coll Mapping.| `cipher_uuid`, `collection_uuid` | Maps ciphers to collections. |

## 3. Data Relationships (ERD Mapping)

### User-Centric View
*   **User** `1:N` **Device**
*   **User** `1:N` **Cipher** (Personal items)
*   **User** `1:N` **Folder**
*   **User** `M:N` **Organization** (via `users_organizations`)

### Organization-Centric View
*   **Organization** `1:N` **Collection**
*   **Organization** `1:N` **Group**
*   **Organization** `1:N` **Cipher** (Owned by Org, not User)
*   **Group** `M:N` **User** (via `groups_users`)
*   **Group** `M:N` **Collection** (via `collections_groups`)

## 4. Extraction & Transformation Strategy

### Change Data Capture (CDC)
*   **Incremental extraction** is supported on major tables (`users`, `ciphers`, `devices`) using the `updated_at` timestamp column.
*   **Hard Deletes**: Most tables (except `ciphers`) use hard deletes. If sync history is required, you must capture delete events from the application logic (WebSockets/Events) or use database-level WAL logs, as querying `updated_at` will miss deleted rows.
    *   *Note*: `ciphers` uses **Soft Deletes** (`deleted_at` is set). Only "Purge" operations hard-delete rows.

### Data Privacy & Encryption
*   **Target Warehouse**: If loading into a Data Warehouse (Snowflake, BigQuery, etc.), ensure columns like `data`, `akey`, `private_key`, `totp_secret` are marked as **Sensitive/Variant/Blob**.
*   **Analysis Potential**:
    *   **Count Analysis**: You *can* count users, organizations, ciphers (types), and activity levels securely.
    *   **Content Analysis**: You *cannot* analyze password strength, URL reuse, or note contents server-side.

### Example Query: Active User Statistics

```sql
-- Count of items per user
SELECT
    u.uuid as user_id,
    u.created_at,
    COUNT(c.uuid) as cipher_count
FROM users u
LEFT JOIN ciphers c ON u.uuid = c.user_uuid
WHERE u.enabled = 1
GROUP BY u.uuid, u.created_at;
```

### Example Query: Account Recovery Status

```sql
-- Check which users have e-mail hints or password hints enabled
SELECT
    uuid,
    email,
    CASE WHEN password_hint IS NOT NULL THEN 1 ELSE 0 END as has_hint
FROM users;
```
