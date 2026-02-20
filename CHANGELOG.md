# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with Rust workspace
- `keychainpgp-core`: OpenPGP crypto engine with Sequoia-PGP backend
- `keychainpgp-keys`: Keyring management with SQLite and OS credential storage
- `keychainpgp-clipboard`: Clipboard monitoring, PGP detection, and auto-clear
- `keychainpgp-ui`: Tauri desktop application with Svelte frontend
- `keychainpgp-cli`: Command-line interface
