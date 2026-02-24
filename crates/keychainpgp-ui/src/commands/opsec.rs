//! OPSEC mode commands for hardened operation.

use std::sync::atomic::Ordering;

#[cfg(desktop)]
use tauri::Manager;
use tauri::{AppHandle, State};

use crate::state::AppState;

/// Enable OPSEC mode: change window title, set flag.
#[tauri::command]
pub fn enable_opsec_mode(
    #[allow(unused_variables)] app: AppHandle,
    state: State<'_, AppState>,
    #[allow(unused_variables)] title: Option<String>,
) -> Result<(), String> {
    state.opsec_mode.store(true, Ordering::SeqCst);

    #[cfg(desktop)]
    {
        let title = title
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "Notes".into());

        if let Some(window) = app.get_webview_window("main") {
            window
                .set_title(&title)
                .map_err(|e| format!("Failed to set title: {e}"))?;
        }
    }

    tracing::info!("OPSEC mode enabled");
    Ok(())
}

/// Disable OPSEC mode: restore window title, clear RAM keys.
#[tauri::command]
pub fn disable_opsec_mode(
    #[allow(unused_variables)] app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.opsec_mode.store(false, Ordering::SeqCst);

    // Zeroize and clear any RAM-only keys (force access even if mutex is poisoned)
    let mut keys = state
        .opsec_secret_keys
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    keys.clear();

    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_title("KeychainPGP")
            .map_err(|e| format!("Failed to set title: {e}"))?;
    }

    tracing::info!("OPSEC mode disabled");
    Ok(())
}

/// Panic wipe: immediately zeroize all secrets and close the app.
#[tauri::command]
pub fn panic_wipe(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    tracing::warn!("OPSEC panic wipe triggered");

    // Zeroize all in-memory secret keys (force access even if mutex is poisoned)
    state
        .opsec_secret_keys
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();

    // Clear passphrase cache
    state
        .passphrase_cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear_all();

    // Clear clipboard (desktop only)
    #[cfg(desktop)]
    {
        let _ = keychainpgp_clipboard::clear::clear_clipboard();
    }

    // Exit the application
    app.exit(0);

    Ok(())
}

/// Get whether OPSEC mode is currently active.
#[tauri::command]
pub fn get_opsec_status(state: State<'_, AppState>) -> bool {
    state.opsec_mode.load(Ordering::SeqCst)
}
