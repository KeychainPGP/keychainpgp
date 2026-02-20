use std::io::Write;

use sequoia_openpgp::cert::prelude::*;
use sequoia_openpgp::crypto::SessionKey;
use sequoia_openpgp::parse::stream::*;
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_openpgp::serialize::stream::*;
use sequoia_openpgp::serialize::Marshal;
use sequoia_openpgp::types::{KeyFlags, PublicKeyAlgorithm};
use sequoia_openpgp::Cert;

use secrecy::ExposeSecret;

use crate::engine::CryptoEngine;
use crate::error::{Error, Result};
use crate::types::{
    CertInfo, Fingerprint, GeneratedKeyPair, KeyAlgorithm, KeyCapability, KeyGenOptions,
    SubkeyInfo, UserId, VerifyResult,
};

/// Sequoia-PGP backed implementation of [`CryptoEngine`].
pub struct SequoiaEngine {
    policy: StandardPolicy<'static>,
}

impl SequoiaEngine {
    /// Create a new `SequoiaEngine` with the standard policy.
    #[must_use]
    pub fn new() -> Self {
        Self {
            policy: StandardPolicy::new(),
        }
    }
}

impl Default for SequoiaEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a Sequoia User ID component value into our UserId type.
fn parse_user_id(uid: &sequoia_openpgp::packet::UserID) -> UserId {
    // Sequoia gives us the raw User ID string, typically "Name <email>"
    let raw = String::from_utf8_lossy(uid.value()).to_string();

    // Try to extract email from angle brackets
    if let (Some(open), Some(close)) = (raw.rfind('<'), raw.rfind('>')) {
        if open < close {
            let email = raw[open + 1..close].trim().to_string();
            let name = raw[..open].trim().to_string();
            return UserId {
                name: if name.is_empty() { None } else { Some(name) },
                email: if email.is_empty() { None } else { Some(email) },
            };
        }
    }

    // If no angle brackets, check if it looks like an email
    if raw.contains('@') {
        UserId {
            name: None,
            email: Some(raw.trim().to_string()),
        }
    } else {
        UserId {
            name: Some(raw.trim().to_string()),
            email: None,
        }
    }
}

/// Map a Sequoia `PublicKeyAlgorithm` to our `KeyAlgorithm`.
fn map_algorithm(algo: PublicKeyAlgorithm, key_size: Option<usize>) -> KeyAlgorithm {
    match algo {
        PublicKeyAlgorithm::EdDSA => KeyAlgorithm::Ed25519,
        PublicKeyAlgorithm::RSAEncryptSign => {
            KeyAlgorithm::Rsa(key_size.unwrap_or(4096) as u32)
        }
        // ECDH/ECDSA with Curve25519 are part of the Ed25519 suite
        PublicKeyAlgorithm::ECDH | PublicKeyAlgorithm::ECDSA => KeyAlgorithm::Ed25519,
        _ => KeyAlgorithm::Ed25519,
    }
}

impl CryptoEngine for SequoiaEngine {
    fn generate_key_pair(&self, options: KeyGenOptions) -> Result<GeneratedKeyPair> {
        let user_id = options.user_id.to_openpgp_string();

        let mut builder = match options.algorithm {
            KeyAlgorithm::Ed25519 => CertBuilder::new()
                .add_userid(user_id)
                .set_cipher_suite(CipherSuite::Cv25519)
                .add_signing_subkey()
                .add_subkey(
                    KeyFlags::empty().set_transport_encryption(),
                    options.expiration,
                    None,
                ),
            KeyAlgorithm::Rsa(bits) => {
                let suite = match bits {
                    3072 => CipherSuite::RSA3k,
                    _ => CipherSuite::RSA4k,
                };
                CertBuilder::new()
                    .add_userid(user_id)
                    .set_cipher_suite(suite)
                    .add_signing_subkey()
                    .add_subkey(
                        KeyFlags::empty().set_transport_encryption(),
                        options.expiration,
                        None,
                    )
            }
        };

        if let Some(expiration) = options.expiration {
            builder = builder.set_validity_period(expiration);
        }

        if let Some(ref passphrase) = options.passphrase {
            builder = builder.set_password(Some(
                sequoia_openpgp::crypto::Password::from(passphrase.expose_secret().as_slice()),
            ));
        }

        let (cert, _revocation) = builder.generate().map_err(|e| Error::KeyGeneration {
            reason: e.to_string(),
        })?;

        let fingerprint = Fingerprint::new(cert.fingerprint().to_hex());

        // Serialize public key (certificate)
        let mut public_key = Vec::new();
        {
            let mut writer =
                sequoia_openpgp::armor::Writer::new(&mut public_key, sequoia_openpgp::armor::Kind::PublicKey)
                    .map_err(|e| Error::KeyGeneration {
                        reason: format!("armor error: {e}"),
                    })?;
            cert.serialize(&mut writer).map_err(|e| Error::KeyGeneration {
                reason: format!("serialize error: {e}"),
            })?;
            writer.finalize().map_err(|e| Error::KeyGeneration {
                reason: format!("finalize error: {e}"),
            })?;
        }

        // Serialize secret key
        let mut secret_key_bytes = Vec::new();
        {
            let mut writer = sequoia_openpgp::armor::Writer::new(
                &mut secret_key_bytes,
                sequoia_openpgp::armor::Kind::SecretKey,
            )
            .map_err(|e| Error::KeyGeneration {
                reason: format!("armor error: {e}"),
            })?;
            cert.as_tsk().serialize(&mut writer).map_err(|e| Error::KeyGeneration {
                reason: format!("serialize error: {e}"),
            })?;
            writer.finalize().map_err(|e| Error::KeyGeneration {
                reason: format!("finalize error: {e}"),
            })?;
        }

        Ok(GeneratedKeyPair {
            public_key,
            secret_key: secrecy::SecretBox::new(Box::new(secret_key_bytes)),
            fingerprint,
        })
    }

    fn encrypt(
        &self,
        plaintext: &[u8],
        recipient_keys: &[Vec<u8>],
    ) -> Result<Vec<u8>> {
        if recipient_keys.is_empty() {
            return Err(Error::NoRecipients);
        }

        let certs: Vec<Cert> = recipient_keys
            .iter()
            .map(|key| {
                Cert::from_bytes(key).map_err(|e| Error::Encryption {
                    reason: format!("invalid recipient key: {e}"),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let mut recipients: Vec<Recipient> = Vec::new();
        for cert in &certs {
            let valid_cert = cert
                .with_policy(&self.policy, None)
                .map_err(|e| Error::Encryption {
                    reason: format!("key policy check failed: {e}"),
                })?;

            for key in valid_cert
                .keys()
                .supported()
                .alive()
                .revoked(false)
                .for_transport_encryption()
                .for_storage_encryption()
            {
                recipients.push(key.into());
            }
        }

        if recipients.is_empty() {
            return Err(Error::Encryption {
                reason: "no valid encryption-capable subkeys found in recipient keys".into(),
            });
        }

        let mut output = Vec::new();
        {
            let mut armored_writer =
                sequoia_openpgp::armor::Writer::new(&mut output, sequoia_openpgp::armor::Kind::Message)
                    .map_err(|e| Error::Encryption {
                        reason: format!("armor error: {e}"),
                    })?;

            let message = Message::new(&mut armored_writer);
            let message = Encryptor2::for_recipients(message, recipients)
                .build()
                .map_err(|e| Error::Encryption {
                    reason: format!("encryptor error: {e}"),
                })?;
            let mut message = LiteralWriter::new(message)
                .build()
                .map_err(|e| Error::Encryption {
                    reason: format!("literal writer error: {e}"),
                })?;

            message.write_all(plaintext).map_err(|e| Error::Encryption {
                reason: format!("write error: {e}"),
            })?;
            message.finalize().map_err(|e| Error::Encryption {
                reason: format!("finalize error: {e}"),
            })?;

            armored_writer.finalize().map_err(|e| Error::Encryption {
                reason: format!("armor finalize error: {e}"),
            })?;
        }

        Ok(output)
    }

    fn decrypt(
        &self,
        ciphertext: &[u8],
        secret_key: &[u8],
        passphrase: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let cert = Cert::from_bytes(secret_key).map_err(|e| Error::Decryption {
            reason: format!("invalid secret key: {e}"),
        })?;

        let helper = DecryptHelper {
            policy: &self.policy,
            cert,
            passphrase,
        };

        let mut decryptor = DecryptorBuilder::from_bytes(ciphertext)
            .map_err(|e| Error::Decryption {
                reason: format!("invalid ciphertext: {e}"),
            })?
            .with_policy(&self.policy, None, helper)
            .map_err(|e| Error::Decryption {
                reason: format!("decryption failed: {e}"),
            })?;

        let mut plaintext = Vec::new();
        std::io::copy(&mut decryptor, &mut plaintext).map_err(|e| Error::Decryption {
            reason: format!("read error: {e}"),
        })?;

        Ok(plaintext)
    }

    fn sign(
        &self,
        data: &[u8],
        secret_key: &[u8],
        passphrase: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let cert = Cert::from_bytes(secret_key).map_err(|e| Error::Signing {
            reason: format!("invalid secret key: {e}"),
        })?;

        let valid_cert = cert.with_policy(&self.policy, None).map_err(|e| Error::Signing {
            reason: format!("key policy check failed: {e}"),
        })?;

        // Find a signing-capable secret key
        let mut keypair = None;

        // Try unencrypted secret keys first
        for ka in valid_cert.keys().supported().alive().revoked(false).for_signing().unencrypted_secret() {
            keypair = Some(ka.key().clone().into_keypair().map_err(|e| Error::Signing {
                reason: format!("keypair conversion failed: {e}"),
            })?);
            break;
        }

        // Try with passphrase
        if keypair.is_none() {
            if let Some(passphrase) = passphrase {
                let password = sequoia_openpgp::crypto::Password::from(passphrase);
                for ka in valid_cert.keys().supported().alive().revoked(false).for_signing().secret() {
                    let key = ka.key().clone();
                    if let Ok(decrypted) = key.decrypt_secret(&password) {
                        if let Ok(kp) = decrypted.into_keypair() {
                            keypair = Some(kp);
                            break;
                        }
                    }
                }
            }
        }

        let signer_keypair = keypair.ok_or_else(|| Error::Signing {
            reason: "no signing-capable secret key found".into(),
        })?;

        let mut output = Vec::new();
        {
            let mut armored_writer =
                sequoia_openpgp::armor::Writer::new(&mut output, sequoia_openpgp::armor::Kind::Message)
                    .map_err(|e| Error::Signing {
                        reason: format!("armor error: {e}"),
                    })?;

            let message = Message::new(&mut armored_writer);
            let message = Signer::new(message, signer_keypair)
                .build()
                .map_err(|e| Error::Signing {
                    reason: format!("signer error: {e}"),
                })?;
            let mut message = LiteralWriter::new(message)
                .build()
                .map_err(|e| Error::Signing {
                    reason: format!("literal writer error: {e}"),
                })?;

            message.write_all(data).map_err(|e| Error::Signing {
                reason: format!("write error: {e}"),
            })?;
            message.finalize().map_err(|e| Error::Signing {
                reason: format!("finalize error: {e}"),
            })?;

            armored_writer.finalize().map_err(|e| Error::Signing {
                reason: format!("armor finalize error: {e}"),
            })?;
        }

        Ok(output)
    }

    fn verify(
        &self,
        signed_data: &[u8],
        signer_key: &[u8],
    ) -> Result<VerifyResult> {
        let signer_cert = Cert::from_bytes(signer_key).map_err(|e| Error::VerificationFailed {
            reason: format!("invalid signer key: {e}"),
        })?;

        let signer_fp = signer_cert.fingerprint().to_hex();

        let helper = VerifyHelper {
            policy: &self.policy,
            cert: signer_cert,
            result: None,
        };

        let mut verifier = VerifierBuilder::from_bytes(signed_data)
            .map_err(|e| Error::VerificationFailed {
                reason: format!("invalid signed data: {e}"),
            })?
            .with_policy(&self.policy, None, helper)
            .map_err(|e| Error::VerificationFailed {
                reason: format!("verification setup failed: {e}"),
            })?;

        // Consume the verified content
        let mut content = Vec::new();
        std::io::copy(&mut verifier, &mut content).map_err(|e| Error::VerificationFailed {
            reason: format!("read error: {e}"),
        })?;

        let helper = verifier.into_helper();

        Ok(helper.result.unwrap_or(VerifyResult {
            valid: false,
            signer_fingerprint: Some(signer_fp),
        }))
    }

    fn inspect_key(&self, key_data: &[u8]) -> Result<CertInfo> {
        let cert = Cert::from_bytes(key_data).map_err(|e| Error::InvalidArmor {
            reason: e.to_string(),
        })?;

        let fingerprint = Fingerprint::new(cert.fingerprint().to_hex());

        // Extract User IDs
        let user_ids: Vec<UserId> = cert.userids().map(|uid| parse_user_id(uid.userid())).collect();

        // Determine algorithm from primary key
        let pk_algo = cert.primary_key().pk_algo();
        let key_size = cert.primary_key().mpis().bits();
        let algorithm = map_algorithm(pk_algo, key_size);

        // Creation time
        let created_at = {
            let ct = cert.primary_key().creation_time();
            chrono::DateTime::<chrono::Utc>::from(ct).to_rfc3339()
        };

        // Expiration time
        let expires_at = cert
            .with_policy(&self.policy, None)
            .ok()
            .and_then(|valid_cert| valid_cert.primary_key().key_expiration_time())
            .map(|et| chrono::DateTime::<chrono::Utc>::from(et).to_rfc3339());

        // Check for secret key material
        let has_secret_key = cert.is_tsk();

        // Extract subkey information
        let subkeys = cert
            .with_policy(&self.policy, None)
            .ok()
            .map(|valid_cert| {
                valid_cert
                    .keys()
                    .subkeys()
                    .map(|ka| {
                        let sk_fp = ka.fingerprint().to_hex();
                        let sk_algo = ka.pk_algo();
                        let sk_size = ka.mpis().bits();
                        let sk_algorithm = map_algorithm(sk_algo, sk_size);
                        let sk_created = {
                            let ct = ka.creation_time();
                            chrono::DateTime::<chrono::Utc>::from(ct).to_rfc3339()
                        };
                        let sk_expires = ka.key_expiration_time()
                            .map(|et| chrono::DateTime::<chrono::Utc>::from(et).to_rfc3339());

                        let mut capabilities = Vec::new();
                        if ka.for_signing() {
                            capabilities.push(KeyCapability::Sign);
                        }
                        if ka.for_transport_encryption() || ka.for_storage_encryption() {
                            capabilities.push(KeyCapability::Encrypt);
                        }
                        if ka.for_certification() {
                            capabilities.push(KeyCapability::Certify);
                        }
                        if ka.for_authentication() {
                            capabilities.push(KeyCapability::Authenticate);
                        }

                        let is_revoked = ka.revocation_status() != sequoia_openpgp::types::RevocationStatus::NotAsFarAsWeKnow;

                        SubkeyInfo {
                            fingerprint: sk_fp,
                            algorithm: sk_algorithm.to_string(),
                            created_at: sk_created,
                            expires_at: sk_expires,
                            capabilities,
                            is_revoked,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(CertInfo {
            fingerprint,
            user_ids,
            algorithm,
            created_at,
            expires_at,
            has_secret_key,
            subkeys,
        })
    }
}

/// Helper struct for the Sequoia decryption streaming API.
struct DecryptHelper<'a> {
    policy: &'a StandardPolicy<'static>,
    cert: Cert,
    passphrase: Option<&'a [u8]>,
}

impl VerificationHelper for DecryptHelper<'_> {
    fn get_certs(
        &mut self,
        _ids: &[sequoia_openpgp::KeyHandle],
    ) -> sequoia_openpgp::Result<Vec<Cert>> {
        Ok(Vec::new())
    }

    fn check(&mut self, _structure: MessageStructure) -> sequoia_openpgp::Result<()> {
        Ok(())
    }
}

impl DecryptionHelper for DecryptHelper<'_> {
    fn decrypt<D>(
        &mut self,
        pkesks: &[sequoia_openpgp::packet::PKESK],
        _skesks: &[sequoia_openpgp::packet::SKESK],
        sym_algo: Option<sequoia_openpgp::types::SymmetricAlgorithm>,
        mut decrypt: D,
    ) -> sequoia_openpgp::Result<Option<sequoia_openpgp::Fingerprint>>
    where
        D: FnMut(sequoia_openpgp::types::SymmetricAlgorithm, &SessionKey) -> bool,
    {
        let valid_cert = self.cert.with_policy(self.policy, None)?;

        // Try unencrypted secret keys first
        for ka in valid_cert
            .keys()
            .supported()
            .unencrypted_secret()
            .for_transport_encryption()
            .for_storage_encryption()
        {
            let mut keypair = ka.key().clone().into_keypair()?;
            for pkesk in pkesks {
                if pkesk
                    .decrypt(&mut keypair, sym_algo)
                    .map(|(algo, sk)| decrypt(algo, &sk))
                    .unwrap_or(false)
                {
                    return Ok(None);
                }
            }
        }

        // Try with passphrase-decrypted keys
        if let Some(passphrase) = self.passphrase {
            let password = sequoia_openpgp::crypto::Password::from(passphrase);

            for ka in valid_cert
                .keys()
                .supported()
                .secret()
                .for_transport_encryption()
                .for_storage_encryption()
            {
                let key = ka.key().clone();
                if let Ok(decrypted) = key.decrypt_secret(&password) {
                    if let Ok(mut keypair) = decrypted.into_keypair() {
                        for pkesk in pkesks {
                            if pkesk
                                .decrypt(&mut keypair, sym_algo)
                                .map(|(algo, sk)| decrypt(algo, &sk))
                                .unwrap_or(false)
                            {
                                return Ok(None);
                            }
                        }
                    }
                }
            }
        }

        Err(sequoia_openpgp::Error::MissingSessionKey(
            "no suitable decryption key found".into(),
        )
        .into())
    }
}

/// Helper struct for the Sequoia signature verification streaming API.
struct VerifyHelper<'a> {
    #[allow(dead_code)]
    policy: &'a StandardPolicy<'static>,
    cert: Cert,
    result: Option<VerifyResult>,
}

impl VerificationHelper for VerifyHelper<'_> {
    fn get_certs(
        &mut self,
        _ids: &[sequoia_openpgp::KeyHandle],
    ) -> sequoia_openpgp::Result<Vec<Cert>> {
        Ok(vec![self.cert.clone()])
    }

    fn check(&mut self, structure: MessageStructure) -> sequoia_openpgp::Result<()> {
        for layer in structure {
            match layer {
                MessageLayer::SignatureGroup { results } => {
                    for result in &results {
                        match result {
                            Ok(GoodChecksum { ka, .. }) => {
                                self.result = Some(VerifyResult {
                                    valid: true,
                                    signer_fingerprint: Some(ka.cert().fingerprint().to_hex()),
                                });
                                return Ok(());
                            }
                            Err(_) => {}
                        }
                    }
                    // No good signature found
                    self.result = Some(VerifyResult {
                        valid: false,
                        signer_fingerprint: Some(self.cert.fingerprint().to_hex()),
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{KeyGenOptions, UserId};

    #[test]
    fn test_generate_ed25519_key_pair() {
        let engine = SequoiaEngine::new();
        let options = KeyGenOptions::new(UserId::new("Test User", "test@example.com"));
        let result = engine.generate_key_pair(options);
        assert!(result.is_ok());

        let key_pair = result.unwrap();
        assert!(!key_pair.public_key.is_empty());
        assert!(!key_pair.fingerprint.0.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let engine = SequoiaEngine::new();

        let options = KeyGenOptions::new(UserId::new("Recipient", "recipient@example.com"));
        let key_pair = engine.generate_key_pair(options).unwrap();

        let plaintext = b"Hello, this is a secret message!";
        let ciphertext = engine
            .encrypt(plaintext, &[key_pair.public_key.clone()])
            .unwrap();

        assert!(!ciphertext.is_empty());
        assert!(String::from_utf8_lossy(&ciphertext).contains("BEGIN PGP MESSAGE"));

        let decrypted = engine
            .decrypt(
                &ciphertext,
                key_pair.secret_key.expose_secret(),
                None,
            )
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_multiple_recipients() {
        let engine = SequoiaEngine::new();

        let kp1 = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Alice", "alice@example.com")))
            .unwrap();
        let kp2 = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Bob", "bob@example.com")))
            .unwrap();

        let plaintext = b"Message for both Alice and Bob";
        let ciphertext = engine
            .encrypt(plaintext, &[kp1.public_key.clone(), kp2.public_key.clone()])
            .unwrap();

        // Both recipients should be able to decrypt
        let dec1 = engine
            .decrypt(&ciphertext, kp1.secret_key.expose_secret(), None)
            .unwrap();
        assert_eq!(dec1, plaintext);

        let dec2 = engine
            .decrypt(&ciphertext, kp2.secret_key.expose_secret(), None)
            .unwrap();
        assert_eq!(dec2, plaintext);
    }

    #[test]
    fn test_encrypt_no_recipients_fails() {
        let engine = SequoiaEngine::new();
        let result = engine.encrypt(b"hello", &[]);
        assert!(matches!(result, Err(Error::NoRecipients)));
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let engine = SequoiaEngine::new();

        let sender = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Sender", "sender@example.com")))
            .unwrap();
        let wrong = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Wrong", "wrong@example.com")))
            .unwrap();

        let ciphertext = engine
            .encrypt(b"secret", &[sender.public_key.clone()])
            .unwrap();

        let result = engine.decrypt(&ciphertext, wrong.secret_key.expose_secret(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_and_verify() {
        let engine = SequoiaEngine::new();

        let kp = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Signer", "signer@example.com")))
            .unwrap();

        let data = b"This message is signed by me.";
        let signed = engine.sign(data, kp.secret_key.expose_secret(), None).unwrap();

        assert!(!signed.is_empty());
        assert!(String::from_utf8_lossy(&signed).contains("BEGIN PGP MESSAGE"));

        let result = engine.verify(&signed, &kp.public_key).unwrap();
        assert!(result.valid);
        assert!(result.signer_fingerprint.is_some());
    }

    #[test]
    fn test_verify_tampered_fails() {
        let engine = SequoiaEngine::new();

        let kp = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Signer", "signer@example.com")))
            .unwrap();
        let wrong = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Other", "other@example.com")))
            .unwrap();

        let signed = engine.sign(b"authentic", kp.secret_key.expose_secret(), None).unwrap();

        // Verify with the wrong key should show invalid
        let result = engine.verify(&signed, &wrong.public_key);
        // This either errors out or returns valid=false
        match result {
            Ok(r) => assert!(!r.valid),
            Err(_) => {} // verification failure is also acceptable
        }
    }

    #[test]
    fn test_inspect_key() {
        let engine = SequoiaEngine::new();

        let kp = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Alice Johnson", "alice@example.com")))
            .unwrap();

        // Inspect public key
        let info = engine.inspect_key(&kp.public_key).unwrap();
        assert_eq!(info.fingerprint.0, kp.fingerprint.0);
        assert_eq!(info.name(), Some("Alice Johnson"));
        assert_eq!(info.email(), Some("alice@example.com"));
        assert!(!info.has_secret_key);
        assert!(!info.created_at.is_empty());

        // Inspect secret key
        let secret_info = engine.inspect_key(kp.secret_key.expose_secret()).unwrap();
        assert!(secret_info.has_secret_key);
        assert_eq!(secret_info.fingerprint.0, kp.fingerprint.0);
    }

    #[test]
    fn test_inspect_key_extracts_expiration() {
        let engine = SequoiaEngine::new();

        let kp = engine
            .generate_key_pair(KeyGenOptions::new(UserId::new("Expiry Test", "exp@test.com")))
            .unwrap();

        let info = engine.inspect_key(&kp.public_key).unwrap();
        // Default key gen has 2-year expiration
        assert!(info.expires_at.is_some());
    }

    #[test]
    fn test_key_fingerprint() {
        let engine = SequoiaEngine::new();
        let options = KeyGenOptions::new(UserId::new("Test", "test@test.com"));
        let key_pair = engine.generate_key_pair(options).unwrap();

        let info = engine.inspect_key(&key_pair.public_key).unwrap();
        assert_eq!(info.fingerprint.0, key_pair.fingerprint.0);
    }
}
