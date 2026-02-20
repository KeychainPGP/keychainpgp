use std::io::{self, Read, Write};

use anyhow::{Context, Result};
use keychainpgp_core::{CryptoEngine, SequoiaEngine};
use keychainpgp_keys::Keyring;

pub fn run(recipient_fingerprints: &[String]) -> Result<()> {
    let engine = SequoiaEngine::new();
    let keyring = Keyring::open_default()?;

    // Look up recipient public keys
    let mut recipient_keys = Vec::new();
    for fp in recipient_fingerprints {
        let record = keyring
            .get_key(fp)?
            .with_context(|| format!("key not found: {fp}"))?;
        recipient_keys.push(record.pgp_data);
    }

    // Read plaintext from stdin
    let mut plaintext = Vec::new();
    io::stdin()
        .read_to_end(&mut plaintext)
        .context("failed to read from stdin")?;

    // Encrypt
    let ciphertext = engine.encrypt(&plaintext, &recipient_keys)?;

    // Write to stdout
    io::stdout()
        .write_all(&ciphertext)
        .context("failed to write to stdout")?;

    Ok(())
}
