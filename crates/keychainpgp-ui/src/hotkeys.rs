//! Global hotkey registration and handling.

// Global hotkeys are registered via tauri-plugin-global-shortcut
// in the frontend JavaScript layer, which invokes Tauri commands.
//
// Default bindings:
//   Ctrl+Shift+E  ->  encrypt_clipboard
//   Ctrl+Shift+D  ->  decrypt_clipboard
//
// On macOS, Ctrl is replaced with Cmd.
//
// This module will be expanded when we add backend-driven hotkey
// registration and customization support.

/// Default hotkey binding for encryption.
pub const DEFAULT_ENCRYPT_HOTKEY: &str = "CmdOrCtrl+Shift+E";

/// Default hotkey binding for decryption.
pub const DEFAULT_DECRYPT_HOTKEY: &str = "CmdOrCtrl+Shift+D";

/// Default hotkey binding for signing.
pub const DEFAULT_SIGN_HOTKEY: &str = "CmdOrCtrl+Shift+S";
