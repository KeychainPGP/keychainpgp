//! Tauri commands for clipboard operations.

use keychainpgp_clipboard::monitor;

/// Read the current clipboard text content.
#[tauri::command]
pub fn read_clipboard() -> Result<Option<String>, String> {
    monitor::read_clipboard_text().map_err(|e| format!("Clipboard error: {e}"))
}

/// Write text to the clipboard.
#[tauri::command]
pub fn write_clipboard(text: String) -> Result<(), String> {
    monitor::write_clipboard_text(&text).map_err(|e| format!("Clipboard error: {e}"))
}

/// Clear the clipboard immediately.
#[tauri::command]
pub fn clear_clipboard() -> Result<(), String> {
    keychainpgp_clipboard::clear::clear_clipboard()
        .map_err(|e| format!("Failed to clear clipboard: {e}"))
}
