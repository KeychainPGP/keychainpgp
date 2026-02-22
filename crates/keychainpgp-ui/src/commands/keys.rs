//! Tauri commands for key management.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use keychainpgp_core::CryptoEngine;
use keychainpgp_core::types::{KeyGenOptions, TrustLevel, UserId};
use keychainpgp_keys::storage::KeyRecord;
use secrecy::{ExposeSecret, SecretBox};

use crate::state::AppState;

/// Read the proxy URL from settings if proxy is enabled.
fn get_proxy_url(app: &AppHandle) -> Option<String> {
    let store = app.store("settings.json").ok()?;
    let val = store.get("settings")?;
    let settings: super::settings::Settings = serde_json::from_value(val).ok()?;
    if !settings.proxy_enabled {
        return None;
    }
    let url = match settings.proxy_preset.as_str() {
        "tor" => "socks5://127.0.0.1:9050".to_string(),
        "lokinet" => "socks5://127.0.0.1:1080".to_string(),
        _ => settings.proxy_url,
    };
    if url.is_empty() { None } else { Some(url) }
}

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

    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    if state.opsec_mode.load(Ordering::Relaxed) {
        // OPSEC mode: store secret key in RAM only, public key in DB
        keyring
            .import_public_key(record.clone())
            .map_err(|e| format!("Failed to store key: {e}"))?;
        let mut opsec_keys = state
            .opsec_secret_keys
            .lock()
            .map_err(|e| format!("Internal error: {e}"))?;
        opsec_keys.insert(
            record.fingerprint.clone(),
            key_pair.secret_key.expose_secret().clone(),
        );
    } else {
        keyring
            .store_generated_key(record.clone(), key_pair.secret_key.expose_secret())
            .map_err(|e| format!("Failed to store key: {e}"))?;
    }

    Ok(KeyInfo::from(record))
}

/// List all keys in the keyring.
#[tauri::command]
pub fn list_keys(state: State<'_, AppState>) -> Result<Vec<KeyInfo>, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
    let keys = keyring
        .list_keys()
        .map_err(|e| format!("Failed to list keys: {e}"))?;
    Ok(keys.into_iter().map(KeyInfo::from).collect())
}

/// Import a key from ASCII-armored text.
#[tauri::command]
pub fn import_key(state: State<'_, AppState>, key_data: String) -> Result<KeyInfo, String> {
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

    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    if cert_info.has_secret_key && state.opsec_mode.load(Ordering::Relaxed) {
        // OPSEC mode: store secret key in RAM only, public key in DB
        keyring
            .import_public_key(record.clone())
            .map_err(|e| format!("Failed to import key: {e}"))?;
        let mut opsec_keys = state
            .opsec_secret_keys
            .lock()
            .map_err(|e| format!("Internal error: {e}"))?;
        opsec_keys.insert(record.fingerprint.clone(), key_data.as_bytes().to_vec());
    } else if cert_info.has_secret_key {
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
pub fn export_key(state: State<'_, AppState>, fingerprint: String) -> Result<String, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
    let record = keyring
        .get_key(&fingerprint)
        .map_err(|e| format!("Failed to look up key: {e}"))?
        .ok_or_else(|| format!("Key not found: {fingerprint}"))?;

    Ok(String::from_utf8_lossy(&record.pgp_data).into_owned())
}

/// Delete a key from the keyring.
#[tauri::command]
pub fn delete_key(state: State<'_, AppState>, fingerprint: String) -> Result<bool, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
    keyring
        .delete_key(&fingerprint)
        .map_err(|e| format!("Failed to delete key: {e}"))
}

/// Search keys by name, email, or fingerprint.
#[tauri::command]
pub fn search_keys(state: State<'_, AppState>, query: String) -> Result<Vec<KeyInfo>, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
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
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
    keyring
        .set_trust(&fingerprint, trust)
        .map_err(|e| format!("Failed to set trust: {e}"))
}

/// Inspect a key and return detailed metadata.
#[tauri::command]
pub fn inspect_key(state: State<'_, AppState>, fingerprint: String) -> Result<KeyInfo, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
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
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
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
pub fn export_key_qr(state: State<'_, AppState>, fingerprint: String) -> Result<String, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;
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

/// Generate a QR code SVG from arbitrary text data.
#[tauri::command]
pub fn generate_qr_svg(data: String) -> Result<String, String> {
    let qr = qrcode::QrCode::new(data.as_bytes())
        .map_err(|e| format!("QR generation failed: {e}"))?;
    let svg = qr
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .build();
    Ok(svg)
}

/// Look up a key via WKD (Web Key Directory) by email address.
#[tauri::command]
pub async fn wkd_lookup(
    app: AppHandle,
    state: State<'_, AppState>,
    email: String,
) -> Result<Option<KeyInfo>, String> {
    let proxy = get_proxy_url(&app);
    let key_bytes = keychainpgp_keys::network::wkd::wkd_lookup(&email, proxy.as_deref())
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
    app: AppHandle,
    state: State<'_, AppState>,
    query: String,
    keyserver_url: Option<String>,
) -> Result<Vec<KeyInfo>, String> {
    let url = keyserver_url.unwrap_or_else(|| "https://keys.openpgp.org".to_string());
    let proxy = get_proxy_url(&app);

    let results =
        keychainpgp_keys::network::keyserver::keyserver_search(&query, &url, proxy.as_deref())
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
    app: AppHandle,
    state: State<'_, AppState>,
    fingerprint: String,
    keyserver_url: Option<String>,
) -> Result<String, String> {
    let url = keyserver_url.unwrap_or_else(|| "https://keys.openpgp.org".to_string());
    let proxy = get_proxy_url(&app);

    let key_data = {
        let keyring = state
            .keyring
            .lock()
            .map_err(|e| format!("Internal error: {e}"))?;
        let record = keyring
            .get_key(&fingerprint)
            .map_err(|e| format!("Failed to look up key: {e}"))?
            .ok_or_else(|| format!("Key not found: {fingerprint}"))?;
        record.pgp_data.clone()
    };

    keychainpgp_keys::network::keyserver::keyserver_upload(&key_data, &url, proxy.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Test a proxy connection by making a simple HTTPS request through it.
#[tauri::command]
pub async fn test_proxy_connection(proxy_url: String) -> Result<String, String> {
    let proxy = reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy URL: {e}"))?;
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create client: {e}"))?;

    client
        .get("https://keys.openpgp.org")
        .send()
        .await
        .map_err(|e| format!("Connection failed: {e}"))?;

    Ok("Proxy connection successful.".into())
}

/// Result of importing an OpenKeychain backup.
#[derive(Debug, Clone, Serialize)]
pub struct BackupImportResult {
    pub imported_count: usize,
    pub keys: Vec<KeyInfo>,
    pub skipped_count: usize,
}

/// Import keys from an OpenKeychain backup file.
///
/// The backup is a symmetrically-encrypted PGP message containing one or more
/// transferable secret keys. The `transfer_code` is a numeric passphrase
/// (typically 9 groups of 4 digits) provided by OpenKeychain during export.
#[tauri::command]
pub fn import_backup(
    state: State<'_, AppState>,
    backup_data: String,
    transfer_code: String,
) -> Result<BackupImportResult, String> {
    // Decrypt the SKESK-encrypted message (engine tries multiple password formats)
    let decrypted = state
        .engine
        .decrypt_skesk(backup_data.as_bytes(), &transfer_code)
        .map_err(|e| format!("Failed to decrypt backup: {e}"))?;

    // Parse decrypted bytes into individual certificates
    let cert_entries = state
        .engine
        .parse_backup_certs(&decrypted)
        .map_err(|e| format!("Failed to parse keys from backup: {e}"))?;

    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    let mut imported_keys: Vec<KeyInfo> = Vec::new();
    let mut skipped = 0;

    for (public_bytes, secret_bytes, cert_info) in cert_entries {
        let fingerprint = cert_info.fingerprint.0.clone();
        let is_own = cert_info.has_secret_key;

        // Check if key already exists in keyring
        if let Ok(Some(existing)) = keyring.get_key(&fingerprint) {
            // Upgrade: if existing is public-only but this cert has secret material,
            // replace it (OpenKeychain backups have PUBLIC block then PRIVATE block
            // for the same key — CertParser may emit them as separate certs).
            if is_own && !existing.is_own_key {
                let record = KeyRecord {
                    fingerprint,
                    name: cert_info.name().map(String::from).or(existing.name),
                    email: cert_info.email().map(String::from).or(existing.email),
                    algorithm: cert_info.algorithm.to_string(),
                    created_at: cert_info.created_at,
                    expires_at: cert_info.expires_at,
                    trust_level: 2,
                    is_own_key: true,
                    pgp_data: public_bytes,
                };
                // Delete the public-only record and re-store with secret material
                let _ = keyring.delete_key(&record.fingerprint);
                keyring
                    .store_generated_key(record.clone(), &secret_bytes)
                    .map_err(|e| format!("Failed to upgrade key: {e}"))?;
                // Update the previously-added KeyInfo in imported_keys
                if let Some(prev) = imported_keys
                    .iter_mut()
                    .find(|k: &&mut KeyInfo| k.fingerprint == record.fingerprint)
                {
                    *prev = KeyInfo::from(record);
                }
            } else {
                skipped += 1;
            }
            continue;
        }

        let record = KeyRecord {
            fingerprint,
            name: cert_info.name().map(String::from),
            email: cert_info.email().map(String::from),
            algorithm: cert_info.algorithm.to_string(),
            created_at: cert_info.created_at,
            expires_at: cert_info.expires_at,
            trust_level: if is_own { 2 } else { 1 },
            is_own_key: is_own,
            pgp_data: public_bytes,
        };

        if is_own {
            keyring
                .store_generated_key(record.clone(), &secret_bytes)
                .map_err(|e| format!("Failed to store key: {e}"))?;
        } else {
            keyring
                .import_public_key(record.clone())
                .map_err(|e| format!("Failed to import key: {e}"))?;
        }

        imported_keys.push(KeyInfo::from(record));
    }

    Ok(BackupImportResult {
        imported_count: imported_keys.len(),
        keys: imported_keys,
        skipped_count: skipped,
    })
}
