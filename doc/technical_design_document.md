# Technical Design Document: Vaultwarden

## 1. Introduction

**Vaultwarden** is an unofficial server implementation of the Bitwarden API, written in Rust. It aims to provide a lightweight, self-hosted alternative to the official Bitwarden server, primarily targeting individuals, families, and smaller organizations.

This document outlines the technical architecture, data models, and system design of the Vaultwarden codebase.

## 2. System Overview

### 2.1 High-Level Architecture

Vaultwarden follows a monolithic architecture pattern typical of web applications. It serves as a backend API that interfaces with a database and clients (Web Vault, Mobile Apps, Browser Extensions, CLI).

*   **Runtime**: Single binary executable handling HTTP requests, WebSocket connections, and background jobs.
*   **Web Framework**: **Rocket** (v0.5) is used for routing, request handling, and state management.
*   **Asynchronous Processing**: **Tokio** runtime powers the async I/O.
*   **Database**: **Diesel** ORM provides an abstraction layer over SQLite, MySQL, and PostgreSQL.

### 2.2 Technology Stack

*   **Language**: Rust (Edition 2021)
*   **Web Framework**: Rocket
*   **Database**: SQLite (default), MySQL/MariaDB, PostgreSQL
*   **ORM**: Diesel
*   **Async Runtime**: Tokio
*   **Serialization**: Serde & Serde JSON
*   **Logging**: Fern & Syslog
*   **Template Engine**: Handlebars (for emails)

## 3. Database Design

The database schema is managed via migrations (Diesel). The core entities revolve around Users, Organizations, and Ciphers (vault items).

### 3.1 Key Entities (Tables)

*   **`users`**: Stores user accounts.
    *   `uuid`: Primary Key.
    *   `email`, `password_hash`, `salt`: Auth credentials.
    *   `private_key`, `public_key`: User's RSA keys (encrypted).
    *   `akey`: Account key (encrypted).
*   **`ciphers`**: The core vault items (Logins, Cards, Identities, Notes).
    *   `uuid`: Primary Key.
    *   `user_uuid` / `organization_uuid`: Ownership.
    *   `data`: The encrypted payload (JSON structure containing username, password, etc., encrypted by the client).
    *   `atype`: Type of cipher (1=Login, 2=Note, etc.).
*   **`organizations`**: Groups of users sharing ciphers.
    *   `uuid`: Primary Key.
    *   `private_key`, `public_key`: Organization's asymmetric keys.
*   **`collections`**: Folders/Tags within an organization for access control.
    *   `uuid`, `org_uuid`.
*   **`users_organizations`**: Many-to-Many link between Users and Orgs, storing access levels and keys.
    *   `akey`: The Org's key encrypted with the User's key.
*   **`devices`**: Tracks logged-in client sessions.
    *   `uuid`, `user_uuid`, `refresh_token`.
*   **`twofactor`**: Stores 2FA configurations (TOTP, WebAuthn, Duo, etc.).

### 3.2 Data Relationships

*   **User -> Ciphers**: One-to-Many (Personal vault).
*   **Organization -> Ciphers**: One-to-Many (Shared vault).
*   **User <-> Organization**: Many-to-Many (via `users_organizations`).
*   **User -> Devices**: One-to-Many.

## 4. API Design

The API mirrors the official Bitwarden API to ensure compatibility with official clients.

### 4.1 Route Structure (`src/api/`)

*   **`identity.rs`** (`/identity`): Authentication endpoints.
    *   `/connect/token`: Login (Password, Refresh Token, API Key).
    *   `/accounts/prelogin`: KDF parameters retrieval.
    *   `/accounts/register`: User registration.
*   **`core/`** (`/api`): Main application logic.
    *   `ciphers.rs`: CRUD operations for vault items.
    *   `sends.rs`: Bitwarden Send implementation.
    *   `organizations.rs`: Org management.
*   **`admin.rs`** (`/admin`): Special internal admin interface for server management (not part of Bitwarden spec).
*   **`notifications.rs`** (`/notifications`): WebSocket handler for real-time updates.

### 4.2 Request Handling

*   **Request Guards**: Rocket's `FromRequest` trait is used extensively in `src/auth.rs` to validate headers and populate context (User, Device, DB Connection) before a handler runs.
    *   `Headers`: Validates Bearer token and loads `User` + `Device`.
    *   `AdminHeaders`: Ensures the user is an admin.

## 5. Security Architecture

### 5.1 Authentication

*   **JWT (JSON Web Tokens)**: Used for stateless authentication.
    *   **Access Token**: Short-lived (default 2 hours).
    *   **Refresh Token**: Long-lived, stored in the DB (`devices` table).
*   **KDF (Key Derivation Function)**: PBKDF2 or Argon2id is used to derive:
    1.  **Master Key**: (Client-side) Used to encrypt the `akey`.
    2.  **Master Password Hash**: (Server-side) Sent to server for authentication.

### 5.2 Encryption (Zero-Knowledge)

Vaultwarden (like Bitwarden) operates on a **zero-knowledge** architecture.
*   The server **never** sees unencrypted data for ciphers.
*   **Client**: Encrypts data using the generated symmetric key (`akey`).
*   **Server**: Stores the encrypted `data` blob in the `ciphers` table.
*   **Sharing**: Implementation of sharing involves encrypting the cipher's key with the Organization's key, which is in turn available to members encrypted with their personal keys.

### 5.3 2FA (Two-Factor Authentication)

Supported methods implemented in `src/api/core/two_factor/`:
*   **Authenticator**: Standard TOTP.
*   **Email**: verification code via SMTP.
*   **Duo**: Duo Security integration.
*   **YubiKey**: Yubico OTP.
*   **WebAuthn/FIDO2**: Hardware keys (Passkeys).

## 6. Project Structure

```
src/
├── api/             # HTTP Route Handlers
│   ├── core/        # Core Vault features (Ciphers, Orgs)
│   ├── identity.rs  # Auth routes
│   └── admin.rs     # Admin panel
├── db/              # Database Layer
│   ├── models/      # Diesel Structs & Logic
│   └── schema.rs    # Database Schema
├── auth.rs          # core Auth logic (JWT, Request Guards)
├── main.rs          # Entry point, Config, Startup
├── config.rs        # Configuration management
├── crypto.rs        # Cryptographic utilities
└── mail.rs          # Email sending logic
```

## 7. Operational Details

*   **Configuration**: Handled via `.env` file and environment variables (loaded in `config.rs`).
*   **Background Jobs**: `schedule_jobs` in `main.rs` handles cleanup tasks (purging events, expiration).
*   **WebSocket**: Used to notify clients of database changes (sync requests).

