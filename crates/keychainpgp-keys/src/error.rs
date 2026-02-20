/// Errors that can occur during keyring operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// OS credential store error.
    #[error("credential store error: {reason}")]
    CredentialStore { reason: String },

    /// Key not found in the keyring.
    #[error("key not found: {fingerprint}")]
    KeyNotFound { fingerprint: String },

    /// Duplicate key already exists.
    #[error("key already exists: {fingerprint}")]
    DuplicateKey { fingerprint: String },

    /// Invalid key data.
    #[error("invalid key data: {reason}")]
    InvalidKey { reason: String },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Core crypto error.
    #[error("crypto error: {0}")]
    Core(#[from] keychainpgp_core::Error),
}

/// Convenience type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
