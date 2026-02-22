# KeychainPGP -- Product Requirements Document

**Version:** 1.0.0-draft
**Date:** 2026-02-20
**Authors:** Product & Engineering Team
**Status:** Draft -- Pending Review

---

## Table of Contents

1. [Product Vision](#1-product-vision)
2. [Product Scope & MVP Definition](#2-product-scope--mvp-definition)
3. [Advanced Features (Phase 2+)](#3-advanced-features-phase-2)
4. [Security & Threat Model](#4-security--threat-model)
5. [Technical Architecture](#5-technical-architecture)
6. [UX & Design Principles](#6-ux--design-principles)
7. [Non-Goals](#7-non-goals)
8. [Privacy & Compliance](#8-privacy--compliance)
9. [Performance Requirements](#9-performance-requirements)
10. [Roadmap](#10-roadmap)
11. [Success Metrics](#11-success-metrics)
12. [Open Source Strategy](#12-open-source-strategy)
13. [Deliverables](#13-deliverables)
14. [Appendix A -- ASCII Wireframes](#appendix-a--ascii-wireframes)
15. [Appendix B -- Developer Task Breakdown](#appendix-b--developer-task-breakdown)
16. [Appendix C -- Suggested Crates & Modules](#appendix-c--suggested-crates--modules)
17. [Appendix D -- Repository Structure](#appendix-d--repository-structure)
18. [Appendix E -- Architecture Diagrams](#appendix-e--architecture-diagrams)
19. [Appendix F -- Coding Guidelines](#appendix-f--coding-guidelines)

---

## 1. Product Vision

### 1.1 Problem Statement

PGP encryption has existed since 1991, yet in 2026 it remains inaccessible to the vast majority of people who need it. The current landscape is defined by friction:

**GnuPG / GPG CLI** -- The de facto standard is a command-line tool with over 300 flags, cryptic error messages, and a mental model that assumes deep familiarity with public-key cryptography. Generating a key pair, understanding trust models, and performing basic encrypt/decrypt operations require reading multiple man pages. Error output like `gpg: public key decryption failed: No secret key` conveys nothing actionable to a non-expert.

**Kleopatra (Windows/Linux)** -- The most common desktop GUI wraps GnuPG in a certificate-manager paradigm borrowed from X.509/PKI. Users are confronted with tabs labeled "Certificates," "Notepad," and "Other Certificates" before they can encrypt a single message. The workflow requires 6+ clicks and understanding the difference between "OpenPGP" and "S/MIME" modes.

**macOS GPG Suite** -- A paid product ($23.90 USD) that bundles GPGMail, GPG Keychain, and GPGServices. It is tightly coupled to Apple Mail and has historically broken across macOS major releases, leaving users unable to decrypt their own messages after an OS update.

**Cognitive load** -- All existing tools expose the full complexity of the OpenPGP specification: subkeys, key flags, trust levels (marginal, full, ultimate), key signing policies, revocation certificates, keyserver URLs, and cipher preferences. This complexity is necessary for the protocol but catastrophic for usability. Studies show that even technically skilled users make critical errors with PGP tools (Whitten & Tygar, "Why Johnny Can't Encrypt," 1999; Ruoti et al., follow-up studies through 2016).

The result: people who genuinely need encrypted communication -- journalists, whistleblowers, activists, security researchers -- either avoid PGP entirely, use it incorrectly (encrypting to the wrong key, forgetting to sign, leaking metadata), or rely on centralized alternatives (Signal, ProtonMail) that cannot serve every use case.

### 1.2 Why OpenKeychain Succeeded

OpenKeychain for Android demonstrated that PGP can be made usable:

- **Intent-based API**: Any Android app could call OpenKeychain to encrypt/decrypt, removing the need for a dedicated email client.
- **Contact integration**: Keys were associated with address book entries, not 40-character fingerprints.
- **One-screen encrypt**: Select a contact, paste or type the message, press "Encrypt." Three steps.
- **Visual security indicators**: Green lock icons for verified keys, yellow for unverified, red for problems.
- **No configuration required**: Modern defaults (RSA-4096 at the time) were applied silently.

OpenKeychain proved that the problem is not PGP itself -- it is the tooling.

### 1.3 The Desktop Gap

There is no desktop equivalent of OpenKeychain. The project was Android-only, and its maintainers have moved it to maintenance mode. Desktop users are left with:

| Platform | Best Available Tool | UX Quality |
|----------|-------------------|------------|
| Windows  | Kleopatra (Gpg4win) | Poor |
| macOS    | GPG Suite | Moderate (paid, fragile) |
| Linux    | Seahorse / KGPG | Poor to moderate |
| Cross-platform | GnuPG CLI | Expert-only |

KeychainPGP fills this gap: a single cross-platform desktop application that brings the OpenKeychain philosophy to Windows, macOS, and Linux.

### 1.4 Target Users

**Primary: Journalists and Whistleblowers**
- Need to receive encrypted tips and communicate with sources.
- Cannot afford mistakes -- a mis-encrypted message can endanger lives.
- Often non-technical; trained in short workshops, rarely retain CLI knowledge.
- Workflow: receive an encrypted block via email or messaging, decrypt and read, reply encrypted.

**Primary: Developers and Cybersecurity Students**
- Learning PGP as part of coursework, CTF challenges, or professional development.
- Comfortable with technology but frustrated by GnuPG's complexity as a learning barrier.
- Need a tool that teaches correct mental models through its interface, not documentation.
- Workflow: generate keys, exchange with peers, encrypt/decrypt messages, sign commits.

**Secondary: Privacy-Conscious Individuals**
- Motivated by principle rather than professional necessity.
- Use PGP intermittently to encrypt sensitive files or communications.
- Low tolerance for complexity; will abandon the tool if onboarding takes more than 5 minutes.
- Workflow: encrypt a note before storing it, share a public key, decrypt a received message.

**Secondary: Security Researchers and Pentesters**
- Use PGP for responsible disclosure, encrypted bug reports, and team communication.
- Need fast clipboard operations during engagements where switching contexts is costly.
- May need advanced features (subkeys, multiple identities) but benefit from sane defaults.
- Workflow: rapid encrypt/decrypt cycles during engagements, key management across identities.

### 1.5 Core Value Proposition

**Zero-configuration encryption.** Install KeychainPGP, and you can encrypt your first message within 2 minutes. No terminal commands, no keyserver configuration, no cipher selection. The application generates a modern key pair on first launch and stores it securely.

**Clipboard-first workflow.** The fundamental interaction pattern is:

```
Encrypt:  Copy plaintext  ->  Press hotkey or click "Encrypt"  ->  Paste ciphertext
Decrypt:  Copy ciphertext ->  Press hotkey or click "Decrypt"  ->  Read plaintext
```

This model works with any application -- email clients, messaging apps, web forms, text editors, note-taking tools -- without requiring integration plugins or APIs.

**Minimal cryptography knowledge required.** Users do not need to know what Ed25519, X25519, AEAD, or ASCII armor mean. The interface presents encryption as a lock-and-key metaphor: you have keys, your contacts have keys, and messages are locked to specific people. All cryptographic decisions (algorithm selection, key derivation, padding) happen silently using audited modern defaults.

---

## 2. Product Scope & MVP Definition

KeychainPGP ships with a Tauri-based graphical interface from the first public release. There is no CLI-only phase visible to end users; the CLI exists as an internal development tool and for headless/server use cases.

### 2.1 Key Management

#### 2.1.1 Key Generation

| Requirement | Detail |
|-------------|--------|
| Signing key algorithm | Ed25519 (EdDSA over Curve25519) |
| Encryption subkey algorithm | X25519 (ECDH over Curve25519) |
| Key format | OpenPGP v4 or v6 (per RFC 9580) |
| Default expiration | 2 years from generation, renewable |
| User ID format | `Display Name <email@example.com>` |
| Passphrase | Optional at generation; strongly recommended via UI prompt |
| Generation time target | < 1 second on modern hardware |

On first launch, the application presents a single onboarding screen:

```
Welcome to KeychainPGP

To get started, we'll create your encryption identity.

  Name:   [________________________]
  Email:  [________________________]

  [ ] Protect with a passphrase (recommended)

              [ Create My Keys ]
```

No algorithm selection, no key size dropdown, no advanced options. The "Create My Keys" button generates an Ed25519 signing key and an X25519 encryption subkey, binds them to the provided User ID, and stores them in the local keyring.

Advanced users can access key generation options through Settings > Advanced > Key Generation, where they may select RSA (3072/4096), specify custom expiration, or add multiple User IDs. This panel is deliberately hidden from the default flow.

#### 2.1.2 Key Import / Export

- **Import formats:** ASCII-armored public keys (`.asc`), binary OpenPGP public keys (`.pgp`, `.gpg`), keyring files.
- **Import methods:**
  - File picker dialog.
  - Drag-and-drop onto the application window.
  - Paste ASCII-armored key block directly into the Keys panel.
  - Clipboard detection (if clipboard contains `-----BEGIN PGP PUBLIC KEY BLOCK-----`, offer to import).
- **Export formats:** ASCII-armored (`.asc`), with a "Copy to Clipboard" button.
- **Export scope:** Public key only by default. Private key export requires explicit confirmation dialog with warning text.
- **Batch import:** Support importing a file containing multiple concatenated public keys.

#### 2.1.3 Key Display

Each key in the keyring displays:

| Field | Display Format | Example |
|-------|---------------|---------|
| Name | Full display name | Alice Johnson |
| Email | Email address | alice@example.com |
| Fingerprint | Grouped hex, last 16 chars prominent | `...7A3F 9B2C 4D1E 8F05` |
| Algorithm | Human-readable | "Modern (Ed25519)" or "Classic (RSA-4096)" |
| Created | Relative + absolute | "2 years ago (2024-02-20)" |
| Expires | Relative + absolute, color-coded | "in 6 months (2026-08-20)" / "EXPIRED" in red |
| Trust | Visual indicator | Green shield / Yellow shield / Gray shield |
| Type | Badge | "Your Key" / "Contact" |

#### 2.1.4 Local Keyring Storage

- Private keys are encrypted at rest using the OS credential storage:
  - **Windows:** DPAPI (Data Protection API) via the `windows-credentials` or equivalent crate.
  - **macOS:** Keychain Services via the `security-framework` crate.
  - **Linux:** Secret Service API (GNOME Keyring / KDE Wallet) via the `keyring` crate, with fallback to a passphrase-encrypted file if no service is available.
- Public keys are stored in a local SQLite database (`keyring.db`) within the application data directory.
- The keyring database is located at:
  - Windows: `%APPDATA%\KeychainPGP\keyring.db`
  - macOS: `~/Library/Application Support/com.keychainpgp.app/keyring.db`
  - Linux: `~/.local/share/keychainpgp/keyring.db`
- The keyring is never transmitted over the network unless the user explicitly initiates a keyserver upload.

### 2.2 Clipboard Encryption & Decryption

#### 2.2.1 Encryption Flow

```
User Action                         System Behavior
-----------                         ---------------
1. Copy plaintext to clipboard      (Standard OS copy)
2. Press Ctrl+Shift+E               Application activates
   OR click "Encrypt" button
3. Recipient selection dialog       Show list of known public keys
   appears                          with search/filter
4. Select one or more recipients    Keys are highlighted
5. Press "Encrypt"                  - Read clipboard text
                                    - Encrypt to selected recipients
                                    - Optionally sign with user's key
                                    - Replace clipboard with ASCII-armored
                                      PGP message
6. Paste into target application    (Standard OS paste)
```

**Recipient selection** supports:
- Search by name, email, or fingerprint fragment.
- Multi-select for encrypting to multiple recipients.
- "Always encrypt to self" option (enabled by default) so the sender can decrypt their own messages.
- Most-recently-used recipients appear at the top.

**Output format:** ASCII-armored PGP message:
```
-----BEGIN PGP MESSAGE-----

hQIMAxyz...
...base64 encoded data...
=ABCD
-----END PGP MESSAGE-----
```

#### 2.2.2 Decryption Flow

```
User Action                         System Behavior
-----------                         ---------------
1. Copy PGP message to clipboard    (Standard OS copy)
2. Press Ctrl+Shift+D               Application activates
   OR click "Decrypt" button
3. (If passphrase-protected)        Passphrase dialog appears
   Enter passphrase                 with optional "remember for session"
4. Decrypted text appears in a      - Read clipboard content
   secure viewer window             - Detect PGP message block
                                    - Decrypt with matching private key
                                    - Display plaintext in viewer
                                    - "Copy to Clipboard" button available
5. (Optional) Click "Copy"          Plaintext replaces clipboard content
6. (Optional) Auto-clear triggers   Clipboard cleared after configured delay
```

**Automatic detection:** When the application is running (foreground or tray), it monitors the clipboard for content matching the pattern `-----BEGIN PGP MESSAGE-----`. When detected, a non-intrusive notification appears:

```
+------------------------------------------+
|  Encrypted message detected              |
|  [Decrypt Now]          [Dismiss]        |
+------------------------------------------+
```

This notification is a system toast / notification, not a modal dialog. It does not steal focus.

#### 2.2.3 Auto-Clear Clipboard

| Setting | Default | Options |
|---------|---------|---------|
| Auto-clear after decrypt | Enabled | On / Off |
| Clear delay | 30 seconds | 10s / 30s / 60s / 120s / Never |
| Clear after encrypt | Disabled | On / Off |
| Notification before clear | Enabled | On / Off |

When auto-clear is active, a subtle countdown indicator appears in the system tray icon or application status bar. The clipboard is overwritten with empty content (not just cleared, to defeat clipboard history tools).

### 2.3 GUI Requirements (Tauri)

#### 2.3.1 Application Layout

The main window consists of three views, accessible via a sidebar or tab bar:

**Encrypt/Decrypt View** (default, home screen):
- Central text area for plaintext / ciphertext preview.
- "Encrypt" and "Decrypt" action buttons, prominently placed.
- Drag-and-drop zone for files (future: file encryption).
- Status bar showing clipboard state and auto-clear countdown.

**Keys View:**
- List of all keys in the keyring (own keys at the top, contacts below).
- Search bar with real-time filtering.
- Per-key actions: Export, Delete, View Details.
- "Import Key" button (file picker + paste area).
- "Generate New Key" button.

**Settings View:**
- General: Language, theme (light/dark/system), startup behavior.
- Security: Auto-clear settings, passphrase cache duration, clipboard monitoring on/off.
- Advanced: Key generation defaults, keyserver URLs, export options.
- About: Version, license, links to source code and documentation.

#### 2.3.2 System Tray Mode

- The application minimizes to the system tray (notification area) when the window is closed.
- Tray icon context menu:
  - "Encrypt Clipboard" -- triggers encryption flow.
  - "Decrypt Clipboard" -- triggers decryption flow.
  - "Open KeychainPGP" -- restores the main window.
  - "Quit" -- exits the application entirely.
- Tray icon changes appearance to indicate state:
  - Default: neutral icon (lock).
  - Clipboard contains PGP message: icon changes (lock + arrow, or color shift).
  - Auto-clear countdown active: subtle animation or badge.

#### 2.3.3 Global Hotkeys

| Hotkey | Action | Customizable |
|--------|--------|-------------|
| `Ctrl+Shift+E` | Encrypt clipboard content | Yes |
| `Ctrl+Shift+D` | Decrypt clipboard content | Yes |

- On macOS, `Ctrl` is replaced with `Cmd` by default.
- Hotkeys work when the application is in the system tray (background).
- Hotkey conflicts are detected at startup; if a conflict exists, the user is notified and can rebind.

#### 2.3.4 Error Messages

All error messages are written in plain language with actionable guidance. No raw error codes, no cryptographic jargon.

| Internal Error | User-Facing Message |
|---------------|---------------------|
| `No secret key found for recipient KeyID` | "You don't have the private key needed to decrypt this message. It may have been encrypted for a different key." |
| `Invalid ASCII armor` | "The clipboard doesn't contain a valid encrypted message. Make sure you copied the entire message, including the BEGIN and END lines." |
| `Passphrase incorrect` | "The passphrase you entered is incorrect. Please try again." |
| `Key expired` | "The key for [Name] expired on [Date]. You can still use it to decrypt old messages, but you cannot encrypt new ones to this key. Ask [Name] to send you an updated key." |
| `No recipients selected` | "Select at least one recipient to encrypt this message to." |
| `Clipboard empty` | "Your clipboard is empty. Copy some text first, then try again." |
| `Signature verification failed` | "This message claims to be from [Name], but the signature could not be verified. The message may have been tampered with." |

---

## 3. Advanced Features (Phase 2+)

These features are explicitly out of scope for the MVP but are planned and inform architectural decisions.

### 3.1 Message Signing & Verification

- Sign clipboard text with the user's private key.
- Verify inline signatures and cleartext-signed messages.
- Visual indicator: "Signed by [Name] -- Verified" (green) or "Signature could not be verified" (red).
- Hotkey: `Ctrl+Shift+S` to sign clipboard content.

### 3.2 Passphrase-Protected Private Keys

- Private keys can be additionally protected with a user-chosen passphrase (beyond OS credential storage).
- Passphrase is requested on first use per session; cached in memory for a configurable duration (default: 10 minutes).
- Memory holding the passphrase is allocated using locked/pinned pages (mlock) and zeroized immediately after use.
- Integration with OS biometric authentication where available (Windows Hello, macOS Touch ID) as a passphrase substitute.

### 3.3 QR Code Public Key Export

- Generate a QR code containing the user's ASCII-armored public key.
- Display on screen for scanning by a phone (e.g., importing into OpenKeychain or a mobile KeychainPGP companion).
- For keys too large for a single QR code, use animated QR code sequences or a compact fingerprint-only QR that links to a key retrieval mechanism.

### 3.4 Key Discovery

**Web Key Directory (WKD):**
- Given an email address, automatically look up the corresponding public key via the domain's `/.well-known/openpgpkey/` endpoint.
- Follows the WKD protocol (draft-koch-openpgp-webkey-service).
- Results are presented as "Key found for alice@example.com -- Import?" with fingerprint verification.

**Keyserver Integration:**
- Search public keyservers (keys.openpgp.org by default, configurable).
- Upload own public key to keyservers.
- Configurable keyserver URLs; support for HKP and HKPS protocols.

**File Import:**
- Import keys from `.asc`, `.pgp`, `.gpg` files via file picker or drag-and-drop (already in MVP).

### 3.5 OPSEC Mode

A hardened operating mode for high-risk users, activated via Settings or a toggle in the main UI.

| Feature | Normal Mode | OPSEC Mode |
|---------|-------------|------------|
| Private key storage | OS credential store, on disk | RAM only; keys must be re-imported each session |
| Clipboard after decrypt | Auto-clear in 30s | Auto-clear in 10s |
| Decrypted text display | Scrollable text viewer | Non-selectable text, no copy button, view timeout |
| Session data on quit | Cached passphrases cleared | All session data zeroized; temporary files overwritten |
| Panic hotkey | Not available | `Ctrl+Shift+P` -- immediately wipes all in-memory keys, clears clipboard, closes application |
| Window title | "KeychainPGP" | Generic title (e.g., "Notes") or configurable |
| Taskbar/Dock presence | Normal application icon | Hidden or generic icon |

### 3.6 WASM Browser Version

- A WebAssembly build of the core crypto module, usable in a browser without trusting a server.
- The browser version handles key generation, encryption, and decryption entirely client-side.
- No private keys ever leave the browser tab; they are stored in the Web Crypto API or IndexedDB with encryption.
- Use case: encrypted message composition on a shared or untrusted computer where installing software is not possible.

### 3.7 Anonymous Key Exchange

- Optional integration with Tor (SOCKS5 proxy) for keyserver lookups and key uploads, preventing metadata leakage.
- Optional Lokinet support as an alternative anonymity layer.
- These features are opt-in, never default, to avoid fingerprinting users who enable them.

---

## 4. Security & Threat Model

### 4.1 Threat Actors and Attack Surfaces

#### Threat 1: Clipboard Malware

**Description:** Malicious software running on the same machine intercepts clipboard content to steal plaintext or ciphertext.

**Mitigations:**
- Auto-clear clipboard after a configurable delay.
- In OPSEC mode, decrypted text is displayed in a non-copyable viewer window and never placed on the clipboard automatically.
- Warn users on first launch that clipboard security depends on the host OS being uncompromised.
- Future: Investigate OS-specific secure clipboard mechanisms (if any become available).

**Residual risk:** If the operating system itself is compromised (rootkit, kernel-level malware), no application-level mitigation is sufficient. KeychainPGP documents this limitation clearly in its security model.

#### Threat 2: Key Exfiltration

**Description:** An attacker gains access to the private key material, either by stealing the keyring file from disk or by exploiting a vulnerability in the application.

**Mitigations:**
- Private keys are encrypted at rest using OS-provided credential storage (DPAPI, Keychain, Secret Service).
- Private key material in memory is zeroized immediately after use via the `zeroize` crate.
- The application does not write private keys to temporary files, logs, or crash dumps.
- Address space layout randomization (ASLR) and stack canaries are enabled in release builds.
- OPSEC mode keeps keys in RAM only and zeroizes on exit.

#### Threat 3: Shoulder Surfing

**Description:** An adversary physically observes the user's screen to read decrypted messages or passphrases.

**Mitigations:**
- Passphrase input fields mask characters by default (standard password field behavior).
- Decrypted messages can be set to auto-hide after a configurable timeout (OPSEC mode).
- The application does not display decrypted content in system notifications or window previews (taskbar thumbnails).
- Optional screen-capture protection: request the OS not to include the KeychainPGP window in screenshots (where supported).

#### Threat 4: Disk Forensic Analysis

**Description:** An adversary with physical access to the device extracts data from disk, including deleted files, swap space, or hibernation images.

**Mitigations:**
- Private keys encrypted at rest (see Threat 2).
- Sensitive buffers are zeroized in memory, reducing (but not eliminating) the window for swap-to-disk leakage.
- Memory locking (`mlock`/`VirtualLock`) is used for buffers containing plaintext and passphrases to prevent paging to swap.
- The application does not create temporary files containing plaintext or decrypted content.
- OPSEC mode: all session data is stored in locked, zeroizable memory only.

**Residual risk:** Swap and hibernation files are controlled by the OS. Users requiring protection against disk forensics should enable full-disk encryption (BitLocker, FileVault, LUKS) independently. KeychainPGP recommends this in its documentation but cannot enforce it.

#### Threat 5: Supply Chain Attacks

**Description:** An attacker compromises the build pipeline, dependency chain, or distribution channel to ship a backdoored version of KeychainPGP.

**Mitigations:**
- Reproducible builds: any developer can reproduce the exact binary from source.
- All release binaries are code-signed (Windows Authenticode, macOS notarization, Linux GPG-signed packages).
- Dependency auditing: `cargo-audit` and `cargo-vet` are integrated into CI.
- Minimal dependency tree: avoid unnecessary crates; prefer well-audited, widely-used dependencies.
- SBOM (Software Bill of Materials) published with each release.
- Signed Git tags for all releases.

### 4.2 Security Requirements

| Requirement | Implementation |
|-------------|---------------|
| Memory safety | Rust language guarantees; no `unsafe` blocks except where audited and justified (e.g., OS API FFI) |
| Zeroization | All buffers containing private keys, passphrases, or plaintext use `zeroize::Zeroize` and are dropped via `zeroize::ZeroizeOnDrop` |
| Constant-time operations | All cryptographic comparisons (MACs, fingerprints) use constant-time equality checks (`subtle` crate) |
| OS credential storage | Private keys stored via DPAPI (Windows), Keychain (macOS), Secret Service (Linux) |
| Memory locking | Sensitive buffers use `mlock` / `VirtualLock` to prevent paging to swap |
| No logging of secrets | Log framework explicitly excludes fields tagged as sensitive; log level is `info` by default with no plaintext or key material at any level |
| Reproducible builds | Deterministic Rust compilation with pinned dependencies (`Cargo.lock` committed) |
| Code signing | Release binaries signed; CI pipeline produces attestations |
| Security audit | Mandatory third-party audit before v1.0 stable release |
| Dependency vetting | `cargo-vet` and `cargo-audit` run on every CI build; new dependencies require justification |

### 4.3 Cryptographic Standards

| Operation | Algorithm | Standard |
|-----------|-----------|----------|
| Signing | Ed25519 (EdDSA) | RFC 8032, bound to OpenPGP via RFC 9580 |
| Key agreement | X25519 (ECDH) | RFC 7748, bound to OpenPGP via RFC 9580 |
| Symmetric encryption | AES-256 with OCB (AEAD) | RFC 9580 AEAD mechanism |
| Hash | SHA-256 | FIPS 180-4 |
| Key derivation (passphrase) | Argon2id | RFC 9106, via OpenPGP S2K |
| Compression | None by default | Disabled to prevent oracle attacks; optional zlib for compatibility |

---

## 5. Technical Architecture

### 5.1 Core Cryptography Module

The cryptographic engine is built on **Sequoia-PGP** (`sequoia-openpgp` crate), the most mature and actively maintained Rust-native OpenPGP implementation.

**Why Sequoia-PGP:**
- Pure Rust (no C dependencies for core crypto, uses Rust crypto backends).
- Implements RFC 9580 (the current OpenPGP standard).
- Supports modern algorithms (Ed25519, X25519, AEAD).
- Well-documented API with strong type safety.
- Actively maintained with security-focused development practices.
- Supports pluggable crypto backends (Rust Crypto, Nettle, OpenSSL) -- we default to the pure-Rust backend for reproducibility and portability.

**Abstraction layer:** All cryptographic operations are accessed through a `CryptoEngine` trait, allowing:
- Unit testing with mock implementations.
- Future swapping of the crypto backend without affecting the rest of the application.
- Clear separation between "what" (encrypt this message to these recipients) and "how" (Sequoia's internal implementation).

```
keychainpgp-core/
  src/
    lib.rs              -- Public API
    engine.rs           -- CryptoEngine trait definition
    sequoia_engine.rs   -- Sequoia-PGP implementation
    types.rs            -- Domain types (Fingerprint, KeyID, UserID, etc.)
    error.rs            -- Typed error hierarchy
```

### 5.2 Application Architecture

KeychainPGP is structured as a Rust workspace with five crates:

```
keychainpgp/
  Cargo.toml            -- Workspace root
  crates/
    keychainpgp-core/   -- Crypto operations (encrypt, decrypt, sign, verify)
    keychainpgp-keys/   -- Keyring management (storage, import, export, search)
    keychainpgp-clipboard/ -- Clipboard monitoring, read/write, auto-clear
    keychainpgp-ui/     -- Tauri application (backend commands + frontend)
    keychainpgp-cli/    -- Headless CLI for scripting and server use
```

**Dependency graph (arrows indicate "depends on"):**

```
keychainpgp-ui ──────┐
                     ├──> keychainpgp-core
keychainpgp-cli ─────┤
                     ├──> keychainpgp-keys ──> keychainpgp-core
                     │
                     └──> keychainpgp-clipboard
```

#### keychainpgp-core

The foundation crate. Contains:
- `CryptoEngine` trait and Sequoia implementation.
- Key generation functions.
- Encrypt/decrypt functions accepting byte slices and returning byte slices.
- Sign/verify functions.
- ASCII armor serialization/deserialization.
- Error types for all cryptographic failures.

No I/O, no filesystem access, no clipboard interaction. Pure functions operating on in-memory data.

#### keychainpgp-keys

Keyring management:
- `Keyring` struct encapsulating key storage.
- SQLite-backed persistent storage for public keys and metadata.
- OS credential storage integration for private keys.
- Key search (by email, name, fingerprint, KeyID).
- Import/export in multiple formats.
- Trust level tracking.
- Key expiration monitoring.

#### keychainpgp-clipboard

Cross-platform clipboard operations:
- Read current clipboard content.
- Write content to clipboard.
- Detect PGP message blocks in clipboard content (regex: `-----BEGIN PGP (MESSAGE|PUBLIC KEY BLOCK|SIGNATURE)-----`).
- Auto-clear mechanism: schedule clipboard wipe after a delay.
- Clipboard change notifications (platform-specific).
- Secure clipboard write (overwrite with zeros before clearing, where platform allows).

Uses `arboard` crate for cross-platform clipboard access.

#### keychainpgp-ui

Tauri application:
- Rust backend implementing Tauri commands (`#[tauri::command]`).
- Frontend in **Svelte** (SvelteKit) -- chosen for its small bundle size, performance, and developer experience.
- System tray integration via Tauri's tray API.
- Global hotkey registration via Tauri's global shortcut API.
- Window management (main window, passphrase dialog, decrypted text viewer).
- IPC between frontend and Rust backend via Tauri's invoke system.

#### keychainpgp-cli

Headless command-line interface:
- `keychainpgp encrypt --recipient <email> < plaintext.txt > encrypted.asc`
- `keychainpgp decrypt < encrypted.asc > plaintext.txt`
- `keychainpgp keys list`
- `keychainpgp keys import <file>`
- `keychainpgp keys export <fingerprint>`
- `keychainpgp keys generate --name "Name" --email "email"`
- Shares the same `keychainpgp-core` and `keychainpgp-keys` crates as the GUI.
- Useful for automation, CI pipelines, and server environments.

### 5.3 GUI Technology Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Backend | Rust + Tauri v2 | Native performance, small binary, system API access |
| Frontend framework | Svelte 5 | Minimal bundle size (~5KB runtime), reactive, fast rendering |
| Styling | Tailwind CSS | Utility-first, consistent design, small CSS output with purging |
| Component library | Custom + Headless UI patterns | Full control over accessibility and design |
| Build tool | Vite | Fast HMR during development, optimized production builds |
| State management | Svelte stores + Tauri event system | Reactive state synchronized with Rust backend |
| Icons | Lucide Icons (tree-shaken) | Consistent, MIT-licensed icon set |

### 5.4 Data Flow Diagrams

#### Encryption Flow

```
  ┌──────────┐      ┌──────────┐      ┌──────────────┐      ┌──────────┐
  │ Clipboard │ ──>  │  UI      │ ──>  │ core:encrypt │ ──>  │ Clipboard│
  │ (read)   │      │ (invoke) │      │ (Sequoia)    │      │ (write)  │
  └──────────┘      └──────────┘      └──────────────┘      └──────────┘
                          │                   ▲
                          │                   │
                          ▼                   │
                    ┌──────────┐        ┌──────────┐
                    │ Recipient│        │ keys:    │
                    │ Selection│ ────>  │ lookup   │
                    └──────────┘        └──────────┘
```

#### Decryption Flow

```
  ┌──────────┐      ┌──────────┐      ┌──────────────┐      ┌──────────┐
  │ Clipboard │ ──>  │  UI      │ ──>  │ core:decrypt │ ──>  │ Secure   │
  │ (read)   │      │ (detect) │      │ (Sequoia)    │      │ Viewer   │
  └──────────┘      └──────────┘      └──────────────┘      └──────────┘
                                             │                     │
                                             ▼                     ▼
                                       ┌──────────┐         ┌──────────┐
                                       │ keys:    │         │ Clipboard│
                                       │ private  │         │ (write + │
                                       │ key fetch│         │  auto-   │
                                       └──────────┘         │  clear)  │
                                                            └──────────┘
```

---

## 6. UX & Design Principles

### 6.1 Principle: Hide Cryptographic Terminology

Users interact with "keys," "locks," and "contacts" -- not "certificates," "subkeys," "key flags," or "cipher suites."

| Cryptographic Term | User-Facing Language |
|-------------------|---------------------|
| Public key | "Your sharing key" or "Contact's key" |
| Private key | "Your private key" (kept, but explained as "the key only you have") |
| Fingerprint | "Key ID" (with a tooltip: "A unique identifier for this key") |
| Key pair | "Your encryption identity" |
| ASCII armor | "Encrypted text block" |
| Subkey | (hidden entirely; managed automatically) |
| Trust model | "How much you trust this contact's key" |
| Revocation | "Cancel this key" |
| Keyserver | "Key directory" |
| Web of Trust | (not exposed in the UI) |

### 6.2 Principle: Secure Defaults Without Configuration

| Decision | Default | Why |
|----------|---------|-----|
| Algorithm | Ed25519 + X25519 | Modern, fast, small keys, widely supported |
| AEAD | AES-256-OCB | Authenticated encryption; prevents tampering |
| Compress | Off | Prevents compression oracle attacks |
| Encrypt to self | On | Users can always decrypt their own messages |
| Key expiration | 2 years | Limits damage from compromised keys |
| Clipboard auto-clear | On (30s) | Reduces clipboard theft window |

Users never see these choices unless they navigate to Advanced settings.

### 6.3 Principle: One-Click Onboarding

The first-launch experience is:

1. Application opens to onboarding screen (name + email fields).
2. User clicks "Create My Keys."
3. Keys are generated (<1s).
4. Main screen appears with "Your keys are ready. Copy your sharing key to give it to others."
5. Total time: under 60 seconds.

No configuration wizards, no algorithm selection, no keyserver registration.

### 6.4 Principle: Visual Trust Indicators

Trust levels are communicated through color and iconography, not text labels:

```
  Green shield  ✓   You have personally verified this key
                    (imported via secure channel, verified fingerprint)

  Yellow shield ~   This key was imported but not verified
                    (fetched from keyserver, received via email)

  Gray shield   ?   Unknown trust level

  Red shield    ✕   Key has a problem (expired, revoked, invalid signature)
```

These indicators appear next to every key in the recipient selection dialog and the keyring view.

### 6.5 Principle: Explain via Tooltips and Microcopy

Every non-obvious UI element has an `(i)` icon or hover tooltip:

- Fingerprint display: "This is a unique identifier for the key. Compare it with the owner in person or over a trusted channel to verify the key is authentic."
- Encrypt to self: "When enabled, you'll be able to decrypt messages you send. If disabled, only the recipient can read the message."
- Auto-clear: "Your clipboard will be automatically cleared after the timer expires to prevent other applications from accessing decrypted content."

### 6.6 Principle: Maximum Two Clicks to Encrypt

The critical path for encryption (assuming keys are already set up):

1. **Click 1:** Press "Encrypt" button (or hotkey -- zero clicks).
2. **Click 2:** Select recipient and confirm.

Clipboard reading and writing are automatic. No copy-paste from/to text fields within the application.

---

## 7. Non-Goals

The following are explicitly out of scope for KeychainPGP at any version:

| Non-Goal | Rationale |
|----------|-----------|
| **Email client integration** | KeychainPGP is clipboard-based. It works *with* any email client but does not embed into one. Building email integration (MIME parsing, PGP/MIME, S/MIME) would massively increase scope and maintenance burden. |
| **Full GnuPG replacement** | GnuPG supports features (smartcards, GPGME API, gpg-agent protocol, SSH agent) that serve a different audience. KeychainPGP does not aim to replicate the GnuPG ecosystem. |
| **Blockchain / cryptocurrency wallets** | Ed25519 keys are used in some blockchain systems, but KeychainPGP is a communication encryption tool, not a financial application. No cryptocurrency functionality will be added. |
| **Cloud key storage** | Private keys are never stored on remote servers by default. Users who want cloud backup can export their keys and manage backup independently. A future plugin system might support this, but it will never be the default. |
| **File encryption (MVP)** | The MVP focuses on text/clipboard encryption. File encryption (encrypting arbitrary files to recipients) is a Phase 2+ feature. |
| **Key signing parties / Web of Trust** | The classic PGP Web of Trust model has seen limited adoption. KeychainPGP uses a simplified trust model (verified / unverified) rather than implementing the full WoT. |
| **S/MIME support** | S/MIME is a different standard with different key formats and trust models (X.509 certificates, CAs). Supporting both would dilute the product's focus. |

---

## 8. Privacy & Compliance

### 8.1 No Telemetry by Default

KeychainPGP collects zero data about its users by default:

- No usage analytics.
- No crash reporting.
- No phone-home checks (update checks are opt-in).
- No unique identifiers or installation tracking.

### 8.2 Optional Crash Reports

Users can opt in to anonymous crash reports in Settings. If enabled:

- Crash reports contain: stack trace, OS version, application version, and error message.
- Crash reports do NOT contain: key material, clipboard content, file paths, user names, email addresses, or any PGP-related data.
- Reports are sent to a self-hosted Sentry instance (not a third-party SaaS) or a simple HTTPS endpoint under the project's control.
- The crash report consent dialog clearly states what is collected and where it is sent.

### 8.3 GDPR Compliance

| GDPR Requirement | Implementation |
|-----------------|---------------|
| Data minimization | No personal data collected by the application itself |
| Right to erasure | Users can delete their keys and all application data via Settings > "Delete All Data" |
| Data portability | Keys can be exported in standard OpenPGP format |
| Consent | Crash reporting is opt-in with clear disclosure |
| Privacy policy | Published on the project website, linked from Settings > About |

### 8.4 Offline-First Architecture

KeychainPGP functions fully without an internet connection. Network access is only used for:

- Keyserver lookups (opt-in, user-initiated).
- WKD key discovery (opt-in, user-initiated).
- Update checks (opt-in).
- Crash reports (opt-in).

No feature is degraded or unavailable when offline.

---

## 9. Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Clipboard encrypt (1KB text) | < 50ms | Time from hotkey press to clipboard write |
| Clipboard encrypt (100KB text) | < 100ms | Time from hotkey press to clipboard write |
| Clipboard decrypt (1KB message) | < 50ms | Time from hotkey press to plaintext display (excluding passphrase entry) |
| Clipboard decrypt (100KB message) | < 100ms | Time from hotkey press to plaintext display |
| Key generation (Ed25519 + X25519) | < 500ms | Time from button press to key stored |
| Application startup (cold) | < 500ms | Time from process launch to window visible |
| Application startup (tray, warm) | < 200ms | Time from tray icon click to window visible |
| Installed binary size | < 25MB | Total size of the application bundle (all platforms) |
| Runtime memory (idle) | < 50MB | RSS memory with application in tray, no operations active |
| Runtime memory (active) | < 100MB | RSS memory during encrypt/decrypt operations |

### 9.1 Optimization Strategies

- **Binary size:** Use Tauri v2's optimized bundling; strip debug symbols in release builds; enable LTO (Link-Time Optimization); use `opt-level = "s"` for size-optimized crypto operations where performance is still acceptable.
- **Startup time:** Lazy-load the keyring database; defer tray icon rendering to after window display; precompile Svelte components at build time.
- **Memory:** Avoid loading entire keyring into memory; use streaming decryption for large messages; free crypto context immediately after operation.
- **Frontend:** Svelte's compiled output is significantly smaller than React/Vue equivalents. Tree-shake all dependencies. Target < 100KB total frontend bundle (gzipped).

---

## 10. Roadmap

### Phase 0: Foundation (Weeks 1-4)

**Goal:** Working Rust library that can encrypt and decrypt OpenPGP messages.

- [ ] Set up Rust workspace with crate structure.
- [ ] Implement `keychainpgp-core` with Sequoia-PGP:
  - Key generation (Ed25519 + X25519).
  - Encrypt to one or more recipients.
  - Decrypt with private key.
  - ASCII armor serialization.
- [ ] Implement `keychainpgp-keys`:
  - SQLite-backed keyring.
  - OS credential storage integration (Windows DPAPI, macOS Keychain, Linux Secret Service).
  - Import/export of ASCII-armored keys.
- [ ] Implement `keychainpgp-cli` with basic commands.
- [ ] Unit and integration tests for all crypto operations.
- [ ] CI pipeline (GitHub Actions): build, test, lint, audit.

**Exit criteria:** CLI can generate keys, encrypt a message to a recipient, and decrypt it.

### Phase 1: Desktop MVP (Weeks 5-12)

**Goal:** Shippable Tauri desktop application with clipboard encryption.

- [ ] Implement `keychainpgp-clipboard`:
  - Cross-platform clipboard read/write.
  - PGP block detection.
  - Auto-clear with countdown.
- [ ] Implement `keychainpgp-ui` (Tauri + Svelte):
  - Onboarding / key generation screen.
  - Encrypt/decrypt home view.
  - Keys manager panel.
  - Recipient selection dialog.
  - Settings panel.
  - System tray integration.
  - Global hotkeys.
  - Error dialogs with human-readable messages.
- [ ] Platform-specific packaging:
  - Windows: MSI/NSIS installer.
  - macOS: DMG with notarization.
  - Linux: AppImage, `.deb`, `.rpm`.
- [ ] End-to-end testing on all three platforms.
- [ ] User testing with 5-10 participants from target user groups.

**Exit criteria:** Non-technical user can install, generate keys, import a contact's key, and exchange encrypted messages via clipboard.

### Phase 2: Trust & Signing (Weeks 13-20)

- [ ] Message signing and cleartext signature verification.
- [ ] Key trust management (verified / unverified with visual indicators).
- [ ] WKD key discovery.
- [ ] Keyserver search and upload (keys.openpgp.org).
- [ ] QR code public key export.
- [ ] Passphrase caching with configurable timeout.
- [ ] Improved key details view (subkeys, capabilities, usage history).

### Phase 3: Internationalization (Weeks 21-26)

**Goal:** Full multilingual support with right-to-left (RTL) layout and CJK typographic handling, enabling worldwide adoption.

- [ ] i18n framework integration (`svelte-i18n` or `paraglide-js` for frontend, `rust-i18n` or compile-time approach for Rust error messages).
- [ ] Extract all user-facing strings from Svelte components into locale files (JSON/YAML).
- [ ] Extract all Rust-side user-facing strings (error messages, tray menu labels, notifications) into translatable catalogs.
- [ ] Implement locale detection (OS locale → user preference → fallback to English).
- [ ] Language selector in Settings with instant preview (no restart required).
- [ ] **Latin/Western European languages:**
  - [ ] English (en) — base locale, already complete.
  - [ ] French (fr).
  - [ ] German (de).
  - [ ] Spanish (es).
  - [ ] Portuguese (pt-BR, pt-PT).
  - [ ] Italian (it).
  - [ ] Dutch (nl).
- [ ] **Cyrillic languages:**
  - [ ] Russian (ru).
  - [ ] Ukrainian (uk).
- [ ] **CJK languages (Chinese, Japanese, Korean):**
  - [ ] Simplified Chinese (zh-CN).
  - [ ] Traditional Chinese (zh-TW).
  - [ ] Japanese (ja).
  - [ ] Korean (ko).
  - [ ] CJK font stack configuration (Noto Sans CJK or system fonts).
  - [ ] Handle CJK text wrapping and line-break rules (CSS `word-break: keep-all` for Korean, `line-break: strict` for Japanese).
- [ ] **Right-to-left (RTL) languages:**
  - [ ] Arabic (ar).
  - [ ] Hebrew (he).
  - [ ] RTL layout mirroring (CSS `dir="rtl"`, logical properties `margin-inline-start` instead of `margin-left`).
  - [ ] Bidirectional text handling in clipboard preview and decrypted message viewer.
- [ ] **Additional languages:**
  - [ ] Turkish (tr).
  - [ ] Polish (pl).
  - [ ] Hindi (hi).
  - [ ] Thai (th).
- [ ] Pluralization rules per locale (ICU MessageFormat or equivalent).
- [ ] Date/time formatting per locale (relative dates like "2 years ago" adapt to language).
- [ ] Number formatting per locale (fingerprint grouping remains hex-universal).
- [ ] Keyboard shortcut labels adapt to locale (e.g., `Ctrl` → `Strg` on German layouts).
- [ ] Accessibility: screen reader announcements respect the active locale (`lang` attribute on root element).
- [ ] Translation workflow: document contributor translation process, provide template files, CI validation that all keys exist in all locales.
- [ ] CLI internationalization: `keychainpgp-cli` output messages respect `LANG`/`LC_ALL` environment variables.

**Exit criteria:** A user in any supported locale can install KeychainPGP and use all features entirely in their language, with correct text direction, date formatting, and pluralization.

### Phase 4: Mobile Companion (Weeks 27-38)

- [ ] Android companion app (Kotlin/Rust FFI or Tauri Mobile).
- [ ] iOS companion app.
- [ ] Key sync between desktop and mobile (manual export/import or encrypted sync).
- [ ] QR code key exchange between desktop and mobile.

### Phase 5: Browser & OPSEC (Weeks 39-50)

- [ ] WASM build of `keychainpgp-core` for browser-based encryption.
- [ ] Browser extension or standalone web app.
- [ ] OPSEC mode implementation (RAM-only keys, panic hotkey, stealth UI).
- [ ] Tor/Lokinet integration for anonymous keyserver access.
- [ ] Third-party security audit.
- [ ] v1.0 stable release.

### Phase 6: Support & Donations (Weeks 51+)

- [ ] In-app "Buy Me a Coffee" page accessible from Settings > About.
- [ ] Display donation wallet addresses with QR codes:
  - [ ] Bitcoin (BTC) — on-chain + Lightning Network for instant low-fee payments.
  - [ ] Ethereum (ETH).
  - [ ] Monero (XMR) — privacy-preserving option for anonymous donations.
- [ ] No third-party payment processor — direct wallet-to-wallet transfers only.
- [ ] Donation addresses hardcoded or loaded from a signed configuration file (prevents address-swap attacks).
- [ ] Static donation page on the project website with the same addresses and QR codes.
- [ ] Optional "Supporters" section in About panel (opt-in, pseudonymous).

### Phase 7: Website & Web App (Weeks 53+)

**Goal:** Two separate web presences — a marketing website on `keychainpgp.org` that presents the project and drives adoption, and the existing WASM web app on `keychainpgp.github.io` polished to production quality.

#### 7a. Marketing website (`keychainpgp.org`)

A lightweight static site (Astro, Hugo, or plain HTML) in a separate repo, deployed to GitHub Pages with a custom domain.

- [ ] Hero section: tagline, one-line value proposition, animated product screenshot/mockup, and prominent download buttons (Windows / macOS / Linux / Android).
- [ ] Features section: 3-4 key selling points with icons (clipboard-first, zero-config, cross-platform, open source).
- [ ] "How it works" section: 3-step visual walkthrough (Copy → Encrypt → Paste).
- [ ] "Try in browser" CTA linking to the web app at `keychainpgp.github.io`.
- [ ] Download section: platform-detected auto-suggestion ("It looks like you're on Windows — [Download for Windows]") with manual override for all platforms.
- [ ] Verification section: GPG signatures and SHA-256 checksums for every release binary.
- [ ] Documentation hub: links to user guide, FAQ, security model, and contribution guide.
- [ ] Donation section: BTC / ETH / XMR addresses with QR codes (mirrors in-app Phase 6).
- [ ] Dark/light theme toggle, responsive design (mobile-friendly).
- [ ] SEO: Open Graph tags, structured data (JSON-LD SoftwareApplication), sitemap, meta descriptions.
- [ ] i18n: site available in all Phase 3 languages (at minimum EN, FR, DE, ES, RU, ZH-CN, JA, AR).
- [ ] Privacy-respecting analytics (Plausible, Umami, or self-hosted — no Google Analytics).
- [ ] Deployment: separate repo with CI/CD, custom domain `keychainpgp.org` via GitHub Pages CNAME.

#### 7b. Web app polish (`keychainpgp.github.io`)

The WASM-based web app is already deployed from `web/` via `deploy-web.yml`. This sub-phase brings it to production quality.

- [ ] Dark/light theme toggle (currently dark-only).
- [ ] Responsive design audit (mobile-friendly layout).
- [ ] i18n: web app available in Phase 3 languages.
- [ ] Improved onboarding: first-visit guidance or help tooltips for new users.
- [ ] Open Graph tags and favicon set for shared links.

**Exit criteria:** Visiting `keychainpgp.org` presents a professional, multilingual landing page where any user can understand the product, download the correct installer, verify its authenticity, or try the app in their browser via `keychainpgp.github.io`. The web app supports multiple languages and both light/dark themes.

### Phase 8: iOS App (Weeks 55+)

**Goal:** Native iOS companion app with feature parity to the Android version.

- [ ] Tauri Mobile iOS target configuration and build pipeline.
- [ ] Xcode project setup, provisioning profiles, and code signing.
- [ ] Adapt UI for iOS design conventions (safe areas, haptics, swipe gestures).
- [ ] iOS Keychain Services integration for private key storage.
- [ ] Share Extension: encrypt/decrypt text from any iOS app via the share sheet.
- [ ] Key sync with desktop (same mechanism as Android: manual export/import or encrypted sync).
- [ ] QR code key exchange (camera-based scan + generation).
- [ ] i18n: all Phase 3 locales available on iOS.
- [ ] TestFlight beta distribution for testing.
- [ ] App Store submission (review guidelines compliance, export compliance for encryption — ECCN 5D002).
- [ ] CI/CD: GitHub Actions with macOS runner for iOS builds.

**Exit criteria:** iOS users can install KeychainPGP from TestFlight or the App Store and perform all core operations (generate keys, import/export, encrypt, decrypt) with the same experience as Android.

---

## 11. Success Metrics

### 11.1 Usability Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Time to first encryption | < 2 minutes from install | Usability testing with new users |
| Task success rate (encrypt) | > 95% | Usability testing: "Encrypt this message to Alice" |
| Task success rate (decrypt) | > 95% | Usability testing: "Decrypt this message" |
| Task success rate (key import) | > 90% | Usability testing: "Import this person's key" |
| Error recovery rate | > 80% | Usability testing: user encounters an error and recovers without help |
| System Usability Scale (SUS) | > 80 (Grade A) | Post-task SUS questionnaire |

### 11.2 Adoption Metrics

| Metric | Target (Year 1) | Measurement |
|--------|-----------------|-------------|
| GitHub stars | > 1,000 | GitHub API |
| Downloads | > 10,000 | Release download counts |
| Monthly active users (opt-in telemetry) | > 1,000 | Anonymous usage ping (opt-in only) |
| Security community adoption | Mentioned in 3+ security training courses | Community outreach tracking |
| Bug reports (usability) | < 5/month after stabilization | GitHub Issues |

### 11.3 Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Crash rate | < 0.1% of sessions | Opt-in crash reports |
| User-reported errors per month | < 10 | GitHub Issues |
| Interoperability | Decrypt messages from GnuPG, OpenKeychain, Proton | Automated interoperability test suite |
| CVE count | 0 critical, 0 high (pre-audit) | Security audit report |

### 11.4 Comparative Metrics

| Metric | GnuPG CLI | Kleopatra | KeychainPGP Target |
|--------|-----------|-----------|-------------------|
| Steps to first encryption | 12+ (install, generate, import, encrypt) | 8+ | 3 (install, create keys, encrypt) |
| Clicks to encrypt a message | N/A (CLI) | 6+ | 2 |
| Time to learn (novice) | Hours | 30-60 minutes | < 5 minutes |
| Error message clarity | Cryptic | Moderate | Plain language |

---

## 12. Open Source Strategy

### 12.1 License

**Dual-licensed under MIT and Apache-2.0** (at the user's choice).

Rationale:
- **MIT** is maximally permissive and widely understood; compatible with nearly all open-source and commercial use cases.
- **Apache-2.0** provides explicit patent protection, which is valuable for a cryptography tool.
- Dual licensing is the standard approach in the Rust ecosystem (used by Rust itself, Serde, Tokio, and most major crates).

All dependencies must be compatible with MIT/Apache-2.0 dual licensing. Dependencies under GPL, AGPL, or other copyleft licenses are not permitted.

### 12.2 Repository Structure

- Single monorepo on GitHub: `github.com/keychainpgp/keychainpgp`
- Rust workspace at the repository root.
- Frontend code co-located in `crates/keychainpgp-ui/frontend/`.
- Documentation in `docs/`.
- GitHub Actions for CI/CD.

### 12.3 Contribution Process

- `CONTRIBUTING.md` with clear guidelines.
- Issues labeled: `good-first-issue`, `help-wanted`, `bug`, `enhancement`, `security`.
- Pull requests require:
  - Passing CI (build, test, lint, audit).
  - At least one maintainer review.
  - Sign-off on Developer Certificate of Origin (DCO).
- Code of conduct: Contributor Covenant v2.1.

### 12.4 Reproducible Builds

- `Cargo.lock` committed to the repository.
- CI builds use pinned Rust toolchain versions (via `rust-toolchain.toml`).
- Docker-based build environment for reproducible builds.
- Build hashes published with each release for verification.
- Detailed instructions for reproducing release binaries from source.

### 12.5 Security Process

- `SECURITY.md` with disclosure instructions.
- Dedicated security email: `security@keychainpgp.org` (or equivalent).
- 90-day disclosure policy: reported vulnerabilities are fixed and disclosed within 90 days.
- Security advisories published via GitHub Security Advisories.
- Bug bounty: initially informal (acknowledgment + credit); formal program when funding allows.

---

## 13. Deliverables

This section enumerates every artifact that constitutes a complete project handoff.

### 13.1 Documentation

| Deliverable | Status |
|-------------|--------|
| Product Requirements Document (this document) | Complete |
| Architecture diagrams | See Appendix E |
| ASCII wireframes for UI | See Appendix A |
| Developer task breakdown with milestones | See Appendix B |
| Suggested Rust crates and Tauri modules | See Appendix C |
| Initial repository structure | See Appendix D |
| Coding guidelines | See Appendix F |

### 13.2 Software Artifacts (to be produced during development)

| Artifact | Phase |
|----------|-------|
| `keychainpgp-core` crate | Phase 0 |
| `keychainpgp-keys` crate | Phase 0 |
| `keychainpgp-cli` binary | Phase 0 |
| `keychainpgp-clipboard` crate | Phase 1 |
| `keychainpgp-ui` Tauri application | Phase 1 |
| Windows installer (MSI) | Phase 1 |
| macOS disk image (DMG) | Phase 1 |
| Linux packages (AppImage, deb, rpm) | Phase 1 |
| Automated test suite | Phase 0-1 |
| CI/CD pipeline | Phase 0 |

---

## Appendix A -- ASCII Wireframes

### A.1 Onboarding Screen (First Launch)

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                       🔐 KeychainPGP                        ║
║                                                              ║
║            Welcome! Let's set up your encryption.            ║
║                                                              ║
║    ┌──────────────────────────────────────────────────┐      ║
║    │  Your Name                                       │      ║
║    └──────────────────────────────────────────────────┘      ║
║                                                              ║
║    ┌──────────────────────────────────────────────────┐      ║
║    │  your@email.com                                  │      ║
║    └──────────────────────────────────────────────────┘      ║
║                                                              ║
║    [x] Protect with a passphrase (recommended)               ║
║                                                              ║
║    ┌──────────────────────────────────────────────────┐      ║
║    │  ••••••••••••                                    │      ║
║    └──────────────────────────────────────────────────┘      ║
║                                                              ║
║              ┌────────────────────────┐                      ║
║              │    Create My Keys      │                      ║
║              └────────────────────────┘                      ║
║                                                              ║
║    Already have keys?  [Import from file]                    ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

### A.2 Main Screen (Encrypt/Decrypt View)

```
╔══════════════════════════════════════════════════════════════╗
║  [Encrypt/Decrypt]    [Keys]    [Settings]         [_][□][X]║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║   Clipboard Status:  "Hello, this is a secret..."  (142 ch) ║
║   ─────────────────────────────────────────────────────────  ║
║                                                              ║
║   ┌────────────────────────────────────────────────────────┐ ║
║   │                                                        │ ║
║   │   Your clipboard content will appear here.             │ ║
║   │   Copy some text, then click Encrypt or Decrypt.       │ ║
║   │                                                        │ ║
║   │   Or drag and drop a file here (coming soon).          │ ║
║   │                                                        │ ║
║   └────────────────────────────────────────────────────────┘ ║
║                                                              ║
║   ┌─────────────────────┐   ┌─────────────────────┐         ║
║   │                     │   │                     │         ║
║   │   ENCRYPT           │   │   DECRYPT           │         ║
║   │   Ctrl+Shift+E      │   │   Ctrl+Shift+D      │         ║
║   │                     │   │                     │         ║
║   └─────────────────────┘   └─────────────────────┘         ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  Ready                              Auto-clear: 30s  ● Tray ║
╚══════════════════════════════════════════════════════════════╝
```

### A.3 Recipient Selection Dialog

```
╔══════════════════════════════════════════════════════════════╗
║  Select Recipients                                    [X]   ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║   ┌──────────────────────────────────────────────────┐      ║
║   │  Search by name or email...                       │      ║
║   └──────────────────────────────────────────────────┘      ║
║                                                              ║
║   ┌──────────────────────────────────────────────────────┐  ║
║   │ [x]  Alice Johnson                        ✓ Green   │  ║
║   │      alice@example.com                              │  ║
║   │      Key ID: 7A3F 9B2C                              │  ║
║   ├──────────────────────────────────────────────────────┤  ║
║   │ [ ]  Bob Smith                             ~ Yellow  │  ║
║   │      bob@company.org                                │  ║
║   │      Key ID: 4D1E 8F05                              │  ║
║   ├──────────────────────────────────────────────────────┤  ║
║   │ [ ]  Charlie Dev                           ? Gray    │  ║
║   │      charlie@dev.io                                 │  ║
║   │      Key ID: 2C7B A193                              │  ║
║   └──────────────────────────────────────────────────────┘  ║
║                                                              ║
║   [x] Also encrypt to myself                                ║
║                                                              ║
║              ┌────────────────────────┐                      ║
║              │    Encrypt Message     │                      ║
║              └────────────────────────┘                      ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

### A.4 Keys Manager Panel

```
╔══════════════════════════════════════════════════════════════╗
║  [Encrypt/Decrypt]    [Keys]    [Settings]         [_][□][X]║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  ┌──────────────────────────────────────┐  [+ Generate New] ║
║  │  Search keys...                      │  [  Import Key  ] ║
║  └──────────────────────────────────────┘                    ║
║                                                              ║
║  YOUR KEYS                                                   ║
║  ┌──────────────────────────────────────────────────────┐   ║
║  │  You (your@email.com)                  [Your Key]    │   ║
║  │  Key ID: A1B2 C3D4 E5F6 7890                        │   ║
║  │  Created: 2026-02-20     Expires: 2028-02-20        │   ║
║  │  Modern (Ed25519)                                    │   ║
║  │                    [Export]  [Details]  [Delete]      │   ║
║  └──────────────────────────────────────────────────────┘   ║
║                                                              ║
║  CONTACTS                                                    ║
║  ┌──────────────────────────────────────────────────────┐   ║
║  │ ✓ Alice Johnson (alice@example.com)     [Verified]   │   ║
║  │   Key ID: 7A3F 9B2C 4D1E 8F05                       │   ║
║  │   Expires: 2027-06-15                                │   ║
║  │                    [Export]  [Details]  [Delete]      │   ║
║  ├──────────────────────────────────────────────────────┤   ║
║  │ ~ Bob Smith (bob@company.org)         [Unverified]   │   ║
║  │   Key ID: 4D1E 8F05 2C7B A193                       │   ║
║  │   Expires: 2026-12-01                                │   ║
║  │                    [Export]  [Details]  [Delete]      │   ║
║  └──────────────────────────────────────────────────────┘   ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  3 keys in keyring                                           ║
╚══════════════════════════════════════════════════════════════╝
```

### A.5 Decrypted Message Viewer

```
╔══════════════════════════════════════════════════════════════╗
║  Decrypted Message                                    [X]   ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║   From: Alice Johnson (alice@example.com)     ✓ Verified    ║
║   ─────────────────────────────────────────────────────────  ║
║                                                              ║
║   ┌──────────────────────────────────────────────────────┐  ║
║   │                                                      │  ║
║   │  Hey, here are the documents you requested.          │  ║
║   │  The password for the archive is "correct horse       │  ║
║   │  battery staple."                                    │  ║
║   │                                                      │  ║
║   │  Let me know if you have any issues.                 │  ║
║   │                                                      │  ║
║   │  - Alice                                             │  ║
║   │                                                      │  ║
║   └──────────────────────────────────────────────────────┘  ║
║                                                              ║
║   ┌─────────────────┐                                       ║
║   │ Copy to Clipboard│        Auto-clear in: 28s            ║
║   └─────────────────┘                                       ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

### A.6 System Tray Context Menu

```
     ┌────────────────────────┐
     │ Encrypt Clipboard      │
     │ Decrypt Clipboard      │
     ├────────────────────────┤
     │ Open KeychainPGP       │
     ├────────────────────────┤
     │ Quit                   │
     └────────────────────────┘
            ▲
         [tray icon]
```

---

## Appendix B -- Developer Task Breakdown

### Phase 0: Foundation (Weeks 1-4)

#### Sprint 1 (Weeks 1-2): Project Setup & Core Crypto

| ID | Task | Estimate | Dependencies |
|----|------|----------|-------------|
| P0-01 | Initialize Rust workspace with crate structure | 2h | None |
| P0-02 | Set up CI pipeline (GitHub Actions: build, test, clippy, audit) | 4h | P0-01 |
| P0-03 | Implement `CryptoEngine` trait in `keychainpgp-core` | 4h | P0-01 |
| P0-04 | Implement Sequoia-PGP backend for `CryptoEngine` | 8h | P0-03 |
| P0-05 | Key generation: Ed25519 signing + X25519 encryption subkey | 6h | P0-04 |
| P0-06 | Encrypt function: plaintext + recipient keys -> OpenPGP message | 8h | P0-04 |
| P0-07 | Decrypt function: OpenPGP message + private key -> plaintext | 8h | P0-04 |
| P0-08 | ASCII armor serialization / deserialization | 4h | P0-04 |
| P0-09 | Error type hierarchy for `keychainpgp-core` | 3h | P0-03 |
| P0-10 | Unit tests for all core crypto operations | 8h | P0-05..P0-08 |

#### Sprint 2 (Weeks 3-4): Keyring & CLI

| ID | Task | Estimate | Dependencies |
|----|------|----------|-------------|
| P0-11 | SQLite schema for keyring database | 4h | P0-01 |
| P0-12 | Keyring CRUD operations (add, get, list, delete keys) | 8h | P0-11 |
| P0-13 | Key import (ASCII-armored, binary) | 6h | P0-12, P0-08 |
| P0-14 | Key export (ASCII-armored) | 4h | P0-12, P0-08 |
| P0-15 | OS credential storage integration (DPAPI/Keychain/Secret Service) | 12h | P0-12 |
| P0-16 | Key search (by email, name, fingerprint, KeyID) | 4h | P0-12 |
| P0-17 | CLI: `generate`, `encrypt`, `decrypt`, `keys list/import/export` | 8h | P0-05..P0-08, P0-12..P0-14 |
| P0-18 | Integration tests: generate -> encrypt -> decrypt round-trip | 6h | P0-17 |
| P0-19 | Interoperability tests: decrypt GnuPG-encrypted messages | 6h | P0-07 |
| P0-20 | Documentation: crate-level rustdoc for core and keys | 4h | All above |

### Phase 1: Desktop MVP (Weeks 5-12)

#### Sprint 3 (Weeks 5-6): Clipboard & Tauri Scaffold

| ID | Task | Estimate | Dependencies |
|----|------|----------|-------------|
| P1-01 | Cross-platform clipboard read/write (`arboard` integration) | 6h | P0-01 |
| P1-02 | PGP block detection in clipboard content | 4h | P1-01 |
| P1-03 | Auto-clear mechanism with configurable delay | 6h | P1-01 |
| P1-04 | Initialize Tauri v2 application with Svelte frontend | 6h | P0-01 |
| P1-05 | Tauri command bindings: bridge Rust backend to frontend | 8h | P1-04 |
| P1-06 | Application data directory setup (per-platform paths) | 4h | P1-04 |
| P1-07 | Settings persistence (JSON config file) | 4h | P1-06 |

#### Sprint 4 (Weeks 7-8): Core UI

| ID | Task | Estimate | Dependencies |
|----|------|----------|-------------|
| P1-08 | Onboarding screen (key generation flow) | 8h | P1-05, P0-05 |
| P1-09 | Main encrypt/decrypt view with clipboard preview | 8h | P1-05, P1-01 |
| P1-10 | Recipient selection dialog (search, multi-select) | 10h | P1-05, P0-16 |
| P1-11 | Encryption flow: clipboard -> select recipients -> encrypt -> clipboard | 8h | P1-09, P1-10 |
| P1-12 | Decryption flow: clipboard -> decrypt -> secure viewer | 8h | P1-09 |
| P1-13 | Passphrase entry dialog | 4h | P1-12 |
| P1-14 | Human-readable error dialogs | 4h | P1-05, P0-09 |

#### Sprint 5 (Weeks 9-10): Key Management UI & System Integration

| ID | Task | Estimate | Dependencies |
|----|------|----------|-------------|
| P1-15 | Keys manager panel (list, search, visual indicators) | 10h | P1-05, P0-12 |
| P1-16 | Key import UI (file picker, drag-and-drop, paste) | 6h | P1-15, P0-13 |
| P1-17 | Key export UI (copy to clipboard, save to file) | 4h | P1-15, P0-14 |
| P1-18 | Key details view (fingerprint, expiration, algorithm) | 6h | P1-15 |
| P1-19 | System tray integration (icon, context menu) | 8h | P1-04 |
| P1-20 | Global hotkey registration (Ctrl+Shift+E, Ctrl+Shift+D) | 6h | P1-04, P1-11, P1-12 |
| P1-21 | Clipboard monitoring notification (PGP detected toast) | 6h | P1-02, P1-19 |
| P1-22 | Settings UI panel | 6h | P1-07 |

#### Sprint 6 (Weeks 11-12): Polish, Packaging & Testing

| ID | Task | Estimate | Dependencies |
|----|------|----------|-------------|
| P1-23 | UI polish: consistent styling, responsive layout, dark/light theme | 12h | All UI tasks |
| P1-24 | Accessibility audit (keyboard navigation, screen reader labels, contrast) | 8h | P1-23 |
| P1-25 | Windows packaging (MSI/NSIS installer, code signing) | 8h | All |
| P1-26 | macOS packaging (DMG, notarization) | 8h | All |
| P1-27 | Linux packaging (AppImage, .deb, .rpm) | 8h | All |
| P1-28 | End-to-end tests on all platforms | 12h | P1-25..P1-27 |
| P1-29 | Usability testing with 5-10 target users | 16h | P1-28 |
| P1-30 | Bug fixes from usability testing | 16h | P1-29 |
| P1-31 | README, CONTRIBUTING, SECURITY, LICENSE files | 4h | None |

**Total estimated effort for MVP (Phase 0 + Phase 1): ~360 hours**

---

## Appendix C -- Suggested Crates & Modules

### Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `sequoia-openpgp` | 1.x | OpenPGP implementation (key gen, encrypt, decrypt, sign, verify) |
| `sequoia-cert-store` | 0.x | Certificate (key) storage and management |
| `zeroize` | 1.x | Secure memory zeroing for sensitive data |
| `subtle` | 2.x | Constant-time comparison operations |
| `secrecy` | 0.x | Wrapper types that zeroize on drop |

### Keyring & Storage

| Crate | Version | Purpose |
|-------|---------|---------|
| `rusqlite` | 0.x | SQLite database for keyring storage |
| `keyring` | 3.x | Cross-platform OS credential storage (DPAPI, Keychain, Secret Service) |
| `directories` | 5.x | Platform-specific application data directories |
| `serde` + `serde_json` | 1.x | Settings serialization |

### Clipboard

| Crate | Version | Purpose |
|-------|---------|---------|
| `arboard` | 3.x | Cross-platform clipboard read/write |
| `regex` | 1.x | PGP block detection patterns |

### GUI (Tauri)

| Crate / Package | Purpose |
|-----------------|---------|
| `tauri` (v2) | Application framework, window management, IPC |
| `tauri-plugin-global-shortcut` | Global hotkey registration |
| `tauri-plugin-notification` | System notifications (PGP detected toast) |
| `tauri-plugin-clipboard-manager` | Clipboard operations (alternative to `arboard`) |
| `tauri-plugin-autostart` | Optional launch-on-startup |
| `tauri-plugin-log` | Structured logging |
| `tauri-plugin-store` | Persistent key-value settings storage |

### Frontend (npm)

| Package | Purpose |
|---------|---------|
| `svelte` (5.x) | UI framework |
| `@sveltejs/kit` | Svelte application framework |
| `tailwindcss` (4.x) | Utility-first CSS |
| `lucide-svelte` | Icon library |
| `@tauri-apps/api` (v2) | Tauri JavaScript API bindings |

### CLI

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.x | Command-line argument parsing |
| `indicatif` | 0.x | Progress bars and spinners |
| `console` | 0.x | Terminal colors and styling |

### Development & Testing

| Crate / Tool | Purpose |
|-------------|---------|
| `tokio` | Async runtime (for clipboard monitoring, tray events) |
| `tracing` + `tracing-subscriber` | Structured logging |
| `thiserror` | Derive macro for error types |
| `anyhow` | Error handling in CLI and integration tests |
| `tempfile` | Temporary files for testing |
| `assert_cmd` | CLI integration testing |
| `cargo-audit` | Dependency vulnerability scanning |
| `cargo-vet` | Dependency vetting |
| `cargo-deny` | License and advisory checks |
| `cargo-llvm-cov` | Code coverage |

---

## Appendix D -- Repository Structure

```
keychainpgp/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                  # Build, test, lint, audit on every PR
│   │   ├── release.yml             # Build release binaries + installers
│   │   └── security-audit.yml      # Weekly dependency audit
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.yml
│   │   ├── feature_request.yml
│   │   └── security_vulnerability.yml
│   └── PULL_REQUEST_TEMPLATE.md
│
├── crates/
│   ├── keychainpgp-core/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs              # Public API re-exports
│   │   │   ├── engine.rs           # CryptoEngine trait
│   │   │   ├── sequoia_engine.rs   # Sequoia-PGP implementation
│   │   │   ├── types.rs            # Fingerprint, KeyID, UserID, etc.
│   │   │   ├── armor.rs            # ASCII armor helpers
│   │   │   └── error.rs            # Error types
│   │   └── tests/
│   │       ├── encrypt_decrypt.rs
│   │       ├── key_generation.rs
│   │       └── interop/            # GnuPG / OpenKeychain interop tests
│   │           └── fixtures/       # Test vectors (encrypted messages, keys)
│   │
│   ├── keychainpgp-keys/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── keyring.rs          # Keyring struct and operations
│   │   │   ├── storage.rs          # SQLite backend
│   │   │   ├── credential.rs       # OS credential store abstraction
│   │   │   ├── import.rs           # Key import logic
│   │   │   ├── export.rs           # Key export logic
│   │   │   └── error.rs
│   │   ├── migrations/
│   │   │   └── 001_initial.sql     # SQLite schema
│   │   └── tests/
│   │
│   ├── keychainpgp-clipboard/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── monitor.rs          # Clipboard monitoring daemon
│   │   │   ├── detect.rs           # PGP block detection
│   │   │   ├── clear.rs            # Auto-clear logic
│   │   │   └── error.rs
│   │   └── tests/
│   │
│   ├── keychainpgp-ui/
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── tauri.conf.json         # Tauri configuration
│   │   ├── capabilities/           # Tauri v2 permission capabilities
│   │   ├── icons/                  # Application icons (all sizes)
│   │   ├── src/
│   │   │   ├── main.rs             # Tauri entry point
│   │   │   ├── commands/           # Tauri command handlers
│   │   │   │   ├── mod.rs
│   │   │   │   ├── crypto.rs       # encrypt, decrypt commands
│   │   │   │   ├── keys.rs         # key management commands
│   │   │   │   ├── clipboard.rs    # clipboard commands
│   │   │   │   └── settings.rs     # settings commands
│   │   │   ├── tray.rs             # System tray setup
│   │   │   ├── hotkeys.rs          # Global hotkey registration
│   │   │   └── state.rs            # Application state management
│   │   └── frontend/
│   │       ├── package.json
│   │       ├── svelte.config.js
│   │       ├── vite.config.ts
│   │       ├── tailwind.config.js
│   │       ├── src/
│   │       │   ├── app.html
│   │       │   ├── app.css
│   │       │   ├── lib/
│   │       │   │   ├── components/
│   │       │   │   │   ├── EncryptDecryptView.svelte
│   │       │   │   │   ├── KeysManager.svelte
│   │       │   │   │   ├── RecipientSelector.svelte
│   │       │   │   │   ├── KeyCard.svelte
│   │       │   │   │   ├── PassphraseDialog.svelte
│   │       │   │   │   ├── DecryptedViewer.svelte
│   │       │   │   │   ├── Settings.svelte
│   │       │   │   │   ├── Onboarding.svelte
│   │       │   │   │   ├── ErrorDialog.svelte
│   │       │   │   │   └── TrustBadge.svelte
│   │       │   │   ├── stores/
│   │       │   │   │   ├── keys.ts
│   │       │   │   │   ├── clipboard.ts
│   │       │   │   │   └── settings.ts
│   │       │   │   └── tauri.ts    # Tauri API wrappers
│   │       │   └── routes/
│   │       │       ├── +layout.svelte
│   │       │       └── +page.svelte
│   │       └── static/
│   │           └── fonts/
│   │
│   └── keychainpgp-cli/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/
│       │   │   ├── mod.rs
│       │   │   ├── encrypt.rs
│       │   │   ├── decrypt.rs
│       │   │   ├── generate.rs
│       │   │   └── keys.rs
│       │   └── output.rs           # Formatting helpers
│       └── tests/
│
├── docs/
│   ├── architecture.md
│   ├── security-model.md
│   ├── user-guide.md
│   └── contributing.md
│
├── Cargo.toml                      # Workspace definition
├── Cargo.lock                      # Pinned dependencies
├── rust-toolchain.toml             # Pinned Rust version
├── .cargo/
│   └── config.toml                 # Cargo configuration (e.g., target-specific settings)
├── deny.toml                       # cargo-deny configuration
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── CONTRIBUTING.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
└── CHANGELOG.md
```

---

## Appendix E -- Architecture Diagrams

### E.1 System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        KeychainPGP                              │
│                                                                 │
│  ┌────────────────┐    ┌────────────────┐                       │
│  │  keychainpgp-  │    │  keychainpgp-  │                       │
│  │  cli            │    │  ui (Tauri)    │                       │
│  │                │    │                │                       │
│  │  Terminal I/O  │    │  ┌──────────┐  │                       │
│  │                │    │  │  Svelte  │  │                       │
│  │                │    │  │ Frontend │  │                       │
│  │                │    │  └────┬─────┘  │                       │
│  │                │    │       │ IPC    │                       │
│  │                │    │  ┌────┴─────┐  │                       │
│  │                │    │  │  Tauri   │  │                       │
│  │                │    │  │  Backend │  │                       │
│  └───────┬────────┘    │  └────┬─────┘  │                       │
│          │             └───────┼────────┘                       │
│          │                     │                                │
│          ▼                     ▼                                │
│  ┌─────────────────────────────────────────┐                   │
│  │           keychainpgp-core              │                   │
│  │                                         │                   │
│  │  CryptoEngine (trait)                   │                   │
│  │    ├── generate_keypair()               │                   │
│  │    ├── encrypt(plaintext, recipients)   │                   │
│  │    ├── decrypt(ciphertext, secret_key)  │                   │
│  │    ├── sign(data, secret_key)           │                   │
│  │    └── verify(data, signature, pubkey)  │                   │
│  │                                         │                   │
│  │  SequoiaEngine (impl CryptoEngine)      │                   │
│  │    └── uses: sequoia-openpgp            │                   │
│  └─────────────────────────────────────────┘                   │
│          │                     │                                │
│          ▼                     ▼                                │
│  ┌──────────────────┐  ┌──────────────────┐                    │
│  │ keychainpgp-keys │  │ keychainpgp-     │                    │
│  │                  │  │ clipboard        │                    │
│  │ Keyring          │  │                  │                    │
│  │ ├── SQLite DB    │  │ Monitor          │                    │
│  │ ├── OS Cred Store│  │ ├── Read/Write   │                    │
│  │ ├── Import/Export│  │ ├── PGP Detect   │                    │
│  │ └── Search       │  │ └── Auto-Clear   │                    │
│  └──────────────────┘  └──────────────────┘                    │
│          │                     │                                │
│          ▼                     ▼                                │
│  ┌──────────────────────────────────────────┐                  │
│  │            Operating System              │                  │
│  │                                          │                  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ │                  │
│  │  │ Keychain │ │ Clipboard│ │  System  │ │                  │
│  │  │ /DPAPI/  │ │   API    │ │  Tray    │ │                  │
│  │  │ SecretSvc│ │          │ │          │ │                  │
│  │  └──────────┘ └──────────┘ └──────────┘ │                  │
│  └──────────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

### E.2 Data Model

```
┌─────────────────────────────────────────────┐
│                  Keyring                     │
│                                             │
│  ┌───────────────────────────────────────┐  │
│  │  Public Keys (SQLite)                 │  │
│  │                                       │  │
│  │  ┌─────────────┐  ┌─────────────┐    │  │
│  │  │ KeyRecord   │  │ KeyRecord   │    │  │
│  │  │             │  │             │    │  │
│  │  │ fingerprint │  │ fingerprint │    │  │
│  │  │ user_ids[]  │  │ user_ids[]  │    │  │
│  │  │ algorithm   │  │ algorithm   │    │  │
│  │  │ created_at  │  │ created_at  │    │  │
│  │  │ expires_at  │  │ expires_at  │    │  │
│  │  │ trust_level │  │ trust_level │    │  │
│  │  │ is_own_key  │  │ is_own_key  │    │  │
│  │  │ pgp_data    │  │ pgp_data    │    │  │
│  │  │ (blob)      │  │ (blob)      │    │  │
│  │  └─────────────┘  └─────────────┘    │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  ┌───────────────────────────────────────┐  │
│  │  Private Keys (OS Credential Store)   │  │
│  │                                       │  │
│  │  Key: "keychainpgp:{fingerprint}"     │  │
│  │  Value: encrypted private key blob    │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### E.3 Encryption Sequence

```
User          Frontend (Svelte)     Backend (Tauri/Rust)     Core        Keys        Clipboard
 │                  │                      │                  │            │              │
 │  Ctrl+Shift+E    │                      │                  │            │              │
 │─────────────────>│                      │                  │            │              │
 │                  │  invoke: read_clip   │                  │            │              │
 │                  │─────────────────────>│                  │            │           read()
 │                  │                      │──────────────────────────────────────────>│
 │                  │                      │<─────────────────────────────────────────── plaintext
 │                  │<─────────────────────│                  │            │              │
 │                  │                      │                  │            │              │
 │  show recipient  │                      │                  │            │              │
 │<─────────────────│                      │                  │            │              │
 │                  │                      │                  │            │              │
 │  select & confirm│                      │                  │            │              │
 │─────────────────>│                      │                  │            │              │
 │                  │  invoke: encrypt     │                  │            │              │
 │                  │─────────────────────>│                  │            │              │
 │                  │                      │  get_public_keys │            │              │
 │                  │                      │────────────────────────────>│              │
 │                  │                      │<───────────────────────────── keys          │
 │                  │                      │  encrypt()       │            │              │
 │                  │                      │─────────────────>│            │              │
 │                  │                      │<──────────────── ciphertext   │              │
 │                  │                      │                  │            │           write()
 │                  │                      │──────────────────────────────────────────>│
 │                  │<─────────────────────│                  │            │              │
 │  "Encrypted!"    │                      │                  │            │              │
 │<─────────────────│                      │                  │            │              │
 │                  │                      │                  │            │              │
 │  Ctrl+V (paste)  │                      │                  │            │              │
 │                  │                      │                  │            │              │
```

---

## Appendix F -- Coding Guidelines

### F.1 Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Format all code with `rustfmt` (default configuration).
- Lint with `clippy` at the `pedantic` level; suppress individual lints only with justification comments.
- Target the latest stable Rust toolchain (pinned in `rust-toolchain.toml`).

### F.2 Error Handling

- Use `thiserror` for library crate error types (`keychainpgp-core`, `keychainpgp-keys`, `keychainpgp-clipboard`).
- Use `anyhow` in application crates (`keychainpgp-cli`, `keychainpgp-ui`) for ergonomic error propagation.
- Every error variant must have a human-readable message suitable for display to the user (after mapping through the error message table in Section 2.3.4).
- Never expose internal error details (file paths, SQL errors, crypto library internals) to the frontend; map them to user-facing messages in the Tauri command handlers.

### F.3 Security Practices

- **No `unsafe` without audit:** Every `unsafe` block must have a `// SAFETY:` comment explaining why it is sound. New `unsafe` blocks require review by a second developer.
- **Zeroize everything sensitive:** All types holding private keys, passphrases, or plaintext must implement `ZeroizeOnDrop`. Use `secrecy::SecretString` and `secrecy::SecretVec<u8>` for sensitive data in transit.
- **No logging of secrets:** The `tracing` framework must be configured to redact fields marked `#[sensitive]`. Manual `dbg!()` and `println!()` for sensitive data are forbidden; CI runs a custom lint to catch them.
- **Dependency policy:** New dependencies must be justified in the PR description. Prefer crates that are: widely used (>1M downloads), actively maintained (commit in last 6 months), audited or vetted (listed in `cargo-vet` registry). Avoid crates with `unsafe` code unless they are well-known (e.g., `libc`, `winapi`).

### F.4 Testing Strategy

| Level | Scope | Tool |
|-------|-------|------|
| Unit tests | Individual functions in core, keys, clipboard | `#[cfg(test)]` modules, `cargo test` |
| Integration tests | Cross-crate workflows (generate -> encrypt -> decrypt) | `tests/` directories, `cargo test` |
| Interoperability tests | Decrypt messages from GnuPG, OpenKeychain | Test fixture files in `tests/interop/fixtures/` |
| CLI tests | End-to-end CLI command execution | `assert_cmd` crate |
| UI tests | Tauri application E2E tests | Playwright or WebdriverIO with Tauri driver |
| Performance tests | Benchmark encrypt/decrypt latency | `criterion` crate |

### F.5 Git Workflow

- `main` branch is always deployable.
- Feature branches: `feature/short-description`.
- Bug fix branches: `fix/short-description`.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `ci:`.
- All PRs are squash-merged to `main`.
- Release tags: `v0.1.0`, `v1.0.0`, etc., following SemVer.

### F.6 Frontend Guidelines

- Components use PascalCase file names: `RecipientSelector.svelte`.
- Stores use camelCase: `clipboardStore.ts`.
- All user-facing strings are extracted into a locale file for future i18n support.
- Accessibility: all interactive elements have ARIA labels; the application is navigable by keyboard alone; color is never the sole indicator of state (always paired with icons or text).
- Tailwind classes are ordered: layout, sizing, spacing, typography, colors, effects (enforced by Prettier plugin).

---

*End of Product Requirements Document*

*Document version: 1.0.0-draft*
*Last updated: 2026-02-20*
