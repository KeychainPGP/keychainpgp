//! # keychainpgp-clipboard
//!
//! Clipboard monitoring and management for KeychainPGP.
//!
//! This crate provides:
//! - Cross-platform clipboard read/write operations.
//! - Automatic detection of PGP blocks in clipboard content.
//! - Timed auto-clear of clipboard content after decryption.

pub mod clear;
pub mod detect;
pub mod error;
pub mod monitor;

pub use error::{Error, Result};
