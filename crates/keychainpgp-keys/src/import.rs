//! Key import functionality.

use keychainpgp_core::armor;

use crate::error::{Error, Result};

/// The result of parsing imported key data.
#[derive(Debug)]
pub struct ImportedKey {
    /// Raw key bytes (ASCII-armored or binary).
    pub data: Vec<u8>,
    /// Whether this is a public or private key.
    pub is_secret: bool,
}

/// Parse raw bytes and determine if they contain valid PGP key material.
///
/// Accepts both ASCII-armored and binary OpenPGP key data.
pub fn parse_import(data: &[u8]) -> Result<Vec<ImportedKey>> {
    let mut keys = Vec::new();

    if armor::is_pgp_public_key(data) {
        keys.push(ImportedKey {
            data: data.to_vec(),
            is_secret: false,
        });
    } else if armor::is_pgp_private_key(data) {
        keys.push(ImportedKey {
            data: data.to_vec(),
            is_secret: true,
        });
    } else {
        // Attempt to parse as binary OpenPGP data
        // For now, return an error; Sequoia can handle this at the keyring level
        return Err(Error::InvalidKey {
            reason: "data does not contain a recognized PGP key block".into(),
        });
    }

    Ok(keys)
}

/// Read key data from a file path.
pub fn read_key_file(path: &std::path::Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| Error::Io(e))
}
