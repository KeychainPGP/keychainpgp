use crate::error::Result;
use crate::types::{CertInfo, GeneratedKeyPair, KeyGenOptions, VerifyResult};

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

    /// Create a cleartext signature of the given data.
    ///
    /// - `data`: The raw bytes to sign.
    /// - `secret_key`: ASCII-armored secret key.
    /// - `passphrase`: Optional passphrase if the secret key is protected.
    ///
    /// Returns the ASCII-armored cleartext signed message.
    fn sign(
        &self,
        data: &[u8],
        secret_key: &[u8],
        passphrase: Option<&[u8]>,
    ) -> Result<Vec<u8>>;

    /// Verify a cleartext-signed or inline-signed OpenPGP message.
    ///
    /// - `signed_data`: The signed message (cleartext or inline).
    /// - `signer_key`: ASCII-armored public key of the expected signer.
    ///
    /// Returns verification result including validity and signer fingerprint.
    fn verify(
        &self,
        signed_data: &[u8],
        signer_key: &[u8],
    ) -> Result<VerifyResult>;

    /// Parse a key (public or secret) and extract metadata.
    fn inspect_key(&self, key_data: &[u8]) -> Result<CertInfo>;

    /// Encrypt data symmetrically with a passphrase (SKESK).
    ///
    /// Creates an OpenPGP message encrypted with a symmetric key derived from
    /// the passphrase. Used for key sync bundles and encrypted backups.
    fn encrypt_symmetric(&self, plaintext: &[u8], passphrase: &[u8]) -> Result<Vec<u8>>;
}
