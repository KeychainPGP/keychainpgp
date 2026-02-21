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

export interface SignResult {
  success: boolean;
  message: string;
}

export interface VerifyResultInfo {
  valid: boolean;
  signer_name: string | null;
  signer_email: string | null;
  signer_fingerprint: string | null;
  trust_level: number;
  message: string;
}

export interface SubkeyInfoDto {
  fingerprint: string;
  algorithm: string;
  created_at: string;
  expires_at: string | null;
  capabilities: string[];
  is_revoked: boolean;
}

export interface UserIdDto {
  name: string | null;
  email: string | null;
}

export interface KeyDetailedInfo {
  fingerprint: string;
  name: string | null;
  email: string | null;
  algorithm: string;
  created_at: string;
  expires_at: string | null;
  trust_level: number;
  is_own_key: boolean;
  user_ids: UserIdDto[];
  subkeys: SubkeyInfoDto[];
}

export interface Settings {
  auto_clear_enabled: boolean;
  auto_clear_delay_secs: number;
  auto_clear_after_encrypt: boolean;
  clipboard_monitoring: boolean;
  encrypt_to_self: boolean;
  theme: string;
  passphrase_cache_secs: number;
  keyserver_url: string;
  include_armor_headers: boolean;
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

export async function signClipboard(
  passphrase?: string
): Promise<SignResult> {
  return invoke("sign_clipboard", { passphrase: passphrase ?? null });
}

export async function verifyClipboard(): Promise<VerifyResultInfo> {
  return invoke("verify_clipboard");
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

export async function inspectKey(fingerprint: string): Promise<KeyInfo> {
  return invoke("inspect_key", { fingerprint });
}

export async function setKeyTrust(fingerprint: string, trustLevel: number): Promise<boolean> {
  return invoke("set_key_trust", { fingerprint, trustLevel });
}

export async function inspectKeyDetailed(fingerprint: string): Promise<KeyDetailedInfo> {
  return invoke("inspect_key_detailed", { fingerprint });
}

export async function clearPassphraseCache(): Promise<void> {
  return invoke("clear_passphrase_cache");
}

export async function exportKeyQr(fingerprint: string): Promise<string> {
  return invoke("export_key_qr", { fingerprint });
}

export async function wkdLookup(email: string): Promise<KeyInfo | null> {
  return invoke("wkd_lookup", { email });
}

export async function keyserverSearch(query: string, keyserverUrl?: string): Promise<KeyInfo[]> {
  return invoke("keyserver_search", { query, keyserverUrl: keyserverUrl ?? null });
}

export async function keyserverUpload(fingerprint: string, keyserverUrl?: string): Promise<string> {
  return invoke("keyserver_upload", { fingerprint, keyserverUrl: keyserverUrl ?? null });
}

export interface BackupImportResult {
  imported_count: number;
  keys: KeyInfo[];
  skipped_count: number;
}

export async function importBackup(
  backupData: string,
  transferCode: string
): Promise<BackupImportResult> {
  return invoke("import_backup", { backupData, transferCode });
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
