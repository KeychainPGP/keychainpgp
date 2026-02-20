//! Tauri commands for application settings.

use serde::{Deserialize, Serialize};

/// Application settings exposed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Auto-clear clipboard after decryption.
    pub auto_clear_enabled: bool,
    /// Auto-clear delay in seconds.
    pub auto_clear_delay_secs: u64,
    /// Also auto-clear after encryption.
    pub auto_clear_after_encrypt: bool,
    /// Monitor clipboard for PGP content.
    pub clipboard_monitoring: bool,
    /// Always encrypt to self.
    pub encrypt_to_self: bool,
    /// UI theme: "light", "dark", or "system".
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_clear_enabled: true,
            auto_clear_delay_secs: 30,
            auto_clear_after_encrypt: false,
            clipboard_monitoring: true,
            encrypt_to_self: true,
            theme: "system".into(),
        }
    }
}

/// Get the current application settings.
#[tauri::command]
pub fn get_settings() -> Settings {
    // TODO: load from persistent store
    Settings::default()
}

/// Update application settings.
#[tauri::command]
pub fn update_settings(settings: Settings) -> Result<(), String> {
    // TODO: persist to store
    tracing::info!("settings updated: {settings:?}");
    Ok(())
}
