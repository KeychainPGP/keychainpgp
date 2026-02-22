/// Checks whether the given bytes contain an ASCII-armored PGP message.
#[must_use]
pub fn is_pgp_message(data: &[u8]) -> bool {
    let text = String::from_utf8_lossy(data);
    text.contains("-----BEGIN PGP MESSAGE-----")
}

/// Checks whether the given bytes contain an ASCII-armored PGP public key block.
#[must_use]
pub fn is_pgp_public_key(data: &[u8]) -> bool {
    let text = String::from_utf8_lossy(data);
    text.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----")
}

/// Checks whether the given bytes contain an ASCII-armored PGP private key block.
#[must_use]
pub fn is_pgp_private_key(data: &[u8]) -> bool {
    let text = String::from_utf8_lossy(data);
    text.contains("-----BEGIN PGP PRIVATE KEY BLOCK-----")
}

/// Checks whether the given bytes contain a cleartext-signed PGP message.
#[must_use]
pub fn is_cleartext_signed(data: &[u8]) -> bool {
    let text = String::from_utf8_lossy(data);
    text.contains("-----BEGIN PGP SIGNED MESSAGE-----")
}

/// Checks whether the given bytes contain any recognized PGP ASCII armor.
#[must_use]
pub fn is_pgp_armored(data: &[u8]) -> bool {
    is_pgp_message(data)
        || is_pgp_public_key(data)
        || is_pgp_private_key(data)
        || is_cleartext_signed(data)
}

/// The type of PGP block detected in the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PgpBlockKind {
    Message,
    PublicKey,
    PrivateKey,
    Signature,
    SignedMessage,
}

/// Detect what kind of PGP block is present in the given data.
#[must_use]
pub fn detect_pgp_block(data: &[u8]) -> Option<PgpBlockKind> {
    let text = String::from_utf8_lossy(data);
    // Check cleartext signed first — it contains both SIGNED MESSAGE and SIGNATURE headers
    if text.contains("-----BEGIN PGP SIGNED MESSAGE-----") {
        Some(PgpBlockKind::SignedMessage)
    } else if text.contains("-----BEGIN PGP MESSAGE-----") {
        Some(PgpBlockKind::Message)
    } else if text.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----") {
        Some(PgpBlockKind::PublicKey)
    } else if text.contains("-----BEGIN PGP PRIVATE KEY BLOCK-----") {
        Some(PgpBlockKind::PrivateKey)
    } else if text.contains("-----BEGIN PGP SIGNATURE-----") {
        Some(PgpBlockKind::Signature)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pgp_message() {
        let data = b"-----BEGIN PGP MESSAGE-----\ndata\n-----END PGP MESSAGE-----";
        assert!(is_pgp_message(data));
        assert_eq!(detect_pgp_block(data), Some(PgpBlockKind::Message));
    }

    #[test]
    fn test_detect_pgp_public_key() {
        let data =
            b"-----BEGIN PGP PUBLIC KEY BLOCK-----\ndata\n-----END PGP PUBLIC KEY BLOCK-----";
        assert!(is_pgp_public_key(data));
        assert_eq!(detect_pgp_block(data), Some(PgpBlockKind::PublicKey));
    }

    #[test]
    fn test_plain_text_not_detected() {
        let data = b"Hello, this is just plain text.";
        assert!(!is_pgp_armored(data));
        assert_eq!(detect_pgp_block(data), None);
    }
}
