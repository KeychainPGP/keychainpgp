//! Tauri commands for encryption and decryption.

use serde::{Deserialize, Serialize};
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

    let passphrase_bytes = passphrase.as_deref().map(|p| p.as_bytes());

    // Try each own key
    for key_record in &own_keys {
        let secret_key = match keyring.get_secret_key(&key_record.fingerprint) {
            Ok(sk) => sk,
            Err(_) => continue,
        };

        match state.engine.decrypt(
            clipboard_text.as_bytes(),
            secret_key.expose_secret(),
            passphrase_bytes,
        ) {
            Ok(plaintext) => {
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
