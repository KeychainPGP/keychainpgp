use crate::error::Result;
use crate::types::{GeneratedKeyPair, KeyGenOptions};

/// Trait abstracting all OpenPGP cryptographic operations.
///
/// This allows the crypto backend to be swapped (e.g. for testing with
/// a mock implementation) without affecting the rest of the application.
pub trait CryptoEngine: Send + Sync {
    /// Generate a new OpenPGP key pair.
    ///
    /// Returns the generated key pair containing both public and secret
    /// key material in ASCII-armored form.
    fn generate_key_pair(&self, options: KeyGenOptions) -> Result<GeneratedKeyPair>;

    /// Encrypt plaintext for the given recipients.
    ///
    /// - `plaintext`: The raw message bytes to encrypt.
    /// - `recipient_keys`: ASCII-armored public keys of the recipients.
    ///
    /// Returns the ASCII-armored OpenPGP encrypted message.
    fn encrypt(
        &self,
        plaintext: &[u8],
        recipient_keys: &[Vec<u8>],
    ) -> Result<Vec<u8>>;

    /// Decrypt an OpenPGP message using the provided secret key.
    ///
    /// - `ciphertext`: ASCII-armored (or binary) OpenPGP message.
    /// - `secret_key`: ASCII-armored secret key.
    /// - `passphrase`: Optional passphrase if the secret key is protected.
    ///
    /// Returns the decrypted plaintext bytes.
    fn decrypt(
        &self,
        ciphertext: &[u8],
        secret_key: &[u8],
        passphrase: Option<&[u8]>,
    ) -> Result<Vec<u8>>;

    /// Parse an ASCII-armored public key and return its fingerprint.
    fn key_fingerprint(&self, public_key: &[u8]) -> Result<String>;
}
