//! Browser-based WASM tests for keychainpgp-wasm.

use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use keychainpgp_wasm::*;

#[wasm_bindgen_test]
fn test_init() {
    init();
}

#[wasm_bindgen_test]
fn test_keygen_encrypt_decrypt_roundtrip() {
    init();

    // Generate a key pair
    let kp_js = generate_key_pair("Test User", "test@example.com", None).unwrap();

    // Extract fields from JS object
    let public_key = js_sys::Reflect::get(&kp_js, &JsValue::from_str("public_key"))
        .unwrap()
        .as_string()
        .unwrap();
    let secret_key = js_sys::Reflect::get(&kp_js, &JsValue::from_str("secret_key"))
        .unwrap()
        .as_string()
        .unwrap();
    let fingerprint = js_sys::Reflect::get(&kp_js, &JsValue::from_str("fingerprint"))
        .unwrap()
        .as_string()
        .unwrap();

    assert!(!fingerprint.is_empty());
    assert!(public_key.contains("BEGIN PGP PUBLIC KEY BLOCK"));
    assert!(secret_key.contains("BEGIN PGP PRIVATE KEY BLOCK"));

    // Encrypt
    let plaintext = "Hello from WASM!";
    let recipient_keys_json = format!("[{:?}]", public_key);
    let ciphertext = encrypt(plaintext, &recipient_keys_json).unwrap();
    assert!(ciphertext.contains("BEGIN PGP MESSAGE"));

    // Decrypt
    let decrypted = decrypt(&ciphertext, &secret_key, None).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[wasm_bindgen_test]
fn test_sign_verify_roundtrip() {
    init();

    let kp_js = generate_key_pair("Signer", "signer@example.com", None).unwrap();

    let public_key = js_sys::Reflect::get(&kp_js, &JsValue::from_str("public_key"))
        .unwrap()
        .as_string()
        .unwrap();
    let secret_key = js_sys::Reflect::get(&kp_js, &JsValue::from_str("secret_key"))
        .unwrap()
        .as_string()
        .unwrap();

    // Sign
    let message = "This message is authentic.";
    let signed = sign(message, &secret_key, None).unwrap();
    assert!(signed.contains("BEGIN PGP MESSAGE"));

    // Verify
    let verify_js = verify(&signed, &public_key).unwrap();
    let valid = js_sys::Reflect::get(&verify_js, &JsValue::from_str("valid"))
        .unwrap()
        .as_bool()
        .unwrap();
    assert!(valid);
}

#[wasm_bindgen_test]
fn test_inspect_key() {
    init();

    let kp_js = generate_key_pair("Alice", "alice@example.com", None).unwrap();

    let public_key = js_sys::Reflect::get(&kp_js, &JsValue::from_str("public_key"))
        .unwrap()
        .as_string()
        .unwrap();

    let info_js = inspect_key(&public_key).unwrap();
    let fingerprint = js_sys::Reflect::get(&info_js, &JsValue::from_str("fingerprint"))
        .unwrap()
        .as_string()
        .unwrap();
    let has_secret_key = js_sys::Reflect::get(&info_js, &JsValue::from_str("has_secret_key"))
        .unwrap()
        .as_bool()
        .unwrap();

    assert!(!fingerprint.is_empty());
    assert!(!has_secret_key);
}

#[wasm_bindgen_test]
fn test_keygen_with_passphrase() {
    init();

    let kp_js = generate_key_pair(
        "Protected",
        "protected@example.com",
        Some("my-passphrase".into()),
    )
    .unwrap();

    let public_key = js_sys::Reflect::get(&kp_js, &JsValue::from_str("public_key"))
        .unwrap()
        .as_string()
        .unwrap();
    let secret_key = js_sys::Reflect::get(&kp_js, &JsValue::from_str("secret_key"))
        .unwrap()
        .as_string()
        .unwrap();

    // Encrypt with public key
    let ciphertext = encrypt("secret", &format!("[{:?}]", public_key)).unwrap();

    // Decrypt with passphrase
    let decrypted = decrypt(&ciphertext, &secret_key, Some("my-passphrase".into())).unwrap();
    assert_eq!(decrypted, "secret");

    // Decrypt without passphrase should fail
    let result = decrypt(&ciphertext, &secret_key, None);
    assert!(result.is_err());
}
