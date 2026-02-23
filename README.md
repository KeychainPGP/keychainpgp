<p align="center">
  <img src="crates/keychainpgp-ui/icons/128x128@2x.png" alt="KeychainPGP" width="128" height="128">
</p>

<h1 align="center">KeychainPGP</h1>

<p align="center">
  <strong>Simple, cross-platform OpenPGP encryption for your clipboard.</strong><br>
  <em>Copy. Encrypt. Paste.</em> &mdash; <em>Copy. Decrypt. Read.</em>
</p>

<p align="center">
  <a href="https://github.com/keychainpgp/keychainpgp/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/keychainpgp/keychainpgp/ci.yml?branch=master&style=flat-square&logo=github&label=CI" alt="CI"></a>
  <a href="https://github.com/keychainpgp/keychainpgp/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/keychainpgp/keychainpgp/release.yml?style=flat-square&logo=github&label=Release" alt="Release"></a>
  <a href="https://github.com/keychainpgp/keychainpgp/releases/latest"><img src="https://img.shields.io/github/v/release/keychainpgp/keychainpgp?style=flat-square&color=blue" alt="Latest Release"></a>
  <a href="https://github.com/keychainpgp/keychainpgp/blob/master/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square" alt="License"></a>
</p>

<p align="center">
  <a href="https://github.com/nicedoc/rust-toolchain"><img src="https://img.shields.io/badge/rust-1.88%2B-orange?style=flat-square&logo=rust" alt="Rust 1.88+"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/tauri-v2-24C8D8?style=flat-square&logo=tauri&logoColor=white" alt="Tauri v2"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/svelte-5-FF3E00?style=flat-square&logo=svelte&logoColor=white" alt="Svelte 5"></a>
  <a href="https://sequoia-pgp.org/"><img src="https://img.shields.io/badge/crypto-Sequoia--PGP-green?style=flat-square" alt="Sequoia-PGP"></a>
</p>

<p align="center">
  <a href="#-installation">Installation</a> &bull;
  <a href="#-features">Features</a> &bull;
  <a href="#-quick-start">Quick Start</a> &bull;
  <a href="#-building-from-source">Build</a> &bull;
  <a href="#-contributing">Contributing</a>
</p>

---

KeychainPGP brings the simplicity of [OpenKeychain](https://www.openkeychain.org/) to the desktop. No command-line flags, no certificate managers, no configuration &mdash; just encryption that works.

## Screenshots

<!-- Replace these with actual screenshots of your app -->
<!--
<p align="center">
  <img src=".github/assets/screenshot-home.png" alt="Home" width="400">
  <img src=".github/assets/screenshot-keys.png" alt="Key Management" width="400">
</p>
-->

> Screenshots coming soon. Want to help? See [Contributing](#-contributing).

## Download

### Desktop

| Platform | Formats |
|----------|---------|
| **Windows** | [`.exe` (NSIS)](https://github.com/keychainpgp/keychainpgp/releases/latest) &middot; [`.msi`](https://github.com/keychainpgp/keychainpgp/releases/latest) |
| **macOS** | [`.dmg`](https://github.com/keychainpgp/keychainpgp/releases/latest) |
| **Linux** | [`.AppImage`](https://github.com/keychainpgp/keychainpgp/releases/latest) &middot; [`.deb`](https://github.com/keychainpgp/keychainpgp/releases/latest) |

### Android

<a href="https://github.com/keychainpgp/keychainpgp/releases/latest"><img src="https://img.shields.io/badge/Get_it_on-GitHub_Releases-blue?style=for-the-badge&logo=github" alt="GitHub Releases" height="50"></a>

**Via [Obtainium](https://obtainium.imranr.dev/)** &mdash; get automatic updates directly from GitHub:

<a href="https://apps.obtainium.imranr.dev/redirect?r=obtainium://app/%7B%22id%22%3A%22com.keychainpgp.app%22%2C%22url%22%3A%22https%3A%2F%2Fgithub.com%2Fkeychainpgp%2Fkeychainpgp%22%2C%22author%22%3A%22keychainpgp%22%2C%22name%22%3A%22KeychainPGP%22%7D"><img src="https://img.shields.io/badge/Get_it_on-Obtainium-teal?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJ3aGl0ZSI+PHBhdGggZD0iTTEyIDJMMTIgMTZNMTIgMTZMNiAxME0xMiAxNkwxOCAxME0yIDE4TDIgMjJMMjIgMjJMMjIgMTgiLz48L3N2Zz4=" alt="Get it on Obtainium" height="50"></a>

> [!TIP]
> [Obtainium](https://github.com/ImranR98/Obtainium) lets you install and update apps directly from their GitHub releases. No app store required.

### Web App

Try KeychainPGP directly in your browser (WASM-powered, nothing leaves your device):

> **[web.keychainpgp.org](https://keychainpgp.org/web)** *(coming soon)*

## Features

- **Clipboard-first workflow** &mdash; encrypt and decrypt without leaving your app
- **Global hotkeys** &mdash; <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd> to encrypt, <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>D</kbd> to decrypt
- **Modern cryptography** &mdash; Ed25519 + X25519, powered by [Sequoia-PGP](https://sequoia-pgp.org/)
- **System tray** &mdash; runs quietly in the background
- **Auto-clear clipboard** &mdash; decrypted text is wiped after 30 seconds
- **Compose mode** &mdash; draft, encrypt, and sign messages in one window
- **OPSEC mode** &mdash; RAM-only keys, no disk traces, Tor proxy support
- **20 languages** &mdash; full i18n via [Paraglide](https://inlang.com/m/gerre34r/library-inlang-paraglideJs)
- **Mobile companion** &mdash; Android app with key sync via QR codes
- **Portable mode** &mdash; drop a `.portable` marker and run from a USB stick
- **Cross-platform** &mdash; Windows, macOS, Linux, Android, and Web (WASM)

## Quick Start

1. **Install** KeychainPGP (see [Download](#download))
2. **Create your keys** &mdash; name + email, one click
3. **Import a contact's** public key
4. **Encrypt** &mdash; copy text &rarr; <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd> &rarr; select recipient &rarr; paste
5. **Decrypt** &mdash; copy PGP message &rarr; <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>D</kbd> &rarr; read

## Architecture

KeychainPGP is a Rust workspace with six crates:

```
keychainpgp/
├── crates/
│   ├── keychainpgp-core       # OpenPGP crypto engine (Sequoia-PGP)
│   ├── keychainpgp-keys       # Keyring: SQLite + OS credential store
│   ├── keychainpgp-clipboard  # Clipboard monitoring, PGP detection, auto-clear
│   ├── keychainpgp-ui         # Tauri v2 desktop + Android app (Svelte 5)
│   ├── keychainpgp-cli        # CLI binary
│   └── keychainpgp-wasm       # WASM bindings for the web app
└── web/                        # Standalone Vite + Svelte 5 SPA (WASM)
```

| Crate | Purpose |
|-------|---------|
| `keychainpgp-core` | Pure-Rust OpenPGP operations &mdash; encrypt, decrypt, sign, verify. No I/O. |
| `keychainpgp-keys` | Key storage with SQLite and OS credential store (Keychain/DPAPI/Secret Service) |
| `keychainpgp-clipboard` | Clipboard watching, PGP message detection, timed auto-clear |
| `keychainpgp-ui` | Tauri v2 desktop & mobile app with Svelte 5 frontend |
| `keychainpgp-cli` | Command-line interface for scripting and headless use |
| `keychainpgp-wasm` | WebAssembly bindings &mdash; same crypto engine in the browser |

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.88+
- [Node.js](https://nodejs.org/) 20+ and npm
- Platform-specific Tauri v2 dependencies &mdash; see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

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
# Run the desktop app in dev mode (hot-reload)
cd crates/keychainpgp-ui && cargo tauri dev

# Run all tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

### WASM / Web App

```bash
# Build the WASM package
wasm-pack build crates/keychainpgp-wasm --target web --out-dir ../../web/pkg

# Run the web app locally
cd web && npm install && npm run dev
```

### Android

```bash
# Requires JDK 17+ and Android NDK
cd crates/keychainpgp-ui
npx @tauri-apps/cli android build --debug
```

## Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md) before submitting a PR.

**TL;DR:**

1. Fork & clone
2. `git checkout -b feature/your-feature`
3. Make changes, add tests
4. `cargo test --workspace && cargo clippy --workspace -- -D warnings`
5. Commit with [Conventional Commits](https://www.conventionalcommits.org/) format
6. Open a pull request

This project follows the [Contributor Covenant v2.1](CODE_OF_CONDUCT.md).

## Security

If you discover a security vulnerability, **do NOT open a public issue**. Please email **keychainpgp@protonmail.com** instead.

See [SECURITY.md](SECURITY.md) for full details.

## Donate

If you find KeychainPGP useful, consider supporting its development:

<a href="https://keychainpgp.org/#donate"><img src="https://img.shields.io/badge/Donate-KeychainPGP-orange?style=for-the-badge&logo=heart&logoColor=white" alt="Donate" height="35"></a>

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

## Acknowledgments

- [Sequoia-PGP](https://sequoia-pgp.org/) &mdash; the modern OpenPGP implementation in Rust
- [Tauri](https://tauri.app/) &mdash; lightweight cross-platform app framework
- [OpenKeychain](https://www.openkeychain.org/) &mdash; the inspiration for this project
- [Svelte](https://svelte.dev/) &mdash; the reactive UI framework

---

<p align="center">
  <sub>Made with care for journalists, activists, and anyone who needs simple encryption.</sub>
</p>