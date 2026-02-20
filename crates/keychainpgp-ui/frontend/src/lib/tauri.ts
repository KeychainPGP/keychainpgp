/**
 * Tauri API wrappers for type-safe invoke calls.
 */
import { invoke } from "@tauri-apps/api/core";

// --- Types ---

export interface KeyInfo {
  fingerprint: string;
  name: string | null;
  email: string | null;
  algorithm: string;
  created_at: string;
  expires_at: string | null;
  trust_level: number;
  is_own_key: boolean;
}

export interface EncryptResult {
  success: boolean;
  message: string;
}

export interface DecryptResult {
  success: boolean;
  plaintext: string;
  message: string;
}

export interface Settings {
  auto_clear_enabled: boolean;
  auto_clear_delay_secs: number;
  auto_clear_after_encrypt: boolean;
  clipboard_monitoring: boolean;
  encrypt_to_self: boolean;
  theme: string;
}

// --- Crypto ---

export async function encryptClipboard(
  recipientFingerprints: string[]
): Promise<EncryptResult> {
  return invoke("encrypt_clipboard", {
    recipientFingerprints: recipientFingerprints,
  });
}

export async function decryptClipboard(
  passphrase?: string
): Promise<DecryptResult> {
  return invoke("decrypt_clipboard", { passphrase: passphrase ?? null });
}

// --- Keys ---

export async function generateKeyPair(
  name: string,
  email: string,
  passphrase?: string
): Promise<KeyInfo> {
  return invoke("generate_key_pair", {
    name,
    email,
    passphrase: passphrase ?? null,
  });
}

export async function listKeys(): Promise<KeyInfo[]> {
  return invoke("list_keys");
}

export async function importKey(keyData: string): Promise<KeyInfo> {
  return invoke("import_key", { keyData });
}

export async function exportKey(fingerprint: string): Promise<string> {
  return invoke("export_key", { fingerprint });
}

export async function deleteKey(fingerprint: string): Promise<boolean> {
  return invoke("delete_key", { fingerprint });
}

export async function searchKeys(query: string): Promise<KeyInfo[]> {
  return invoke("search_keys", { query });
}

// --- Clipboard ---

export async function readClipboard(): Promise<string | null> {
  return invoke("read_clipboard");
}

export async function writeClipboard(text: string): Promise<void> {
  return invoke("write_clipboard", { text });
}

export async function clearClipboard(): Promise<void> {
  return invoke("clear_clipboard");
}

// --- Settings ---

export async function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export async function updateSettings(settings: Settings): Promise<void> {
  return invoke("update_settings", { settings });
}
