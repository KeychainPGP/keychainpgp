//! Key sync commands for transferring keys between devices.

use serde::Serialize;
use tauri::State;

use keychainpgp_core::CryptoEngine;
use secrecy::ExposeSecret;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SyncBundle {
    /// Passphrase to share with the receiving device.
    pub passphrase: String,
    /// QR code parts (each is an SVG string).
    pub qr_parts: Vec<String>,
    /// The entire encrypted bundle as base64 for file export.
    pub file_data: String,
}

/// Export all own keys as an encrypted bundle for sync.
#[tauri::command]
pub fn export_key_bundle(
    state: State<'_, AppState>,
    qr_part_size: Option<u32>,
) -> Result<SyncBundle, String> {
    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    let all_keys = keyring
        .list_keys()
        .map_err(|e| format!("Failed to list keys: {e}"))?;

    // Build bundle of all keys (own keys with secret material + contact public keys)
    let mut entries = Vec::new();
    for key in &all_keys {
        let secret_key = if key.is_own_key && keyring.has_secret_key(&key.fingerprint) {
            keyring
                .get_secret_key(&key.fingerprint)
                .ok()
                .map(|s| s.expose_secret().clone())
        } else {
            None
        };

        entries.push(keychainpgp_keys::sync::KeyBundleEntry {
            fingerprint: key.fingerprint.clone(),
            public_key: key.pgp_data.clone(),
            secret_key,
            trust_level: key.trust_level,
        });
    }

    if entries.is_empty() {
        return Err("No keys to export. Generate or import a key first.".into());
    }

    let bundle = keychainpgp_keys::sync::KeyBundle {
        version: 2,
        keys: entries,
    };

    // Generate random passphrase
    let passphrase = keychainpgp_keys::sync::generate_sync_passphrase();

    // Serialize, compress, and encrypt with passphrase (SKESK)
    let bundle_json =
        serde_json::to_vec(&bundle).map_err(|e| format!("Serialization failed: {e}"))?;

    let compressed = keychainpgp_keys::sync::compress(&bundle_json)
        .map_err(|e| format!("Compression failed: {e}"))?;

    let encrypted = state
        .engine
        .encrypt_symmetric(&compressed, passphrase.as_bytes())
        .map_err(|e| format!("Encryption failed: {e}"))?;

    // Split into QR-sized parts with fountain parity codes
    let part_size = qr_part_size.unwrap_or(200) as usize;
    let parts = keychainpgp_keys::sync::split_for_qr_with_size(&encrypted, part_size);

    // First QR code: passphrase (so the scanner can auto-fill it)
    let pass_qr_data = format!("KCPGP-PASS:{passphrase}");
    let pass_qr = qrcode::QrCode::new(pass_qr_data.as_bytes())
        .map_err(|e| format!("QR generation failed: {e}"))?;
    let pass_svg = pass_qr
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(256, 256)
        .build();

    let data_qr_parts: Result<Vec<String>, String> = parts
        .iter()
        .map(|data| {
            let code = qrcode::QrCode::new(data.as_bytes())
                .map_err(|e| format!("QR generation failed: {e}"))?;
            Ok(code
                .render::<qrcode::render::svg::Color>()
                .min_dimensions(256, 256)
                .build())
        })
        .collect();

    // Prepend passphrase QR, then data QRs (including fountain parity)
    let mut qr_parts = vec![pass_svg];
    qr_parts.extend(data_qr_parts?);

    let file_data = keychainpgp_keys::sync::base64_encode(&encrypted);

    Ok(SyncBundle {
        passphrase,
        qr_parts,
        file_data,
    })
}

/// Import a key bundle (from QR scan sequence or file).
#[tauri::command]
pub fn import_key_bundle(
    state: State<'_, AppState>,
    encrypted_data: String,
    passphrase: String,
) -> Result<usize, String> {
    let encrypted = keychainpgp_keys::sync::base64_decode(&encrypted_data)
        .map_err(|e| format!("Invalid data: {e}"))?;

    let decrypted = state
        .engine
        .decrypt_skesk(&encrypted, &passphrase)
        .map_err(|e| format!("Decryption failed — check the passphrase: {e}"))?;

    // Decompress if compressed (v2), or use raw JSON (v1 backward compat)
    let json_data = keychainpgp_keys::sync::decompress_or_raw(&decrypted)
        .map_err(|e| format!("Decompression failed: {e}"))?;

    let bundle: keychainpgp_keys::sync::KeyBundle =
        serde_json::from_slice(&json_data).map_err(|e| format!("Invalid bundle format: {e}"))?;

    if bundle.version > 2 {
        return Err(format!(
            "Unsupported bundle version: {}. Please update KeychainPGP.",
            bundle.version
        ));
    }

    let keyring = state
        .keyring
        .lock()
        .map_err(|e| format!("Internal error: {e}"))?;

    let mut imported = 0;
    for entry in &bundle.keys {
        // Check if key already exists
        let existing = keyring.get_key(&entry.fingerprint).ok().flatten();

        if let Some(ref existing_key) = existing {
            if existing_key.is_own_key {
                // Already have this key with secret material — skip
                continue;
            }
            if entry.secret_key.is_some() {
                // Upgrade: existing public-only -> own key with secret material
                let _ = keyring.delete_key(&existing_key.fingerprint);
            } else {
                // Already have this public key — skip
                continue;
            }
        }

        // Import using the engine to inspect and validate
        let cert_info = state
            .engine
            .inspect_key(&entry.public_key)
            .map_err(|e| format!("Invalid key in bundle: {e}"))?;

        let record = keychainpgp_keys::KeyRecord {
            fingerprint: cert_info.fingerprint.0,
            name: cert_info.user_ids.first().and_then(|u| u.name.clone()),
            email: cert_info.user_ids.first().and_then(|u| u.email.clone()),
            algorithm: format!("{}", cert_info.algorithm),
            created_at: cert_info.created_at,
            expires_at: cert_info.expires_at,
            trust_level: entry.trust_level,
            is_own_key: entry.secret_key.is_some(),
            is_revoked: cert_info.is_revoked,
            pgp_data: entry.public_key.clone(),
        };

        if let Some(ref secret_key) = entry.secret_key {
            keyring
                .store_generated_key(record, secret_key)
                .map_err(|e| format!("Failed to import key: {e}"))?;
        } else {
            keyring
                .import_public_key(record)
                .map_err(|e| format!("Failed to import key: {e}"))?;
        }

        imported += 1;
    }

    Ok(imported)
}

/// Save sync bundle data to a file chosen by the user.
#[tauri::command]
pub fn save_sync_file(path: String, data: String) -> Result<(), String> {
    std::fs::write(&path, data.as_bytes()).map_err(|e| format!("Failed to save file: {e}"))
}
