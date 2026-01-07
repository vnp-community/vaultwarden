# Vaultwarden API Endpoints

This document lists the available API endpoints in Vaultwarden.

## Identity & Authentication (`/identity`)

| Method | Endpoint | Description | Request Body / Params |
| :--- | :--- | :--- | :--- |
| `POST` | `/connect/token` | **Login**. Issues access tokens. | `grant_type`, `username`, `password`, `scope`, `client_id` (x-www-form-urlencoded) |
| `POST` | `/accounts/prelogin` | **Prelogin**. Gets KDF info. | `{ "email": "user@example.com" }` |
| `POST` | `/accounts/register` | **Register**. Create account. | `{ "email": "...", "masterPasswordHash": "...", "key": "...", ... }` |
| `POST` | `/accounts/verify-email` | **Verify Email**. | `{ "token": "..." }` |
| `GET` | `/accounts/check-email` | **Check Email**. | `?email=user@example.com` |

## Core API (`/api`)

### Accounts (`/api/accounts`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/accounts/profile` | Get user profile info. |
| `PUT` | `/accounts/profile` | Update profile (Name). |
| `POST` | `/accounts/keys` | Update public/private keys. |
| `POST` | `/accounts/password` | Change master password. |
| `POST` | `/accounts/kdf` | Update KDF settings. |
| `POST` | `/accounts/delete` | Delete account. |
| `POST` | `/accounts/revision-date` | Get latest revision date. |

### Sync (`/api/sync`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/sync` | **Full Sync**. Returns ciphers, folders, etc. |

### Ciphers (`/api/ciphers`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/ciphers` | List all ciphers. |
| `GET` | `/ciphers/<uuid>` | Get specific cipher details. |
| `POST` | `/ciphers` | Create a new cipher. Request body includes `type`, `name`, `login` info, `folderId`. |
| `PUT` | `/ciphers/<uuid>` | Update a cipher. |
| `PUT` | `/ciphers/<uuid>/partial` | Partial update (e.g., move folder). |
| `DELETE` | `/ciphers/<uuid>` | Soft delete (trash). |
| `POST` | `/ciphers/<uuid>/attachment` | Upload attachment (multipart/form-data). |

### Organizations (`/api/organizations`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/organizations` | Create Organization. |
| `GET` | `/organizations/<uuid>` | Get Organization info. |
| `GET` | `/organizations/<uuid>/collections` | List Collections. |
| `POST` | `/organizations/<uuid>/collections` | Create Collection. |
| `GET` | `/organizations/<uuid>/members` | List Members. |
| `POST` | `/organizations/<uuid>/users/invite` | Invite Members. |

### Collections (`/api/collections`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/collections` | List user's collections. |
| `GET` | `/collections/<uuid>/details` | Get collection details. |

### Folders (`/api/folders`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/folders` | List folders. |
| `POST` | `/folders` | Create folder. |

### Sends (`/api/sends`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/sends` | List Sends. |
| `POST` | `/sends` | Create Send. |
| `POST` | `/sends/file/v2` | Upload Send File. |

## 2FA (`/api/two-factor`)
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/two-factor` | List providers. |
| `POST` | `/two-factor/get-authenticator` | Generate TOTP secret. |
| `POST` | `/two-factor/authenticator` | Enable TOTP. |

## System
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/config` | Server configuration. |
| `GET` | `/api/version` | Server version. |
| `GET` | `/items` | **Alias** for `/ciphers`. |
| `GET` | `/projects` | **Alias** for `/folders`. |
