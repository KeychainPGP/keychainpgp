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
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

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
        .map_err(|e| {
            format!("Your clipboard is empty. Copy some text first, then try again. ({e})")
        })?
        .ok_or_else(|| {
            "Your clipboard is empty. Copy some text first, then try again.".to_string()
        })?;

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
        return Err("The text doesn't contain a valid encrypted message. \
             Make sure you have the entire message, including the BEGIN and END lines."
            .into());
    }

    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
    let own_keys = keyring
        .list_keys()
        .map_err(|e| format!("Failed to list keys: {e}"))?
        .into_iter()
        .filter(|k| k.is_own_key)
        .collect::<Vec<_>>();

    if own_keys.is_empty() {
        return Err("You don't have any private keys. Generate or import a key first.".into());
    }

    let is_opsec = state.opsec_mode.load(Ordering::SeqCst);

    for key_record in &own_keys {
        let secret_key: SecretBox<Vec<u8>> = if is_opsec {
            let opsec_keys = state
                .opsec_secret_keys
                .lock()
                .map_err(|e| format!("Internal error: {e}"))?;
            match opsec_keys.get(&key_record.fingerprint) {
                Some(k) => SecretBox::new(Box::new((**k).clone())),
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
            state
                .passphrase_cache
                .lock()
                .ok()
                .and_then(|c| c.get(&key_record.fingerprint).map(|b| b.to_vec()))
        } else {
            None
        };
        let pp = passphrase.map(|p| p.as_bytes()).or(cached.as_deref());

        match state
            .engine
            .decrypt(ciphertext.as_bytes(), secret_key.expose_secret(), pp)
        {
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
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
    let own_keys = keyring
        .list_keys()
        .map_err(|e| format!("Failed to list keys: {e}"))?
        .into_iter()
        .filter(|k| k.is_own_key)
        .collect::<Vec<_>>();

    if own_keys.is_empty() {
        return Err("You don't have any private keys. Generate or import a key first.".into());
    }

    let is_opsec = state.opsec_mode.load(Ordering::SeqCst);

    for key_record in &own_keys {
        let secret_key: SecretBox<Vec<u8>> = if is_opsec {
            let opsec_keys = state
                .opsec_secret_keys
                .lock()
                .map_err(|e| format!("Internal error: {e}"))?;
            match opsec_keys.get(&key_record.fingerprint) {
                Some(k) => SecretBox::new(Box::new((**k).clone())),
                None => match keyring.get_secret_key(&key_record.fingerprint) {
                    Ok(sk) => sk,
                    Err(_) => continue,
                },
            }
        } else {
            match keyring.get_secret_key(&key_record.fingerprint) {
                Ok(sk) => sk,
                Err(_) => continue,
            }
        };

        let cached = if passphrase.is_none() {
            state
                .passphrase_cache
                .lock()
                .ok()
                .and_then(|c| c.get(&key_record.fingerprint).map(|b| b.to_vec()))
        } else {
            None
        };
        let pp = passphrase.map(|p| p.as_bytes()).or(cached.as_deref());

        match state
            .engine
            .sign(plaintext.as_bytes(), secret_key.expose_secret(), pp)
        {
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
fn verify_impl(state: &AppState, signed_text: &str) -> Result<VerifyResultInfo, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
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
        match state
            .engine
            .verify(signed_text.as_bytes(), &key_record.pgp_data)
        {
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
        message: "Signature could not be verified. The signer's key may not be in your keyring."
            .into(),
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
pub fn verify_clipboard(state: State<'_, AppState>) -> Result<VerifyResultInfo, String> {
    let clipboard_text = keychainpgp_clipboard::monitor::read_clipboard_text()
        .map_err(|e| format!("Could not read clipboard: {e}"))?
        .ok_or_else(|| "Your clipboard is empty. Copy a signed message first.".to_string())?;

    verify_impl(&state, &clipboard_text)
}

/// Verify a signed message from text (does not touch clipboard).
#[tauri::command]
pub fn verify_text(state: State<'_, AppState>, text: String) -> Result<VerifyResultInfo, String> {
    verify_impl(&state, &text)
}

/// Clear all cached passphrases.
#[tauri::command]
pub fn clear_passphrase_cache(state: State<'_, AppState>) -> Result<(), String> {
    let mut cache = state
        .passphrase_cache
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
    cache.clear_all();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use keychainpgp_core::types::{KeyGenOptions, UserId};
    use keychainpgp_keys::storage::KeyRecord;
    use secrecy::ExposeSecret;

    /// Create an AppState backed by a temporary directory.
    fn setup() -> (AppState, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState::initialize_with_dir(tmp.path()).unwrap();
        (state, tmp)
    }

    /// Generate a key pair, store it in the keyring, and return the fingerprint.
    fn generate_and_store(state: &AppState, name: &str, email: &str) -> String {
        generate_and_store_with_passphrase(state, name, email, None)
    }

    /// Generate a key pair with optional passphrase, store it, return the fingerprint.
    fn generate_and_store_with_passphrase(
        state: &AppState,
        name: &str,
        email: &str,
        passphrase: Option<&str>,
    ) -> String {
        let user_id = UserId::new(name, email);
        let mut options = KeyGenOptions::new(user_id);
        if let Some(pp) = passphrase {
            options = options.with_passphrase(SecretBox::new(Box::new(pp.as_bytes().to_vec())));
        }

        let key_pair = state.engine.generate_key_pair(options).unwrap();
        let info = state.engine.inspect_key(&key_pair.public_key).unwrap();

        let record = KeyRecord {
            fingerprint: key_pair.fingerprint.0.clone(),
            name: Some(name.to_string()),
            email: Some(email.to_string()),
            algorithm: info.algorithm.to_string(),
            created_at: info.created_at,
            expires_at: info.expires_at,
            trust_level: 2,
            is_own_key: true,
            is_revoked: info.is_revoked,
            pgp_data: key_pair.public_key.clone(),
        };

        let keyring = state.keyring.lock().unwrap();
        keyring
            .store_generated_key(record, key_pair.secret_key.expose_secret())
            .unwrap();

        key_pair.fingerprint.0.clone()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (state, _tmp) = setup();
        let fp = generate_and_store(&state, "Alice", "alice@test.com");

        let plaintext = "Hello, this is a secret message!";
        let ciphertext = encrypt_impl(&state, plaintext, &[fp]).unwrap();
        assert!(ciphertext.contains("BEGIN PGP MESSAGE"));

        let result = decrypt_impl(&state, &ciphertext, None).unwrap();
        assert!(result.success);
        assert_eq!(result.plaintext, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_with_passphrase() {
        let (state, _tmp) = setup();
        let fp = generate_and_store_with_passphrase(
            &state,
            "Bob",
            "bob@test.com",
            Some("strong-passphrase"),
        );

        let plaintext = "Passphrase-protected message";
        let ciphertext = encrypt_impl(&state, plaintext, &[fp]).unwrap();

        let result = decrypt_impl(&state, &ciphertext, Some("strong-passphrase")).unwrap();
        assert!(result.success);
        assert_eq!(result.plaintext, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_passphrase_fails() {
        let (state, _tmp) = setup();
        let fp = generate_and_store_with_passphrase(
            &state,
            "Carol",
            "carol@test.com",
            Some("correct-passphrase"),
        );

        let plaintext = "Secret";
        let ciphertext = encrypt_impl(&state, plaintext, &[fp]).unwrap();

        let result = decrypt_impl(&state, &ciphertext, Some("wrong-passphrase"));
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let (state, _tmp) = setup();
        generate_and_store(&state, "Dave", "dave@test.com");

        let plaintext = "This message is authentic.";
        let signed = sign_impl(&state, plaintext, None).unwrap();
        assert!(signed.contains("BEGIN PGP MESSAGE"));

        let result = verify_impl(&state, &signed).unwrap();
        assert!(result.valid);
        assert_eq!(result.signer_name.as_deref(), Some("Dave"));
    }

    #[test]
    fn test_sign_verify_with_passphrase() {
        let (state, _tmp) = setup();
        generate_and_store_with_passphrase(&state, "Eve", "eve@test.com", Some("sign-passphrase"));

        let plaintext = "Signed with passphrase";
        let signed = sign_impl(&state, plaintext, Some("sign-passphrase")).unwrap();

        let result = verify_impl(&state, &signed).unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_decrypt_no_own_keys() {
        let (state, _tmp) = setup();
        // No keys stored — decrypt should fail
        let pgp_msg = "-----BEGIN PGP MESSAGE-----\n\nwA0D\n-----END PGP MESSAGE-----";
        let result = decrypt_impl(&state, pgp_msg, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("don't have any private keys"));
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let (state, _tmp) = setup();

        // Generate two key pairs
        let fp_alice = generate_and_store(&state, "Alice2", "alice2@test.com");
        let _fp_bob = generate_and_store(&state, "Bob2", "bob2@test.com");

        // Encrypt only for Alice
        let ciphertext = encrypt_impl(&state, "For Alice only", &[fp_alice.clone()]).unwrap();

        // Delete Alice's key, keep only Bob's
        {
            let keyring = state.keyring.lock().unwrap();
            keyring.delete_key(&fp_alice).unwrap();
        }

        // Bob cannot decrypt Alice's message
        let result = decrypt_impl(&state, &ciphertext, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_key_fails() {
        let (state, _tmp) = setup();
        let fp_signer = generate_and_store(&state, "Signer", "signer@test.com");

        let signed = sign_impl(&state, "Authentic message", None).unwrap();

        // Delete the signer key
        {
            let keyring = state.keyring.lock().unwrap();
            keyring.delete_key(&fp_signer).unwrap();
        }

        // Add a different key
        generate_and_store(&state, "Other", "other@test.com");

        // Verification should fail (wrong key)
        let result = verify_impl(&state, &signed).unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn test_full_crypto_roundtrip() {
        let (state, _tmp) = setup();

        // Generate sender and recipient
        let fp_sender = generate_and_store(&state, "Sender", "sender@test.com");
        let fp_recipient = generate_and_store(&state, "Recipient", "recipient@test.com");

        let plaintext = "Confidential message from Sender to Recipient";

        // Encrypt for recipient
        let ciphertext = encrypt_impl(&state, plaintext, &[fp_recipient.clone()]).unwrap();

        // Recipient decrypts
        let decrypted = decrypt_impl(&state, &ciphertext, None).unwrap();
        assert!(decrypted.success);
        assert_eq!(decrypted.plaintext, plaintext);

        // Sign by sender (sign_impl tries all own keys, sender's key should work)
        let signed = sign_impl(&state, plaintext, None).unwrap();

        // Verify signature
        let verified = verify_impl(&state, &signed).unwrap();
        assert!(verified.valid);

        // Delete sender, keep recipient — re-sign should still work with recipient's key
        {
            let keyring = state.keyring.lock().unwrap();
            keyring.delete_key(&fp_sender).unwrap();
        }
        let signed2 = sign_impl(&state, "Another message", None).unwrap();
        let verified2 = verify_impl(&state, &signed2).unwrap();
        assert!(verified2.valid);
        assert_eq!(verified2.signer_name.as_deref(), Some("Recipient"));
    }
}
