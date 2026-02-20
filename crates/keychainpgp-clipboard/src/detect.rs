//! PGP block detection in clipboard content.

use keychainpgp_core::armor::{self, PgpBlockKind};

/// Check the clipboard text and return what kind of PGP block it contains, if any.
#[must_use]
pub fn detect_pgp_content(text: &str) -> Option<PgpBlockKind> {
    armor::detect_pgp_block(text.as_bytes())
}

/// Returns `true` if the text contains an encrypted PGP message.
#[must_use]
pub fn is_encrypted_message(text: &str) -> bool {
    detect_pgp_content(text) == Some(PgpBlockKind::Message)
}

/// Returns `true` if the text contains a PGP public key block.
#[must_use]
pub fn is_public_key(text: &str) -> bool {
    detect_pgp_content(text) == Some(PgpBlockKind::PublicKey)
}
