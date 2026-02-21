//! Tauri commands for application settings.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::state::AppState;

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
    /// Which own keys to encrypt to (fingerprints). Empty = all own keys.
    #[serde(default)]
    pub encrypt_to_self_keys: Vec<String>,
    /// UI theme: "light", "dark", or "system".
    pub theme: String,
    /// Passphrase cache duration in seconds (0 = disabled).
    pub passphrase_cache_secs: u64,
    /// Keyserver URL for key discovery.
    pub keyserver_url: String,
    /// Include armor headers (Version, Comment) in PGP output.
    #[serde(default = "default_true")]
    pub include_armor_headers: bool,
    /// User's preferred display language. "auto" = detect from OS.
    #[serde(default = "default_locale")]
    pub locale: String,
}

fn default_true() -> bool { true }
fn default_locale() -> String { "auto".into() }

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_clear_enabled: true,
            auto_clear_delay_secs: 30,
            auto_clear_after_encrypt: false,
            clipboard_monitoring: true,
            encrypt_to_self: true,
            encrypt_to_self_keys: Vec::new(),
            theme: "system".into(),
            passphrase_cache_secs: 600,
            keyserver_url: "https://keys.openpgp.org".into(),
            include_armor_headers: true,
            locale: "auto".into(),
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
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), String> {
    // Sync armor header setting to the engine
    state.engine.set_include_armor_headers(settings.include_armor_headers);

    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open settings store: {e}"))?;

    let val = serde_json::to_value(&settings).map_err(|e| format!("Serialize error: {e}"))?;
    store.set(SETTINGS_KEY, val);

    tracing::info!("settings updated: {settings:?}");
    Ok(())
}
