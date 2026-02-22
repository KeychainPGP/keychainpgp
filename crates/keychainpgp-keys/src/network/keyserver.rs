//! Keyserver (HKP/VKS) key search and upload.
//!
//! Supports keys.openpgp.org (Hagrid VKS) and standard HKP keyservers.

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
        builder = builder.proxy(proxy);
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

/// Upload a public key to a keyserver.
pub async fn keyserver_upload(
    key_data: &[u8],
    keyserver_url: &str,
    proxy_url: Option<&str>,
) -> Result<String, String> {
    let client = build_client(15, proxy_url)?;

    let key_text = String::from_utf8_lossy(key_data).into_owned();

    // Try VKS API first (keys.openpgp.org)
    let vks_url = format!("{}/vks/v1/upload", keyserver_url.trim_end_matches('/'));

    let response = client
        .post(&vks_url)
        .header("Content-Type", "application/pgp-keys")
        .body(key_text.clone())
        .send()
        .await;

    if let Ok(resp) = response {
        if resp.status().is_success() {
            return Ok("Key uploaded successfully. Check your email to verify.".into());
        }
    }

    // Fall back to HKP upload
    let hkp_url = format!("{}/pks/add", keyserver_url.trim_end_matches('/'));

    let form_body = format!("keytext={}", urlencoding(&key_text));

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
