# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Full HKP/VKS keyserver support with machine-readable index parsing
- Multi-server parallel search support (comma-separated URLs)

### Fixed
- Broken keyserver search functionality where WKD lookup failures (e.g., for Gmail) would block all results from keyservers
- App hang/database conflict when importing duplicate keys (added manual existence check)

### Added (Original)
- Initial project structure with Rust workspace
- `keychainpgp-core`: OpenPGP crypto engine with Sequoia-PGP backend
- `keychainpgp-keys`: Keyring management with SQLite and OS credential storage
- `keychainpgp-clipboard`: Clipboard monitoring, PGP detection, and auto-clear
- `keychainpgp-ui`: Tauri desktop application with Svelte frontend
- `keychainpgp-cli`: Command-line interface
