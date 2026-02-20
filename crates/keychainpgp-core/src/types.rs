use std::fmt;

use secrecy::SecretBox;
use zeroize::ZeroizeOnDrop;

/// A generated OpenPGP key pair containing both the public certificate
/// and the secret key material.
#[derive(ZeroizeOnDrop)]
pub struct GeneratedKeyPair {
    /// ASCII-armored public key (certificate).
    #[zeroize(skip)]
    pub public_key: Vec<u8>,

    /// ASCII-armored secret key.
    pub secret_key: SecretBox<Vec<u8>>,

    /// Human-readable fingerprint of the primary key.
    #[zeroize(skip)]
    pub fingerprint: Fingerprint,
}

/// An OpenPGP key fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint(pub String);

impl Fingerprint {
    /// Create a new fingerprint from a hex string.
    #[must_use]
    pub fn new(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// Return the fingerprint as a grouped hex string for display.
    /// Example: `"7A3F 9B2C 4D1E 8F05"`
    #[must_use]
    pub fn display_grouped(&self) -> String {
        self.0
            .chars()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Return the last 16 hex characters (short form).
    #[must_use]
    pub fn short(&self) -> &str {
        let len = self.0.len();
        if len >= 16 {
            &self.0[len - 16..]
        } else {
            &self.0
        }
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_grouped())
    }
}

/// A User ID associated with an OpenPGP key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserId {
    /// Display name (e.g. "Alice Johnson").
    pub name: Option<String>,

    /// Email address (e.g. "alice@example.com").
    pub email: Option<String>,
}

impl UserId {
    /// Create a User ID with both name and email.
    #[must_use]
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            email: Some(email.into()),
        }
    }

    /// Format as an OpenPGP User ID string: `"Name <email>"`.
    #[must_use]
    pub fn to_openpgp_string(&self) -> String {
        match (&self.name, &self.email) {
            (Some(name), Some(email)) => format!("{name} <{email}>"),
            (Some(name), None) => name.clone(),
            (None, Some(email)) => format!("<{email}>"),
            (None, None) => String::new(),
        }
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_openpgp_string())
    }
}

/// The algorithm used by a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAlgorithm {
    /// Ed25519 for signing, X25519 for encryption (modern default).
    Ed25519,
    /// RSA with the given bit size.
    Rsa(u32),
}

impl fmt::Display for KeyAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519 => write!(f, "Modern (Ed25519)"),
            Self::Rsa(bits) => write!(f, "Classic (RSA-{bits})"),
        }
    }
}

/// Trust level for a key in the keyring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    /// Trust has not been established.
    Unknown,
    /// Key was imported but not verified out-of-band.
    Unverified,
    /// Key has been verified by the user (e.g. fingerprint comparison).
    Verified,
}

impl fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Unverified => write!(f, "Unverified"),
            Self::Verified => write!(f, "Verified"),
        }
    }
}

/// Options for key generation.
pub struct KeyGenOptions {
    /// The user identity to bind to the key.
    pub user_id: UserId,

    /// Optional passphrase to protect the private key.
    pub passphrase: Option<SecretBox<Vec<u8>>>,

    /// Key algorithm (defaults to Ed25519).
    pub algorithm: KeyAlgorithm,

    /// Expiration duration from now. `None` means no expiration.
    pub expiration: Option<std::time::Duration>,
}

impl KeyGenOptions {
    /// Create default key generation options with Ed25519 and 2-year expiration.
    #[must_use]
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            passphrase: None,
            algorithm: KeyAlgorithm::Ed25519,
            expiration: Some(std::time::Duration::from_secs(2 * 365 * 24 * 60 * 60)),
        }
    }

    /// Set the passphrase for the generated key.
    #[must_use]
    pub fn with_passphrase(mut self, passphrase: SecretBox<Vec<u8>>) -> Self {
        self.passphrase = Some(passphrase);
        self
    }

    /// Set the key algorithm.
    #[must_use]
    pub fn with_algorithm(mut self, algorithm: KeyAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Set the expiration duration.
    #[must_use]
    pub fn with_expiration(mut self, expiration: Option<std::time::Duration>) -> Self {
        self.expiration = expiration;
        self
    }
}
