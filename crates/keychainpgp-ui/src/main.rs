//! KeychainPGP Tauri Application
//!
//! The main entry point for the desktop GUI.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod passphrase_cache;
mod state;
mod tray;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("starting KeychainPGP v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // Initialize application state
            let app_state = state::AppState::initialize()?;

            // Load persisted settings and apply to engine
            if let Ok(store) = tauri_plugin_store::StoreExt::store(app, "settings.json") {
                if let Some(val) = store.get("settings") {
                    if let Ok(settings) = serde_json::from_value::<commands::settings::Settings>(val) {
                        app_state.engine.set_include_armor_headers(settings.include_armor_headers);
                    }
                }
            }

            app.manage(app_state);

            // Set up system tray
            tray::setup_tray(app)?;

            tracing::info!("KeychainPGP initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::crypto::encrypt_clipboard,
            commands::crypto::decrypt_clipboard,
            commands::crypto::sign_clipboard,
            commands::crypto::verify_clipboard,
            commands::crypto::clear_passphrase_cache,
            commands::keys::generate_key_pair,
            commands::keys::list_keys,
            commands::keys::import_key,
            commands::keys::export_key,
            commands::keys::delete_key,
            commands::keys::search_keys,
            commands::keys::inspect_key,
            commands::keys::set_key_trust,
            commands::keys::inspect_key_detailed,
            commands::keys::export_key_qr,
            commands::keys::wkd_lookup,
            commands::keys::keyserver_search,
            commands::keys::keyserver_upload,
            commands::keys::import_backup,
            commands::clipboard::read_clipboard,
            commands::clipboard::write_clipboard,
            commands::clipboard::clear_clipboard,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running KeychainPGP");
}
