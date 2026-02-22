use std::io::{self, Read};

use anyhow::{Context, Result};
use keychainpgp_core::{CryptoEngine, SequoiaEngine};

pub fn run(file: &str) -> Result<()> {
    let data = if file == "-" {
        let mut buf = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .context("failed to read from stdin")?;
        buf
    } else {
        std::fs::read(file).with_context(|| format!("failed to read {file}"))?
    };

    if data.is_empty() {
        anyhow::bail!("no key data provided");
    }

    let engine = SequoiaEngine::new();
    let info = engine.inspect_key(&data).context("failed to parse key")?;

    let key_type = if info.has_secret_key {
        "Secret key (contains private material)"
    } else {
        "Public key"
    };

    println!("Type:        {key_type}");
    println!("Fingerprint: {}", info.fingerprint);
    println!("Algorithm:   {}", info.algorithm);
    println!("Created:     {}", format_date(&info.created_at));

    if let Some(ref exp) = info.expires_at {
        println!("Expires:     {}", format_date(exp));
    } else {
        println!("Expires:     never");
    }

    if info.user_ids.is_empty() {
        println!("User IDs:    (none)");
    } else {
        for (i, uid) in info.user_ids.iter().enumerate() {
            let prefix = if i == 0 {
                "User ID:    "
            } else {
                "            "
            };
            println!("{prefix} {uid}");
        }
    }

    Ok(())
}

/// Format an ISO 8601 date to just the date portion for display.
fn format_date(iso: &str) -> &str {
    iso.split('T').next().unwrap_or(iso)
}
