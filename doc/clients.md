# Vaultwarden Compatible Clients

Vaultwarden implements the Bitwarden API, making it compatible with the vast majority of official Bitwarden clients. Below is a list of the primary client implementations available.

## 1. Official Bitwarden Clients

These are the standard open-source clients developed by Bitwarden Inc. They are fully compatible with Vaultwarden when the "Self-hosted URL" is configured in the app settings.

### Web Vault
*   **Repository**: [bitwarden/clients](https://github.com/bitwarden/clients) (specifically `apps/web`)
*   **Vaultwarden Build**: [dani-garcia/bw_web_builds](https://github.com/dani-garcia/bw_web_builds)
    *   *Note*: Vaultwarden distributes a specific build of the web vault to align with the server version and patch any minor incompatibilities.

### Browser Extensions
*   **Repository**: [bitwarden/clients](https://github.com/bitwarden/clients) (specifically `apps/browser`)
*   **Supported Browsers**: Chrome, Firefox, Opera, Edge, Safari, Vivaldi, Brave, Tor.
*   **Technology**: Angular, Web Extension API.

### Mobile Apps
*   **Repository**: [bitwarden/mobile](https://github.com/bitwarden/mobile)
*   **Platforms**: Android, iOS.
*   **Technology**: C#, Xamarin (Android, iOS, Forms).

### Desktop Applications
*   **Repository**: [bitwarden/clients](https://github.com/bitwarden/clients) (specifically `apps/desktop`)
*   **Platforms**: Windows, macOS, Linux.
*   **Technology**: Electron.

### Command Line Interface (CLI)
*   **Repository**: [bitwarden/clients](https://github.com/bitwarden/clients) (specifically `apps/cli`)
*   **Platforms**: Windows, macOS, Linux.
*   **Technology**: TypeScript, Node.js.

## 2. Community & Third-Party Clients

These are alternative clients developed by the open-source community, often aiming for different performance characteristics (e.g., native code vs Electron) or UI preferences.

### rbw (Unofficial CLI)
*   **Repository**: [doy/rbw](https://git.tozt.net/rbw) (GitHub mirror often available)
*   **Description**: A lightweight, unofficial command-line client written in Rust. It acts as a stateful agent, avoiding the need to frequently re-enter credentials compared to the official CLI.

### Keyguard (Android)
*   **Repository**: [AChep/keyguard-app](https://github.com/AChep/keyguard-app)
*   **Description**: An alternative Android client featuring a modern Material Design UI and support for Bitwarden servers.

### Goldwarden
*   **Repository**: [quexten/goldwarden](https://github.com/quexten/goldwarden)
*   **Description**: A desktop client focused on Linux integration (Flatpak available), offering features like system-wide autofill and better validatibility.
