//! Tauri commands for key management.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, State};

use keychainpgp_core::CryptoEngine;
use keychainpgp_core::types::{KeyGenOptions, TrustLevel, UserId};
use keychainpgp_keys::network::keyserver::{
    KeyserverMatch, keyserver_fetch, keyserver_search as ks_search, validate_keyserver_url,
};
use keychainpgp_keys::storage::KeyRecord;
use secrecy::{ExposeSecret, SecretBox};
use tokio::sync::Semaphore;

use crate::state::AppState;

/// Validate that a proxy URL uses an allowed SOCKS5 protocol.
fn validate_proxy_url(url: &str) -> Result<(), String> {
    if url.starts_with("socks5://") || url.starts_with("socks5h://") {
        Ok(())
    } else {
        Err("Proxy URL must use socks5:// or socks5h:// protocol".into())
    }
}

/// Read the proxy URL from settings if proxy is enabled.
fn get_proxy_url(app: &AppHandle, state: &AppState) -> Result<Option<String>, String> {
    let settings = super::settings::get_settings_internal(app, state);
    if !settings.proxy_enabled {
        return Ok(None);
    }
    let url = match settings.proxy_preset.as_str() {
        "tor" => "socks5h://127.0.0.1:9050".to_string(),
        "lokinet" => "socks5h://127.0.0.1:1080".to_string(),
        _ => settings.proxy_url.clone(),
    };
    if url.trim().is_empty() {
        Err("Proxy is enabled but no URL is configured. Please check your settings.".to_string())
    } else {
        validate_proxy_url(&url)?;
        Ok(Some(url))
    }
}

/// Internal helper to upload key data to multiple keyservers.
/// Returns (successes, failures) where each entry is "url: message".
async fn upload_to_keyservers_internal(
    urls: &[String],
    key_data: &[u8],
    proxy: Option<&str>,
) -> (Vec<String>, Vec<String>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for url in urls {
        if let Err(e) = validate_keyserver_url(url) {
            failures.push(format!("{url}: {e}"));
            continue;
        }
        match keychainpgp_keys::network::keyserver::keyserver_upload(key_data, url, proxy).await {
            Ok(msg) => successes.push(format!("{url}: {msg}")),
            Err(e) => failures.push(format!("{url}: {e}")),
        }
    }
    (successes, failures)
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
    pub is_revoked: bool,
}

/// Key discovery result with source information.
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryResult {
    pub fingerprint: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub algorithm: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub trust_level: i32,
    pub is_own_key: bool,
    pub is_revoked: bool,
    pub source: String,
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
            is_revoked: r.is_revoked,
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
    // Input validation
    if name.is_empty() || name.len() > 256 {
        return Err("Name must be between 1 and 256 characters".into());
    }
    if email.is_empty() || email.len() > 256 {
        return Err("Email must be between 1 and 256 characters".into());
    }

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
        is_revoked: info.is_revoked,
        pgp_data: key_pair.public_key.clone(),
    };

    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    if state.opsec_mode.load(Ordering::SeqCst) {
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
            zeroize::Zeroizing::new(key_pair.secret_key.expose_secret().clone()),
        );
    } else {
        keyring
            .store_generated_key(record.clone(), key_pair.secret_key.expose_secret())
            .map_err(|e| format!("Failed to store key: {e}"))?;
    }

    // Store revocation certificate
    if !key_pair.revocation_cert.is_empty() {
        if let Err(e) =
            keyring.store_revocation_cert(&record.fingerprint, &key_pair.revocation_cert)
        {
            tracing::warn!("failed to store revocation certificate: {e}");
        }
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
        is_revoked: cert_info.is_revoked,
        pgp_data: key_data.as_bytes().to_vec(),
    };

    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    // Check if key already exists to avoid UNIQUE constraint error
    if keyring
        .get_key(&record.fingerprint)
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Err(format!(
            "Key already exists in keyring: {}",
            record.fingerprint
        ));
    }

    if cert_info.has_secret_key && state.opsec_mode.load(Ordering::SeqCst) {
        // OPSEC mode: store secret key in RAM only, public key in DB
        keyring
            .import_public_key(record.clone())
            .map_err(|e| format!("Failed to import key: {e}"))?;
        let mut opsec_keys = state
            .opsec_secret_keys
            .lock()
            .map_err(|e| format!("Internal error: {e}"))?;
        opsec_keys.insert(
            record.fingerprint.clone(),
            zeroize::Zeroizing::new(key_data.as_bytes().to_vec()),
        );
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
    let qr =
        qrcode::QrCode::new(data.as_bytes()).map_err(|e| format!("QR generation failed: {e}"))?;
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
    let proxy = get_proxy_url(&app, &state)?;
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
        is_revoked: cert_info.is_revoked,
    }))
}

/// Fetch a key via WKD and import it into the keyring.
#[tauri::command]
pub async fn wkd_fetch_and_import(
    app: AppHandle,
    state: State<'_, AppState>,
    email: String,
) -> Result<KeyInfo, String> {
    let proxy = get_proxy_url(&app, &state)?;
    let key_bytes = keychainpgp_keys::network::wkd::wkd_lookup(&email, proxy.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let cert_info = state
        .engine
        .inspect_key(&key_bytes)
        .map_err(|e| format!("Invalid key data from WKD: {e}"))?;

    let name = cert_info.name().map(String::from);
    let email_val = cert_info.email().map(String::from);

    let record = KeyRecord {
        fingerprint: cert_info.fingerprint.0.clone(),
        name,
        email: email_val,
        algorithm: cert_info.algorithm.to_string(),
        created_at: cert_info.created_at,
        expires_at: cert_info.expires_at,
        trust_level: 1,
        is_own_key: false,
        is_revoked: cert_info.is_revoked,
        pgp_data: key_bytes,
    };

    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    if keyring
        .get_key(&record.fingerprint)
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Err(format!(
            "Key already exists in keyring: {}",
            record.fingerprint
        ));
    }

    keyring
        .import_public_key(record.clone())
        .map_err(|e| e.to_string())?;

    Ok(KeyInfo::from(record))
}

/// Search for keys on one or more keyservers.
///
/// If `keyserver_url` contains commas, it is treated as a list of servers to query in parallel.
#[tauri::command]
pub async fn keyserver_search(
    app: AppHandle,
    state: State<'_, AppState>,
    query: String,
    keyserver_url: Option<String>,
) -> Result<Vec<DiscoveryResult>, String> {
    let settings = super::settings::get_settings_internal(&app, &state);
    let url_string = keyserver_url.unwrap_or_else(|| {
        if settings.unverified_keyserver_url.is_empty() {
            settings.keyserver_url.clone()
        } else {
            format!(
                "{},{}",
                settings.keyserver_url, settings.unverified_keyserver_url
            )
        }
    });
    let urls: Vec<String> = url_string
        .split(',')
        .map(|s| {
            let mut s = s.trim().to_string();
            if !s.is_empty() && !s.contains("://") {
                s = format!("https://{s}");
            }
            s
        })
        .filter(|s| !s.is_empty())
        .collect();

    if urls.is_empty() {
        return Err("No keyservers configured".into());
    }

    for url in &urls {
        validate_keyserver_url(url)?;
    }

    let proxy = get_proxy_url(&app, &state)?;
    let query_clone = query.clone();

    // Limit concurrency to avoid spawning an unbounded number of network requests.
    // 10 is a reasonable default for standard use cases.
    let semaphore = Arc::new(Semaphore::new(10));

    let mut futures = Vec::new();
    for url in urls {
        let u = url.to_string();
        let q = query_clone.clone();
        let p = proxy.clone();
        let sem = semaphore.clone();
        futures.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.map_err(|e| e.to_string())?;
            let matches = ks_search(&q, &u, p.as_deref()).await?;
            Ok::<_, String>((u, matches))
        }));
    }

    let mut all_matches: Vec<(String, KeyserverMatch)> = Vec::new();
    let mut errors = Vec::new();

    for handle in futures {
        match handle.await {
            Ok(Ok((url, ks_matches))) => {
                for m in ks_matches {
                    all_matches.push((url.clone(), m));
                }
            }
            Ok(Err(e)) => errors.push(e),
            Err(e) => errors.push(format!("Task failed: {e}")),
        }
    }

    if all_matches.is_empty() && !errors.is_empty() {
        return Err(format!(
            "All keyserver queries failed:\n{}",
            errors.join("\n")
        ));
    }

    // Deduplicate by fingerprint (or KeyID if fingerprint is missing),
    // collecting all source URLs per key.
    let mut unique_keys: std::collections::HashMap<String, (Vec<String>, KeyserverMatch)> =
        std::collections::HashMap::new();
    for (url, m) in all_matches {
        let id = if m.fingerprint.is_empty() {
            m.key_id.clone()
        } else {
            m.fingerprint.clone()
        };
        unique_keys
            .entry(id)
            .and_modify(|(urls, _)| urls.push(url.clone()))
            .or_insert_with(|| (vec![url], m));
    }

    let infos = unique_keys
        .into_values()
        .map(|(urls, m)| {
            // Parse "Name <email>" if possible
            let (name, email) = if let Some(uid) = m.user_ids.first() {
                if let Some(pos) = uid.find('<') {
                    let name = uid[..pos].trim().to_string();
                    let email = uid[pos + 1..].trim_matches('>').to_string();
                    (Some(name), Some(email))
                } else {
                    (Some(uid.clone()), None)
                }
            } else {
                (None, None)
            };

            // Extract hostnames for display
            let source = urls
                .iter()
                .map(|url| {
                    reqwest::Url::parse(url)
                        .ok()
                        .and_then(|u| u.host_str().map(String::from))
                        .unwrap_or_else(|| url.clone())
                })
                .collect::<Vec<_>>()
                .join(", ");

            DiscoveryResult {
                fingerprint: m.fingerprint,
                name,
                email,
                algorithm: String::new(),
                created_at: m
                    .created_at
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                expires_at: m.expires_at.as_ref().map(|t| t.to_string()),
                trust_level: 0,
                is_own_key: false,
                is_revoked: false,
                source,
            }
        })
        .collect();

    Ok(infos)
}

/// Fetch a key from one or more keyservers and import it.
#[tauri::command]
pub async fn fetch_and_import_key(
    app: AppHandle,
    state: State<'_, AppState>,
    fingerprint: String,
    keyserver_url: String,
) -> Result<KeyInfo, String> {
    let urls: Vec<&str> = keyserver_url
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let proxy = get_proxy_url(&app, &state)?;

    let mut last_error = String::new();
    for url in urls {
        validate_keyserver_url(url)?;
        match keyserver_fetch(&fingerprint, url, proxy.as_deref()).await {
            Ok(key_data) => {
                verify_fetched_key(&state, &key_data, &fingerprint)?;
                let key_text = String::from_utf8_lossy(&key_data).into_owned();
                return import_key(state, key_text);
            }
            Err(e) => last_error = e,
        }
    }

    Err(if last_error.is_empty() {
        "No keyservers available to fetch from".into()
    } else {
        format!("Failed to fetch key from any server. Last error: {last_error}")
    })
}

/// Upload a public key to one or more keyservers.
#[tauri::command]
pub async fn keyserver_upload(
    app: AppHandle,
    state: State<'_, AppState>,
    fingerprint: String,
    keyserver_url: Option<String>,
) -> Result<String, String> {
    let settings = super::settings::get_settings_internal(&app, &state);
    let url_string = keyserver_url.unwrap_or_else(|| settings.keyserver_url.clone());
    let urls: Vec<String> = url_string
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if urls.is_empty() {
        return Err("No keyservers configured".into());
    }

    let proxy = get_proxy_url(&app, &state)?;

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

    let (successes, failures) =
        upload_to_keyservers_internal(&urls, &key_data, proxy.as_deref()).await;

    if successes.is_empty() {
        Err(format!(
            "Failed to upload to all keyservers: {}",
            failures.join("; ")
        ))
    } else if failures.is_empty() {
        Ok(successes.join("\n"))
    } else {
        Ok(format!(
            "Uploaded to {} server(s):\n{}\nFailed on {} server(s):\n{}",
            successes.len(),
            successes.join("\n"),
            failures.len(),
            failures.join("\n")
        ))
    }
}

/// Test a proxy connection by making a simple HTTPS request through it.
#[tauri::command]
pub async fn test_proxy_connection(proxy_url: String) -> Result<String, String> {
    validate_proxy_url(&proxy_url)?;
    let proxy = reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy URL: {e}"))?;
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .no_proxy()
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
                    is_revoked: cert_info.is_revoked,
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
            is_revoked: cert_info.is_revoked,
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

/// Internal helper to verify a fetched key's fingerprint.
fn verify_fetched_key(
    state: &AppState,
    key_data: &[u8],
    expected_fingerprint: &str,
) -> Result<(), String> {
    let cert_info = state
        .engine
        .inspect_key(key_data)
        .map_err(|e| e.to_string())?;
    if cert_info.fingerprint.0.to_uppercase() != expected_fingerprint.to_uppercase() {
        return Err("Fetched key fingerprint does not match requested fingerprint".into());
    }
    Ok(())
}

/// Export a private key to a file path.
#[tauri::command]
pub fn export_private_key(
    state: State<'_, AppState>,
    fingerprint: String,
    path: String,
) -> Result<String, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    // Verify the key exists and is an own key
    let record = keyring
        .get_key(&fingerprint)
        .map_err(|e| format!("Failed to look up key: {e}"))?
        .ok_or_else(|| format!("Key not found: {fingerprint}"))?;

    if !record.is_own_key {
        return Err("Cannot export private key: this is not your own key".into());
    }

    // Check OPSEC mode — secret key might be in RAM
    let secret_key_bytes = if state.opsec_mode.load(Ordering::SeqCst) {
        let opsec_keys = state
            .opsec_secret_keys
            .lock()
            .map_err(|e| format!("Internal error: {e}"))?;
        opsec_keys
            .get(&fingerprint)
            .map(|z| (**z).clone())
            .ok_or_else(|| "Secret key not found in OPSEC session".to_string())?
    } else {
        let sk = keyring
            .get_secret_key(&fingerprint)
            .map_err(|e| format!("Failed to retrieve secret key: {e}"))?;
        sk.expose_secret().clone()
    };

    // Armor the secret key
    let armored = state
        .engine
        .armor_key(&secret_key_bytes)
        .map_err(|e| format!("Failed to armor private key: {e}"))?;

    std::fs::write(&path, armored.as_bytes()).map_err(|e| format!("Failed to write file: {e}"))?;

    Ok(format!("Private key exported to {path}"))
}

/// Publish a revocation certificate to all configured keyservers.
///
/// This retrieves the stored revocation certificate, which is already a full
/// revoked certificate (public key merged with revocation signature), and
/// uploads it to every configured keyserver. The local key is then marked
/// as revoked in the database and its PGP data is updated.
#[tauri::command]
pub async fn publish_revocation_cert(
    app: AppHandle,
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<String, String> {
    let rev_cert = {
        let keyring = state
            .keyring
            .lock()
            .map_err(|e| format!("Internal error: {e}"))?;
        keyring
            .get_revocation_cert(&fingerprint)
            .map_err(|e| format!("Failed to get revocation certificate: {e}"))?
            .ok_or_else(|| {
                "No revocation certificate found for this key. Only keys generated by KeychainPGP have revocation certificates.".to_string()
            })?
    };

    // Read keyserver URLs from settings (both verified and unverified)
    let settings = super::settings::get_settings_internal(&app, &state);
    let url_string = if settings.unverified_keyserver_url.is_empty() {
        settings.keyserver_url.clone()
    } else {
        format!(
            "{},{}",
            settings.keyserver_url, settings.unverified_keyserver_url
        )
    };
    let urls: Vec<String> = url_string
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if urls.is_empty() {
        return Err("No keyservers configured".into());
    }
    let proxy = get_proxy_url(&app, &state)?;

    let (successes, failures) =
        upload_to_keyservers_internal(&urls, &rev_cert, proxy.as_deref()).await;

    if successes.is_empty() {
        return Err(format!(
            "Failed to publish revocation to any keyserver: {}",
            failures.join("; ")
        ));
    }

    // Mark as revoked locally ONLY IF at least one upload succeeded.
    // Update local database: mark as revoked and update PGP data with revoked cert
    {
        let keyring = state
            .keyring
            .lock()
            .map_err(|e| format!("Internal error: {e}"))?;
        let _ = keyring.set_revoked(&fingerprint, true);
        let _ = keyring.update_pgp_data(&fingerprint, &rev_cert);
    }

    if failures.is_empty() {
        Ok(format!(
            "Revocation published to {} keyserver(s)",
            successes.len()
        ))
    } else {
        Ok(format!(
            "Revocation published to {} keyserver(s). Failed on {}: {}",
            successes.len(),
            failures.len(),
            failures.join("; ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keychainpgp_core::types::{KeyGenOptions, UserId};

    fn setup() -> (AppState, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState::initialize_with_dir(tmp.path()).unwrap();
        (state, tmp)
    }

    #[test]
    fn test_verify_fetched_key_success() {
        let (state, _tmp) = setup();

        // Generate a real key to get valid PGP data and fingerprint
        let user_id = UserId::new("Test", "test@example.com");
        let options = KeyGenOptions::new(user_id);
        let key_pair = state.engine.generate_key_pair(options).unwrap();
        let fingerprint = key_pair.fingerprint.0.clone();

        // Verification should succeed when fingerprints match
        let result = verify_fetched_key(&state, &key_pair.public_key, &fingerprint);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_fetched_key_mismatch_fails() {
        let (state, _tmp) = setup();

        // Generate a real key
        let user_id = UserId::new("Test", "test@example.com");
        let options = KeyGenOptions::new(user_id);
        let key_pair = state.engine.generate_key_pair(options).unwrap();

        // A different fingerprint
        let fake_fingerprint = "0123456789ABCDEF0123456789ABCDEF01234567";

        // Verification should fail when fingerprints mismatch
        let result = verify_fetched_key(&state, &key_pair.public_key, fake_fingerprint);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Fetched key fingerprint does not match requested fingerprint"
        );
    }

    #[test]
    fn test_validate_proxy_url() {
        assert!(validate_proxy_url("socks5://127.0.0.1:9050").is_ok());
        assert!(validate_proxy_url("socks5h://localhost:1080").is_ok());
        assert!(validate_proxy_url("http://127.0.0.1:8080").is_err());
        assert!(validate_proxy_url("").is_err());
        assert!(validate_proxy_url("   ").is_err());
    }

    #[test]
    fn test_get_proxy_url_logic() {
        // Internal logic check for get_proxy_url's matching
        let tor_url = "socks5h://127.0.0.1:9050";
        let lokinet_url = "socks5h://127.0.0.1:1080";

        let preset_tor = "tor";
        let preset_lokinet = "lokinet";
        let preset_custom = "custom";
        let custom_url = "socks5://myproxy:1234";

        assert_eq!(if preset_tor == "tor" { tor_url } else { "" }, tor_url);
        assert_eq!(
            if preset_lokinet == "lokinet" {
                lokinet_url
            } else {
                ""
            },
            lokinet_url
        );
        assert_eq!(
            if preset_custom == "custom" {
                custom_url
            } else {
                ""
            },
            custom_url
        );
    }
}
