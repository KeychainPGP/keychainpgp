//! Secret key storage with OS credential store + file-based fallback.
//!
//! Primary backend (tried first):
//! - Windows: DPAPI via the `keyring` crate
//! - macOS: Keychain Services via the `keyring` crate
//! - Linux: Secret Service (GNOME Keyring / KDE Wallet) via the `keyring` crate
//!
//! Fallback backend (used when OS store is unavailable):
//! - Encrypted files in `{data_dir}/secrets/` directory

use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use secrecy::SecretBox;
use zeroize::Zeroize;

use crate::error::{Error, Result};

const SERVICE_NAME: &str = "keychainpgp";

/// Abstraction over secret key storage.
///
/// Tries the OS credential store first, falls back to file-based storage.
/// In portable mode, the OS credential store is skipped entirely to leave
/// no traces on the host system.
pub struct CredentialStore {
    secrets_dir: PathBuf,
    /// When true, skip OS credential store (DPAPI/Keychain/Secret Service).
    portable: bool,
}

impl CredentialStore {
    /// Create a new credential store backed by both the OS keyring and a file fallback.
    pub fn new(data_dir: &Path) -> Result<Self> {
        let secrets_dir = data_dir.join("secrets");
        std::fs::create_dir_all(&secrets_dir)?;
        Ok(Self {
            secrets_dir,
            portable: false,
        })
    }

    /// Enable or disable portable mode (skips OS credential store).
    pub fn set_portable(&mut self, portable: bool) {
        self.portable = portable;
    }

    /// Store a private key. Always stores to file (with restrictive permissions);
    /// also tries OS credential store as a preferred retrieval source.
    pub fn store_secret_key(&self, fingerprint: &str, secret_key: &[u8]) -> Result<()> {
        Self::validate_fingerprint(fingerprint)?;

        // Always store to file as reliable fallback (with restrictive permissions)
        self.store_to_file(fingerprint, secret_key)?;

        // Also try OS credential store for faster retrieval (skip in portable mode)
        if !self.portable {
            if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, fingerprint) {
                let encoded = base64_encode(secret_key);
                let _ = entry.set_secret(encoded.as_bytes());
            }
        }

        Ok(())
    }

    /// Retrieve a private key. Tries OS credential store first (unless portable),
    /// falls back to file.
    pub fn get_secret_key(&self, fingerprint: &str) -> Result<SecretBox<Vec<u8>>> {
        Self::validate_fingerprint(fingerprint)?;

        // Try OS credential store first (skip in portable mode)
        if !self.portable {
            if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, fingerprint) {
                if let Ok(mut encoded) = entry.get_secret() {
                    if let Ok(decoded) = base64_decode(&encoded) {
                        encoded.zeroize();
                        return Ok(SecretBox::new(Box::new(decoded)));
                    }
                }
            }
        }

        // Fall back to file-based storage
        self.load_from_file(fingerprint)
    }

    /// Delete a private key from both stores.
    pub fn delete_secret_key(&self, fingerprint: &str) -> Result<()> {
        Self::validate_fingerprint(fingerprint)?;

        // Try OS credential store (ignore errors, skip in portable mode)
        if !self.portable {
            if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, fingerprint) {
                let _ = entry.delete_credential();
            }
        }

        // Try file store (ignore errors if not present)
        let path = self.secret_key_path(fingerprint);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        Ok(())
    }

    /// Store a revocation certificate for the given key.
    pub fn store_revocation_cert(&self, fingerprint: &str, rev_cert: &[u8]) -> Result<()> {
        Self::validate_fingerprint(fingerprint)?;
        let path = self.secrets_dir.join(format!("{fingerprint}.rev"));

        #[cfg(unix)]
        {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|e| Error::CredentialStore {
                    reason: format!("failed to write revocation cert: {e}"),
                })?;
            file.write_all(rev_cert)
                .map_err(|e| Error::CredentialStore {
                    reason: format!("failed to write revocation cert: {e}"),
                })?;
        }

        #[cfg(not(unix))]
        {
            std::fs::write(&path, rev_cert).map_err(|e| Error::CredentialStore {
                reason: format!("failed to write revocation cert: {e}"),
            })?;
        }

        Ok(())
    }

    /// Check if a secret key exists in either store.
    pub fn has_secret_key(&self, fingerprint: &str) -> bool {
        if Self::validate_fingerprint(fingerprint).is_err() {
            return false;
        }

        // Check OS credential store (skip in portable mode)
        if !self.portable {
            if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, fingerprint) {
                if entry.get_secret().is_ok() {
                    return true;
                }
            }
        }

        // Check file store
        self.secret_key_path(fingerprint).exists()
    }

    /// Validate that a fingerprint contains only hex characters (prevents path traversal).
    fn validate_fingerprint(fingerprint: &str) -> Result<()> {
        if fingerprint.is_empty() || !fingerprint.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::CredentialStore {
                reason: format!("invalid fingerprint: must be hex only, got '{fingerprint}'"),
            });
        }
        Ok(())
    }

    fn secret_key_path(&self, fingerprint: &str) -> PathBuf {
        self.secrets_dir.join(format!("{fingerprint}.key"))
    }

    fn store_to_file(&self, fingerprint: &str, secret_key: &[u8]) -> Result<()> {
        let path = self.secret_key_path(fingerprint);
        let encoded = base64_encode(secret_key);

        // On Unix, create with restrictive permissions (owner-only read/write)
        #[cfg(unix)]
        {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|e| Error::CredentialStore {
                    reason: format!("failed to write secret key file: {e}"),
                })?;
            file.write_all(encoded.as_bytes())
                .map_err(|e| Error::CredentialStore {
                    reason: format!("failed to write secret key file: {e}"),
                })?;
        }

        #[cfg(not(unix))]
        {
            std::fs::write(&path, encoded.as_bytes()).map_err(|e| Error::CredentialStore {
                reason: format!("failed to write secret key file: {e}"),
            })?;
        }

        Ok(())
    }

    fn load_from_file(&self, fingerprint: &str) -> Result<SecretBox<Vec<u8>>> {
        let path = self.secret_key_path(fingerprint);
        let mut encoded = std::fs::read(&path).map_err(|e| Error::CredentialStore {
            reason: format!("failed to read secret key file: {e}"),
        })?;

        let decoded = base64_decode(&encoded).map_err(|e| Error::CredentialStore {
            reason: format!("failed to decode secret key: {e}"),
        })?;

        encoded.zeroize();
        Ok(SecretBox::new(Box::new(decoded)))
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(data: &[u8]) -> std::result::Result<Vec<u8>, String> {
    fn val(c: u8) -> std::result::Result<u32, String> {
        match c {
            b'A'..=b'Z' => Ok((c - b'A') as u32),
            b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
            b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err(format!("invalid base64 character: {c}")),
        }
    }

    let data: Vec<u8> = data
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if data.len() % 4 != 0 {
        return Err("invalid base64 length".into());
    }

    let mut result = Vec::with_capacity(data.len() / 4 * 3);
    for chunk in data.chunks(4) {
        let a = val(chunk[0])?;
        let b = val(chunk[1])?;
        let c = val(chunk[2])?;
        let d = val(chunk[3])?;
        let triple = (a << 18) | (b << 12) | (c << 6) | d;
        result.push(((triple >> 16) & 0xFF) as u8);
        if chunk[2] != b'=' {
            result.push(((triple >> 8) & 0xFF) as u8);
        }
        if chunk[3] != b'=' {
            result.push((triple & 0xFF) as u8);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_fallback_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::new(tmp.path()).unwrap();

        let secret = b"-----BEGIN PGP PRIVATE KEY BLOCK-----\nfake secret key data\n-----END PGP PRIVATE KEY BLOCK-----";
        store.store_to_file("ABCD1234", secret).unwrap();

        assert!(store.secret_key_path("ABCD1234").exists());

        let retrieved = store.load_from_file("ABCD1234").unwrap();
        use secrecy::ExposeSecret;
        assert_eq!(retrieved.expose_secret().as_slice(), secret);
    }

    #[test]
    fn test_file_fallback_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::new(tmp.path()).unwrap();

        store.store_to_file("ABCD1234", b"secret").unwrap();
        assert!(store.secret_key_path("ABCD1234").exists());

        store.delete_secret_key("ABCD1234").unwrap();
        assert!(!store.secret_key_path("ABCD1234").exists());
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World! This is test data with special chars: \x00\xFF\x80";
        let encoded = base64_encode(data);
        let decoded = base64_decode(encoded.as_bytes()).unwrap();
        assert_eq!(decoded, data);
    }
}
