//! Keyserver (HKP/VKS) key search and upload.
//!
//! Supports keys.openpgp.org (Hagrid VKS) and standard HKP keyservers.
use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Result from a keyserver search (machine-readable index).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyserverMatch {
    /// Key ID (long or short).
    pub key_id: String,
    /// Key fingerprint (hex).
    pub fingerprint: String,
    /// Creation date (seconds since epoch).
    pub created_at: Option<u64>,
    /// Expiration date (seconds since epoch).
    pub expires_at: Option<u64>,
    /// List of User IDs (name <email>).
    pub user_ids: Vec<String>,
}

/// Result from a keyserver search (full key data).
#[derive(Debug, Clone)]
pub struct KeyserverResult {
    /// Email address associated with the key (if known from query).
    pub email: Option<String>,
    /// ASCII-armored public key data.
    pub key_data: Vec<u8>,
}

/// Build a reqwest client with optional SOCKS5 proxy support.
fn build_client(timeout_secs: u64, proxy_url: Option<&str>) -> Result<reqwest::Client, String> {
    let mut builder =
        reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout_secs));

    if let Some(proxy) = proxy_url {
        let proxy = reqwest::Proxy::all(proxy).map_err(|e| format!("Invalid proxy URL: {e}"))?;
        builder = builder.proxy(proxy).no_proxy();
    }

    builder
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))
}

/// Search for keys on a keyserver by email or name.
///
/// Returns a list of machine-readable matches.
pub async fn keyserver_search(
    query: &str,
    keyserver_url: &str,
    proxy_url: Option<&str>,
) -> Result<Vec<KeyserverMatch>, String> {
    let client = build_client(15, proxy_url)?;

    let url = format!(
        "{}/pks/lookup?search={}&op=index&options=mr",
        keyserver_url.trim_end_matches('/'),
        urlencoding(query)
    );

    // Retry up to 2 times on 404 — HKP clusters can return inconsistent
    // results when load-balanced across nodes with different replication state.
    let max_attempts = 3;
    for attempt in 1..=max_attempts {
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Keyserver search failed: {e}"))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            if attempt < max_attempts {
                continue;
            }
            return Ok(vec![]);
        }

        if !response.status().is_success() {
            return Err(format!(
                "Keyserver search failed with status: {}",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read keyserver response: {e}"))?;

        return Ok(parse_mr_index(&body));
    }

    Ok(vec![])
}

/// Fetch a full ASCII-armored public key from a keyserver by fingerprint.
pub async fn keyserver_fetch(
    fingerprint: &str,
    keyserver_url: &str,
    proxy_url: Option<&str>,
) -> Result<Vec<u8>, String> {
    if !fingerprint.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "Invalid fingerprint: must be hexadecimal: {fingerprint}"
        ));
    }

    let client = build_client(15, proxy_url)?;

    let url = format!(
        "{}/pks/lookup?search=0x{}&op=get&options=mr",
        keyserver_url.trim_end_matches('/'),
        fingerprint
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Keyserver fetch failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Key not found or server error: {}",
            response.status()
        ));
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read keyserver response: {e}"))?;

    if body.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----") {
        Ok(body.into_bytes())
    } else {
        Err("Response did not contain a valid PGP key block".into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyserverKind {
    KeysOpenPgpOrg,
    Generic,
}

const KEYS_OPENPGP_ORG_HOST: &str = "keys.openpgp.org";
const VKS_UNPUBLISHED_STATUS: &str = "unpublished";
const VKS_PENDING_STATUS: &str = "pending";
const DEFAULT_VERIFICATION_LOCALE: &str = "en_US";

#[derive(Debug, Serialize)]
struct VksUploadRequest<'a> {
    keytext: &'a str,
}

#[derive(Debug, Deserialize)]
struct VksUploadResponse {
    token: Option<String>,
    #[serde(default)]
    status: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct VksRequestVerifyRequest {
    token: String,
    addresses: Vec<String>,
    locale: Vec<String>,
}

async fn post_json<T: Serialize>(
    client: &reqwest::Client,
    url: &str,
    payload: &T,
) -> Result<reqwest::Response, String> {
    let body =
        serde_json::to_string(payload).map_err(|e| format!("Failed to encode JSON body: {e}"))?;
    client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))
}

async fn parse_json_response<T: DeserializeOwned>(body: &str, context: &str) -> Result<T, String> {
    serde_json::from_str(body).map_err(|e| format!("Failed to parse {context} response: {e}"))
}

fn detect_keyserver_kind(keyserver_url: &str) -> KeyserverKind {
    let host = reqwest::Url::parse(keyserver_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()));

    match host.as_deref() {
        Some(KEYS_OPENPGP_ORG_HOST) => KeyserverKind::KeysOpenPgpOrg,
        _ => KeyserverKind::Generic,
    }
}

fn verification_addresses(status: &HashMap<String, String>) -> Vec<String> {
    status
        .iter()
        .filter_map(|(email, state)| {
            if state.eq_ignore_ascii_case(VKS_UNPUBLISHED_STATUS) {
                Some(email.clone())
            } else {
                None
            }
        })
        .collect()
}

fn pending_verification_count(status: &HashMap<String, String>) -> usize {
    status
        .values()
        .filter(|state| state.eq_ignore_ascii_case(VKS_PENDING_STATUS))
        .count()
}

async fn keyserver_upload_keys_openpgp_org(
    client: &reqwest::Client,
    key_text: &str,
    keyserver_url: &str,
) -> Result<String, String> {
    validate_keyserver_url(keyserver_url)?;
    let base = keyserver_url.trim_end_matches('/');
    let upload_url = format!("{base}/vks/v1/upload");

    let upload_resp = post_json(client, &upload_url, &VksUploadRequest { keytext: key_text })
        .await
        .map_err(|e| format!("Upload failed: {e}"))?;

    let status = upload_resp.status();
    let body = upload_resp
        .text()
        .await
        .map_err(|e| format!("Failed to read upload response body: {e}"))?;

    tracing::debug!(?status, %body, "VKS upload response");

    if !status.is_success() {
        return Err(format!("Upload failed with status: {status}. Body: {body}"));
    }

    let upload_body: VksUploadResponse = parse_json_response(&body, "VKS upload").await?;

    let Some(token) = upload_body.token else {
        return Ok("Key uploaded successfully.".into());
    };

    let addresses = verification_addresses(&upload_body.status);
    if addresses.is_empty() {
        let pending = pending_verification_count(&upload_body.status);
        if pending > 0 {
            tracing::debug!(
                pending,
                ?upload_body.status,
                "Skipping VKS request-verify: verification already pending"
            );
            return Ok(format!(
                "Key uploaded successfully. Verification email already pending for {pending} address(es). Check your inbox."
            ));
        }
        return Ok("Key uploaded successfully.".into());
    }

    let request_verify_url = format!("{base}/vks/v1/request-verify");
    let verify_request = VksRequestVerifyRequest {
        token,
        addresses,
        locale: vec![DEFAULT_VERIFICATION_LOCALE.into()],
    };
    let address_count = verify_request.addresses.len();
    let verify_resp = post_json(client, &request_verify_url, &verify_request)
        .await
        .map_err(|e| format!("Verification email request failed: {e}"))?;

    let status = verify_resp.status();
    let body = verify_resp
        .text()
        .await
        .map_err(|e| format!("Failed to read verification response body: {e}"))?;

    tracing::debug!(?status, %body, "VKS verification request response");

    if !status.is_success() {
        return Err(format!(
            "Verification email request failed with status: {status}. Body: {body}"
        ));
    }

    Ok(format!(
        "Key uploaded successfully. Verification email requested for {address_count} address(es)."
    ))
}

async fn keyserver_upload_generic(
    client: &reqwest::Client,
    key_text: &str,
    keyserver_url: &str,
) -> Result<String, String> {
    // Try VKS-like endpoint first for compatibility.
    let vks_url = format!("{}/vks/v1/upload", keyserver_url.trim_end_matches('/'));

    let response = client
        .post(&vks_url)
        .header("Content-Type", "application/pgp-keys")
        .body(key_text.to_owned())
        .send()
        .await;

    if let Ok(resp) = response {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::debug!("VKS upload status: {}, body: {}", status, body);
        if status.is_success() {
            return Ok("Key uploaded successfully. Check your email to verify.".into());
        }
    }

    // Fall back to HKP upload.
    let hkp_url = format!("{}/pks/add", keyserver_url.trim_end_matches('/'));
    let form_body = format!("keytext={}", urlencoding(key_text));

    let response = client
        .post(&hkp_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_body)
        .send()
        .await
        .map_err(|e| format!("Upload failed: {e}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read keyserver response: {e}"))?;

    tracing::debug!("HKP upload status: {}, body: {}", status, body);

    if status.is_success() {
        Ok("Key uploaded successfully.".into())
    } else {
        Err(format!(
            "Upload failed with status: {status}. Response: {body}"
        ))
    }
}

/// Validates and normalizes a keyserver URL.
/// Prepends "https://" if no scheme is present.
pub fn validate_keyserver_url(url: &str) -> Result<(), String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(format!(
            "Invalid keyserver URL: '{url}'. It must start with http:// or https://"
        ))
    }
}

/// Parse the HKP machine-readable index format (options=mr).
///
/// Format: colon-separated records.
/// info:version:count
/// pub:keyid:algo:keylen:creationdate:expirationdate:flags
/// uid:escaped_uid:creationdate:expirationdate:flags
fn parse_mr_index(body: &str) -> Vec<KeyserverMatch> {
    let mut results = Vec::new();
    let mut current_key: Option<KeyserverMatch> = None;

    for line in body.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "pub" => {
                // If we were building a key, push it
                if let Some(k) = current_key.take() {
                    results.push(k);
                }

                if parts.len() < 5 {
                    continue;
                }

                let key_id = parts[1].to_string();
                // Fingerprint is not always in 'pub' record in old HKP,
                // but some modern ones (Hagrid) use it as keyid or add it.
                // We'll treat the keyid as the primary identifier for now.
                let created_at = parts.get(4).and_then(|s| s.parse().ok());
                let expires_at = parts.get(5).and_then(|s| s.parse().ok());

                current_key = Some(KeyserverMatch {
                    key_id,
                    fingerprint: String::new(), // Will be updated if we find more info or just use KeyID
                    created_at,
                    expires_at,
                    user_ids: Vec::new(),
                });
            }
            "uid" => {
                if let Some(ref mut k) = current_key {
                    if parts.len() > 1 {
                        // HKP escapes certain characters in UIDs
                        let uid = hkr_unescape(parts[1]);
                        k.user_ids.push(uid);
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(k) = current_key {
        results.push(k);
    }

    // Post-process: many keyservers return the fingerprint in the 'pub' field if it's 40 chars
    for k in &mut results {
        if k.key_id.len() >= 32 {
            k.fingerprint = k.key_id.to_uppercase();
        }
    }

    results
}

/// Unescape HKP 'escaped_uid' field.
fn hkr_unescape(input: &str) -> String {
    let mut result_bytes = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex_bytes = &bytes[i + 1..=i + 2];
            // Safely attempt to parse the next two bytes as a hex string
            if let Ok(hex_str) = std::str::from_utf8(hex_bytes) {
                if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                    result_bytes.push(byte);
                    i += 3;
                    continue;
                }
            }
        }

        // Fallback for non-escaped bytes or invalid hex after '%'
        result_bytes.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&result_bytes).into_owned()
}

/// Upload a public key to a keyserver.
pub async fn keyserver_upload(
    key_data: &[u8],
    keyserver_url: &str,
    proxy_url: Option<&str>,
) -> Result<String, String> {
    let client = build_client(15, proxy_url)?;
    let key_text = String::from_utf8_lossy(key_data).into_owned();

    match detect_keyserver_kind(keyserver_url) {
        KeyserverKind::KeysOpenPgpOrg => {
            keyserver_upload_keys_openpgp_org(&client, &key_text, keyserver_url).await
        }
        KeyserverKind::Generic => keyserver_upload_generic(&client, &key_text, keyserver_url).await,
    }
}

/// Simple percent-encoding for URL query parameters.
fn urlencoding(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push('%');
                result.push_str(&format!("{byte:02X}"));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkr_unescape_basic() {
        assert_eq!(hkr_unescape("Alice%20Smith"), "Alice Smith");
        assert_eq!(
            hkr_unescape("Alice%3cemail%40example.com%3e"),
            "Alice<email@example.com>"
        );
        assert_eq!(hkr_unescape("No%20changes"), "No changes");
    }

    #[test]
    fn test_hkr_unescape_multibyte_utf8() {
        // "é" is %C3%A9
        assert_eq!(hkr_unescape("Alice%20%C3%A9"), "Alice é");

        // "🦀" is %F0%9F%A6%80
        assert_eq!(hkr_unescape("Ferris%20%F0%9F%A6%80"), "Ferris 🦀");
    }

    #[test]
    fn test_hkr_unescape_invalid_utf8() {
        // %FF is an invalid UTF-8 start byte.
        // `from_utf8_lossy` should replace it with the Unicode replacement character ().
        assert_eq!(hkr_unescape("Bad%FFData"), "Bad\u{FFFD}Data");
    }

    #[test]
    fn test_hkr_unescape_malformed_escapes() {
        // Handle '%' at the end of the string
        assert_eq!(hkr_unescape("Trailing%"), "Trailing%");

        // Handle invalid hex digits
        assert_eq!(hkr_unescape("Invalid%ZZHex"), "Invalid%ZZHex");
    }

    #[test]
    fn test_parse_mr_index() {
        let body = "info:1:2\n\
                    pub:ED7001AAAD902CA416FB7A5148007E3D:1:2048:1620000000::\n\
                    uid:Alice%20Smith%20<alice@example.com>:1620000000::\n\
                    pub:F22FD696C8875505:1:4096:1610000000:1700000000:\n\
                    uid:Bob%20Brown:1610000000::\n\
                    uid:Bob%20Admin%20<admin@example.com>:1610000000::";

        let results = parse_mr_index(body);
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].key_id, "ED7001AAAD902CA416FB7A5148007E3D");
        assert_eq!(results[0].fingerprint, "ED7001AAAD902CA416FB7A5148007E3D");
        assert_eq!(results[0].user_ids.len(), 1);
        assert_eq!(results[0].user_ids[0], "Alice Smith <alice@example.com>");

        assert_eq!(results[1].key_id, "F22FD696C8875505");
        assert_eq!(results[1].fingerprint, ""); // Short ID, not enough for fingerprint
        assert_eq!(results[1].user_ids.len(), 2);
        assert_eq!(results[1].user_ids[0], "Bob Brown");
        assert_eq!(results[1].user_ids[1], "Bob Admin <admin@example.com>");
    }

    #[tokio::test]
    async fn test_keyserver_fetch_invalid_fingerprint() {
        let result = keyserver_fetch(
            "AAAA&op=delete&search=",
            "https://keyserver.ubuntu.com",
            None,
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid fingerprint"));

        let result = keyserver_fetch("not-hex", "https://keyserver.ubuntu.com", None).await;
        assert!(result.is_err());

        let result = keyserver_fetch("123G", "https://keyserver.ubuntu.com", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_keyserver_fetch_valid_fingerprint_format() {
        // We don't want to actually hit the network in unit tests if possible,
        // but here we are just testing the validation step which happens before the network call.
        // However, since it's an async function and build_client will be called,
        // it might try to make a request if we don't mock it or use an invalid URL.

        let result = keyserver_fetch("ABCDEF0123456789", "invalid-url", None).await;
        // It should pass validation and then fail on build_client or the actual request
        match result {
            Err(e) => assert!(!e.contains("Invalid fingerprint")),
            _ => panic!("Should have failed with network/URL error, not validation error"),
        }
    }

    #[test]
    fn detect_keyserver_kind_matches_keys_openpgp_org() {
        assert_eq!(
            detect_keyserver_kind("https://keys.openpgp.org"),
            KeyserverKind::KeysOpenPgpOrg
        );
        assert_eq!(
            detect_keyserver_kind("https://keys.openpgp.org/"),
            KeyserverKind::KeysOpenPgpOrg
        );
    }

    #[test]
    fn detect_keyserver_kind_defaults_to_generic() {
        assert_eq!(
            detect_keyserver_kind("https://keyserver.ubuntu.com"),
            KeyserverKind::Generic
        );
        assert_eq!(detect_keyserver_kind("not-a-url"), KeyserverKind::Generic);
    }

    #[test]
    fn verification_addresses_selects_only_unpublished() {
        let mut status = HashMap::new();
        status.insert("a@example.com".to_string(), "unpublished".to_string());
        status.insert("b@example.com".to_string(), "published".to_string());
        status.insert("c@example.com".to_string(), "UnPublished".to_string());

        let mut out = verification_addresses(&status);
        out.sort();
        assert_eq!(out, vec!["a@example.com", "c@example.com"]);
    }

    #[test]
    fn pending_verification_count_counts_pending_case_insensitive() {
        let mut status = HashMap::new();
        status.insert("a@example.com".to_string(), "pending".to_string());
        status.insert("b@example.com".to_string(), "Pending".to_string());
        status.insert("c@example.com".to_string(), "published".to_string());
        assert_eq!(pending_verification_count(&status), 2);
    }
}
