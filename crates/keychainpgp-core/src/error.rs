/// Errors that can occur during cryptographic operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Key generation failed.
    #[error("failed to generate key pair: {reason}")]
    KeyGeneration { reason: String },

    /// Encryption failed.
    #[error("failed to encrypt message: {reason}")]
    Encryption { reason: String },

    /// Decryption failed.
    #[error("failed to decrypt message: {reason}")]
    Decryption { reason: String },

    /// No suitable private key was found for decryption.
    #[error("no private key found that can decrypt this message")]
    NoSecretKey,

    /// The provided passphrase was incorrect.
    #[error("incorrect passphrase")]
    BadPassphrase,

    /// Signing failed.
    #[error("failed to sign data: {reason}")]
    Signing { reason: String },

    /// Signature verification failed.
    #[error("signature verification failed: {reason}")]
    VerificationFailed { reason: String },

    /// ASCII armor parsing failed.
    #[error("invalid ASCII armor: {reason}")]
    InvalidArmor { reason: String },

    /// A key has expired.
    #[error("key expired on {expiration}")]
    KeyExpired { expiration: String },

    /// A key has been revoked.
    #[error("key has been revoked")]
    KeyRevoked,

    /// No recipients were specified for encryption.
    #[error("no recipients specified")]
    NoRecipients,

    /// An internal error in the underlying crypto library.
    #[error("internal crypto error: {0}")]
    Internal(String),
}

/// Convenience type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
