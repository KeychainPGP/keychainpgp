# Privacy Policy

**Last updated:** February 23, 2026

KeychainPGP is free, open-source software for OpenPGP encryption. This policy explains what data the software handles and how.

## Core Principle

KeychainPGP is designed to operate entirely offline and locally. It does not collect, transmit, or store any user data on external servers.

## Data Handled Locally

### Cryptographic Keys

- **Private keys** are stored on your device in an encrypted SQLite database, protected by your operating system's credential store (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux). A file-based fallback is used when no OS credential store is available.
- **Public keys** are stored locally in the same SQLite database.
- In OPSEC mode, private keys are held only in RAM and are never written to disk.
- You have full control over your keys. You can export, delete, or revoke them at any time.

### Clipboard Data

- KeychainPGP reads your clipboard only when you explicitly invoke an encrypt or decrypt action (via global hotkey or UI button).
- Decrypted text placed on the clipboard is automatically cleared after 30 seconds by default.
- Clipboard data is never transmitted anywhere.

### Application Settings

- User preferences (language, theme, hotkeys, etc.) are stored locally using Tauri's store plugin.
- Settings never leave your device.

## Network Activity

- **KeychainPGP does not phone home.** There is no telemetry, analytics, crash reporting, or automatic update mechanism.
- **Keyserver access**: If you choose to search for or upload keys to a keyserver, that network request is made directly to the keyserver you select. KeychainPGP does not proxy or intercept these requests (unless Tor proxy is enabled in OPSEC mode, in which case traffic is routed through the Tor network).
- **OPSEC mode Tor proxy**: When enabled, network requests are routed through Tor. No data is sent to KeychainPGP maintainers.

## Web Application (WASM)

The browser-based version of KeychainPGP runs entirely in your browser via WebAssembly. No data is sent to any server. All cryptographic operations happen locally in your browser's sandbox.

## Third-Party Services

KeychainPGP does not integrate with any third-party analytics, advertising, or tracking services.

## Data We Do NOT Collect

- No personal information
- No usage statistics or telemetry
- No crash reports
- No IP addresses
- No cookies or browser fingerprints
- No encrypted or decrypted message content

## Children's Privacy

KeychainPGP does not collect any data from anyone, including children.

## Changes to This Policy

Changes to this policy will be documented in the project's Git history and release notes. The "Last updated" date above will be revised accordingly.

## Contact

For privacy-related questions: **keychainpgp@protonmail.com**

For general inquiries, open an issue on [GitHub](https://github.com/keychainpgp/keychainpgp/issues).