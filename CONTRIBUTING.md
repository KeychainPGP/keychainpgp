# Contributing to KeychainPGP

Thank you for your interest in contributing to KeychainPGP!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/keychainpgp.git`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests: `cargo test --workspace`
6. Run lints: `cargo clippy --workspace -- -D warnings`
7. Commit with [Conventional Commits](https://www.conventionalcommits.org/): `git commit -m "feat: add feature"`
8. Push and open a pull request

## Development Setup

### Prerequisites

- Rust 1.85+ (via [rustup](https://rustup.rs/))
- Node.js 20+ and npm
- Platform-specific Tauri dependencies

### Build & Test

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Pull Request Requirements

- All CI checks must pass (build, test, clippy, format)
- At least one maintainer review
- Sign-off on the Developer Certificate of Origin (DCO)
- Commit messages follow Conventional Commits format

## Adding Dependencies

New crate dependencies require justification in the PR description:
- Why is this dependency needed?
- What alternatives were considered?
- Is the crate well-maintained and audited?
- Does its license comply with MIT/Apache-2.0?

## Internationalization (i18n)

The frontend supports 20 locales via [Paraglide](https://inlang.com/m/gerre34r/library-inlang-paraglideJs). When adding new UI strings:

- **Only add your keys to `messages/en.json`** — you don't need to touch the other 19 locale files.
- A pre-build script (`scripts/sync-translations.js`) automatically copies missing keys from `en.json` into all other locales as fallback.
- Proper translations are handled separately by maintainers after merge.
- Use the `m.your_key()` pattern in Svelte components (see existing code for examples).

## Security

- No `unsafe` blocks without a `// SAFETY:` comment and second reviewer approval
- All sensitive data must use `zeroize`/`secrecy` types
- No logging of private keys, passphrases, or plaintext at any log level
- Run `cargo audit` before submitting PRs that change dependencies

## Code of Conduct

This project follows the [Contributor Covenant v2.1](CODE_OF_CONDUCT.md).
