//! Tauri commands for application settings.

use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::state::AppState;

/// Settings file name used in portable mode.
const PORTABLE_SETTINGS_FILE: &str = "settings.json";

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
    /// SOCKS5 proxy URL for anonymous keyserver access (e.g., "socks5h://127.0.0.1:9050").
    #[serde(default = "default_proxy_url")]
    pub proxy_url: String,
    /// Whether the proxy is active for keyserver requests.
    #[serde(default)]
    pub proxy_enabled: bool,
    /// Proxy preset: "tor", "lokinet", or "custom".
    #[serde(default = "default_proxy_preset")]
    pub proxy_preset: String,
    /// Close to system tray instead of quitting when the window is closed.
    #[serde(default)]
    pub close_to_tray: bool,
    /// OPSEC mode: hardened operation for high-risk users.
    #[serde(default)]
    pub opsec_mode: bool,
    /// OPSEC: custom window title (empty = "Notes").
    #[serde(default = "default_opsec_title")]
    pub opsec_window_title: String,
    /// OPSEC: view timeout in seconds for decrypted text (0 = no timeout).
    #[serde(default = "default_opsec_view_timeout")]
    pub opsec_view_timeout_secs: u64,
}

fn default_true() -> bool {
    true
}
fn default_locale() -> String {
    "auto".into()
}
fn default_proxy_url() -> String {
    "socks5h://127.0.0.1:9050".into()
}
fn default_proxy_preset() -> String {
    "tor".into()
}
fn default_opsec_title() -> String {
    "Notes".into()
}
fn default_opsec_view_timeout() -> u64 {
    30
}

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
            proxy_url: "socks5h://127.0.0.1:9050".into(),
            proxy_enabled: false,
            proxy_preset: "tor".into(),
            close_to_tray: false,
            opsec_mode: false,
            opsec_window_title: "Notes".into(),
            opsec_view_timeout_secs: 30,
        }
    }
}

const SETTINGS_KEY: &str = "settings";

/// Check if the app is running in portable mode.
#[tauri::command]
pub fn is_portable(state: State<'_, AppState>) -> bool {
    state.portable
}

/// Get the current application settings.
#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> Settings {
    // In portable mode, read directly from the portable data dir
    if let Some(ref portable_dir) = state.portable_dir {
        let path = portable_dir.join(PORTABLE_SETTINGS_FILE);
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(s) = serde_json::from_str::<Settings>(&data) {
                return s;
            }
        }
        return Settings::default();
    }

    // Normal mode: use Tauri plugin store
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
    // Sync settings to runtime state
    state
        .engine
        .set_include_armor_headers(settings.include_armor_headers);
    state
        .close_to_tray
        .store(settings.close_to_tray, Ordering::Relaxed);

    // Sync passphrase cache TTL
    if let Ok(mut cache) = state.passphrase_cache.lock() {
        cache.set_ttl(settings.passphrase_cache_secs);
    }

    // In portable mode, write directly to the portable data dir
    if let Some(ref portable_dir) = state.portable_dir {
        let path = portable_dir.join(PORTABLE_SETTINGS_FILE);
        let json =
            serde_json::to_string_pretty(&settings).map_err(|e| format!("Serialize error: {e}"))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write portable settings: {e}"))?;
        tracing::info!("settings updated (portable)");
        return Ok(());
    }

    // Normal mode: use Tauri plugin store
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open settings store: {e}"))?;

    let val = serde_json::to_value(&settings).map_err(|e| format!("Serialize error: {e}"))?;
    store.set(SETTINGS_KEY, val);

    tracing::info!("settings updated");
    Ok(())
}
