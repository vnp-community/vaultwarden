# Identity Management in Vaultwarden

Vaultwarden implements a comprehensive identity management system compatible with Bitwarden clients, handling user registration, authentication (including SSO and 2FA), and cryptographic key management.

## 1. User Registration (`/accounts/register`)

Registration is the entry point for new identities. It handles multiple scenarios:

1.  **Open Signups**: If enabled in config (`SIGNUPS_ALLOWED=true`), any user can register.
2.  **Invitations**: Users invited to an Organization can register even if open signups are disabled.
3.  **Emergency Access**: Users granted emergency access can register to accept it.
4.  **Email Verification**: If configured, a verification token (JWT) is required or sent during the process.

**Data Flow**:
*   Input: Email, Master Password Hash, User Symmetric Key, Public/Private Key Pair (encrypted), KDF settings.
*   Storage: A new `User` record is created in the database.
*   Key Interaction: The User's "Key" is stored encrypted with the Master Password. The `Private Key` is also stored encrypted.

## 2. Authentication Flows

### A. Password Login (`/connect/token` grant_type="password")
The standard login flow:
1.  **Validation**: Checks client_id, scope, and user existence.
2.  **Proof of Work**: User sends credentials. Server validates the `Password Hash`.
3.  **KDF Upgrade**: If the client is using older KDF settings than stored, they are updated.
4.  **2FA Check**: If 2FA is enabled, the server returns a temporary token and requires a second step.
5.  **Success**: Returns Access Token (JWT), Refresh Token, and encrypted User Key/Private Key.

### B. Single Sign-On (SSO)
Leverages OIDC (OpenID Connect) for authentication.
*   **Flow**: `authorization_code` grant type.
*   **Logic**:
    1.  Client redirects user to Identity Provider (IdP).
    2.  IdP redirects back with a `code`.
    3.  Vaultwarden exchanges `code` for user info.
    4.  **Linking**: Matches email to existing User. If `SSO_ONLY` is enabled, password login is blocked.
    5.  **Note**: SSO handles *authentication* (who you are), but *decryption* still requires the Master Password (or Key Connector, though strictly Key Connector is enterprise-only).

### C. API Key Login (`client_credentials`)
Used for CLI and automated tools.
*   **User API Key**: `client_id=user.<uuid>`, `client_secret=<user_api_key>`.
*   **Org API Key**: `client_id=organization.<uuid>`, `client_secret=<org_api_key>`.
*   **Characteristics**: Bypasses 2FA. Returns limited scope tokens.

## 3. Two-Factor Authentication (2FA)

Vaultwarden supports multiple 2FA methods, managed via `src/api/core/two_factor/`:

*   **Authenticator (TOTP)**: Standard Time-based One-Time Password.
*   **WebAuthn (FIDO2)**: Hardware keys (YubiKey) or Biometrics (FaceID/TouchID).
*   **YubiKey OTP**: Classic YubiKey OTP protocol.
*   **Duo**: Push notifications via Duo Security (supports both Iframe and OIDC).
*   **Email**: Sending a verification code to the user's email.

**Policy Enforcement**:
Organizations can enforce "Two-Step Login" policies. If a user in such an org logs in without 2FA, they are prompted to set it up or blocked.

## 4. Cryptographic Identity

A user's identity is fundamentally tied to their cryptographic keys, not just their database record.

*   **Master Password**: Never stored. Only a hash is stored.
*   **Master Key**: Derived from Master Password using PBKDF2 or Argon2id.
*   **User Key**: A symmetric key used to encrypt the vault. Stored in DB, *encrypted by the Master Key*.
*   **Key Rotation**:
    *   **Password Change**: Re-encrypts the User Key with the new Master Key.
    *   **Key Rotation**: Generates a *new* User Key, re-encrypts all vault data (ciphers, folders) with the new key.

## 5. Session Management

*   **Access Token**: JWT, valid for 2 hours.
*   **Refresh Token**: Random string (stored in DB against `Device`), valid indefinitely (until revoked). Used to request new Access Tokens.
*   **Security Stamp**: A UUID in the User record. Changing password/keys rotates this stamp, invalidating all active JWTs instantly.

## 6. Account Lifecycle

*   **Locking**: Too many failed attempts trigger a temporary IP ban (Rate Limiting).
*   **Deletion**: Users can delete their account. This removes all personal data and removes them from Organizations.
*   **Recover Delete**: A specific token-based flow to delete an account if the password is lost (optional feature).
