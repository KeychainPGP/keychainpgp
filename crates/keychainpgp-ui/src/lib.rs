//! KeychainPGP Tauri Application — shared library entry point.
//!
//! This module contains the app builder and setup logic shared between
//! the desktop binary (`main.rs`) and the mobile library entry point.

mod commands;
mod passphrase_cache;
mod state;

#[cfg(desktop)]
mod tray;

use tauri::Manager;

#[cfg(desktop)]
fn create_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            // Shared crypto commands
            commands::crypto::encrypt_text,
            commands::crypto::decrypt_text,
            commands::crypto::sign_text,
            commands::crypto::verify_text,
            commands::crypto::clear_passphrase_cache,
            // Desktop-only clipboard commands
            commands::crypto::encrypt_clipboard,
            commands::crypto::decrypt_clipboard,
            commands::crypto::sign_clipboard,
            commands::crypto::verify_clipboard,
            commands::clipboard::read_clipboard,
            commands::clipboard::write_clipboard,
            commands::clipboard::clear_clipboard,
            // Shared key commands
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
            // Shared settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            // Shared sync commands
            commands::sync::export_key_bundle,
            commands::sync::import_key_bundle,
        ])
}

#[cfg(mobile)]
fn create_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            // Shared crypto commands
            commands::crypto::encrypt_text,
            commands::crypto::decrypt_text,
            commands::crypto::sign_text,
            commands::crypto::verify_text,
            commands::crypto::clear_passphrase_cache,
            // Shared key commands
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
            // Shared settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            // Shared sync commands
            commands::sync::export_key_bundle,
            commands::sync::import_key_bundle,
        ])
}

/// Run the KeychainPGP application.
///
/// On mobile this is the entry point invoked by the native host.
/// On desktop this is called from `main()`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("starting KeychainPGP v{}", env!("CARGO_PKG_VERSION"));

    create_builder()
        .setup(|app| {
            // Initialize application state.
            // On desktop, use platform-default directories (via `directories` crate).
            // On mobile, `directories::ProjectDirs` doesn't work, so we use
            // the app data dir provided by Tauri's path resolver.
            #[cfg(desktop)]
            let app_state = state::AppState::initialize()?;
            #[cfg(mobile)]
            let app_state = {
                let data_dir = app.path().app_data_dir()?;
                state::AppState::initialize_with_dir(&data_dir)?
            };

            // Load persisted settings and apply to engine
            #[cfg(desktop)]
            let mut locale = "auto".to_string();
            if let Ok(store) = tauri_plugin_store::StoreExt::store(app, "settings.json") {
                if let Some(val) = store.get("settings") {
                    if let Ok(settings) =
                        serde_json::from_value::<commands::settings::Settings>(val)
                    {
                        app_state
                            .engine
                            .set_include_armor_headers(settings.include_armor_headers);
                        #[cfg(desktop)]
                        {
                            locale = settings.locale;
                        }
                    }
                }
            }

            app.manage(app_state);

            // Set up system tray with locale-aware labels (desktop only)
            #[cfg(desktop)]
            tray::setup_tray(app, &locale)?;

            tracing::info!("KeychainPGP initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running KeychainPGP");
}
