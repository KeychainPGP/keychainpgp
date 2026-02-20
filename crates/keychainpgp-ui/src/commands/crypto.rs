//! Tauri commands for encryption and decryption.

use serde::Serialize;
use tauri::State;

use keychainpgp_core::CryptoEngine;
use secrecy::ExposeSecret;

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

/// Encrypt the current clipboard content for the given recipients.
#[tauri::command]
pub fn encrypt_clipboard(
    state: State<'_, AppState>,
    recipient_fingerprints: Vec<String>,
) -> Result<EncryptResult, String> {
    // Read clipboard
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Your clipboard is empty. Copy some text first, then try again. ({e})"))?
        .ok_or_else(|| "Your clipboard is empty. Copy some text first, then try again.".to_string())?;

    // Look up recipient public keys
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;

    let mut recipient_keys = Vec::new();
    for fp in &recipient_fingerprints {
        let record = keyring
            .get_key(fp)
            .map_err(|e| format!("Failed to look up key: {e}"))?
            .ok_or_else(|| format!("Key not found: {fp}"))?;
        recipient_keys.push(record.pgp_data);
    }

    drop(keyring);

    // Encrypt
    let ciphertext = state
        .engine
        .encrypt(clipboard_text.as_bytes(), &recipient_keys)
        .map_err(|e| format!("Encryption failed: {e}"))?;

    // Write ciphertext to clipboard
    let armored = String::from_utf8(ciphertext)
        .map_err(|_| "Internal error: encrypted output is not valid text".to_string())?;

    keychainpgp_clipboard::monitor::write_clipboard_text(&armored)
        .map_err(|e| format!("Failed to write to clipboard: {e}"))?;

    Ok(EncryptResult {
        success: true,
        message: "Message encrypted and copied to clipboard.".into(),
    })
}

/// Decrypt the current clipboard content.
#[tauri::command]
pub fn decrypt_clipboard(
    state: State<'_, AppState>,
    passphrase: Option<String>,
) -> Result<DecryptResult, String> {
    // Read clipboard
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Could not read clipboard: {e}"))?
        .ok_or_else(|| "Your clipboard is empty. Copy an encrypted message first.".to_string())?;

    if !keychainpgp_clipboard::detect::is_encrypted_message(&clipboard_text) {
        return Err(
            "The clipboard doesn't contain a valid encrypted message. \
             Make sure you copied the entire message, including the BEGIN and END lines."
                .into(),
        );
    }

    // Find a matching secret key
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

    // Try each own key
    for key_record in &own_keys {
        let secret_key = match keyring.get_secret_key(&key_record.fingerprint) {
            Ok(sk) => sk,
            Err(_) => continue,
        };

        // Check passphrase cache if no explicit passphrase provided
        let cached = if passphrase.is_none() {
            state.passphrase_cache.lock().ok()
                .and_then(|c| c.get(&key_record.fingerprint).map(|b| b.to_vec()))
        } else {
            None
        };
        let pp = passphrase.as_deref().map(|p| p.as_bytes())
            .or(cached.as_deref());

        match state.engine.decrypt(
            clipboard_text.as_bytes(),
            secret_key.expose_secret(),
            pp,
        ) {
            Ok(plaintext) => {
                // Cache the passphrase on success
                if let Some(ref p) = passphrase {
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

/// Sign the current clipboard content with the user's private key.
#[tauri::command]
pub fn sign_clipboard(
    state: State<'_, AppState>,
    passphrase: Option<String>,
) -> Result<SignResult, String> {
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Could not read clipboard: {e}"))?
        .ok_or_else(|| "Your clipboard is empty. Copy some text first.".to_string())?;

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

    for key_record in &own_keys {
        let secret_key = match keyring.get_secret_key(&key_record.fingerprint) {
            Ok(sk) => sk,
            Err(_) => continue,
        };

        // Check passphrase cache if no explicit passphrase provided
        let cached = if passphrase.is_none() {
            state.passphrase_cache.lock().ok()
                .and_then(|c| c.get(&key_record.fingerprint).map(|b| b.to_vec()))
        } else {
            None
        };
        let pp = passphrase.as_deref().map(|p| p.as_bytes())
            .or(cached.as_deref());

        match state.engine.sign(
            clipboard_text.as_bytes(),
            secret_key.expose_secret(),
            pp,
        ) {
            Ok(signed_data) => {
                // Cache the passphrase on success
                if let Some(ref p) = passphrase {
                    if let Ok(mut cache) = state.passphrase_cache.lock() {
                        cache.store(&key_record.fingerprint, p.as_bytes());
                    }
                }

                let signed_text = String::from_utf8(signed_data)
                    .map_err(|_| "Internal error: signed output is not valid text".to_string())?;

                keychainpgp_clipboard::monitor::write_clipboard_text(&signed_text)
                    .map_err(|e| format!("Failed to write to clipboard: {e}"))?;

                return Ok(SignResult {
                    success: true,
                    message: "Message signed and copied to clipboard.".into(),
                });
            }
            Err(_) => continue,
        }
    }

    Err("Failed to sign. Your key may require a passphrase.".into())
}

/// Verify a signed message on the clipboard.
#[tauri::command]
pub fn verify_clipboard(
    state: State<'_, AppState>,
) -> Result<VerifyResultInfo, String> {
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Could not read clipboard: {e}"))?
        .ok_or_else(|| "Your clipboard is empty. Copy a signed message first.".to_string())?;

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
        match state.engine.verify(clipboard_text.as_bytes(), &key_record.pgp_data) {
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

/// Clear all cached passphrases.
#[tauri::command]
pub fn clear_passphrase_cache(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut cache = state.passphrase_cache.lock().map_err(|e| format!("Internal error: {e}"))?;
    cache.clear_all();
    Ok(())
}
