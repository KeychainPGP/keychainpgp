//! Key synchronization between devices.
//!
//! Exports and imports encrypted key bundles containing both public and secret
//! keys, suitable for transfer via QR code sequences or file sharing.

use std::collections::HashMap;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

/// Custom serde: serialize `Vec<u8>` as base64 string, deserialize from either
/// base64 string (v2) or number array (v1) for backward compatibility.
mod serde_b64 {
    use serde::de;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&super::base64_encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        d.deserialize_any(B64OrArray)
    }

    struct B64OrArray;
    impl<'de> de::Visitor<'de> for B64OrArray {
        type Value = Vec<u8>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("base64 string or byte array")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<u8>, E> {
            super::base64_decode(v).map_err(E::custom)
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<u8>, A::Error> {
            let mut bytes = Vec::new();
            while let Some(b) = seq.next_element()? {
                bytes.push(b);
            }
            Ok(bytes)
        }
    }
}

/// Custom serde for `Option<Vec<u8>>` as optional base64 string.
mod serde_b64_opt {
    use serde::de;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &Option<Vec<u8>>, s: S) -> Result<S::Ok, S::Error> {
        match data {
            Some(d) => s.serialize_some(&super::base64_encode(d)),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<u8>>, D::Error> {
        d.deserialize_option(OptB64OrArray)
    }

    struct OptB64OrArray;
    impl<'de> de::Visitor<'de> for OptB64OrArray {
        type Value = Option<Vec<u8>>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("null, base64 string, or byte array")
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<Vec<u8>>, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Option<Vec<u8>>, D::Error> {
            super::serde_b64::deserialize(d).map(Some)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<Vec<u8>>, E> {
            Ok(None)
        }
    }
}

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
    /// Public key data (serialized as base64 string in v2, number array in v1).
    #[serde(with = "serde_b64")]
    pub public_key: Vec<u8>,
    /// Secret key data, only for own keys.
    #[serde(with = "serde_b64_opt")]
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

/// Default maximum bytes per QR code data part.
///
/// 200 bytes keeps QR codes at version 5–7 (ECC-L), which phone cameras
/// scan reliably even with fast carousel autoplay.
const QR_PART_SIZE: usize = 200;

/// Prefix for multi-part QR codes.
const QR_PREFIX: &str = "KCPGP";

/// Split encrypted data into QR-code-sized parts with fountain parity codes.
///
/// Each data part has the format `KCPGP:<part>/<total>:<base64_chunk>`.
/// Fountain parity parts use `KCPGP:F<i>+<j>/<total>:<base64_xor>`.
pub fn split_for_qr(encrypted: &[u8]) -> Vec<String> {
    split_for_qr_with_size(encrypted, QR_PART_SIZE)
}

/// Split encrypted data with a custom part size.
///
/// Generates data parts followed by fountain parity parts (XOR of consecutive
/// pairs) for redundancy. Parity parts allow recovering a missed data part
/// if its pair was scanned successfully.
pub fn split_for_qr_with_size(encrypted: &[u8], part_size: usize) -> Vec<String> {
    let encoded = base64_encode(encrypted);
    let chunks: Vec<&[u8]> = encoded.as_bytes().chunks(part_size).collect();
    let total = chunks.len();

    if total == 0 {
        return vec![format!("{QR_PREFIX}:1/1:")];
    }

    // Data parts
    let mut parts: Vec<String> = chunks
        .iter()
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
        .collect();

    // Fountain parity parts: XOR of consecutive pairs for redundancy.
    if total > 1 {
        let parity_count = total.div_ceil(2);
        for p in 0..parity_count {
            let i = p * 2 % total;
            let j = (p * 2 + 1) % total;
            if i == j {
                continue;
            }

            let a = chunks[i];
            let b = chunks[j];
            let len = part_size;
            let mut xor = Vec::with_capacity(len);
            for k in 0..len {
                xor.push(a.get(k).copied().unwrap_or(0) ^ b.get(k).copied().unwrap_or(0));
            }

            parts.push(format!(
                "{}:F{}+{}/{}:{}",
                QR_PREFIX,
                i + 1,
                j + 1,
                total,
                base64_encode(&xor)
            ));
        }
    }

    parts
}

/// Reassemble parts from QR scans back into encrypted data.
///
/// Handles both regular data parts and fountain parity parts.
/// Parts can be provided in any order; missing data parts are
/// recovered via fountain codes when possible.
pub fn reassemble_from_qr(parts: &[String]) -> Result<Vec<u8>, String> {
    if parts.is_empty() {
        return Err("No QR parts provided".into());
    }

    let prefix = format!("{QR_PREFIX}:");
    let mut data_map: HashMap<usize, String> = HashMap::new();
    let mut fountain: Vec<(usize, usize, String)> = Vec::new();
    let mut expected_total = 0;

    for p in parts {
        let Some(rest) = p.strip_prefix(&prefix) else {
            continue;
        };
        let Some((header, data)) = rest.split_once(':') else {
            continue;
        };

        if let Some(fheader) = header.strip_prefix('F') {
            // Fountain parity part: F<i>+<j>/<total>
            if let Some((indices, total_s)) = fheader.split_once('/') {
                if let Some((i_s, j_s)) = indices.split_once('+') {
                    if let (Ok(i), Ok(j), Ok(total)) = (
                        i_s.parse::<usize>(),
                        j_s.parse::<usize>(),
                        total_s.parse::<usize>(),
                    ) {
                        expected_total = expected_total.max(total);
                        fountain.push((i, j, data.to_string()));
                    }
                }
            }
        } else if let Some((part_s, total_s)) = header.split_once('/') {
            if let (Ok(part), Ok(total)) = (part_s.parse::<usize>(), total_s.parse::<usize>()) {
                expected_total = expected_total.max(total);
                data_map.insert(part, data.to_string());
            }
        }
    }

    if data_map.is_empty() && fountain.is_empty() {
        return Err("No valid KCPGP parts found".into());
    }

    // Fountain recovery: try to fill missing data parts using parity XOR
    if data_map.len() < expected_total && !fountain.is_empty() {
        let mut progress = true;
        while progress && data_map.len() < expected_total {
            progress = false;
            for (i, j, fdata) in &fountain {
                let has_i = data_map.contains_key(i);
                let has_j = data_map.contains_key(j);
                if (has_i && has_j) || (!has_i && !has_j) {
                    continue;
                }

                let known_key = if has_i { *i } else { *j };
                let missing_key = if has_i { *j } else { *i };

                let xor_bytes = base64_decode(fdata)?;
                let known_bytes = data_map[&known_key].as_bytes();
                let mut recovered: Vec<u8> = xor_bytes
                    .iter()
                    .enumerate()
                    .map(|(k, &xb)| xb ^ known_bytes.get(k).copied().unwrap_or(0))
                    .collect();
                // Trim trailing zeros (base64 chars are never 0x00)
                while recovered.last() == Some(&0) {
                    recovered.pop();
                }
                let recovered_str = String::from_utf8(recovered)
                    .map_err(|e| format!("fountain recovery produced invalid UTF-8: {e}"))?;
                data_map.insert(missing_key, recovered_str);
                progress = true;
            }
        }
    }

    if data_map.len() != expected_total {
        return Err(format!(
            "Incomplete scan: got {} of {} parts",
            data_map.len(),
            expected_total
        ));
    }

    let mut sorted: Vec<(usize, &String)> = data_map.iter().map(|(k, v)| (*k, v)).collect();
    sorted.sort_by_key(|(k, _)| *k);
    let combined: String = sorted.iter().map(|(_, d)| d.as_str()).collect();
    base64_decode(&combined)
}

/// Generate a cryptographically random sync passphrase.
///
/// Format: 6 groups of 4 digits separated by dashes (e.g., `1234-5678-9012-3456-7890-1234`).
/// Uses rejection sampling to avoid modulo bias, batched random for fewer syscalls.
pub fn generate_sync_passphrase() -> String {
    use keychainpgp_core::crypto_random;

    let mut out = String::with_capacity(6 * 5 - 1); // 4 digits + '-' x5
    let mut groups = 0;

    while groups < 6 {
        let mut buf = [0u8; 32];
        crypto_random(&mut buf);

        for chunk in buf.chunks_exact(2) {
            let val = u16::from_be_bytes([chunk[0], chunk[1]]);

            // Reject values >= 60000 to avoid modulo bias
            // (65536 % 10000 = 5536, so values 60000..65535 are biased)
            if val < 60000 {
                let num = val % 10000;

                if groups > 0 {
                    out.push('-');
                }

                out.push(char::from(b'0' + ((num / 1000) % 10) as u8));
                out.push(char::from(b'0' + ((num / 100) % 10) as u8));
                out.push(char::from(b'0' + ((num / 10) % 10) as u8));
                out.push(char::from(b'0' + (num % 10) as u8));

                groups += 1;
                if groups == 6 {
                    break;
                }
            }
        }
    }

    out
}

/// Compress data with deflate for smaller bundles.
pub fn compress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
    encoder
        .write_all(data)
        .map_err(|e| format!("compression failed: {e}"))?;
    encoder
        .finish()
        .map_err(|e| format!("compression failed: {e}"))
}

/// Decompress deflate-compressed data.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = flate2::read::DeflateDecoder::new(data);
    let mut buf = Vec::new();
    decoder
        .read_to_end(&mut buf)
        .map_err(|e| format!("decompression failed: {e}"))?;
    Ok(buf)
}

/// Decompress if compressed, otherwise return as-is.
///
/// Detects compressed data by checking if the first byte is `{` (raw JSON).
pub fn decompress_or_raw(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.first() == Some(&b'{') {
        // Raw JSON (v1 uncompressed format)
        Ok(data.to_vec())
    } else {
        // Compressed (v2 format)
        decompress(data)
    }
}

/// Encode bytes to base64 (standard alphabet with padding).
pub fn base64_encode(data: &[u8]) -> String {
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
        let data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        let parts = split_for_qr(&data);

        // Count data parts vs fountain parts
        let data_parts: Vec<_> = parts
            .iter()
            .filter(|p| p.starts_with("KCPGP:") && !p.starts_with("KCPGP:F"))
            .collect();
        let fountain_parts: Vec<_> = parts.iter().filter(|p| p.starts_with("KCPGP:F")).collect();

        assert!(data_parts.len() > 1, "should have multiple data parts");
        assert!(
            !fountain_parts.is_empty(),
            "should have fountain parity parts"
        );

        // Verify data parts have correct format
        for (i, part) in data_parts.iter().enumerate() {
            let expected_header = format!("KCPGP:{}/{}", i + 1, data_parts.len());
            assert!(
                part.starts_with(&expected_header),
                "part {i} header mismatch: expected {expected_header}, got {}",
                &part[..expected_header.len().min(part.len())]
            );
        }

        // Reassemble with all parts (data + fountain)
        let reassembled = reassemble_from_qr(&parts).unwrap();
        assert_eq!(data, reassembled);
    }

    #[test]
    fn test_qr_split_reassemble_out_of_order() {
        let data: Vec<u8> = (0..3000).map(|i| (i % 256) as u8).collect();
        let mut parts = split_for_qr(&data);
        parts.reverse();
        let reassembled = reassemble_from_qr(&parts).unwrap();
        assert_eq!(data, reassembled);
    }

    #[test]
    fn test_fountain_recovery() {
        let data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let parts = split_for_qr(&data);

        // Separate data and fountain parts
        let data_parts: Vec<_> = parts
            .iter()
            .filter(|p| p.starts_with("KCPGP:") && !p.starts_with("KCPGP:F"))
            .cloned()
            .collect();
        let fountain_parts: Vec<_> = parts
            .iter()
            .filter(|p| p.starts_with("KCPGP:F"))
            .cloned()
            .collect();

        assert!(data_parts.len() >= 2, "need at least 2 data parts");
        assert!(!fountain_parts.is_empty(), "need fountain parts");

        // Remove second data part (index 1) — the fountain parity of parts 1+2 should recover it
        let mut incomplete: Vec<String> = data_parts
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 1)
            .map(|(_, p)| p.clone())
            .collect();
        incomplete.extend(fountain_parts);

        let reassembled = reassemble_from_qr(&incomplete).unwrap();
        assert_eq!(data, reassembled);
    }

    #[test]
    fn test_qr_custom_part_size() {
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let small = split_for_qr_with_size(&data, 100);
        let large = split_for_qr_with_size(&data, 500);

        let small_data: Vec<_> = small.iter().filter(|p| !p.starts_with("KCPGP:F")).collect();
        let large_data: Vec<_> = large.iter().filter(|p| !p.starts_with("KCPGP:F")).collect();

        assert!(
            small_data.len() > large_data.len(),
            "smaller parts should produce more QR codes"
        );

        // Both should reassemble correctly
        let r1 = reassemble_from_qr(&small).unwrap();
        let r2 = reassemble_from_qr(&large).unwrap();
        assert_eq!(data, r1);
        assert_eq!(data, r2);
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
        // Only provide first data part, no fountain
        let first_data: Vec<_> = parts.iter().take(1).cloned().collect();
        let result = reassemble_from_qr(&first_data);
        assert!(result.is_err(), "should error on incomplete scan");
    }

    #[test]
    fn test_compress_decompress_round_trip() {
        let data = b"Hello, KeychainPGP! This is a test of compression.";
        let compressed = compress(data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_compress_reduces_size() {
        let data = r#"{"keys":[{"fingerprint":"AABB","public_key":[1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0]}]}"#;
        let repeated: String = std::iter::repeat_n(data, 50).collect::<Vec<_>>().join(",");
        let compressed = compress(repeated.as_bytes()).unwrap();
        assert!(
            compressed.len() < repeated.len() / 2,
            "compressed ({}) should be much smaller than original ({})",
            compressed.len(),
            repeated.len()
        );
    }

    #[test]
    fn test_decompress_or_raw_json() {
        let json = b"{\"version\":1,\"keys\":[]}";
        let result = decompress_or_raw(json).unwrap();
        assert_eq!(json.as_slice(), result.as_slice());
    }

    #[test]
    fn test_decompress_or_raw_compressed() {
        let json = b"{\"version\":1,\"keys\":[]}";
        let compressed = compress(json).unwrap();
        assert_ne!(
            compressed[0], b'{',
            "compressed data should not start with open brace"
        );
        let result = decompress_or_raw(&compressed).unwrap();
        assert_eq!(json.as_slice(), result.as_slice());
    }

    #[test]
    fn test_bundle_v2_base64_serde() {
        let bundle = KeyBundle {
            version: 2,
            keys: vec![
                KeyBundleEntry {
                    fingerprint: "AABBCCDD".into(),
                    public_key: vec![0x99, 0x01, 0x02, 0x03],
                    secret_key: Some(vec![0x95, 0x04, 0x05, 0x06]),
                    trust_level: 2,
                },
                KeyBundleEntry {
                    fingerprint: "EEFF0011".into(),
                    public_key: vec![0x99, 0x07, 0x08, 0x09],
                    secret_key: None,
                    trust_level: 1,
                },
            ],
        };

        let json = serde_json::to_string(&bundle).unwrap();
        // V2 should serialize keys as base64 strings, not arrays
        assert!(
            !json.contains("[153,"),
            "v2 should not contain number arrays"
        );
        assert!(
            json.contains("\"mQECAw==\""),
            "v2 should contain base64 string for public_key"
        );

        // Deserialize back
        let restored: KeyBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.version, 2);
        assert_eq!(restored.keys[0].public_key, vec![0x99, 0x01, 0x02, 0x03]);
        assert_eq!(
            restored.keys[0].secret_key.as_deref(),
            Some(&[0x95, 0x04, 0x05, 0x06][..])
        );
        assert!(restored.keys[1].secret_key.is_none());
    }

    #[test]
    fn test_bundle_v1_backward_compat() {
        // V1 format uses number arrays — our deserializer should accept both
        let v1_json = r#"{"version":1,"keys":[{"fingerprint":"AABB","public_key":[153,1,2,3],"secret_key":null,"trust_level":0}]}"#;
        let bundle: KeyBundle = serde_json::from_str(v1_json).unwrap();
        assert_eq!(bundle.keys[0].public_key, vec![153, 1, 2, 3]);
        assert!(bundle.keys[0].secret_key.is_none());
    }

    #[test]
    fn test_bundle_serialize_compress_round_trip() {
        let bundle = KeyBundle {
            version: 2,
            keys: vec![
                KeyBundleEntry {
                    fingerprint: "AABBCCDD".into(),
                    public_key: vec![0x99, 0x01, 0x02, 0x03],
                    secret_key: Some(vec![0x95, 0x04, 0x05, 0x06]),
                    trust_level: 2,
                },
                KeyBundleEntry {
                    fingerprint: "EEFF0011".into(),
                    public_key: vec![0x99, 0x07, 0x08, 0x09],
                    secret_key: None,
                    trust_level: 1,
                },
            ],
        };

        // Serialize -> compress -> decompress -> deserialize
        let json = serde_json::to_vec(&bundle).unwrap();
        let compressed = compress(&json).unwrap();
        let decompressed = decompress_or_raw(&compressed).unwrap();
        let restored: KeyBundle = serde_json::from_slice(&decompressed).unwrap();

        assert_eq!(restored.version, 2);
        assert_eq!(restored.keys.len(), 2);
        assert_eq!(restored.keys[0].fingerprint, "AABBCCDD");
        assert_eq!(restored.keys[1].fingerprint, "EEFF0011");
        assert!(restored.keys[0].secret_key.is_some());
        assert!(restored.keys[1].secret_key.is_none());
    }
}
