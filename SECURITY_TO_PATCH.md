# Security Audit — Vulnerabilities to Patch

**Date:** 2026-02-24
**Scope:** Full codebase — core, keys, clipboard, UI, CLI, WASM, web, CI/CD
**Total:** 4 High, 14 Medium, 15 Low, 3 Info
**Status:** 4/4 High FIXED, 12/14 Medium FIXED (M3 fixed with H1, M4 deferred, M5 fixed with H2), 13/15 Low FIXED (L3 deferred, L9 documented), I3 FIXED

---

## HIGH

### H1 — Secret keys stored as plaintext base64 on disk ✅ FIXED

**File:** `crates/keychainpgp-keys/src/credential.rs:120-127`

The file-based fallback writes secret key material as base64 (not encryption) with default file permissions. On Linux, files are often created `0644` (world-readable). Any process or user with read access to `{data_dir}/secrets/*.key` can extract all private keys.

**Fix:**
1. Set file permissions to `0o600` on Unix immediately after creation.
2. Only write the file fallback when the OS credential store fails (currently it always writes both).
3. Long-term: encrypt the file with a key derived from the OS credential store.

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true).create(true).truncate(true)
        .mode(0o600)
        .open(&path)?
        .write_all(encoded.as_bytes())?;
}
```

---

### H2 — OPSEC secret keys stored in plain `Vec<u8>` without guaranteed zeroization ✅ FIXED

**File:** `crates/keychainpgp-ui/src/state.rs:42`

`opsec_secret_keys: Mutex<HashMap<String, Vec<u8>>>` — plain `Vec<u8>` does not zeroize on drop. If the mutex is poisoned, `if let Ok(mut keys)` silently skips cleanup. If the process is killed (SIGKILL), exit handlers never run. HashMap reallocation can leave copies in freed memory.

**Fix:** Use `Zeroizing<Vec<u8>>` and force-unlock poisoned mutexes:

```rust
pub opsec_secret_keys: Mutex<HashMap<String, Zeroizing<Vec<u8>>>>,
```

In cleanup code, replace `if let Ok(mut keys)` with:
```rust
let mut keys = state.opsec_secret_keys.lock().unwrap_or_else(|e| e.into_inner());
```

---

### H3 — SVG injection via `{@html}` in QR code rendering ✅ FIXED

**Files:**
- `crates/keychainpgp-ui/frontend/src/components/modals/QrExportModal.svelte:27`
- `crates/keychainpgp-ui/frontend/src/components/modals/KeySyncExportModal.svelte:136`
- `crates/keychainpgp-ui/frontend/src/components/modals/DonateModal.svelte:78`

`{@html svgData}` bypasses Svelte's HTML escaping. While the `qrcode` crate generates safe SVG, this is an implicit trust dependency. If the SVG generation ever changes, this becomes XSS.

**Fix:** Render QR as a data URI image to prevent script execution:

```typescript
const safeDataUri = `data:image/svg+xml;base64,${btoa(svgData)}`;
// <img src={safeDataUri} alt="QR Code" />
```

---

### H4 — CLI passphrase exposed in process arguments ✅ FIXED

**File:** `crates/keychainpgp-cli/src/main.rs:37-38, 51-52, 63-64`

The `--passphrase` argument is visible via `ps aux`, `/proc/<pid>/cmdline`, and shell history on multi-user systems.

**Fix:** Add interactive TTY prompt via `rpassword::prompt_password()` as default. Add `--passphrase-fd <N>` for scripted use. Add a warning in `--help`.

---

## MEDIUM

### M1 — Decryption silently ignores signature verification ✅ FIXED

**File:** `crates/keychainpgp-core/src/sequoia_engine.rs:853-864`

`DecryptHelper` implements `VerificationHelper::check()` returning `Ok(())` unconditionally, and `get_certs()` returns an empty vec. Signed-and-encrypted messages are decrypted without any signature check.

**Fix:** Return signer information so the UI can display whether the message was authenticated. At minimum, log a warning.

---

### M2 — Revocation certificate discarded at key generation ✅ FIXED

**File:** `crates/keychainpgp-core/src/sequoia_engine.rs:408`

`let (cert, _revocation) = builder.generate()` — the revocation cert is silently dropped. If the user loses their passphrase or their key is compromised, they cannot revoke it.

**Fix:** Store the revocation certificate alongside the key, or let the user export it at generation time.

---

### M3 — Path traversal via unsanitized fingerprint in file paths ✅ FIXED (with H1)

**File:** `crates/keychainpgp-keys/src/credential.rs:116-118`

`self.secrets_dir.join(format!("{fingerprint}.key"))` — a fingerprint containing `../../` could read/write outside the secrets directory.

**Fix:**
```rust
fn secret_key_path(&self, fingerprint: &str) -> PathBuf {
    assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()), "invalid fingerprint");
    self.secrets_dir.join(format!("{fingerprint}.key"))
}
```

---

### M4 — OPSEC mode still writes key metadata to SQLite on disk

**File:** `crates/keychainpgp-ui/src/commands/keys.rs:102-114`

Even in OPSEC mode, public key and metadata (name, email, fingerprint, algorithm, `is_own_key`) are persisted to the SQLite database. This defeats the "no disk traces" promise.

**Fix:** In OPSEC mode, use an in-memory SQLite database, or store public key records in the RAM-only HashMap alongside secret keys.

---

### M5 — Poisoned mutex prevents OPSEC secret key cleanup ✅ FIXED (with H2)

**File:** `crates/keychainpgp-ui/src/commands/opsec.rs:47-52, 71-76`

`if let Ok(mut keys) = state.opsec_secret_keys.lock()` — if a prior panic poisoned the mutex, cleanup silently fails and secret keys remain in memory.

**Fix:** `lock().unwrap_or_else(|e| e.into_inner())` to force access even when poisoned.

---

### M6 — Tor DNS leak via `socks5://` instead of `socks5h://` ✅ FIXED

**File:** `crates/keychainpgp-keys/src/network/keyserver.rs:15-27`

Using `socks5://` performs DNS resolution locally, leaking the keyserver domain to the local resolver. For Tor users, this reveals they're contacting a keyserver.

**Fix:** Default to `socks5h://127.0.0.1:9050` (the `h` suffix routes DNS through the proxy). Also applies to WKD lookups in `wkd.rs`.

---

### M7 — Expired passphrases persist in memory indefinitely ✅ FIXED

**File:** `crates/keychainpgp-ui/src/passphrase_cache.rs:47-55`

`get()` returns `None` for expired entries but does not remove them. Expired passphrases stay in RAM until `clear_all()` or process exit.

**Fix:** Change `get` to `&mut self`, remove expired entries when encountered:
```rust
pub fn get(&mut self, fingerprint: &str) -> Option<&[u8]> {
    let expired = self.entries.get(fingerprint)
        .is_some_and(|e| e.stored_at.elapsed() >= self.ttl);
    if expired {
        self.entries.remove(fingerprint);
        return None;
    }
    self.entries.get(fingerprint).map(|e| e.passphrase.as_slice())
}
```

---

### M8 — Passphrase cache TTL setting has no effect ✅ FIXED

**File:** `crates/keychainpgp-ui/src/state.rs:62, 86`

The cache is initialized with `DEFAULT_CACHE_TTL` (600s). The user-facing `passphrase_cache_secs` setting is never applied to the `PassphraseCache` instance.

**Fix:** Add `set_ttl()` to `PassphraseCache` and call it from `update_settings`.

---

### M9 — Multiple secret key buffers not zeroized on drop ✅ FIXED

**Files:**
- `crates/keychainpgp-core/src/sequoia_engine.rs:443-461` — `secret_key_bytes` Vec on error path
- `crates/keychainpgp-keys/src/sync.rs:17-28` — `KeyBundleEntry.secret_key: Option<Vec<u8>>`
- `crates/keychainpgp-ui/src/commands/keys.rs:526-620` — decrypted backup bytes

Multiple code paths handle secret key material in plain `Vec<u8>` that is not zeroized when dropped.

**Fix:** Use `Zeroizing<Vec<u8>>` for all buffers containing secret key material.

---

### M10 — SSRF via user-controlled keyserver and proxy URLs ✅ FIXED

**File:** `crates/keychainpgp-ui/src/commands/keys.rs:431-432, 471-473, 495-496`

`keyserver_search`, `keyserver_upload`, and `test_proxy_connection` accept arbitrary URLs from the frontend with no validation. A compromised webview could target internal network addresses or cloud metadata endpoints.

**Fix:** Require `https://` for keyserver URLs. Reject private/reserved IP ranges. For proxy URLs, only allow `socks5://` and `http://` targeting localhost.

---

### M11 — Web app missing CSP headers ✅ FIXED

**File:** `web/index.html`

The standalone web app has no Content-Security-Policy. It handles PGP key material and WASM crypto with no protection against script injection.

**Fix:** Add to `web/index.html`:
```html
<meta http-equiv="Content-Security-Policy"
      content="default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; object-src 'none'; base-uri 'self'">
```

---

### M12 — Web app wrapping key extractable from sessionStorage ✅ FIXED

**File:** `web/src/lib/keystore.ts:41-62`

The AES-256-GCM wrapping key is generated with `extractable: true` and stored as base64 in `sessionStorage`. Any XSS can extract it and decrypt all secret keys from IndexedDB.

**Fix:** Set `extractable: false`. Keep the `CryptoKey` object in a module-level variable instead of serializing to sessionStorage.

---

### M13 — `wasm-pack` installed via curl pipe to shell in CI ✅ FIXED

**File:** `.github/workflows/deploy-web.yml:41`

`curl ... | sh` is vulnerable to supply chain attacks with no integrity check.

**Fix:** `cargo install wasm-pack --locked --version 0.13.1`

---

### M14 — No hardening flags for macOS builds ✅ FIXED

**File:** `.cargo/config.toml`

Linux and Windows have hardening flags, but macOS has none.

**Fix:** Add `[target.x86_64-apple-darwin]` and `[target.aarch64-apple-darwin]` sections with PIE flags.

---

## LOW

### L1 — `allow-variable-time-crypto` enabled on Sequoia ✅ DOCUMENTED

**File:** `Cargo.toml:28`

May allow non-constant-time crypto operations, opening timing side-channels.

**Fix:** Evaluate if this flag is actually required for `crypto-rust` backend. Document accepted risk if needed.

---

### L2 — `subtle` crate declared but never used ✅ FIXED

**File:** `crates/keychainpgp-core/Cargo.toml:18`

Constant-time comparison crate listed but never imported. Suggests intended but unimplemented security measure.

**Fix:** Use `subtle::ConstantTimeEq` for fingerprint comparisons, or remove the dependency.

---

### L3 — SQLite database not encrypted at rest

**File:** `crates/keychainpgp-keys/src/storage.rs:39`

Public keys and metadata (names, emails, fingerprints, `is_own_key`) are stored unencrypted. Especially concerning in OPSEC mode.

**Fix:** Consider SQLCipher for at-rest encryption, especially in OPSEC mode.

---

### L4 — Secret key files not securely deleted ✅ FIXED

**File:** `crates/keychainpgp-keys/src/credential.rs:93-96`

`std::fs::remove_file()` unlinks the file but data remains on disk until overwritten.

**Fix:** Overwrite with zeros before unlinking. Document SSD wear-leveling limitation.

---

### L5 — OPSEC mode flag uses `Ordering::Relaxed` ✅ FIXED (with H2)

**Files:** `crates/keychainpgp-ui/src/commands/opsec.rs:19, 44`

No memory ordering guarantee. A thread could read `false` and use the non-OPSEC code path after another thread set it to `true`.

**Fix:** Use `Ordering::SeqCst` or `Acquire`/`Release`.

---

### L6 — Settings logged with full Debug representation ✅ FIXED

**File:** `crates/keychainpgp-ui/src/commands/settings.rs:165, 177`

`tracing::info!("settings updated: {settings:?}")` may log proxy URL credentials and OPSEC configuration.

**Fix:** Log only `"settings updated"` or implement a custom `Debug` that redacts sensitive fields.

---

### L7 — Sync passphrase modulo bias ✅ FIXED

**File:** `crates/keychainpgp-keys/src/sync.rs:110-126`

`u16 % 10000` introduces modulo bias (~0.07 bits per group).

**Fix:** Use rejection sampling: discard values >= 60000.

---

### L8 — `withGlobalTauri: true` exposes IPC globally ✅ FIXED

**File:** `crates/keychainpgp-ui/tauri.conf.json:14`

`window.__TAURI__` is accessible from dev console or any injected script.

**Fix:** Set to `false` and use ES module imports exclusively (already used throughout `tauri.ts`).

---

### L9 — `unsafe-inline` in CSP style-src ⚠️ DOCUMENTED

**File:** `crates/keychainpgp-ui/tauri.conf.json:27`

Allows CSS injection, which can be used for CSS-based data exfiltration.

**Fix:** Use nonce-based CSP for inline styles if framework allows.

---

### L10 — No input validation on key generation parameters ✅ FIXED

**File:** `crates/keychainpgp-ui/src/commands/keys.rs:62-67`

`name` and `email` accept arbitrary strings with no length limits. Megabyte-length inputs could cause DoS.

**Fix:** Validate length < 256 chars and non-empty name.

---

### L11 — No input size limit on CLI stdin ✅ FIXED

**File:** `crates/keychainpgp-cli/src/commands/encrypt.rs:42-45` (and decrypt, sign, verify, inspect)

`read_to_end()` with no size limit allows memory exhaustion.

**Fix:** Use `Read::take(MAX_INPUT_SIZE)` before `read_to_end()`.

---

### L12 — Non-atomic secret key file writes ✅ FIXED

**File:** `crates/keychainpgp-keys/src/credential.rs:120-127`

Process interruption during `std::fs::write()` can corrupt the file, losing the secret key.

**Fix:** Write to a temp file, then atomically rename.

---

### L13 — Yanked dependencies only warned, not denied ✅ FIXED

**File:** `deny.toml:4`

`yanked = "warn"` allows building with yanked (potentially vulnerable) crates.

**Fix:** Change to `yanked = "deny"`.

---

### L14 — CI keystore and key.properties not cleaned up ✅ FIXED

**File:** `.github/workflows/release.yml:133-143`

Android signing secrets written to disk during build are never removed.

**Fix:** Add cleanup step with `if: always()`.

---

### L15 — Clipboard clear may not defeat all history managers ✅ FIXED

**File:** `crates/keychainpgp-clipboard/src/clear.rs:48-62`

Writing empty string may not clear clipboard history on Windows (built-in history captures on write) or third-party tools.

**Fix:** Use platform-specific APIs (Windows `ClearClipboardHistory`). Document limitation.

---

## INFO

### I1 — SQL injection not possible (positive)

All SQLite queries use `params![]`. No SQL injection found.

### I2 — `unsafe` code globally denied (positive)

`[workspace.lints.rust] unsafe_code = "deny"` — no `unsafe` blocks found anywhere.

### I3 — WASM exposes secret key as immutable JS string ✅ FIXED

**File:** `crates/keychainpgp-wasm/src/lib.rs:85-91`

Inherent WASM/JS limitation. JS strings cannot be zeroed. Document for web users. Consider `Uint8Array` (manually zeroable) instead.

**Fix applied:** Secret key now returned as `Uint8Array` (via `serde_bytes`). Callers zeroize with `.fill(0)` after use. Keystore encrypts/decrypts raw bytes directly.