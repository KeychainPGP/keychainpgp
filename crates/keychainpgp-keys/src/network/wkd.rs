//! Web Key Directory (WKD) key lookup.
//!
//! Implements the WKD protocol (draft-koch-openpgp-webkey-service)
//! to discover OpenPGP public keys by email address.

use sha1::{Digest, Sha1};

/// z-base-32 alphabet (RFC 6189).
const ZBASE32_ALPHABET: &[u8; 32] = b"ybndrfg8ejkmcpqxot1uwisza345h769";

/// Encode bytes as z-base-32.
fn zbase32_encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits_in_buffer = 0;

    for &byte in data {
        buffer = (buffer << 8) | u64::from(byte);
        bits_in_buffer += 8;

        while bits_in_buffer >= 5 {
            bits_in_buffer -= 5;
            let index = ((buffer >> bits_in_buffer) & 0x1F) as usize;
            result.push(ZBASE32_ALPHABET[index] as char);
        }
    }

    if bits_in_buffer > 0 {
        let index = ((buffer << (5 - bits_in_buffer)) & 0x1F) as usize;
        result.push(ZBASE32_ALPHABET[index] as char);
    }

    result
}

/// WKD hash of the local part of an email address.
fn wkd_hash(local_part: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(local_part.to_lowercase().as_bytes());
    let hash = hasher.finalize();
    zbase32_encode(&hash)
}

/// Look up a public key via WKD.
///
/// Tries the "advanced" method first, then falls back to the "direct" method.
/// Returns the raw key bytes if found.
pub async fn wkd_lookup(email: &str, proxy_url: Option<&str>) -> Result<Vec<u8>, String> {
    let (local, domain) = email
        .split_once('@')
        .ok_or_else(|| format!("Invalid email address: {email}"))?;

    let hash = wkd_hash(local);

    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10));

    if let Some(proxy) = proxy_url {
        let proxy = reqwest::Proxy::all(proxy)
            .map_err(|e| format!("Invalid proxy URL: {e}"))?;
        builder = builder.proxy(proxy);
    }

    let client = builder.build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    // Try advanced method first
    let advanced_url = format!(
        "https://openpgpkey.{domain}/.well-known/openpgpkey/{domain}/hu/{hash}?l={local}"
    );

    if let Ok(response) = client.get(&advanced_url).send().await {
        if response.status().is_success() {
            if let Ok(bytes) = response.bytes().await {
                if !bytes.is_empty() {
                    return Ok(bytes.to_vec());
                }
            }
        }
    }

    // Fall back to direct method
    let direct_url = format!(
        "https://{domain}/.well-known/openpgpkey/hu/{hash}?l={local}"
    );

    let response = client
        .get(&direct_url)
        .send()
        .await
        .map_err(|e| format!("WKD lookup failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("No WKD key found for {email}"));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read WKD response: {e}"))?;

    if bytes.is_empty() {
        return Err(format!("No WKD key found for {email}"));
    }

    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zbase32_encode() {
        // Test vector: SHA-1 of "Joe" is a known value
        let hash = {
            let mut h = Sha1::new();
            h.update(b"joe");
            h.finalize()
        };
        let encoded = zbase32_encode(&hash);
        // z-base-32 encoding of SHA-1 should be 32 chars
        assert_eq!(encoded.len(), 32);
    }

    #[test]
    fn test_wkd_hash_deterministic() {
        let h1 = wkd_hash("alice");
        let h2 = wkd_hash("alice");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_wkd_hash_case_insensitive() {
        let h1 = wkd_hash("Alice");
        let h2 = wkd_hash("alice");
        assert_eq!(h1, h2);
    }
}
