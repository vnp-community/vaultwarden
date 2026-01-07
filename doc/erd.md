# Vaultwarden Entity Relationship Diagram (ERD)

This document outlines the data models and their relationships within the Vaultwarden database.

## List of Models

The following is a list of tables (models) defined in the database schema:

| Table Name | Description | Key Relationships |
| :--- | :--- | :--- |
| `users` | The central user entity. | Related to Devices, Ciphers, Folders, Organizations (via users_organizations). |
| `devices` | Client devices/sessions. | Belongs to `users`. |
| `twofactor` | 2FA configurations. | Belongs to `users`. |
| `ciphers` | Vault items (logins, notes, etc.). | Belongs to `users` OR `organizations`. |
| `folders` | User sub-folders for organization. | Belongs to `users`. |
| `folders_ciphers` | Many-to-Many link. | Links `folders` and `ciphers`. |
| `organizations` | Organization entities. | Related to Users, Collections, Groups, Ciphers. |
| `users_organizations` | Org Members. | Links `users` and `organizations`. Defines membership status/keys. |
| `collections` | Logical groupings in Orgs. | Belongs to `organizations`. |
| `users_collections` | Access control. | Links `users` and `collections` (Direct access). |
| `ciphers_collections`| Organization. | Links `ciphers` and `collections`. |
| `groups` | User groups in Orgs. | Belongs to `organizations`. |
| `groups_users` | Group membership. | Links `groups` and `users_organizations`. |
| `collections_groups` | Group Info Access. | Links `collections` and `groups`. |
| `org_policies` | Enterprise policies. | Belongs to `organizations`. |
| `sends` | Bitwarden Sends. | Belongs to `users` or `organizations`. |
| `attachments` | File metadata. | Belongs to `ciphers`. |
| `favorites` | User favorites. | Links `users` and `ciphers`. |
| `emergency_access` | Trusted contacts. | Links `users` (grantor) and `users` (grantee). |
| `invitations` | Pending email invites. | Independent (references email). |
| `auth_requests` | Mobile/Desktop biometric auth. | Belongs to `users`. |

## Database Schema Diagram

```mermaid
erDiagram
    %% User Core
    USERS ||--o{ DEVICES : "has"
    USERS ||--o{ TWOFACTOR : "enables"
    USERS ||--o{ CIPHERS : "owns (personal)"
    USERS ||--o{ FOLDERS : "manages"
    USERS ||--o{ USERS_ORGANIZATIONS : "joins"
    USERS ||--o{ EMERGENCY_ACCESS : "grants/receives"
    USERS ||--o{ SENDS : "creates"

    %% Organization Core
    ORGANIZATIONS ||--o{ USERS_ORGANIZATIONS : "has members"
    ORGANIZATIONS ||--o{ COLLECTIONS : "contains"
    ORGANIZATIONS ||--o{ GROUPS : "defines"
    ORGANIZATIONS ||--o{ ORG_POLICIES : "enforces"
    ORGANIZATIONS ||--o{ CIPHERS : "owns (shared)"

    %% Vault Items
    CIPHERS ||--o{ ATTACHMENTS : "has"
    CIPHERS ||--o{ CIPHERS_COLLECTIONS : "in"
    CIPHERS ||--o{ FAVORITES : "favorited by"
    FOLDERS ||--o{ FOLDERS_CIPHERS : "contains"
    CIPHERS ||--o{ FOLDERS_CIPHERS : "belongs to"

    %% Collections & Sharing
    COLLECTIONS ||--o{ CIPHERS_COLLECTIONS : "contains"
    COLLECTIONS ||--o{ USERS_COLLECTIONS : "assigned to user"
    COLLECTIONS ||--o{ COLLECTIONS_GROUPS : "assigned to group"

    %% Groups
    GROUPS ||--o{ GROUPS_USERS : "has members"
    USERS_ORGANIZATIONS ||--o{ GROUPS_USERS : "member of"
    GROUPS ||--o{ COLLECTIONS_GROUPS : "accesses"

    %% Table Definitions
    USERS {
        uuid PK
        email string
        password_hash blob
        akey string
        private_key string
        public_key string
    }

    DEVICES {
        uuid PK
        user_uuid FK
        push_token string
    }

    CIPHERS {
        uuid PK
        user_uuid FK "nullable"
        organization_uuid FK "nullable"
        data blob "Encrypted content"
        type int "Login, Note, Card, etc"
    }

    ORGANIZATIONS {
        uuid PK
        name string
        private_key string
        public_key string
    }

    COLLECTIONS {
        uuid PK
        org_uuid FK
        name string
    }

    USERS_ORGANIZATIONS {
        uuid PK
        user_uuid FK
        org_uuid FK
        access_all boolean
        type int "Owner, Admin, User..."
    }

    GROUPS {
        uuid PK
        organizations_uuid FK
        name string
    }
```
