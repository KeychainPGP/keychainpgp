export type Locale =
  | "en" | "fr" | "de" | "es" | "it" | "nl"
  | "pt-BR" | "pt-PT" | "ru" | "uk" | "pl" | "tr"
  | "zh-CN" | "zh-TW" | "ja" | "ko"
  | "ar" | "he" | "hi" | "th";

export type MessageKey =
  /* App shell */
  | "app_subtitle" | "app_footer" | "loading" | "loading_error"
  /* Tabs */
  | "tab_encrypt" | "tab_decrypt" | "tab_sign" | "tab_verify" | "tab_keys"
  /* Encrypt */
  | "encrypt_title" | "encrypt_placeholder" | "encrypt_recipients"
  | "encrypt_no_keys" | "encrypt_btn" | "encrypt_error_empty" | "encrypt_error_no_recipient"
  | "encrypt_own_label"
  /* Decrypt */
  | "decrypt_title" | "decrypt_placeholder" | "decrypt_passphrase"
  | "decrypt_btn" | "decrypt_error_empty" | "decrypt_error_no_key" | "decrypt_error_failed"
  /* Sign */
  | "sign_title" | "sign_placeholder" | "sign_passphrase"
  | "sign_btn" | "sign_error_empty" | "sign_error_no_key" | "sign_error_failed"
  /* Verify */
  | "verify_title" | "verify_placeholder" | "verify_btn"
  | "verify_error_empty" | "verify_error_no_keys"
  | "verify_valid" | "verify_signed_by" | "verify_failed" | "verify_signer_not_found"
  /* Key Manager */
  | "keys_title" | "keys_generate_btn" | "keys_import_btn"
  | "keys_empty" | "keys_own_label" | "keys_unnamed"
  /* Key Generation */
  | "keygen_title" | "keygen_name" | "keygen_email" | "keygen_passphrase"
  | "keygen_error_required" | "keygen_cancel" | "keygen_submit" | "keygen_loading"
  | "keygen_success"
  /* Import */
  | "import_title" | "import_placeholder" | "import_error_empty"
  | "import_cancel" | "import_submit" | "import_loading" | "import_success"
  /* Key actions */
  | "key_export_btn" | "key_delete_btn" | "key_deleted" | "key_exported"
  /* Common */
  | "copy_btn"
  /* Theme */
  | "theme_light" | "theme_dark"
  /* Onboarding */
  | "onboarding_title" | "onboarding_desc" | "onboarding_dismiss"
  /* Language */
  | "language_label";

export type Messages = Record<MessageKey, string>;
