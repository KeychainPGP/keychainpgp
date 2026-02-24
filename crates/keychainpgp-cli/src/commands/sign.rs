use std::io::{self, Read, Write};

use anyhow::{Context, Result};
use keychainpgp_core::{CryptoEngine, SequoiaEngine};
use keychainpgp_keys::Keyring;
use secrecy::ExposeSecret;

pub fn run(key_fingerprint: Option<&str>, passphrase: Option<&str>) -> Result<()> {
    let engine = SequoiaEngine::new();
    let keyring = Keyring::open_default()?;

    // Find the signing key
    let secret_key = if let Some(fp) = key_fingerprint {
        keyring
            .get_secret_key(fp)
            .with_context(|| format!("could not retrieve secret key for {fp}"))?
    } else {
        // Use the first own key
        let own_keys = keyring
            .list_keys()?
            .into_iter()
            .filter(|k| k.is_own_key)
            .collect::<Vec<_>>();

        let first = own_keys
            .first()
            .context("no private keys in keyring; generate or import one first")?;

        keyring.get_secret_key(&first.fingerprint)?
    };

    // Read data from stdin (limit to 64 MB to prevent memory exhaustion)
    const MAX_INPUT: u64 = 64 * 1024 * 1024;
    let mut data = Vec::new();
    io::stdin()
        .take(MAX_INPUT)
        .read_to_end(&mut data)
        .context("failed to read from stdin")?;

    let passphrase_bytes = passphrase.map(|p| p.as_bytes());

    let signed = engine.sign(&data, secret_key.expose_secret(), passphrase_bytes)?;

    io::stdout()
        .write_all(&signed)
        .context("failed to write to stdout")?;

    Ok(())
}
