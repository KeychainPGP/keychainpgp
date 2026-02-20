//! Clipboard monitoring daemon.
//!
//! Watches the system clipboard for changes and notifies when
//! PGP-armored content is detected.

use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, trace};

use keychainpgp_core::armor::PgpBlockKind;

use crate::detect;
use crate::error::{Error, Result};

/// Events emitted by the clipboard monitor.
#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    /// A PGP block was detected in the clipboard.
    PgpDetected {
        kind: PgpBlockKind,
        content: String,
    },
    /// The clipboard content changed to non-PGP text.
    TextChanged {
        preview: String,
        length: usize,
    },
    /// The clipboard was cleared or is empty.
    Empty,
}

/// Configuration for the clipboard monitor.
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// How often to poll the clipboard (default: 500ms).
    pub poll_interval: Duration,
    /// Maximum length of text preview in events (default: 100 chars).
    pub preview_max_len: usize,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(500),
            preview_max_len: 100,
        }
    }
}

/// Start the clipboard monitor. Returns a receiver for clipboard events
/// and a handle to stop the monitor.
pub fn start_monitor(
    config: MonitorConfig,
) -> (mpsc::Receiver<ClipboardEvent>, MonitorHandle) {
    let (tx, rx) = mpsc::channel(32);
    let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

    tokio::spawn(async move {
        let mut last_content = String::new();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(config.poll_interval) => {
                    match read_clipboard_text() {
                        Ok(Some(text)) => {
                            if text != last_content {
                                last_content.clone_from(&text);

                                let event = if let Some(kind) = detect::detect_pgp_content(&text) {
                                    debug!("PGP block detected in clipboard: {kind:?}");
                                    ClipboardEvent::PgpDetected {
                                        kind,
                                        content: text,
                                    }
                                } else {
                                    let preview_len = text.len().min(config.preview_max_len);
                                    let preview = text[..preview_len].to_string();
                                    ClipboardEvent::TextChanged {
                                        preview,
                                        length: text.len(),
                                    }
                                };

                                if tx.send(event).await.is_err() {
                                    debug!("clipboard monitor receiver dropped, stopping");
                                    break;
                                }
                            }
                        }
                        Ok(None) => {
                            if !last_content.is_empty() {
                                last_content.clear();
                                let _ = tx.send(ClipboardEvent::Empty).await;
                            }
                        }
                        Err(e) => {
                            trace!("clipboard read error (likely non-text content): {e}");
                        }
                    }
                }
                _ = stop_rx.recv() => {
                    debug!("clipboard monitor stopped");
                    break;
                }
            }
        }
    });

    (rx, MonitorHandle { stop_tx })
}

/// Handle to stop the clipboard monitor.
pub struct MonitorHandle {
    stop_tx: mpsc::Sender<()>,
}

impl MonitorHandle {
    /// Stop the clipboard monitor.
    pub async fn stop(self) {
        let _ = self.stop_tx.send(()).await;
    }
}

/// Read the current clipboard text content.
pub fn read_clipboard_text() -> Result<Option<String>> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| Error::Clipboard {
        reason: e.to_string(),
    })?;

    match clipboard.get_text() {
        Ok(text) if text.is_empty() => Ok(None),
        Ok(text) => Ok(Some(text)),
        Err(arboard::Error::ContentNotAvailable) => Ok(None),
        Err(e) => Err(Error::Clipboard {
            reason: e.to_string(),
        }),
    }
}

/// Write text to the clipboard.
pub fn write_clipboard_text(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| Error::Clipboard {
        reason: e.to_string(),
    })?;

    clipboard
        .set_text(text.to_string())
        .map_err(|e| Error::Clipboard {
            reason: e.to_string(),
        })?;

    Ok(())
}
