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

#[cfg(desktop)]
fn create_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
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
            commands::keys::export_private_key,
            commands::keys::publish_revocation_cert,
            commands::keys::delete_key,
            commands::keys::search_keys,
            commands::keys::inspect_key,
            commands::keys::set_key_trust,
            commands::keys::inspect_key_detailed,
            commands::keys::export_key_qr,
            commands::keys::wkd_lookup,
            commands::keys::wkd_fetch_and_import,
            commands::keys::keyserver_search,
            commands::keys::keyserver_upload,
            commands::keys::fetch_and_import_key,
            commands::keys::import_backup,
            commands::keys::test_proxy_connection,
            commands::keys::generate_qr_svg,
            // Shared settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::is_portable,
            // Shared sync commands
            commands::sync::export_key_bundle,
            commands::sync::import_key_bundle,
            commands::sync::save_sync_file,
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
            commands::keys::export_private_key,
            commands::keys::publish_revocation_cert,
            commands::keys::delete_key,
            commands::keys::search_keys,
            commands::keys::inspect_key,
            commands::keys::set_key_trust,
            commands::keys::inspect_key_detailed,
            commands::keys::export_key_qr,
            commands::keys::wkd_lookup,
            commands::keys::wkd_fetch_and_import,
            commands::keys::keyserver_search,
            commands::keys::keyserver_upload,
            commands::keys::fetch_and_import_key,
            commands::keys::import_backup,
            commands::keys::test_proxy_connection,
            commands::keys::generate_qr_svg,
            // Shared settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::is_portable,
            // Shared sync commands
            commands::sync::export_key_bundle,
            commands::sync::import_key_bundle,
            commands::sync::save_sync_file,
            // Clipboard commands (via tauri-plugin-clipboard-manager)
            commands::clipboard_mobile::read_clipboard,
            commands::clipboard_mobile::write_clipboard,
            commands::clipboard_mobile::clear_clipboard,
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
            // On desktop: check for portable mode (.portable marker), otherwise use
            // platform-default directories (via `directories` crate).
            // On mobile: `directories::ProjectDirs` doesn't work, so we use
            // the app data dir provided by Tauri's path resolver.
            #[cfg(desktop)]
            let app_state = if let Some(portable_dir) = state::detect_portable_dir() {
                tracing::info!("portable mode: data dir = {}", portable_dir.display());
                let mut s = state::AppState::initialize_with_dir(&portable_dir)?;
                s.portable = true;
                s.portable_dir = Some(portable_dir);
                // Skip OS keyring in portable mode
                s.keyring.lock().unwrap().set_portable(true);
                s
            } else {
                state::AppState::initialize()?
            };
            #[cfg(mobile)]
            let app_state = {
                let data_dir = app.path().app_data_dir()?;
                state::AppState::initialize_with_dir(&data_dir)?
            };

            // Load persisted settings and apply to engine.
            // In portable mode, read from the portable data dir directly.
            // In normal mode, use the Tauri plugin store.
            #[cfg(desktop)]
            let mut locale = "auto".to_string();
            #[cfg(desktop)]
            let mut opsec_settings = None;

            let loaded_settings: Option<commands::settings::Settings> =
                if let Some(ref portable_dir) = app_state.portable_dir {
                    let path = portable_dir.join("settings.json");
                    std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|data| serde_json::from_str(&data).ok())
                } else if let Ok(store) = tauri_plugin_store::StoreExt::store(app, "settings.json")
                {
                    store
                        .get("settings")
                        .and_then(|val| serde_json::from_value(val).ok())
                } else {
                    None
                };

            if let Some(settings) = loaded_settings {
                app_state
                    .engine
                    .set_include_armor_headers(settings.include_armor_headers);
                if settings.opsec_mode {
                    app_state.opsec_mode.store(true, Ordering::SeqCst);
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
                    if app_state.opsec_mode.load(Ordering::SeqCst) {
                        // Zeroize all in-memory secret keys (force access even if mutex is poisoned)
                        app_state
                            .opsec_secret_keys
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .clear();
                        // Clear passphrase cache
                        app_state
                            .passphrase_cache
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .clear_all();
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    /// Extract command names from an `invoke_handler` block in the source text.
    ///
    /// Looks for lines like `commands::crypto::encrypt_text,` and returns
    /// the full path (e.g. "crypto::encrypt_text").
    fn extract_commands(block: &str) -> HashSet<String> {
        let mut commands = HashSet::new();
        for line in block.lines() {
            let trimmed = line.trim().trim_end_matches(',');
            if let Some(rest) = trimmed.strip_prefix("commands::") {
                if !rest.is_empty() && !rest.starts_with("//") {
                    commands.insert(rest.to_string());
                }
            }
        }
        commands
    }

    /// Find the invoke_handler block starting after `needle` in the source.
    fn find_handler_block(source: &str, needle: &str) -> String {
        let start = source.find(needle).expect("could not find handler marker");
        let after = &source[start..];
        let handler_start = after
            .find("invoke_handler(")
            .expect("no invoke_handler after marker");
        let block_start = &after[handler_start..];
        // Find the matching `])` that closes the generate_handler![] macro
        let mut depth = 0;
        let mut end = 0;
        for (i, ch) in block_start.char_indices() {
            if ch == '[' {
                depth += 1;
            } else if ch == ']' {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
        }
        block_start[..end].to_string()
    }

    /// Verify that desktop and mobile command handlers stay in sync.
    ///
    /// This test prevents regressions like issue #27 where a command was
    /// registered for desktop but missing from mobile, causing a runtime
    /// "Command not found" error on Android.
    #[test]
    fn test_command_registration_sync() {
        let source = include_str!("lib.rs");

        let desktop_block = find_handler_block(source, "#[cfg(desktop)]");
        let mobile_block = find_handler_block(source, "#[cfg(mobile)]");

        let desktop_cmds = extract_commands(&desktop_block);
        let mobile_cmds = extract_commands(&mobile_block);

        assert!(
            !desktop_cmds.is_empty(),
            "Failed to parse desktop commands from lib.rs"
        );
        assert!(
            !mobile_cmds.is_empty(),
            "Failed to parse mobile commands from lib.rs"
        );

        // Commands that are expected on only one platform
        let desktop_only: HashSet<&str> = [
            "crypto::encrypt_clipboard",
            "crypto::decrypt_clipboard",
            "crypto::sign_clipboard",
            "crypto::verify_clipboard",
            "clipboard::read_clipboard",
            "clipboard::write_clipboard",
            "clipboard::clear_clipboard",
        ]
        .into();

        let mobile_only: HashSet<&str> = [
            "clipboard_mobile::read_clipboard",
            "clipboard_mobile::write_clipboard",
            "clipboard_mobile::clear_clipboard",
        ]
        .into();

        // Verify desktop-only commands are NOT in mobile
        for cmd in &desktop_only {
            assert!(
                desktop_cmds.contains(*cmd),
                "Desktop-only command {cmd} missing from desktop handler"
            );
            assert!(
                !mobile_cmds.contains(*cmd),
                "Desktop-only command {cmd} should NOT be in mobile handler"
            );
        }

        // Verify mobile-only commands are NOT in desktop
        for cmd in &mobile_only {
            assert!(
                mobile_cmds.contains(*cmd),
                "Mobile-only command {cmd} missing from mobile handler"
            );
            assert!(
                !desktop_cmds.contains(*cmd),
                "Mobile-only command {cmd} should NOT be in desktop handler"
            );
        }

        // All remaining desktop commands (shared) must also be in mobile
        let shared_desktop: HashSet<_> = desktop_cmds
            .iter()
            .filter(|c| !desktop_only.contains(c.as_str()))
            .collect();
        let shared_mobile: HashSet<_> = mobile_cmds
            .iter()
            .filter(|c| !mobile_only.contains(c.as_str()))
            .collect();

        let missing_from_mobile: Vec<_> = shared_desktop.difference(&shared_mobile).collect();
        let missing_from_desktop: Vec<_> = shared_mobile.difference(&shared_desktop).collect();

        assert!(
            missing_from_mobile.is_empty(),
            "Commands registered on desktop but missing from mobile (issue #27 regression): {missing_from_mobile:?}"
        );
        assert!(
            missing_from_desktop.is_empty(),
            "Commands registered on mobile but missing from desktop: {missing_from_desktop:?}"
        );
    }
}
