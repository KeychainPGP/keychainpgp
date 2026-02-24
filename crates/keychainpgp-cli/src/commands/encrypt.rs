use std::io::{self, Read, Write};

use anyhow::{Context, Result};
use keychainpgp_core::{CryptoEngine, SequoiaEngine};
use keychainpgp_keys::Keyring;

pub fn run(recipient_fingerprints: &[String]) -> Result<()> {
    let engine = SequoiaEngine::new();
    let keyring = Keyring::open_default()?;

    // Look up recipient public keys (by fingerprint or name/email search)
    let mut recipient_keys = Vec::new();
    for query in recipient_fingerprints {
        // Try exact fingerprint first
        if let Some(record) = keyring.get_key(query)? {
            recipient_keys.push(record.pgp_data);
            continue;
        }

        // Fall back to search
        let results = keyring.search_keys(query)?;
        match results.len() {
            0 => anyhow::bail!("no key found matching '{query}'"),
            1 => recipient_keys.push(results.into_iter().next().unwrap().pgp_data),
            n => {
                eprintln!("Ambiguous recipient '{query}' matched {n} keys:");
                for r in &results {
                    let name = r.name.as_deref().unwrap_or("(no name)");
                    let email = r
                        .email
                        .as_deref()
                        .map(|e| format!(" <{e}>"))
                        .unwrap_or_default();
                    eprintln!("  {} {name}{email}", &r.fingerprint[..16]);
                }
                anyhow::bail!("specify a more precise recipient (use full fingerprint)");
            }
        }
    }

    // Read plaintext from stdin (limit to 64 MB to prevent memory exhaustion)
    const MAX_INPUT: u64 = 64 * 1024 * 1024;
    let mut plaintext = Vec::new();
    io::stdin()
        .take(MAX_INPUT)
        .read_to_end(&mut plaintext)
        .context("failed to read from stdin")?;

    if plaintext.is_empty() {
        anyhow::bail!("no input data (stdin was empty)");
    }

    // Encrypt
    let ciphertext = engine.encrypt(&plaintext, &recipient_keys)?;

    // Write to stdout
    io::stdout()
        .write_all(&ciphertext)
        .context("failed to write to stdout")?;

    eprintln!(
        "Encrypted for {} recipient(s).",
        recipient_fingerprints.len()
    );

    Ok(())
}
