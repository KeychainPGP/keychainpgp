//! Key synchronization between devices.
//!
//! Exports and imports encrypted key bundles containing both public and secret
//! keys, suitable for transfer via QR code sequences or file sharing.

use serde::{Deserialize, Serialize};

/// A bundle containing all keys for sync.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyBundle {
    /// Bundle format version.
    pub version: u32,
    /// Each key entry.
    pub keys: Vec<KeyBundleEntry>,
}

/// A single key entry within a sync bundle.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyBundleEntry {
    /// Key fingerprint (hex).
    pub fingerprint: String,
    /// ASCII-armored public key data.
    pub public_key: Vec<u8>,
    /// ASCII-armored secret key data (only for own keys).
    pub secret_key: Option<Vec<u8>>,
    /// Trust level.
    pub trust_level: i32,
}

impl Drop for KeyBundleEntry {
    fn drop(&mut self) {
        if let Some(ref mut sk) = self.secret_key {
            zeroize::Zeroize::zeroize(sk);
        }
    }
}

/// Maximum bytes per QR code part.
///
/// Lower values produce simpler (lower-version) QR codes that phone cameras
/// can scan reliably from a screen.  500 bytes + the `KCPGP:N/M:` header
/// lands around QR version 10–15 at ECC-L, which is quick to detect even
/// when the codes rotate rapidly.
const QR_PART_SIZE: usize = 500;

/// Prefix for multi-part QR codes.
const QR_PREFIX: &str = "KCPGP";

/// Split encrypted data into QR-code-sized parts.
///
/// Each part has the format `KCPGP:<part>/<total>:<base64_chunk>`.
pub fn split_for_qr(encrypted: &[u8]) -> Vec<String> {
    let encoded = base64_encode(encrypted);
    let total = encoded.len().div_ceil(QR_PART_SIZE);

    if total == 0 {
        return vec![format!("{QR_PREFIX}:1/1:")];
    }

    encoded
        .as_bytes()
        .chunks(QR_PART_SIZE)
        .enumerate()
        .map(|(i, chunk)| {
            format!(
                "{}:{}/{}:{}",
                QR_PREFIX,
                i + 1,
                total,
                String::from_utf8_lossy(chunk)
            )
        })
        .collect()
}

/// Reassemble parts from QR scans back into encrypted data.
///
/// Parts can be provided in any order; they are sorted by part number.
pub fn reassemble_from_qr(parts: &[String]) -> Result<Vec<u8>, String> {
    if parts.is_empty() {
        return Err("No QR parts provided".into());
    }

    let mut parsed: Vec<(usize, usize, &str)> = parts
        .iter()
        .filter_map(|p| {
            let rest = p.strip_prefix(&format!("{QR_PREFIX}:"))?;
            let (header, data) = rest.split_once(':')?;
            let (part_s, total_s) = header.split_once('/')?;
            let part: usize = part_s.parse().ok()?;
            let total: usize = total_s.parse().ok()?;
            Some((part, total, data))
        })
        .collect();

    if parsed.is_empty() {
        return Err("No valid KCPGP parts found".into());
    }

    let expected_total = parsed[0].1;
    if parsed.len() != expected_total {
        return Err(format!(
            "Incomplete scan: got {} of {} parts",
            parsed.len(),
            expected_total
        ));
    }

    parsed.sort_by_key(|(n, _, _)| *n);

    let combined: String = parsed.iter().map(|(_, _, d)| *d).collect();
    base64_decode(&combined)
}

/// Generate a cryptographically random sync passphrase.
///
/// Format: 6 groups of 4 digits separated by dashes (e.g., `1234-5678-9012-3456-7890-1234`).
/// Uses rejection sampling to avoid modulo bias.
pub fn generate_sync_passphrase() -> String {
    use keychainpgp_core::crypto_random;

    let mut groups = Vec::with_capacity(6);
    while groups.len() < 6 {
        let mut pair = [0u8; 2];
        crypto_random(&mut pair);
        let val = u16::from_be_bytes(pair);
        // Reject values >= 60000 to avoid modulo bias
        // (65536 % 10000 = 5536, so values 60000..65535 are biased)
        if val < 60000 {
            groups.push(format!("{:04}", val % 10000));
        }
    }

    groups.join("-")
}

/// Encode bytes to base64 (URL-safe, no padding).
pub fn base64_encode(data: &[u8]) -> String {
    // Simple base64 encoding using standard alphabet.
    // We use a no-dependency implementation to avoid adding another crate.
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Decode base64 string to bytes.
pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    fn val(c: u8) -> Result<u32, String> {
        match c {
            b'A'..=b'Z' => Ok((c - b'A') as u32),
            b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
            b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err(format!("invalid base64 character: {}", c as char)),
        }
    }

    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let bytes = input.as_bytes();
    if bytes.len() % 4 != 0 {
        return Err("invalid base64 length".into());
    }

    let mut result = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let a = val(chunk[0])?;
        let b = val(chunk[1])?;
        let c = val(chunk[2])?;
        let d = val(chunk[3])?;
        let triple = (a << 18) | (b << 12) | (c << 6) | d;

        result.push((triple >> 16) as u8);
        if chunk[2] != b'=' {
            result.push((triple >> 8) as u8);
        }
        if chunk[3] != b'=' {
            result.push(triple as u8);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_round_trip() {
        let data = b"Hello, KeychainPGP! This is a test of the sync module.";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_qr_split_reassemble() {
        // Create data larger than QR_PART_SIZE
        let data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        let parts = split_for_qr(&data);
        assert!(parts.len() > 1, "should split into multiple parts");

        // Verify all parts have correct format
        for (i, part) in parts.iter().enumerate() {
            assert!(part.starts_with("KCPGP:"), "part should start with prefix");
            let expected_header = format!("KCPGP:{}/{}", i + 1, parts.len());
            assert!(
                part.starts_with(&expected_header),
                "part {i} header mismatch: expected {expected_header}, got {}",
                &part[..expected_header.len().min(part.len())]
            );
        }

        // Reassemble and verify
        let reassembled = reassemble_from_qr(&parts).unwrap();
        assert_eq!(data, reassembled);
    }

    #[test]
    fn test_qr_split_reassemble_out_of_order() {
        let data: Vec<u8> = (0..3000).map(|i| (i % 256) as u8).collect();
        let mut parts = split_for_qr(&data);
        parts.reverse(); // Shuffle order
        let reassembled = reassemble_from_qr(&parts).unwrap();
        assert_eq!(data, reassembled);
    }

    #[test]
    fn test_passphrase_format() {
        let passphrase = generate_sync_passphrase();
        let groups: Vec<&str> = passphrase.split('-').collect();
        assert_eq!(groups.len(), 6, "should have 6 groups");
        for group in &groups {
            assert_eq!(group.len(), 4, "each group should be 4 digits");
            assert!(
                group.chars().all(|c| c.is_ascii_digit()),
                "should be all digits"
            );
        }
    }

    #[test]
    fn test_incomplete_scan_error() {
        let data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        let parts = split_for_qr(&data);
        // Only provide first part
        let result = reassemble_from_qr(&parts[..1]);
        assert!(result.is_err(), "should error on incomplete scan");
    }
}
