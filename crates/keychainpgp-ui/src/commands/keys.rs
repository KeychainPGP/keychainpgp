//! Tauri commands for key management.

use serde::Serialize;
use tauri::State;

use keychainpgp_core::types::{KeyGenOptions, TrustLevel, UserId};
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

/// Set the trust level of a key.
#[tauri::command]
pub fn set_key_trust(
    state: State<'_, AppState>,
    fingerprint: String,
    trust_level: i32,
) -> Result<bool, String> {
    let trust = match trust_level {
        0 => TrustLevel::Unknown,
        1 => TrustLevel::Unverified,
        2 => TrustLevel::Verified,
        _ => return Err(format!("Invalid trust level: {trust_level}")),
    };
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    keyring
        .set_trust(&fingerprint, trust)
        .map_err(|e| format!("Failed to set trust: {e}"))
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

/// Subkey information returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct SubkeyInfoDto {
    pub fingerprint: String,
    pub algorithm: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub capabilities: Vec<String>,
    pub is_revoked: bool,
}

/// User ID information returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct UserIdDto {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Detailed key information including subkeys and all User IDs.
#[derive(Debug, Clone, Serialize)]
pub struct KeyDetailedInfo {
    pub fingerprint: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub algorithm: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub trust_level: i32,
    pub is_own_key: bool,
    pub user_ids: Vec<UserIdDto>,
    pub subkeys: Vec<SubkeyInfoDto>,
}

/// Inspect a key and return detailed metadata including subkeys and all User IDs.
#[tauri::command]
pub fn inspect_key_detailed(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<KeyDetailedInfo, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let record = keyring
        .get_key(&fingerprint)
        .map_err(|e| format!("Failed to look up key: {e}"))?
        .ok_or_else(|| format!("Key not found: {fingerprint}"))?;

    let cert_info = state
        .engine
        .inspect_key(&record.pgp_data)
        .map_err(|e| format!("Failed to inspect key: {e}"))?;

    let user_ids = cert_info
        .user_ids
        .iter()
        .map(|uid| UserIdDto {
            name: uid.name.clone(),
            email: uid.email.clone(),
        })
        .collect();

    let subkeys = cert_info
        .subkeys
        .iter()
        .map(|sk| SubkeyInfoDto {
            fingerprint: sk.fingerprint.clone(),
            algorithm: sk.algorithm.clone(),
            created_at: sk.created_at.clone(),
            expires_at: sk.expires_at.clone(),
            capabilities: sk.capabilities.iter().map(|c| c.to_string()).collect(),
            is_revoked: sk.is_revoked,
        })
        .collect();

    Ok(KeyDetailedInfo {
        fingerprint: record.fingerprint,
        name: record.name,
        email: record.email,
        algorithm: record.algorithm,
        created_at: record.created_at,
        expires_at: record.expires_at,
        trust_level: record.trust_level,
        is_own_key: record.is_own_key,
        user_ids,
        subkeys,
    })
}

/// Export a public key as a QR code SVG.
#[tauri::command]
pub fn export_key_qr(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<String, String> {
    let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
    let record = keyring
        .get_key(&fingerprint)
        .map_err(|e| format!("Failed to look up key: {e}"))?
        .ok_or_else(|| format!("Key not found: {fingerprint}"))?;

    let key_text = String::from_utf8_lossy(&record.pgp_data).into_owned();

    let qr = qrcode::QrCode::new(key_text.as_bytes())
        .map_err(|e| format!("Key is too large for a QR code: {e}"))?;

    let svg = qr
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(256, 256)
        .build();

    Ok(svg)
}

/// Look up a key via WKD (Web Key Directory) by email address.
#[tauri::command]
pub async fn wkd_lookup(
    state: State<'_, AppState>,
    email: String,
) -> Result<Option<KeyInfo>, String> {
    let key_bytes = keychainpgp_keys::network::wkd::wkd_lookup(&email)
        .await
        .map_err(|e| e.to_string())?;

    let cert_info = state
        .engine
        .inspect_key(&key_bytes)
        .map_err(|e| format!("Invalid key data from WKD: {e}"))?;

    let name = cert_info.name().map(String::from);
    let email_val = cert_info.email().map(String::from);
    let fp = cert_info.fingerprint.0.clone();

    Ok(Some(KeyInfo {
        fingerprint: fp,
        name,
        email: email_val,
        algorithm: cert_info.algorithm.to_string(),
        created_at: cert_info.created_at,
        expires_at: cert_info.expires_at,
        trust_level: 0,
        is_own_key: false,
    }))
}

/// Search for keys on a keyserver.
#[tauri::command]
pub async fn keyserver_search(
    state: State<'_, AppState>,
    query: String,
    keyserver_url: Option<String>,
) -> Result<Vec<KeyInfo>, String> {
    let url = keyserver_url.unwrap_or_else(|| "https://keys.openpgp.org".to_string());

    let results = keychainpgp_keys::network::keyserver::keyserver_search(&query, &url)
        .await
        .map_err(|e| e.to_string())?;

    let mut keys = Vec::new();
    for result in results {
        match state.engine.inspect_key(&result.key_data) {
            Ok(cert_info) => {
                let name = cert_info.name().map(String::from);
                let email_val = cert_info.email().map(String::from).or(result.email);
                let fp = cert_info.fingerprint.0.clone();
                keys.push(KeyInfo {
                    fingerprint: fp,
                    name,
                    email: email_val,
                    algorithm: cert_info.algorithm.to_string(),
                    created_at: cert_info.created_at,
                    expires_at: cert_info.expires_at,
                    trust_level: 0,
                    is_own_key: false,
                });
            }
            Err(_) => continue,
        }
    }

    Ok(keys)
}

/// Upload a public key to a keyserver.
#[tauri::command]
pub async fn keyserver_upload(
    state: State<'_, AppState>,
    fingerprint: String,
    keyserver_url: Option<String>,
) -> Result<String, String> {
    let url = keyserver_url.unwrap_or_else(|| "https://keys.openpgp.org".to_string());

    let key_data = {
        let keyring = state.keyring.lock().map_err(|e| format!("Internal error: {e}"))?;
        let record = keyring
            .get_key(&fingerprint)
            .map_err(|e| format!("Failed to look up key: {e}"))?
            .ok_or_else(|| format!("Key not found: {fingerprint}"))?;
        record.pgp_data.clone()
    };

    keychainpgp_keys::network::keyserver::keyserver_upload(&key_data, &url)
        .await
        .map_err(|e| e.to_string())
}
