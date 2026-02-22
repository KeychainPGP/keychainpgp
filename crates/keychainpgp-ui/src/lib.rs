//! KeychainPGP Tauri Application — shared library entry point.
//!
//! This module contains the app builder and setup logic shared between
//! the desktop binary (`main.rs`) and the mobile library entry point.

mod commands;
mod passphrase_cache;
mod state;

#[cfg(desktop)]
mod tray;

use std::sync::atomic::Ordering;

use tauri::Manager;
use zeroize::Zeroize;

#[cfg(desktop)]
fn create_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(app_state) = window.try_state::<state::AppState>() {
                    if app_state.close_to_tray.load(Ordering::Relaxed) {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
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
            commands::keys::test_proxy_connection,
            commands::keys::generate_qr_svg,
            // Shared settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            // Shared sync commands
            commands::sync::export_key_bundle,
            commands::sync::import_key_bundle,
            // OPSEC commands
            commands::opsec::enable_opsec_mode,
            commands::opsec::disable_opsec_mode,
            commands::opsec::panic_wipe,
            commands::opsec::get_opsec_status,
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
            commands::keys::test_proxy_connection,
            commands::keys::generate_qr_svg,
            // Shared settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            // Shared sync commands
            commands::sync::export_key_bundle,
            commands::sync::import_key_bundle,
            // OPSEC commands
            commands::opsec::enable_opsec_mode,
            commands::opsec::disable_opsec_mode,
            commands::opsec::panic_wipe,
            commands::opsec::get_opsec_status,
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
            #[cfg(desktop)]
            let mut opsec_settings = None;
            if let Ok(store) = tauri_plugin_store::StoreExt::store(app, "settings.json") {
                if let Some(val) = store.get("settings") {
                    if let Ok(settings) =
                        serde_json::from_value::<commands::settings::Settings>(val)
                    {
                        app_state
                            .engine
                            .set_include_armor_headers(settings.include_armor_headers);
                        if settings.opsec_mode {
                            app_state.opsec_mode.store(true, Ordering::Relaxed);
                        }
                        #[cfg(desktop)]
                        {
                            app_state
                                .close_to_tray
                                .store(settings.close_to_tray, Ordering::Relaxed);
                            locale = settings.locale.clone();
                            if settings.opsec_mode {
                                opsec_settings = Some(settings);
                            }
                        }
                    }
                }
            }

            app.manage(app_state);

            // Apply OPSEC window title if active (desktop only)
            #[cfg(desktop)]
            if let Some(ref settings) = opsec_settings {
                if let Some(window) = app.get_webview_window("main") {
                    let title = if settings.opsec_window_title.is_empty() {
                        "Notes"
                    } else {
                        &settings.opsec_window_title
                    };
                    let _ = window.set_title(title);
                }
            }

            // Set up system tray with locale-aware labels (desktop only)
            #[cfg(desktop)]
            tray::setup_tray(app, &locale)?;

            tracing::info!("KeychainPGP initialized");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building KeychainPGP")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit = event {
                if let Some(app_state) = app.try_state::<state::AppState>() {
                    if app_state.opsec_mode.load(Ordering::Relaxed) {
                        // Zeroize all in-memory secret keys
                        if let Ok(mut keys) = app_state.opsec_secret_keys.lock() {
                            for value in keys.values_mut() {
                                value.zeroize();
                            }
                            keys.clear();
                        }
                        // Clear passphrase cache
                        if let Ok(mut cache) = app_state.passphrase_cache.lock() {
                            cache.clear_all();
                        }
                        // Clear clipboard (desktop only)
                        #[cfg(desktop)]
                        {
                            let _ = keychainpgp_clipboard::clear::clear_clipboard();
                        }
                        tracing::info!("OPSEC session cleanup completed");
                    }
                }
            }
        });
}
