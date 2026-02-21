//! System tray setup and event handling.

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Emitter, Manager,
};

/// Return localized tray menu labels based on the locale prefix.
fn tray_labels(locale: &str) -> (&'static str, &'static str, &'static str, &'static str, &'static str) {
    let base = locale.split('-').next().unwrap_or(locale);
    match base {
        "fr" => ("Chiffrer le presse-papiers", "Déchiffrer le presse-papiers", "Signer le presse-papiers", "Ouvrir KeychainPGP", "Quitter"),
        "de" => ("Zwischenablage verschlüsseln", "Zwischenablage entschlüsseln", "Zwischenablage signieren", "KeychainPGP öffnen", "Beenden"),
        "es" => ("Cifrar portapapeles", "Descifrar portapapeles", "Firmar portapapeles", "Abrir KeychainPGP", "Salir"),
        "it" => ("Cifra appunti", "Decifra appunti", "Firma appunti", "Apri KeychainPGP", "Esci"),
        "pt" => ("Criptografar área de transferência", "Descriptografar área de transferência", "Assinar área de transferência", "Abrir KeychainPGP", "Sair"),
        "ru" => ("Зашифровать буфер", "Расшифровать буфер", "Подписать буфер", "Открыть KeychainPGP", "Выход"),
        "uk" => ("Зашифрувати буфер", "Розшифрувати буфер", "Підписати буфер", "Відкрити KeychainPGP", "Вихід"),
        "ja" => ("クリップボードを暗号化", "クリップボードを復号", "クリップボードに署名", "KeychainPGPを開く", "終了"),
        "ko" => ("클립보드 암호화", "클립보드 복호화", "클립보드 서명", "KeychainPGP 열기", "종료"),
        "zh" => ("加密剪贴板", "解密剪贴板", "签名剪贴板", "打开 KeychainPGP", "退出"),
        "ar" => ("تشفير الحافظة", "فك تشفير الحافظة", "توقيع الحافظة", "فتح KeychainPGP", "خروج"),
        "he" => ("הצפן לוח", "פענח לוח", "חתום לוח", "פתח KeychainPGP", "יציאה"),
        "tr" => ("Panoyu şifrele", "Pano şifresini çöz", "Panoyu imzala", "KeychainPGP'yi aç", "Çıkış"),
        "pl" => ("Zaszyfruj schowek", "Odszyfruj schowek", "Podpisz schowek", "Otwórz KeychainPGP", "Wyjdź"),
        "nl" => ("Klembord versleutelen", "Klembord ontsleutelen", "Klembord ondertekenen", "KeychainPGP openen", "Afsluiten"),
        "hi" => ("क्लिपबोर्ड एन्क्रिप्ट करें", "क्लिपबोर्ड डिक्रिप्ट करें", "क्लिपबोर्ड पर हस्ताक्षर करें", "KeychainPGP खोलें", "बाहर निकलें"),
        "th" => ("เข้ารหัสคลิปบอร์ด", "ถอดรหัสคลิปบอร์ด", "ลงนามคลิปบอร์ด", "เปิด KeychainPGP", "ออก"),
        _ => ("Encrypt Clipboard", "Decrypt Clipboard", "Sign Clipboard", "Open KeychainPGP", "Quit"),
    }
}

/// Set up the system tray icon and menu.
pub fn setup_tray(app: &App, locale: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (encrypt_label, decrypt_label, sign_label, open_label, quit_label) = tray_labels(locale);

    let encrypt_item = MenuItem::with_id(app, "encrypt", encrypt_label, true, None::<&str>)?;
    let decrypt_item = MenuItem::with_id(app, "decrypt", decrypt_label, true, None::<&str>)?;
    let sign_item = MenuItem::with_id(app, "sign", sign_label, true, None::<&str>)?;
    let separator = MenuItem::with_id(app, "sep", "─────────────────", false, None::<&str>)?;
    let open_item = MenuItem::with_id(app, "open", open_label, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&encrypt_item, &decrypt_item, &sign_item, &separator, &open_item, &quit_item],
    )?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("KeychainPGP")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "encrypt" | "decrypt" | "sign" => {
                let action = event.id.as_ref().to_string();
                tracing::info!("tray: {action} requested");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                let _ = app.emit("tray-action", action);
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
