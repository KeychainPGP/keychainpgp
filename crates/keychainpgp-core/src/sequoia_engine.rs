use std::io::Write;

use sequoia_openpgp::cert::prelude::*;
use sequoia_openpgp::crypto::SessionKey;
use sequoia_openpgp::parse::stream::*;
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_openpgp::serialize::stream::*;
use sequoia_openpgp::serialize::Marshal;
use sequoia_openpgp::types::KeyFlags;
use sequoia_openpgp::Cert;

use secrecy::ExposeSecret;

use crate::engine::CryptoEngine;
use crate::error::{Error, Result};
use crate::types::{Fingerprint, GeneratedKeyPair, KeyAlgorithm, KeyGenOptions};

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

    fn key_fingerprint(&self, public_key: &[u8]) -> Result<String> {
        let cert = Cert::from_bytes(public_key).map_err(|e| Error::InvalidArmor {
            reason: e.to_string(),
        })?;
        Ok(cert.fingerprint().to_hex())
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
        // Try unencrypted secret keys first
        let valid_cert = self.cert.with_policy(self.policy, None)?;

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

        // Generate recipient key pair
        let options = KeyGenOptions::new(UserId::new("Recipient", "recipient@example.com"));
        let key_pair = engine.generate_key_pair(options).unwrap();

        // Encrypt
        let plaintext = b"Hello, this is a secret message!";
        let ciphertext = engine
            .encrypt(plaintext, &[key_pair.public_key.clone()])
            .unwrap();

        assert!(!ciphertext.is_empty());
        assert!(String::from_utf8_lossy(&ciphertext).contains("BEGIN PGP MESSAGE"));

        // Decrypt
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
    fn test_encrypt_no_recipients_fails() {
        let engine = SequoiaEngine::new();
        let result = engine.encrypt(b"hello", &[]);
        assert!(matches!(result, Err(Error::NoRecipients)));
    }

    #[test]
    fn test_key_fingerprint() {
        let engine = SequoiaEngine::new();
        let options = KeyGenOptions::new(UserId::new("Test", "test@test.com"));
        let key_pair = engine.generate_key_pair(options).unwrap();

        let fp = engine.key_fingerprint(&key_pair.public_key).unwrap();
        assert_eq!(fp, key_pair.fingerprint.0);
    }
}
