# Authorization & Access Control

Vaultwarden utilizes a hierarchical and role-based access control (RBAC) system, primarily implemented through Rocket Request Guards and JSON Web Tokens (JWT).

## 1. Request Guards

The core of the authorization logic is embedded in `Request Guard` structs. These are used in API endpoint signatures to ensure the caller has the necessary permissions *before* the handler logic is executed.

### Core Guards

| Guard | Checks Performed | Usage |
| :--- | :--- | :--- |
| `Headers` | Validates the **Bearer Token** (JWT). <br> Ensures the `device_uuid` and `user_uuid` in the claim exist in the database. <br> **Crucially**: Validates the `security_stamp`. If the user changed their password or rotated keys, the stamp changes, invalidating old tokens. | Base guard for any authenticated user endpoint. |
| `OrgHeaders`| Extends `Headers`. <br> Checks if the authenticated user is a **member** of the target Organization (provided via URL or Query param). | Base guard for organization-related endpoints. |
| `OrgMemberHeaders` | Extends `OrgHeaders`. <br> Explicitly checks `membership_type >= User`. | Ensures user is not just related but active. |

### Role-Specific Guards (Organization)

These guards enforce specific roles within an Organization context:

| Guard | Role Requirement | Description |
| :--- | :--- | :--- |
| `OwnerHeaders` | **Owner** (Type 0) | Required for critical actions like deleting the org or changing billing settings. |
| `AdminHeaders` | **Admin** (Type 1) or higher | Allows user management, collection creation, and group management. |
| `ManagerHeaders` | **Manager** (Type 3) or higher | Allows management of assigned collections. **Includes a check** to ensure the user has `manage` permission on the *specific collection* being accessed. |
| `ManagerHeadersLoose` | **Manager** (Type 3) or higher | Same as above but without the specific collection check (used for list endpoints). |

## 2. Resource-Level Permission Logic

Beyond the global request guards, specific data access logic handles fine-grained permissions, particularly for **Ciphers** (Vault Items).

### Cipher Access (`get_access_restrictions`)

Access to a shared cipher is calculated dynamically based on:
1.  **Direct Ownership**: If the user created the cipher (personal vault), they have full access.
2.  **Organization Ownership**: If the cipher belongs to an org, access is determined by:
    *   **Implicit Full Access**: If the user is an **Owner** or **Admin** (or has `AccessAll` flag), they have full R/W access.
    *   **Collection Assignment**: If the cipher is in a Collection the user has been assigned to.
    *   **Group Assignment**: If the cipher is in a Collection assigned to a Group the user is in.

**Conflict Resolution**:
*   A cipher can be in multiple collections.
*   Permissions (`ReadOnly`, `HidePasswords`) are aggregated.
*   **Logic**: A user needs access to *at least one* collection containing the cipher.
*   **Rule**: Permissions are generally **restrictive** (AND logic) across collections for flags like `ReadOnly`, but `Manage` is **additive** (OR logic).

### Collection Access (`can_access_collection`)

Determines if a user can see/interact with a Collection:
*   **Owners/Admins**: Always return true.
*   **Users/Managers**: Must have an explicit `users_collections` record OR be in a `group` linked via `collections_groups`.

## 3. Policy Enforcement

Enterprise Policies (e.g., "Disable Send", "Require 2FA") are checked ad-hoc within endpoints.
*   Helper functions like `enforce_disable_send_policy` check the `org_policies` table.
*   These are usually called immediately after the Request Guard.

## 4. Authentication Tokens (JWT)

*   **Login Token**: Short-lived (default 2 hours). Contains `sub` (User UUID), `sstamp` (Security Stamp), `device`, and `client_id`.
*   **Refresh Token**: Long-lived (never expires until revoked/rotated). Used to get new Login Tokens. Stored in the database as `device.refresh_token`.
*   **Invite Token**: Used for joining organizations.
*   **Admin Token**: Specific to the `/admin` page, signed with a separate secret/issuer.

## 5. Security Stamp (`sstamp`)

The security stamp is a critical security feature. It is a random UUID stored on the `User` record and embedded in the JWT.
*   **Rotation**: It is rotated whenever the master password is changed, keys are rotated, or sessions are explicitly revoked.
*   **Effect**: When the stamp rotates in the DB, all existing JWTs (which contain the *old* stamp) fail validation in the `Headers` guard, forcing all devices to re-authenticate.
