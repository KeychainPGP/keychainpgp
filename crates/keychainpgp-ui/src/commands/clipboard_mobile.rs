//! Tauri commands for clipboard operations (mobile).
//!
//! On mobile, the `keychainpgp-clipboard` crate (which uses `arboard`) is not
//! available. Instead we use `tauri-plugin-clipboard-manager` which provides
//! native clipboard access on Android and iOS.

use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Read the current clipboard text content.
#[tauri::command]
pub fn read_clipboard(app: AppHandle) -> Result<Option<String>, String> {
    match app.clipboard().read_text() {
        Ok(text) if text.is_empty() => Ok(None),
        Ok(text) => Ok(Some(text)),
        Err(e) => Err(format!("Clipboard error: {e}")),
    }
}

/// Write text to the clipboard.
#[tauri::command]
pub fn write_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Clipboard error: {e}"))
}

/// Clear the clipboard immediately.
#[tauri::command]
pub fn clear_clipboard(app: AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(String::new())
        .map_err(|e| format!("Failed to clear clipboard: {e}"))
}
