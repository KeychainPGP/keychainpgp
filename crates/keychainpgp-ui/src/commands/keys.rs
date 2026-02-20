//! Tauri commands for key management.

use serde::Serialize;
use tauri::State;

use keychainpgp_core::types::{KeyGenOptions, UserId};
use keychainpgp_core::CryptoEngine;
use keychainpgp_keys::storage::KeyRecord;
use secrecy::{ExposeSecret, SecretBox};

use crate::state::AppState;

/// Key information returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct KeyInfo {
    pub fingerprint: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub algorithm: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub trust_level: i32,
    pub is_own_key: bool,
}

impl From<KeyRecord> for KeyInfo {
    fn from(r: KeyRecord) -> Self {
        Self {
            fingerprint: r.fingerprint,
            name: r.name,
            email: r.email,
            algorithm: r.algorithm,
            created_at: r.created_at,
            expires_at: r.expires_at,
            trust_level: r.trust_level,
            is_own_key: r.is_own_key,
        }
    }
}

/// Generate a new key pair and store it in the keyring.
#[tauri::command]
pub fn generate_key_pair(
    state: State<'_, AppState>,
    name: String,
    email: String,
    passphrase: Option<String>,
) -> Result<KeyInfo, String> {
    let user_id = UserId::new(&name, &email);
    let mut options = KeyGenOptions::new(user_id);

    if let Some(pass) = passphrase {
        options = options.with_passphrase(SecretBox::new(Box::new(pass.into_bytes())));
    }

    let key_pair = state
        .engine
        .generate_key_pair(options)
        .map_err(|e| format!("Key generation failed: {e}"))?;

    let info = state
        .engine
        .inspect_key(&key_pair.public_key)
        .map_err(|e| format!("Failed to inspect generated key: {e}"))?;

    let record = KeyRecord {
        fingerprint: key_pair.fingerprint.0.clone(),
        name: Some(name),
        email: Some(email),
        algorithm: info.algorithm.to_string(),
        created_at: info.created_at,
        expires_at: info.expires_at,
        trust_level: 2, // Own key = verified
        is_own_key: true,
        pgp_data: key_pair.public_key.clone(),
    };

    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    keyring
        .store_generated_key(record.clone(), key_pair.secret_key.expose_secret())
        .map_err(|e| format!("Failed to store key: {e}"))?;

    Ok(KeyInfo::from(record))
}

/// List all keys in the keyring.
#[tauri::command]
pub fn list_keys(state: State<'_, AppState>) -> Result<Vec<KeyInfo>, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let keys = keyring
        .list_keys()
        .map_err(|e| format!("Failed to list keys: {e}"))?;
    Ok(keys.into_iter().map(KeyInfo::from).collect())
}

/// Import a key from ASCII-armored text.
#[tauri::command]
pub fn import_key(
    state: State<'_, AppState>,
    key_data: String,
) -> Result<KeyInfo, String> {
    let cert_info = state
        .engine
        .inspect_key(key_data.as_bytes())
        .map_err(|e| format!("Invalid key data: {e}"))?;

    let name = cert_info.name().map(String::from);
    let email = cert_info.email().map(String::from);

    let record = KeyRecord {
        fingerprint: cert_info.fingerprint.0.clone(),
        name,
        email,
        algorithm: cert_info.algorithm.to_string(),
        created_at: cert_info.created_at,
        expires_at: cert_info.expires_at,
        trust_level: if cert_info.has_secret_key { 2 } else { 1 },
        is_own_key: cert_info.has_secret_key,
        pgp_data: key_data.as_bytes().to_vec(),
    };

    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;

    if cert_info.has_secret_key {
        keyring
            .store_generated_key(record.clone(), key_data.as_bytes())
            .map_err(|e| format!("Failed to import key: {e}"))?;
    } else {
        keyring
            .import_public_key(record.clone())
            .map_err(|e| format!("Failed to import key: {e}"))?;
    }

    Ok(KeyInfo::from(record))
}

/// Export a public key as ASCII-armored text.
#[tauri::command]
pub fn export_key(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<String, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let record = keyring
        .get_key(&fingerprint)
        .map_err(|e| format!("Failed to look up key: {e}"))?
        .ok_or_else(|| format!("Key not found: {fingerprint}"))?;

    Ok(String::from_utf8_lossy(&record.pgp_data).into_owned())
}

/// Delete a key from the keyring.
#[tauri::command]
pub fn delete_key(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<bool, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    keyring
        .delete_key(&fingerprint)
        .map_err(|e| format!("Failed to delete key: {e}"))
}

/// Search keys by name, email, or fingerprint.
#[tauri::command]
pub fn search_keys(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<KeyInfo>, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let keys = keyring
        .search_keys(&query)
        .map_err(|e| format!("Search failed: {e}"))?;
    Ok(keys.into_iter().map(KeyInfo::from).collect())
}

/// Inspect a key and return detailed metadata.
#[tauri::command]
pub fn inspect_key(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<KeyInfo, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let record = keyring
        .get_key(&fingerprint)
        .map_err(|e| format!("Failed to look up key: {e}"))?
        .ok_or_else(|| format!("Key not found: {fingerprint}"))?;

    Ok(KeyInfo::from(record))
}
