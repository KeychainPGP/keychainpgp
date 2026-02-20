//! Key export functionality.

use crate::error::Result;
use crate::storage::KeyRecord;

/// Export a public key as ASCII-armored text.
pub fn export_public_key(record: &KeyRecord) -> Result<String> {
    Ok(String::from_utf8_lossy(&record.pgp_data).into_owned())
}
