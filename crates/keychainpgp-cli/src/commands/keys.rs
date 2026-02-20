use std::path::Path;

use anyhow::{Context, Result};
use keychainpgp_core::{CryptoEngine, SequoiaEngine};
use keychainpgp_keys::storage::KeyRecord;
use keychainpgp_keys::Keyring;

pub fn list() -> Result<()> {
    let keyring = Keyring::open_default()?;
    let keys = keyring.list_keys()?;

    if keys.is_empty() {
        eprintln!("No keys in keyring. Use 'keychainpgp generate' to create one.");
        return Ok(());
    }

    for key in &keys {
        print_key_summary(key);
        println!();
    }

    eprintln!("{} key(s) in keyring.", keys.len());
    Ok(())
}

pub fn import(file: &Path) -> Result<()> {
    let data = std::fs::read(file).with_context(|| format!("failed to read {}", file.display()))?;

    let engine = SequoiaEngine::new();
    let info = engine.inspect_key(&data)
        .with_context(|| format!("failed to parse key from {}", file.display()))?;

    let name = info.name().map(String::from);
    let email = info.email().map(String::from);
    let display = match (&name, &email) {
        (Some(n), Some(e)) => format!("{n} <{e}>"),
        (Some(n), None) => n.clone(),
        (None, Some(e)) => e.clone(),
        (None, None) => info.fingerprint.0.clone(),
    };

    if info.has_secret_key {
        let keyring = Keyring::open_default()?;
        let record = KeyRecord {
            fingerprint: info.fingerprint.0.clone(),
            name,
            email,
            algorithm: info.algorithm.to_string(),
            created_at: info.created_at.clone(),
            expires_at: info.expires_at.clone(),
            trust_level: 2, // own key = verified
            is_own_key: true,
            pgp_data: data.clone(),
        };
        keyring.store_generated_key(record, &data)?;
        eprintln!("Secret key imported: {display}");
    } else {
        let keyring = Keyring::open_default()?;
        let record = KeyRecord {
            fingerprint: info.fingerprint.0.clone(),
            name,
            email,
            algorithm: info.algorithm.to_string(),
            created_at: info.created_at.clone(),
            expires_at: info.expires_at.clone(),
            trust_level: 1, // imported = unverified
            is_own_key: false,
            pgp_data: data,
        };
        keyring.import_public_key(record)?;
        eprintln!("Public key imported: {display}");
    }

    eprintln!("Fingerprint: {}", info.fingerprint);
    Ok(())
}

pub fn export(fingerprint: &str) -> Result<()> {
    let keyring = Keyring::open_default()?;
    let record = keyring
        .get_key(fingerprint)?
        .with_context(|| format!("key not found: {fingerprint}"))?;

    print!("{}", String::from_utf8_lossy(&record.pgp_data));
    Ok(())
}

pub fn delete(fingerprint: &str) -> Result<()> {
    let keyring = Keyring::open_default()?;
    if keyring.delete_key(fingerprint)? {
        eprintln!("Key deleted: {fingerprint}");
    } else {
        eprintln!("Key not found: {fingerprint}");
    }
    Ok(())
}

pub fn search(query: &str) -> Result<()> {
    let keyring = Keyring::open_default()?;
    let results = keyring.search_keys(query)?;

    if results.is_empty() {
        eprintln!("No keys match '{query}'.");
        return Ok(());
    }

    for key in &results {
        print_key_summary(key);
        println!();
    }

    Ok(())
}

fn print_key_summary(key: &KeyRecord) {
    let tag = if key.is_own_key { "sec" } else { "pub" };
    let name = key.name.as_deref().unwrap_or("(no name)");
    let email = key
        .email
        .as_deref()
        .map(|e| format!(" <{e}>"))
        .unwrap_or_default();
    let trust = match key.trust_level {
        0 => "[unknown]",
        1 => "[unverified]",
        2 => "[verified]",
        _ => "[?]",
    };
    let expires = key
        .expires_at
        .as_deref()
        .map(|e| format!("  expires {}", format_date(e)))
        .unwrap_or_default();

    println!("{tag}   {:<12} {trust}", key.algorithm);
    println!("      {}", key.fingerprint);
    println!("      {name}{email}{expires}");
}

/// Format an ISO 8601 date to just the date portion for display.
fn format_date(iso: &str) -> &str {
    // "2026-02-20T00:00:00+00:00" -> "2026-02-20"
    iso.split('T').next().unwrap_or(iso)
}
