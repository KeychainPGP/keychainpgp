//! OS credential store integration for private key storage.
//!
//! - Windows: DPAPI via the `keyring` crate
//! - macOS: Keychain Services via the `keyring` crate
//! - Linux: Secret Service (GNOME Keyring / KDE Wallet) via the `keyring` crate

use secrecy::SecretBox;
use zeroize::Zeroize;

use crate::error::{Error, Result};

const SERVICE_NAME: &str = "keychainpgp";

/// Abstraction over OS credential storage for private keys.
pub struct CredentialStore;

impl CredentialStore {
    /// Store a private key in the OS credential store.
    ///
    /// The key is identified by its fingerprint.
    pub fn store_secret_key(fingerprint: &str, secret_key: &[u8]) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, fingerprint).map_err(|e| {
            Error::CredentialStore {
                reason: format!("failed to create credential entry: {e}"),
            }
        })?;

        // Store as base64 since some credential backends don't handle raw bytes well
        let encoded = base64_encode(secret_key);
        entry
            .set_secret(encoded.as_bytes())
            .map_err(|e| Error::CredentialStore {
                reason: format!("failed to store secret key: {e}"),
            })?;

        Ok(())
    }

    /// Retrieve a private key from the OS credential store.
    pub fn get_secret_key(fingerprint: &str) -> Result<SecretBox<Vec<u8>>> {
        let entry = keyring::Entry::new(SERVICE_NAME, fingerprint).map_err(|e| {
            Error::CredentialStore {
                reason: format!("failed to create credential entry: {e}"),
            }
        })?;

        let mut encoded = entry.get_secret().map_err(|e| Error::CredentialStore {
            reason: format!("failed to retrieve secret key: {e}"),
        })?;

        let decoded = base64_decode(&encoded).map_err(|e| Error::CredentialStore {
            reason: format!("failed to decode secret key: {e}"),
        })?;

        // Zeroize the intermediate encoded form
        encoded.zeroize();

        Ok(SecretBox::new(Box::new(decoded)))
    }

    /// Delete a private key from the OS credential store.
    pub fn delete_secret_key(fingerprint: &str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, fingerprint).map_err(|e| {
            Error::CredentialStore {
                reason: format!("failed to create credential entry: {e}"),
            }
        })?;

        entry
            .delete_credential()
            .map_err(|e| Error::CredentialStore {
                reason: format!("failed to delete secret key: {e}"),
            })?;

        Ok(())
    }

    /// Check if a secret key exists in the credential store.
    pub fn has_secret_key(fingerprint: &str) -> bool {
        let entry = match keyring::Entry::new(SERVICE_NAME, fingerprint) {
            Ok(e) => e,
            Err(_) => return false,
        };
        entry.get_secret().is_ok()
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
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

    let data: Vec<u8> = data.iter().copied().filter(|b| !b.is_ascii_whitespace()).collect();
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
