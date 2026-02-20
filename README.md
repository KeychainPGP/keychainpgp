# KeychainPGP

Simple, cross-platform OpenPGP encryption for your clipboard.

**Copy. Encrypt. Paste.** &mdash; **Copy. Decrypt. Read.**

KeychainPGP brings the simplicity of [OpenKeychain](https://www.openkeychain.org/) to desktop: Windows, macOS, and Linux.

## Features

- **Clipboard-first workflow** &mdash; encrypt and decrypt without leaving your app
- **Global hotkeys** &mdash; Ctrl+Shift+E to encrypt, Ctrl+Shift+D to decrypt
- **Modern cryptography** &mdash; Ed25519 + X25519, zero configuration
- **System tray** &mdash; runs quietly in the background
- **Auto-clear clipboard** &mdash; decrypted text is wiped after 30 seconds
- **Cross-platform** &mdash; one app for Windows, macOS, and Linux

## Quick Start

1. Install KeychainPGP
2. Create your keys (name + email, one click)
3. Import a contact's public key
4. Copy text &rarr; Ctrl+Shift+E &rarr; select recipient &rarr; paste encrypted message

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.85+
- [Node.js](https://nodejs.org/) 20+ and npm
- Platform-specific Tauri dependencies: see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

### Build

```bash
# Clone the repository
git clone https://github.com/keychainpgp/keychainpgp.git
cd keychainpgp

# Install frontend dependencies
cd crates/keychainpgp-ui/frontend && npm install && cd ../../..

# Build the desktop app
cargo build --release -p keychainpgp-ui

# Or build the CLI only
cargo build --release -p keychainpgp-cli
```

### Development

```bash
# Run the desktop app in development mode
cd crates/keychainpgp-ui && cargo tauri dev

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace -- -D warnings
```

## Architecture

KeychainPGP is a Rust workspace with five crates:

| Crate | Purpose |
|-------|---------|
| `keychainpgp-core` | OpenPGP crypto operations (Sequoia-PGP) |
| `keychainpgp-keys` | Keyring management (SQLite + OS credential store) |
| `keychainpgp-clipboard` | Clipboard monitoring and auto-clear |
| `keychainpgp-ui` | Tauri desktop application (Svelte frontend) |
| `keychainpgp-cli` | Command-line interface |

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

## Security

Please report security vulnerabilities via the process described in [SECURITY.md](SECURITY.md).
