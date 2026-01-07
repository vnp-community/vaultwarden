# Vaultwarden Features & Actors

This document outlines the System Actors (roles) and their mapping to the available Features within Vaultwarden.

## 1. Actors (Roles)

Actors represent the different types of users or system identities interacting with Vaultwarden.

### System Level
*   **User**: A standard registered user. Owns a personal vault.
*   **System Admin**: Administrator of the Vaultwarden instance (access to `/admin`).
*   **Anonymous**: An unauthenticated user (e.g., viewing a public Bitwarden Send link or logging in).

### Organization Level
Organizations allow sharing and management of vaults. Roles are hierarchical:

*   **Owner** (Type 0): Full control over the Organization, including billing, deletion, and all settings.
*   **Admin** (Type 1): Can manage users, collections, and groups.
*   **User** (Type 2): Standard member. Can access assigned collections. Read-only or Read-Write depends on permissions.
*   **Manager** (Type 3): A manager with somewhat elevated permissions compared to a User, often used for specific collection management.
*   **Custom** (Type 4): A flexible role where specific permissions (e.g., "Manage Users", "Manage Collections") can be toggled. *Implemented as a Manager variant in Vaultwarden database.*

## 2. Feature List

Key functionalities provided by the system:

1.  **Authentication**: Login, Register, SSO, API Key access.
2.  **Vault Management (Personal)**: CRUD operations on Logins, Cards, Identities, Secure Notes.
3.  **Folder Management**: Organizing personal vault items.
4.  **Sharing (Organization)**: Creating Organizations, Collections, and moving items to them.
5.  **User Management (Org)**: Inviting, confirming, and removing members from an Organization.
6.  **Access Control**: assigning Users/Groups to Collections with specific permissions (Read-Only, Hide Passwords).
7.  **Bitwarden Send**: Creating ephemeral, secure links for sharing text/files with external (Anonymous) users.
8.  **Sync**: Synchronization of vault data across multiple devices.
9.  **Emergency Access**: Granting trusted contacts access to your vault after a timeout.
10. **Attachments**: Uploading files to vault items.
11. **System Administration**: Managing server config, viewing diagnostics, deleting users server-wide.

## 3. Actor-Feature Matrix

| Feature | User | Org Owner | Org Admin | Org User | System Admin | Anonymous |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Register / Login** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Manage Personal Vault** | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Create Organization** | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Delete Organization** | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Invite Members** | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **Manage Collections** | ❌ | ✅ | ✅ | ⚠️ (Assigned) | ❌ | ❌ |
| **Manage Groups** | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **View Org Vault** | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Bitwarden Send (Create)**| ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Bitwarden Send (Read)** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Emergency Access** | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Server Config (/admin)**| ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| **Upload Attachments** | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |

**Legend:**
*   ✅ : Access Granted
*   ❌ : Access Denied
*   ⚠️ : Limited Access (e.g., only specific collections)
