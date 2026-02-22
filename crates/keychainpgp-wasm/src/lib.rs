//! WebAssembly bindings for KeychainPGP cryptographic operations.
//!
//! Exposes the core PGP engine to JavaScript via `wasm-bindgen`.

use wasm_bindgen::prelude::*;

use keychainpgp_core::engine::CryptoEngine;
use keychainpgp_core::sequoia_engine::SequoiaEngine;
use keychainpgp_core::types::{KeyGenOptions, UserId};
use secrecy::ExposeSecret;
use serde::Serialize;

/// Initialize the WASM module (sets up panic hook for better error messages).
#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Result of key generation, returned as a JS object.
#[derive(Serialize)]
struct KeyPairResult {
    public_key: String,
    secret_key: String,
    fingerprint: String,
}

/// Result of signature verification, returned as a JS object.
#[derive(Serialize)]
struct VerifyResultJs {
    valid: bool,
    signer_fingerprint: Option<String>,
}

/// Result of key inspection, returned as a JS object.
#[derive(Serialize)]
struct CertInfoJs {
    fingerprint: String,
    user_ids: Vec<UserIdJs>,
    algorithm: String,
    created_at: String,
    expires_at: Option<String>,
    has_secret_key: bool,
    subkeys: Vec<SubkeyInfoJs>,
}

#[derive(Serialize)]
struct UserIdJs {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Serialize)]
struct SubkeyInfoJs {
    fingerprint: String,
    algorithm: String,
    created_at: String,
    expires_at: Option<String>,
    capabilities: Vec<String>,
    is_revoked: bool,
}

/// Generate a new PGP key pair.
///
/// Returns a JS object: `{ public_key: string, secret_key: string, fingerprint: string }`
#[wasm_bindgen(js_name = generateKeyPair)]
pub fn generate_key_pair(
    name: &str,
    email: &str,
    passphrase: Option<String>,
) -> Result<JsValue, JsError> {
    let engine = SequoiaEngine::new();
    let user_id = UserId::new(name, email);
    let mut options = KeyGenOptions::new(user_id);

    if let Some(pp) = passphrase {
        if !pp.is_empty() {
            options = options.with_passphrase(secrecy::SecretBox::new(Box::new(pp.into_bytes())));
        }
    }

    let key_pair = engine
        .generate_key_pair(options)
        .map_err(|e| JsError::new(&e.to_string()))?;

    let result = KeyPairResult {
        public_key: String::from_utf8_lossy(&key_pair.public_key).into_owned(),
        secret_key: String::from_utf8_lossy(key_pair.secret_key.expose_secret()).into_owned(),
        fingerprint: key_pair.fingerprint.0.clone(),
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Encrypt plaintext for the given recipients.
///
/// `recipient_keys_json` is a JSON array of ASCII-armored public key strings.
/// Returns the ASCII-armored ciphertext.
#[wasm_bindgen(js_name = encrypt)]
pub fn encrypt(plaintext: &str, recipient_keys_json: &str) -> Result<String, JsError> {
    let engine = SequoiaEngine::new();

    let recipient_keys: Vec<String> =
        serde_json::from_str(recipient_keys_json).map_err(|e| JsError::new(&e.to_string()))?;

    let key_bytes: Vec<Vec<u8>> = recipient_keys.into_iter().map(|k| k.into_bytes()).collect();

    let ciphertext = engine
        .encrypt(plaintext.as_bytes(), &key_bytes)
        .map_err(|e| JsError::new(&e.to_string()))?;

    String::from_utf8(ciphertext).map_err(|e| JsError::new(&e.to_string()))
}

/// Decrypt an encrypted PGP message.
///
/// Returns the plaintext string.
#[wasm_bindgen(js_name = decrypt)]
pub fn decrypt(
    ciphertext: &str,
    secret_key: &str,
    passphrase: Option<String>,
) -> Result<String, JsError> {
    let engine = SequoiaEngine::new();

    let pp_bytes = passphrase.as_ref().map(|p| p.as_bytes());

    let plaintext = engine
        .decrypt(ciphertext.as_bytes(), secret_key.as_bytes(), pp_bytes)
        .map_err(|e| JsError::new(&e.to_string()))?;

    String::from_utf8(plaintext).map_err(|e| JsError::new(&e.to_string()))
}

/// Sign a message with the given secret key.
///
/// Returns the ASCII-armored signed message.
#[wasm_bindgen(js_name = sign)]
pub fn sign(data: &str, secret_key: &str, passphrase: Option<String>) -> Result<String, JsError> {
    let engine = SequoiaEngine::new();

    let pp_bytes = passphrase.as_ref().map(|p| p.as_bytes());

    let signed = engine
        .sign(data.as_bytes(), secret_key.as_bytes(), pp_bytes)
        .map_err(|e| JsError::new(&e.to_string()))?;

    String::from_utf8(signed).map_err(|e| JsError::new(&e.to_string()))
}

/// Verify a signed PGP message against a signer's public key.
///
/// Returns a JS object: `{ valid: boolean, signer_fingerprint: string | null }`
#[wasm_bindgen(js_name = verify)]
pub fn verify(signed_data: &str, signer_key: &str) -> Result<JsValue, JsError> {
    let engine = SequoiaEngine::new();

    let result = engine
        .verify(signed_data.as_bytes(), signer_key.as_bytes())
        .map_err(|e| JsError::new(&e.to_string()))?;

    let js_result = VerifyResultJs {
        valid: result.valid,
        signer_fingerprint: result.signer_fingerprint,
    };

    serde_wasm_bindgen::to_value(&js_result).map_err(|e| JsError::new(&e.to_string()))
}

/// Inspect a PGP key and extract metadata.
///
/// Returns a JS object with key information (fingerprint, user IDs, algorithm, dates, subkeys).
#[wasm_bindgen(js_name = inspectKey)]
pub fn inspect_key(key_data: &str) -> Result<JsValue, JsError> {
    let engine = SequoiaEngine::new();

    let info = engine
        .inspect_key(key_data.as_bytes())
        .map_err(|e| JsError::new(&e.to_string()))?;

    let js_info = CertInfoJs {
        fingerprint: info.fingerprint.0,
        user_ids: info
            .user_ids
            .into_iter()
            .map(|uid| UserIdJs {
                name: uid.name,
                email: uid.email,
            })
            .collect(),
        algorithm: info.algorithm.to_string(),
        created_at: info.created_at,
        expires_at: info.expires_at,
        has_secret_key: info.has_secret_key,
        subkeys: info
            .subkeys
            .into_iter()
            .map(|sk| SubkeyInfoJs {
                fingerprint: sk.fingerprint,
                algorithm: sk.algorithm,
                created_at: sk.created_at,
                expires_at: sk.expires_at,
                capabilities: sk.capabilities.into_iter().map(|c| c.to_string()).collect(),
                is_revoked: sk.is_revoked,
            })
            .collect(),
    };

    serde_wasm_bindgen::to_value(&js_info).map_err(|e| JsError::new(&e.to_string()))
}
