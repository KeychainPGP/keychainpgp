/**
 * Typed wrappers around the keychainpgp-wasm module.
 */

import wasmInit, {
  init as wasmPanicHook,
  generateKeyPair as _generateKeyPair,
  encrypt as _encrypt,
  decrypt as _decrypt,
  sign as _sign,
  verify as _verify,
  inspectKey as _inspectKey,
} from "../../pkg/keychainpgp_wasm.js";

export interface KeyPairResult {
  public_key: string;
  /** Raw bytes of the armored secret key — use `.fill(0)` to zeroize after use. */
  secret_key: Uint8Array;
  fingerprint: string;
}

export interface VerifyResult {
  valid: boolean;
  signer_fingerprint: string | null;
}

export interface UserIdInfo {
  name: string | null;
  email: string | null;
}

export interface SubkeyInfo {
  fingerprint: string;
  algorithm: string;
  created_at: string;
  expires_at: string | null;
  capabilities: string[];
  is_revoked: boolean;
}

export interface CertInfo {
  fingerprint: string;
  user_ids: UserIdInfo[];
  algorithm: string;
  created_at: string;
  expires_at: string | null;
  has_secret_key: boolean;
  subkeys: SubkeyInfo[];
}

let initialized = false;

export async function initWasm(): Promise<void> {
  if (initialized) return;
  await wasmInit();
  wasmPanicHook();
  initialized = true;
}

export function generateKeyPair(
  name: string,
  email: string,
  passphrase?: string,
): KeyPairResult {
  return _generateKeyPair(name, email, passphrase ?? undefined) as KeyPairResult;
}

export function encrypt(plaintext: string, recipientKeys: string[]): string {
  return _encrypt(plaintext, JSON.stringify(recipientKeys));
}

export function decrypt(
  ciphertext: string,
  secretKey: string,
  passphrase?: string,
): string {
  return _decrypt(ciphertext, secretKey, passphrase ?? undefined);
}

export function sign(
  data: string,
  secretKey: string,
  passphrase?: string,
): string {
  return _sign(data, secretKey, passphrase ?? undefined);
}

export function verify(signedData: string, signerKey: string): VerifyResult {
  return _verify(signedData, signerKey) as VerifyResult;
}

export function inspectKey(keyData: string): CertInfo {
  return _inspectKey(keyData) as CertInfo;
}
