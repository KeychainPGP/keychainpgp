//! Tauri commands for application settings.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

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

const SETTINGS_KEY: &str = "settings";

/// Get the current application settings.
#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    let store = match app.store("settings.json") {
        Ok(s) => s,
        Err(_) => return Settings::default(),
    };

    match store.get(SETTINGS_KEY) {
        Some(val) => serde_json::from_value(val).unwrap_or_default(),
        None => Settings::default(),
    }
}

/// Update application settings.
#[tauri::command]
pub fn update_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open settings store: {e}"))?;

    let val = serde_json::to_value(&settings).map_err(|e| format!("Serialize error: {e}"))?;
    store.set(SETTINGS_KEY, val);

    tracing::info!("settings updated: {settings:?}");
    Ok(())
}
