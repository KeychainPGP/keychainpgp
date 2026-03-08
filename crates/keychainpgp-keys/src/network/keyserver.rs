//! Keyserver (HKP/VKS) key search and upload.
//!
//! Supports keys.openpgp.org (Hagrid VKS) and standard HKP keyservers.

use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Result from a keyserver search.
#[derive(Debug, Clone)]
pub struct KeyserverResult {
    /// Email address associated with the key.
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
/// Uses the VKS API (keys.openpgp.org) by default.
pub async fn keyserver_search(
    query: &str,
    keyserver_url: &str,
    proxy_url: Option<&str>,
) -> Result<Vec<KeyserverResult>, String> {
    let client = build_client(10, proxy_url)?;

    // Use HKP lookup endpoint
    let url = format!(
        "{}/pks/lookup?search={}&op=get&options=mr",
        keyserver_url.trim_end_matches('/'),
        urlencoding(query)
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Keyserver search failed: {e}"))?;

    if !response.status().is_success() {
        return Err("No keys found on keyserver.".into());
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read keyserver response: {e}"))?;

    // If the response contains a PGP key block, return it as a single result
    if body.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----") {
        return Ok(vec![KeyserverResult {
            email: if query.contains('@') {
                Some(query.to_string())
            } else {
                None
            },
            key_data: body.into_bytes(),
        }]);
    }

    Ok(vec![])
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
        if resp.status().is_success() {
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

    if response.status().is_success() {
        Ok("Key uploaded successfully.".into())
    } else {
        Err(format!("Upload failed with status: {}", response.status()))
    }
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
