use anyhow::Result;
use keychainpgp_core::types::{KeyGenOptions, UserId};
use keychainpgp_core::{CryptoEngine, SequoiaEngine};
use keychainpgp_keys::Keyring;
use keychainpgp_keys::storage::KeyRecord;
use secrecy::{ExposeSecret, SecretBox};

pub fn run(name: &str, email: &str, passphrase: Option<&str>) -> Result<()> {
    let engine = SequoiaEngine::new();
    let user_id = UserId::new(name, email);
    let mut options = KeyGenOptions::new(user_id);

    if let Some(pass) = passphrase {
        options = options.with_passphrase(SecretBox::new(Box::new(pass.as_bytes().to_vec())));
    }

    eprintln!("Generating key pair for {name} <{email}>...");
    let key_pair = engine.generate_key_pair(options)?;

    // Extract proper metadata from the generated key
    let info = engine.inspect_key(&key_pair.public_key)?;

    let keyring = Keyring::open_default()?;
    let record = KeyRecord {
        fingerprint: key_pair.fingerprint.0.clone(),
        name: Some(name.to_string()),
        email: Some(email.to_string()),
        algorithm: info.algorithm.to_string(),
        created_at: info.created_at,
        expires_at: info.expires_at,
        trust_level: 2,
        is_own_key: true,
        pgp_data: key_pair.public_key.clone(),
    };

    keyring.store_generated_key(record, key_pair.secret_key.expose_secret())?;

    // Store revocation certificate alongside the key
    if !key_pair.revocation_cert.is_empty() {
        if let Err(e) =
            keyring.store_revocation_cert(&key_pair.fingerprint.0, &key_pair.revocation_cert)
        {
            eprintln!("Warning: failed to store revocation certificate: {e}");
        }
    }

    eprintln!("Key generated successfully!");
    eprintln!("Fingerprint: {}", key_pair.fingerprint);
    eprintln!("Algorithm:   {}", info.algorithm);

    Ok(())
}
