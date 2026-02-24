//! Clipboard auto-clear functionality.

use std::time::Duration;

use tokio::sync::watch;
use tracing::{debug, warn};

use crate::error::{Error, Result};

/// Handle to a scheduled clipboard clear operation.
/// Drop this handle to cancel the scheduled clear.
pub struct ClearHandle {
    cancel_tx: watch::Sender<bool>,
}

impl ClearHandle {
    /// Cancel the scheduled clipboard clear.
    pub fn cancel(self) {
        let _ = self.cancel_tx.send(true);
    }
}

/// Schedule a clipboard clear after the given delay.
///
/// Returns a [`ClearHandle`] that can be used to cancel the operation.
pub fn schedule_clear(delay: Duration) -> ClearHandle {
    let (cancel_tx, mut cancel_rx) = watch::channel(false);

    tokio::spawn(async move {
        tokio::select! {
            _ = tokio::time::sleep(delay) => {
                if let Err(e) = clear_clipboard() {
                    warn!("failed to clear clipboard: {e}");
                } else {
                    debug!("clipboard cleared after {}s delay", delay.as_secs());
                }
            }
            _ = cancel_rx.changed() => {
                debug!("clipboard clear cancelled");
            }
        }
    });

    ClearHandle { cancel_tx }
}

/// Immediately clear the clipboard by overwriting with empty content.
///
/// **Limitation:** On Windows, the built-in clipboard history may capture
/// content before this clear runs. Third-party clipboard managers on any
/// platform may also retain copies. This function provides best-effort
/// clearing, not guaranteed data destruction.
pub fn clear_clipboard() -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| Error::Clipboard {
        reason: e.to_string(),
    })?;

    // Overwrite with empty string rather than just clearing,
    // to defeat clipboard history tools.
    clipboard
        .set_text(String::new())
        .map_err(|e| Error::Clipboard {
            reason: e.to_string(),
        })?;

    // Also call clear() to remove the clipboard entry entirely
    clipboard.clear().map_err(|e| Error::Clipboard {
        reason: e.to_string(),
    })?;

    Ok(())
}
