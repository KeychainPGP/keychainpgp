//! # keychainpgp-keys
//!
//! Keyring management for KeychainPGP.
//!
//! This crate handles:
//! - Persistent storage of public keys in a SQLite database.
//! - Secure storage of private keys via OS credential stores
//!   (Windows DPAPI, macOS Keychain, Linux Secret Service).
//! - Key import and export in ASCII-armored format.
//! - Key search by name, email, or fingerprint.

pub mod credential;
pub mod error;
pub mod export;
pub mod import;
pub mod keyring;
pub mod storage;

pub use error::{Error, Result};
pub use keyring::Keyring;
