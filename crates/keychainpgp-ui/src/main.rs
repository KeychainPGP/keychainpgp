//! KeychainPGP desktop entry point.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    keychainpgp_ui_lib::run();
}
