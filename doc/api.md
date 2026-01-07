# Vaultwarden API Endpoints

This document lists the available API endpoints in Vaultwarden, categorized by their function. These endpoints are generally compatible with the Bitwarden Client API.

## Identity & Authentication (`/identity`)

These endpoints handle user authentication, registration, and token management.

*   `POST /connect/token` - **Login**. Handles password authentication, API key login, and SSO flow to issue access/refresh tokens.
*   `POST /accounts/prelogin` - **Prelogin**. Returns KDF information (iterations, salt) for the email to allow the client to derive the master key.
*   `POST /accounts/register` - **Register**. Create a new user account.
*   `POST /accounts/verify-email` - **Verify Email**. Verifies a user's email address using a token.
*   `POST /accounts/verify-email-token` - **Resend Verification**. Resends the email verification token.
*   `GET /accounts/check-email` - **Check Email**. Checks if an email is already registered. (Often used during pre-flight).

## Core API (`/api`)

The core API handles the main vault data management.

### Accounts (`/api/accounts`)
*   `GET /accounts/profile` - **Get Profile**. Returns the user's profile information.
*   `PUT /accounts/profile` - **Update Profile**. Updates user profile (name, etc.).
*   `POST /accounts/profile` - **Update Profile** (Alternative).
*   `PUT /accounts/avatar` - **Update Avatar**. Sets a custom color for the user avatar.
*   `POST /accounts/keys` - **Update Keys**. Updates the user's public/private keys (e.g., during rotation).
*   `POST /accounts/password` - **Change Password**. Changes the master password.
*   `POST /accounts/kdf` - **Update KDF**. Updates the KDF settings (PBKDF2/Argon2id) and re-encrypts the master key.
*   `POST /accounts/rotate-key` - **Rotate Key**. Rotates the account's encryption key.
*   `POST /accounts/delete` - **Delete Account**. Deletes the user account.
*   `POST /accounts/revision-date` - **Get Revision Date**. Returns the latest revision date for client sync.

### Sync (`/api/sync`)
*   `GET /sync` - **Sync Vault**. Returns the complete vault state (ciphers, folders, collections, policies, etc.) for the authenticated user.

### Ciphers (`/api/ciphers`)
*   `GET /ciphers` - **List Ciphers**. Returns a list of all visible ciphers.
*   `GET /ciphers/<uuid>` - **Get Cipher**. Returns details of a specific cipher.
*   `POST /ciphers` - **Create Cipher**. Creates a new cipher (Login, Card, Identity, Note).
*   `PUT /ciphers/<uuid>` - **Update Cipher**. Updates an existing cipher.
*   `POST /ciphers/<uuid>` - **Update Cipher** (Alternative).
*   `POST /ciphers/<uuid>/partial` - **Partial Update**. Updates specific fields (folder, favorite) without full write access.
*   `DELETE /ciphers/<uuid>` - **Delete Cipher**. Soft deletes (trashes) a cipher.
*   `PUT /ciphers/<uuid>/restore` - **Restore Cipher**. Restores a cipher from trash.
*   `DELETE /ciphers/<uuid>/delete` - **Permanent Delete**. Permanently removes a cipher (admin/owner only usually).
*   `POST /ciphers/import` - **Import**. Imports ciphers and folders from a structural JSON object.
*   `POST /ciphers/<uuid>/attachment` - **Upload Attachment**. Uploads a file attachment for a cipher.
*   `DELETE /ciphers/<uuid>/attachment/<attachment_id>` - **Delete Attachment**.

### Organizations (`/api/organizations`)
*   `POST /organizations` - **Create Organization**.
*   `GET /organizations/<uuid>` - **Get Organization**.
*   `PUT /organizations/<uuid>` - **Update Organization**.
*   `DELETE /organizations/<uuid>` - **Delete Organization**.
*   `POST /organizations/<uuid>/leave` - **Leave Organization**.
*   `GET /organizations/<uuid>/collections` - **List Collections**.
*   `POST /organizations/<uuid>/collections` - **Create Collection**.
*   `GET /organizations/<uuid>/members` - **List Members**.
*   `POST /organizations/<uuid>/users/invite` - **Invite User**. Invites a user to the organization.
*   `POST /organizations/<uuid>/users/confirm` - **Confirm Invite**. Confirms a user's acceptance of an invite.
*   `DELETE /organizations/<uuid>/users/<user_uuid>` - **Remove User**. Removes a user from the organization.
*   `GET /organizations/<uuid>/policies` - **List Policies**.
*   `PUT /organizations/<uuid>/policies/<type>` - **Update Policy**. Enables/disables/configures an org policy.

### Collections (`/api/collections`)
*   `GET /collections` - **List User Collections**. Lists collections the user has access to.
*   `GET /collections/<uuid>/details` - **Collection Details**. Returns permissions and groups for a collection.

### Folders (`/api/folders`)
*   `GET /folders` - **List Folders**.
*   `GET /folders/<uuid>` - **Get Folder**.
*   `POST /folders` - **Create Folder**.
*   `PUT /folders/<uuid>` - **Update Folder**.
*   `DELETE /folders/<uuid>` - **Delete Folder**.

### Sends (`/api/sends`)
*   `GET /sends` - **List Sends**.
*   `POST /sends` - **Create Send**. Creates a new Send object (Text or File).
*   `GET /sends/<uuid>` - **Get Send**.
*   `PUT /sends/<uuid>` - **Update Send**.
*   `DELETE /sends/<uuid>` - **Delete Send**.
*   `POST /sends/file/v2` - **Upload Send File**. Uploads a file for a Send.
*   `POST /sends/<uuid>/access/file/<file_id>` - **Access Send File**. (Public access endpoint for downloading).

### Two-Factor (`/api/two-factor`)
*   `GET /two-factor` - **List Providers**. Returns available/enabled 2FA providers.
*   `POST /two-factor/get-authenticator` - **Get TOTP Key**. Generates a new TOTP secret.
*   `POST /two-factor/authenticator` - **Enable TOTP**. Validates and enables TOTP.
*   `POST /two-factor/email` - **Enable Email**. Enables Email 2FA.
*   `POST /two-factor/duo` - **Enable Duo**.
*   `POST /two-factor/webauthn` - **Enable WebAuthn**. Registers a FIDO2/WebAuthn credential.
*   `POST /two-factor/disable` - **Disable Provider**. Disables a specific 2FA provider.

## Public / Admin (`/admin`, `/public`)

*   `/admin/*` - **Admin Panel**. Vaultwarden-specific admin interface endpoints (requires admin token).
*   `/public/organization/import` - **Directory Connector**. Endpoint used by the Bitwarden Directory Connector to sync users/groups from LDAP/AD.
*   `/notifications/hub` - **WebSocket Hub**. Endpoint for WebSocket connections to receive real-time updates.
