/// Errors that can occur during clipboard operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Clipboard access failed.
    #[error("clipboard error: {reason}")]
    Clipboard { reason: String },

    /// Clipboard is empty.
    #[error("clipboard is empty")]
    Empty,

    /// The clipboard content is not valid UTF-8 text.
    #[error("clipboard content is not valid text")]
    NotText,
}

/// Convenience type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
