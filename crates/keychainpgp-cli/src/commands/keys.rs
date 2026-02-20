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
        let key_type = if key.is_own_key { "[own]  " } else { "[contact]" };
        let name = key.name.as_deref().unwrap_or("(no name)");
        let email = key
            .email
            .as_deref()
            .map(|e| format!(" <{e}>"))
            .unwrap_or_default();
        let expires = key
            .expires_at
            .as_deref()
            .map(|e| format!("  expires: {e}"))
            .unwrap_or_default();

        println!(
            "{key_type} {name}{email}\n         {}{expires}",
            &key.fingerprint
        );
    }

    eprintln!("\n{} key(s) in keyring.", keys.len());
    Ok(())
}

pub fn import(file: &Path) -> Result<()> {
    let data = std::fs::read(file).with_context(|| format!("failed to read {}", file.display()))?;

    let engine = SequoiaEngine::new();
    let fingerprint = engine.key_fingerprint(&data)?;

    let keyring = Keyring::open_default()?;
    let record = KeyRecord {
        fingerprint: fingerprint.clone(),
        name: None,
        email: None,
        algorithm: "Unknown".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        expires_at: None,
        trust_level: 1,
        is_own_key: false,
        pgp_data: data,
    };

    keyring.import_public_key(record)?;
    eprintln!("Key imported: {fingerprint}");
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
        let name = key.name.as_deref().unwrap_or("(no name)");
        let email = key
            .email
            .as_deref()
            .map(|e| format!(" <{e}>"))
            .unwrap_or_default();
        println!("{name}{email}\n  {}", &key.fingerprint);
    }

    Ok(())
}
