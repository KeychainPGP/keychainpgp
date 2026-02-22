//! End-to-end integration tests for the full KeychainPGP pipeline.
//!
//! Tests generate → store → encrypt → decrypt → sign → verify round trips
//! using a temporary keyring directory.

use keychainpgp_core::types::{KeyGenOptions, UserId};
use keychainpgp_core::{CryptoEngine, SequoiaEngine};
use keychainpgp_keys::Keyring;
use keychainpgp_keys::storage::KeyRecord;
use secrecy::ExposeSecret;

fn setup() -> (SequoiaEngine, Keyring, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let engine = SequoiaEngine::new();
    let keyring = Keyring::open_at(tmp.path()).unwrap();
    (engine, keyring, tmp)
}

fn generate_and_store(
    engine: &SequoiaEngine,
    keyring: &Keyring,
    name: &str,
    email: &str,
) -> String {
    let options = KeyGenOptions::new(UserId::new(name, email));
    let key_pair = engine.generate_key_pair(options).unwrap();
    let info = engine.inspect_key(&key_pair.public_key).unwrap();

    let record = KeyRecord {
        fingerprint: key_pair.fingerprint.0.clone(),
        name: Some(name.to_string()),
        email: Some(email.to_string()),
        algorithm: info.algorithm.to_string(),
        created_at: info.created_at,
        expires_at: info.expires_at,
        trust_level: 2,
        is_own_key: true,
        pgp_data: key_pair.public_key.clone(),
    };

    keyring
        .store_generated_key(record, key_pair.secret_key.expose_secret())
        .unwrap();

    key_pair.fingerprint.0.clone()
}

#[test]
fn test_generate_store_list_roundtrip() {
    let (engine, keyring, _tmp) = setup();

    let fp = generate_and_store(&engine, &keyring, "Alice", "alice@test.com");

    let keys = keyring.list_keys().unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].fingerprint, fp);
    assert_eq!(keys[0].name.as_deref(), Some("Alice"));
    assert_eq!(keys[0].email.as_deref(), Some("alice@test.com"));
    assert!(keys[0].is_own_key);
}

#[test]
fn test_generate_encrypt_decrypt_roundtrip() {
    let (engine, keyring, _tmp) = setup();

    let fp = generate_and_store(&engine, &keyring, "Bob", "bob@test.com");

    let record = keyring.get_key(&fp).unwrap().unwrap();
    let plaintext = b"Hello Bob, this is a secret message!";

    // Encrypt to Bob's public key
    let ciphertext = engine.encrypt(plaintext, &[record.pgp_data]).unwrap();
    assert!(!ciphertext.is_empty());
    assert_ne!(&ciphertext[..], plaintext);

    // Decrypt with Bob's secret key
    let secret_key = keyring.get_secret_key(&fp).unwrap();
    let decrypted = engine
        .decrypt(&ciphertext, secret_key.expose_secret(), None)
        .unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_multi_recipient_encrypt_decrypt() {
    let (engine, keyring, _tmp) = setup();

    let fp_alice = generate_and_store(&engine, &keyring, "Alice", "alice@test.com");
    let fp_bob = generate_and_store(&engine, &keyring, "Bob", "bob@test.com");

    let alice_record = keyring.get_key(&fp_alice).unwrap().unwrap();
    let bob_record = keyring.get_key(&fp_bob).unwrap().unwrap();

    let plaintext = b"Message for both Alice and Bob";
    let ciphertext = engine
        .encrypt(plaintext, &[alice_record.pgp_data, bob_record.pgp_data])
        .unwrap();

    // Alice can decrypt
    let alice_sk = keyring.get_secret_key(&fp_alice).unwrap();
    let dec_alice = engine
        .decrypt(&ciphertext, alice_sk.expose_secret(), None)
        .unwrap();
    assert_eq!(dec_alice, plaintext);

    // Bob can decrypt
    let bob_sk = keyring.get_secret_key(&fp_bob).unwrap();
    let dec_bob = engine
        .decrypt(&ciphertext, bob_sk.expose_secret(), None)
        .unwrap();
    assert_eq!(dec_bob, plaintext);
}

#[test]
fn test_wrong_key_cannot_decrypt() {
    let (engine, keyring, _tmp) = setup();

    let fp_alice = generate_and_store(&engine, &keyring, "Alice", "alice@test.com");
    let fp_eve = generate_and_store(&engine, &keyring, "Eve", "eve@test.com");

    let alice_record = keyring.get_key(&fp_alice).unwrap().unwrap();
    let plaintext = b"Only for Alice";
    let ciphertext = engine.encrypt(plaintext, &[alice_record.pgp_data]).unwrap();

    // Eve should NOT be able to decrypt
    let eve_sk = keyring.get_secret_key(&fp_eve).unwrap();
    let result = engine.decrypt(&ciphertext, eve_sk.expose_secret(), None);
    assert!(result.is_err());
}

#[test]
fn test_sign_verify_roundtrip() {
    let (engine, keyring, _tmp) = setup();

    let fp = generate_and_store(&engine, &keyring, "Signer", "signer@test.com");
    let record = keyring.get_key(&fp).unwrap().unwrap();
    let secret_key = keyring.get_secret_key(&fp).unwrap();

    let data = b"This document is authentic.";
    let signed = engine.sign(data, secret_key.expose_secret(), None).unwrap();

    // Verify with the signer's public key
    let result = engine.verify(&signed, &record.pgp_data).unwrap();
    assert!(result.valid);
    assert!(result.signer_fingerprint.is_some());
}

#[test]
fn test_verify_wrong_key_fails() {
    let (engine, keyring, _tmp) = setup();

    let fp_signer = generate_and_store(&engine, &keyring, "Signer", "signer@test.com");
    let fp_other = generate_and_store(&engine, &keyring, "Other", "other@test.com");

    let signer_sk = keyring.get_secret_key(&fp_signer).unwrap();
    let other_record = keyring.get_key(&fp_other).unwrap().unwrap();

    let signed = engine
        .sign(b"authentic", signer_sk.expose_secret(), None)
        .unwrap();

    // Verify with the WRONG key should fail
    let result = engine.verify(&signed, &other_record.pgp_data);
    match result {
        Ok(r) => assert!(!r.valid),
        Err(_) => {} // verification error is also acceptable
    }
}

#[test]
fn test_search_keys() {
    let (engine, keyring, _tmp) = setup();

    generate_and_store(&engine, &keyring, "Alice Johnson", "alice@example.com");
    generate_and_store(&engine, &keyring, "Bob Smith", "bob@example.com");

    // Search by name
    let results = keyring.search_keys("Alice").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name.as_deref(), Some("Alice Johnson"));

    // Search by email
    let results = keyring.search_keys("bob@").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].email.as_deref(), Some("bob@example.com"));

    // Search returns nothing for unknown
    let results = keyring.search_keys("charlie").unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_delete_key() {
    let (engine, keyring, _tmp) = setup();

    let fp = generate_and_store(&engine, &keyring, "Delete Me", "delete@test.com");

    assert!(keyring.get_key(&fp).unwrap().is_some());
    assert!(keyring.delete_key(&fp).unwrap());
    assert!(keyring.get_key(&fp).unwrap().is_none());
}

#[test]
fn test_import_public_key() {
    let (engine, keyring, _tmp) = setup();

    // Generate a key pair (simulating an external key)
    let options = KeyGenOptions::new(UserId::new("External", "external@test.com"));
    let key_pair = engine.generate_key_pair(options).unwrap();
    let info = engine.inspect_key(&key_pair.public_key).unwrap();

    // Import only the public key
    let record = KeyRecord {
        fingerprint: key_pair.fingerprint.0.clone(),
        name: info.name().map(String::from),
        email: info.email().map(String::from),
        algorithm: info.algorithm.to_string(),
        created_at: info.created_at,
        expires_at: info.expires_at,
        trust_level: 1,
        is_own_key: false,
        pgp_data: key_pair.public_key.clone(),
    };

    keyring.import_public_key(record).unwrap();

    let keys = keyring.list_keys().unwrap();
    assert_eq!(keys.len(), 1);
    assert!(!keys[0].is_own_key);
    assert_eq!(keys[0].trust_level, 1);

    // Should NOT have a secret key
    assert!(!keyring.has_secret_key(&key_pair.fingerprint.0));
}

#[test]
fn test_inspect_key_metadata() {
    let engine = SequoiaEngine::new();
    let options = KeyGenOptions::new(UserId::new("Test User", "test@inspect.com"));
    let key_pair = engine.generate_key_pair(options).unwrap();

    let info = engine.inspect_key(&key_pair.public_key).unwrap();
    assert_eq!(info.fingerprint.0, key_pair.fingerprint.0);
    assert_eq!(info.name(), Some("Test User"));
    assert_eq!(info.email(), Some("test@inspect.com"));
    assert!(!info.has_secret_key);
    assert!(!info.created_at.is_empty());

    // Secret key inspection
    let secret_info = engine
        .inspect_key(key_pair.secret_key.expose_secret())
        .unwrap();
    assert!(secret_info.has_secret_key);
    assert_eq!(secret_info.fingerprint.0, key_pair.fingerprint.0);
}

#[test]
fn test_passphrase_protected_key() {
    let engine = SequoiaEngine::new();
    let options = KeyGenOptions::new(UserId::new("Protected", "protected@test.com"))
        .with_passphrase(secrecy::SecretBox::new(Box::new(b"hunter2".to_vec())));

    let key_pair = engine.generate_key_pair(options).unwrap();

    let plaintext = b"Secret data protected by passphrase";

    // Encrypt to the key
    let ciphertext = engine
        .encrypt(plaintext, &[key_pair.public_key.clone()])
        .unwrap();

    // Decrypt WITH passphrase should succeed
    let decrypted = engine
        .decrypt(
            &ciphertext,
            key_pair.secret_key.expose_secret(),
            Some(b"hunter2"),
        )
        .unwrap();
    assert_eq!(decrypted, plaintext);

    // Decrypt WITHOUT passphrase should fail
    let result = engine.decrypt(&ciphertext, key_pair.secret_key.expose_secret(), None);
    assert!(result.is_err());

    // Decrypt with WRONG passphrase should fail
    let result = engine.decrypt(
        &ciphertext,
        key_pair.secret_key.expose_secret(),
        Some(b"wrong"),
    );
    assert!(result.is_err());
}

#[test]
fn test_sign_with_passphrase() {
    let engine = SequoiaEngine::new();
    let options = KeyGenOptions::new(UserId::new("ProtSigner", "protsigner@test.com"))
        .with_passphrase(secrecy::SecretBox::new(Box::new(b"signpass".to_vec())));

    let key_pair = engine.generate_key_pair(options).unwrap();

    let data = b"Signed with passphrase-protected key";

    // Sign WITH correct passphrase
    let signed = engine
        .sign(data, key_pair.secret_key.expose_secret(), Some(b"signpass"))
        .unwrap();

    // Verify
    let result = engine.verify(&signed, &key_pair.public_key).unwrap();
    assert!(result.valid);

    // Sign WITHOUT passphrase should fail
    let result = engine.sign(data, key_pair.secret_key.expose_secret(), None);
    assert!(result.is_err());
}

#[test]
fn test_full_pipeline_generate_store_encrypt_sign_verify_decrypt() {
    let (engine, keyring, _tmp) = setup();

    // Generate Alice (sender) and Bob (recipient)
    let fp_alice = generate_and_store(&engine, &keyring, "Alice", "alice@pipeline.com");
    let fp_bob = generate_and_store(&engine, &keyring, "Bob", "bob@pipeline.com");

    let bob_record = keyring.get_key(&fp_bob).unwrap().unwrap();
    let alice_sk = keyring.get_secret_key(&fp_alice).unwrap();

    let message = b"Confidential: Project plans for Q3 2026";

    // Alice encrypts to Bob
    let ciphertext = engine
        .encrypt(message, &[bob_record.pgp_data.clone()])
        .unwrap();

    // Alice signs the ciphertext
    let signed = engine
        .sign(&ciphertext, alice_sk.expose_secret(), None)
        .unwrap();

    // Bob verifies Alice's signature
    let alice_record = keyring.get_key(&fp_alice).unwrap().unwrap();
    let verify_result = engine.verify(&signed, &alice_record.pgp_data).unwrap();
    assert!(verify_result.valid);

    // Bob decrypts
    let bob_sk = keyring.get_secret_key(&fp_bob).unwrap();
    let decrypted = engine
        .decrypt(&ciphertext, bob_sk.expose_secret(), None)
        .unwrap();
    assert_eq!(decrypted, message);
}
