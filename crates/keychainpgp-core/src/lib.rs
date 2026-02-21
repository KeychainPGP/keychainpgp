//! # keychainpgp-core
//!
//! Core OpenPGP cryptographic operations for KeychainPGP.
//!
//! This crate provides the foundational cryptographic functionality:
//! - Key generation (Ed25519 signing + X25519 encryption)
//! - Message encryption and decryption
//! - Message signing and signature verification
//! - Key inspection (metadata extraction)
//! - ASCII armor serialization/deserialization
//!
//! All operations are abstracted behind the [`CryptoEngine`] trait,
//! with a concrete implementation backed by Sequoia-PGP.
//!
//! This crate performs no I/O. All functions operate on in-memory data.

pub mod armor;
pub mod engine;
pub mod error;
pub mod sequoia_engine;
pub mod types;

pub use engine::CryptoEngine;
pub use error::{Error, Result};
pub use sequoia_engine::SequoiaEngine;

/// Fill a buffer with cryptographically secure random bytes.
///
/// Wraps the Sequoia-PGP random number generator.
pub fn crypto_random(buf: &mut [u8]) {
    sequoia_openpgp::crypto::random(buf);
}
