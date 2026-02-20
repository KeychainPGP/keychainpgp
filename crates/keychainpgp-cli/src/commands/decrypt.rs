use std::io::{self, Read, Write};

use anyhow::{Context, Result};
use keychainpgp_core::{CryptoEngine, SequoiaEngine};
use keychainpgp_keys::Keyring;
use secrecy::ExposeSecret;

pub fn run(passphrase: Option<&str>) -> Result<()> {
    let engine = SequoiaEngine::new();
    let keyring = Keyring::open_default()?;

    // Read ciphertext from stdin
    let mut ciphertext = Vec::new();
    io::stdin()
        .read_to_end(&mut ciphertext)
        .context("failed to read from stdin")?;

    if ciphertext.is_empty() {
        anyhow::bail!("no input data (stdin was empty)");
    }

    // Find own keys and try each
    let own_keys = keyring
        .list_keys()?
        .into_iter()
        .filter(|k| k.is_own_key)
        .collect::<Vec<_>>();

    if own_keys.is_empty() {
        anyhow::bail!(
            "no private keys found in keyring; generate one with 'keychainpgp generate' first"
        );
    }

    let passphrase_bytes = passphrase.map(|p| p.as_bytes());

    for key_record in &own_keys {
        let secret_key = match keyring.get_secret_key(&key_record.fingerprint) {
            Ok(sk) => sk,
            Err(_) => continue,
        };

        match engine.decrypt(&ciphertext, secret_key.expose_secret(), passphrase_bytes) {
            Ok(plaintext) => {
                io::stdout()
                    .write_all(&plaintext)
                    .context("failed to write to stdout")?;
                return Ok(());
            }
            Err(_) => continue,
        }
    }

    anyhow::bail!(
        "decryption failed: none of the {} private key(s) in the keyring could decrypt this message",
        own_keys.len()
    )
}
