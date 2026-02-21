//! Tauri commands for encryption and decryption.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::State;

use keychainpgp_core::CryptoEngine;
use secrecy::{ExposeSecret, SecretBox};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct EncryptResult {
    /// Whether encryption succeeded.
    pub success: bool,
    /// Human-readable status message.
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DecryptResult {
    /// Whether decryption succeeded.
    pub success: bool,
    /// The decrypted plaintext (empty if failed).
    pub plaintext: String,
    /// Human-readable status message.
    pub message: String,
}

/// Shared encrypt logic: encrypt plaintext for given recipients, return armored ciphertext.
fn encrypt_impl(
    state: &AppState,
    plaintext: &str,
    recipient_fingerprints: &[String],
) -> Result<String, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;

    let mut recipient_keys = Vec::new();
    for fp in recipient_fingerprints {
        let record = keyring
            .get_key(fp)
            .map_err(|e| format!("Failed to look up key: {e}"))?
            .ok_or_else(|| format!("Key not found: {fp}"))?;
        recipient_keys.push(record.pgp_data);
    }

    drop(keyring);

    let ciphertext = state
        .engine
        .encrypt(plaintext.as_bytes(), &recipient_keys)
        .map_err(|e| format!("Encryption failed: {e}"))?;

    String::from_utf8(ciphertext)
        .map_err(|_| "Internal error: encrypted output is not valid text".to_string())
}

/// Encrypt the current clipboard content for the given recipients.
#[cfg(desktop)]
#[tauri::command]
pub fn encrypt_clipboard(
    state: State<'_, AppState>,
    recipient_fingerprints: Vec<String>,
) -> Result<EncryptResult, String> {
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Your clipboard is empty. Copy some text first, then try again. ({e})"))?
        .ok_or_else(|| "Your clipboard is empty. Copy some text first, then try again.".to_string())?;

    let armored = encrypt_impl(&state, &clipboard_text, &recipient_fingerprints)?;

    keychainpgp_clipboard::monitor::write_clipboard_text(&armored)
        .map_err(|e| format!("Failed to write to clipboard: {e}"))?;

    Ok(EncryptResult {
        success: true,
        message: "Message encrypted and copied to clipboard.".into(),
    })
}

/// Encrypt a given text for the given recipients (does not touch clipboard).
#[tauri::command]
pub fn encrypt_text(
    state: State<'_, AppState>,
    text: String,
    recipient_fingerprints: Vec<String>,
) -> Result<EncryptResult, String> {
    let armored = encrypt_impl(&state, &text, &recipient_fingerprints)?;

    Ok(EncryptResult {
        success: true,
        message: armored,
    })
}

/// Shared decrypt logic: decrypt ciphertext, return plaintext.
fn decrypt_impl(
    state: &AppState,
    ciphertext: &str,
    passphrase: Option<&str>,
) -> Result<DecryptResult, String> {
    if keychainpgp_core::armor::detect_pgp_block(ciphertext.as_bytes())
        != Some(keychainpgp_core::armor::PgpBlockKind::Message)
    {
        return Err(
            "The text doesn't contain a valid encrypted message. \
             Make sure you have the entire message, including the BEGIN and END lines."
                .into(),
        );
    }

    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let own_keys = keyring
        .list_keys()
        .map_err(|e| format!("Failed to list keys: {e}"))?
        .into_iter()
        .filter(|k| k.is_own_key)
        .collect::<Vec<_>>();

    if own_keys.is_empty() {
        return Err(
            "You don't have any private keys. Generate or import a key first.".into(),
        );
    }

    let is_opsec = state.opsec_mode.load(Ordering::Relaxed);

    for key_record in &own_keys {
        let secret_key: SecretBox<Vec<u8>> = if is_opsec {
            let opsec_keys = state.opsec_secret_keys.lock()
                .map_err(|e| format!("Internal error: {e}"))?;
            match opsec_keys.get(&key_record.fingerprint) {
                Some(k) => SecretBox::new(Box::new(k.clone())),
                None => {
                    // Also try the regular keyring (keys imported before OPSEC was enabled)
                    match keyring.get_secret_key(&key_record.fingerprint) {
                        Ok(sk) => sk,
                        Err(_) => continue,
                    }
                }
            }
        } else {
            match keyring.get_secret_key(&key_record.fingerprint) {
                Ok(sk) => sk,
                Err(_) => continue,
            }
        };

        let cached = if passphrase.is_none() {
            state.passphrase_cache.lock().ok()
                .and_then(|c| c.get(&key_record.fingerprint).map(|b| b.to_vec()))
        } else {
            None
        };
        let pp = passphrase.map(|p| p.as_bytes())
            .or(cached.as_deref());

        match state.engine.decrypt(
            ciphertext.as_bytes(),
            secret_key.expose_secret(),
            pp,
        ) {
            Ok(plaintext) => {
                if let Some(p) = passphrase {
                    if let Ok(mut cache) = state.passphrase_cache.lock() {
                        cache.store(&key_record.fingerprint, p.as_bytes());
                    }
                }
                let text = String::from_utf8_lossy(&plaintext).into_owned();
                return Ok(DecryptResult {
                    success: true,
                    plaintext: text,
                    message: "Message decrypted successfully.".into(),
                });
            }
            Err(_) => continue,
        }
    }

    Err(
        "You don't have the private key needed to decrypt this message. \
         It may have been encrypted for a different key."
            .into(),
    )
}

/// Decrypt the current clipboard content.
#[cfg(desktop)]
#[tauri::command]
pub fn decrypt_clipboard(
    state: State<'_, AppState>,
    passphrase: Option<String>,
) -> Result<DecryptResult, String> {
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Could not read clipboard: {e}"))?
        .ok_or_else(|| "Your clipboard is empty. Copy an encrypted message first.".to_string())?;

    decrypt_impl(&state, &clipboard_text, passphrase.as_deref())
}

/// Decrypt a given text (does not touch clipboard).
#[tauri::command]
pub fn decrypt_text(
    state: State<'_, AppState>,
    text: String,
    passphrase: Option<String>,
) -> Result<DecryptResult, String> {
    decrypt_impl(&state, &text, passphrase.as_deref())
}

#[derive(Debug, Serialize)]
pub struct SignResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResultInfo {
    pub valid: bool,
    pub signer_name: Option<String>,
    pub signer_email: Option<String>,
    pub signer_fingerprint: Option<String>,
    pub trust_level: i32,
    pub message: String,
}

/// Shared sign logic: sign plaintext, return armored signed text.
fn sign_impl(
    state: &AppState,
    plaintext: &str,
    passphrase: Option<&str>,
) -> Result<String, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let own_keys = keyring
        .list_keys()
        .map_err(|e| format!("Failed to list keys: {e}"))?
        .into_iter()
        .filter(|k| k.is_own_key)
        .collect::<Vec<_>>();

    if own_keys.is_empty() {
        return Err("You don't have any private keys. Generate or import a key first.".into());
    }

    let is_opsec = state.opsec_mode.load(Ordering::Relaxed);

    for key_record in &own_keys {
        let secret_key: SecretBox<Vec<u8>> = if is_opsec {
            let opsec_keys = state.opsec_secret_keys.lock()
                .map_err(|e| format!("Internal error: {e}"))?;
            match opsec_keys.get(&key_record.fingerprint) {
                Some(k) => SecretBox::new(Box::new(k.clone())),
                None => {
                    match keyring.get_secret_key(&key_record.fingerprint) {
                        Ok(sk) => sk,
                        Err(_) => continue,
                    }
                }
            }
        } else {
            match keyring.get_secret_key(&key_record.fingerprint) {
                Ok(sk) => sk,
                Err(_) => continue,
            }
        };

        let cached = if passphrase.is_none() {
            state.passphrase_cache.lock().ok()
                .and_then(|c| c.get(&key_record.fingerprint).map(|b| b.to_vec()))
        } else {
            None
        };
        let pp = passphrase.map(|p| p.as_bytes())
            .or(cached.as_deref());

        match state.engine.sign(
            plaintext.as_bytes(),
            secret_key.expose_secret(),
            pp,
        ) {
            Ok(signed_data) => {
                if let Some(p) = passphrase {
                    if let Ok(mut cache) = state.passphrase_cache.lock() {
                        cache.store(&key_record.fingerprint, p.as_bytes());
                    }
                }

                return String::from_utf8(signed_data)
                    .map_err(|_| "Internal error: signed output is not valid text".to_string());
            }
            Err(_) => continue,
        }
    }

    Err("Failed to sign. Your key may require a passphrase.".into())
}

/// Shared verify logic: verify signed text against all keys in keyring.
fn verify_impl(
    state: &AppState,
    signed_text: &str,
) -> Result<VerifyResultInfo, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let all_keys = keyring
        .list_keys()
        .map_err(|e| format!("Failed to list keys: {e}"))?;

    if all_keys.is_empty() {
        return Ok(VerifyResultInfo {
            valid: false,
            signer_name: None,
            signer_email: None,
            signer_fingerprint: None,
            trust_level: 0,
            message: "No keys in keyring to verify against.".into(),
        });
    }

    for key_record in &all_keys {
        match state.engine.verify(signed_text.as_bytes(), &key_record.pgp_data) {
            Ok(result) if result.valid => {
                return Ok(VerifyResultInfo {
                    valid: true,
                    signer_name: key_record.name.clone(),
                    signer_email: key_record.email.clone(),
                    signer_fingerprint: result.signer_fingerprint,
                    trust_level: key_record.trust_level,
                    message: format!(
                        "Valid signature from {}.",
                        key_record.name.as_deref().unwrap_or("unknown")
                    ),
                });
            }
            _ => continue,
        }
    }

    Ok(VerifyResultInfo {
        valid: false,
        signer_name: None,
        signer_email: None,
        signer_fingerprint: None,
        trust_level: 0,
        message: "Signature could not be verified. The signer's key may not be in your keyring.".into(),
    })
}

/// Sign the current clipboard content with the user's private key.
#[cfg(desktop)]
#[tauri::command]
pub fn sign_clipboard(
    state: State<'_, AppState>,
    passphrase: Option<String>,
) -> Result<SignResult, String> {
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Could not read clipboard: {e}"))?
        .ok_or_else(|| "Your clipboard is empty. Copy some text first.".to_string())?;

    let signed_text = sign_impl(&state, &clipboard_text, passphrase.as_deref())?;

    keychainpgp_clipboard::monitor::write_clipboard_text(&signed_text)
        .map_err(|e| format!("Failed to write to clipboard: {e}"))?;

    Ok(SignResult {
        success: true,
        message: "Message signed and copied to clipboard.".into(),
    })
}

/// Sign a given text (does not touch clipboard, returns signed text in message).
#[tauri::command]
pub fn sign_text(
    state: State<'_, AppState>,
    text: String,
    passphrase: Option<String>,
) -> Result<SignResult, String> {
    let signed_text = sign_impl(&state, &text, passphrase.as_deref())?;

    Ok(SignResult {
        success: true,
        message: signed_text,
    })
}

/// Verify a signed message on the clipboard.
#[cfg(desktop)]
#[tauri::command]
pub fn verify_clipboard(
    state: State<'_, AppState>,
) -> Result<VerifyResultInfo, String> {
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Could not read clipboard: {e}"))?
        .ok_or_else(|| "Your clipboard is empty. Copy a signed message first.".to_string())?;

    verify_impl(&state, &clipboard_text)
}

/// Verify a signed message from text (does not touch clipboard).
#[tauri::command]
pub fn verify_text(
    state: State<'_, AppState>,
    text: String,
) -> Result<VerifyResultInfo, String> {
    verify_impl(&state, &text)
}

/// Clear all cached passphrases.
#[tauri::command]
pub fn clear_passphrase_cache(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut cache = state.passphrase_cache.lock().map_err(|e| format!("Internal error: {e}"))?;
    cache.clear_all();
    Ok(())
}
