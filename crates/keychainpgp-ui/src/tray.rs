//! System tray setup and event handling.

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App,
};

/// Set up the system tray icon and menu.
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let encrypt_item = MenuItem::with_id(app, "encrypt", "Encrypt Clipboard", true, None::<&str>)?;
    let decrypt_item = MenuItem::with_id(app, "decrypt", "Decrypt Clipboard", true, None::<&str>)?;
    let separator = MenuItem::with_id(app, "sep", "─────────────────", false, None::<&str>)?;
    let open_item = MenuItem::with_id(app, "open", "Open KeychainPGP", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&encrypt_item, &decrypt_item, &separator, &open_item, &quit_item],
    )?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("KeychainPGP")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "encrypt" => {
                tracing::info!("tray: encrypt clipboard requested");
                // TODO: trigger encryption flow
            }
            "decrypt" => {
                tracing::info!("tray: decrypt clipboard requested");
                // TODO: trigger decryption flow
            }
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
