/**
 * QR code scanning utilities using qr-scanner (JS-based, WebWorker).
 *
 * Uses getUserMedia for camera access — the camera stays open continuously
 * with no refocus or blinking between scans. The video feed is rendered
 * in a <video> element managed by the scan overlay component.
 */
import QrScanner from "qr-scanner";
import { importKey, type KeyInfo } from "$lib/tauri";
import * as m from "$lib/paraglide/messages.js";

/** Active scanner instance (singleton — only one scan session at a time). */
let activeScanner: QrScanner | null = null;

/**
 * Start continuous QR scanning on a <video> element.
 * Calls `onScan` for each detected QR code. Return `true` from onScan to stop.
 * The camera stays open between detections — no blinking or refocus.
 *
 * Returns a cleanup function that stops the scanner.
 */
export function startContinuousScan(
  videoEl: HTMLVideoElement,
  onScan: (content: string) => boolean,
  onError?: (error: string) => void,
): () => void {
  // Stop any existing scanner
  if (activeScanner) {
    activeScanner.stop();
    activeScanner.destroy();
    activeScanner = null;
  }

  const scanner = new QrScanner(
    videoEl,
    (result) => {
      const shouldStop = onScan(result.data);
      if (shouldStop) {
        scanner.stop();
        scanner.destroy();
        if (activeScanner === scanner) activeScanner = null;
      }
    },
    {
      preferredCamera: "environment",
      highlightScanRegion: false,
      highlightCodeOutline: false,
    },
  );

  activeScanner = scanner;

  scanner.start().catch((err) => {
    const msg = String(err);
    if (onError) onError(msg);
  });

  return () => {
    scanner.stop();
    scanner.destroy();
    if (activeScanner === scanner) activeScanner = null;
  };
}

/** Stop any active scanner. */
export function cancelScan(): void {
  if (activeScanner) {
    activeScanner.stop();
    activeScanner.destroy();
    activeScanner = null;
  }
}

/**
 * Scan a single QR code containing a PGP key and import it.
 * This is used by components that manage their own video element and overlay.
 * Detects KCPGP sync format and throws a user-friendly error.
 */
export async function importScannedContent(content: string): Promise<KeyInfo> {
  if (content.startsWith("KCPGP:")) {
    throw new Error(m.error_sync_qr_wrong_context());
  }
  return await importKey(content);
}

/** Parsed KCPGP QR part. */
export interface QrPart {
  part: number;
  total: number;
  data: string;
}

/** Parse a KCPGP-prefixed QR code string. Returns null if not a valid KCPGP part. */
export function parseKcpgpPart(content: string): QrPart | null {
  if (!content.startsWith("KCPGP:")) return null;
  const rest = content.slice(6);
  // Skip fountain parts (start with F)
  if (rest.startsWith("F")) return null;
  const colonIdx = rest.indexOf(":");
  if (colonIdx < 0) return null;
  const header = rest.slice(0, colonIdx);
  const data = rest.slice(colonIdx + 1);
  const slashIdx = header.indexOf("/");
  if (slashIdx < 0) return null;
  const part = parseInt(header.slice(0, slashIdx), 10);
  const total = parseInt(header.slice(slashIdx + 1), 10);
  if (isNaN(part) || isNaN(total) || part < 1 || total < 1) return null;
  return { part, total, data };
}

/** Parsed fountain parity part. */
export interface FountainPart {
  i: number;
  j: number;
  total: number;
  data: string;
}

/** Parse a KCPGP fountain parity QR code (format: KCPGP:F<i>+<j>/<total>:<base64_xor>). */
export function parseFountainPart(content: string): FountainPart | null {
  if (!content.startsWith("KCPGP:F")) return null;
  const rest = content.slice(7); // after "KCPGP:F"
  const colonIdx = rest.indexOf(":");
  if (colonIdx < 0) return null;
  const header = rest.slice(0, colonIdx);
  const data = rest.slice(colonIdx + 1);
  const slashIdx = header.indexOf("/");
  if (slashIdx < 0) return null;
  const indices = header.slice(0, slashIdx);
  const total = parseInt(header.slice(slashIdx + 1), 10);
  const plusIdx = indices.indexOf("+");
  if (plusIdx < 0) return null;
  const i = parseInt(indices.slice(0, plusIdx), 10);
  const j = parseInt(indices.slice(plusIdx + 1), 10);
  if (isNaN(i) || isNaN(j) || isNaN(total)) return null;
  return { i, j, total, data };
}

/** Decode base64 string to Uint8Array. */
function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes;
}

/**
 * Try to recover missing data parts using fountain parity XOR codes.
 * Mutates `dataParts` in place. Returns true if all parts are now available.
 */
export function fountainRecover(
  dataParts: Map<number, string>,
  fountainParts: FountainPart[],
  total: number,
): boolean {
  let progress = true;
  while (progress && dataParts.size < total) {
    progress = false;
    for (const fp of fountainParts) {
      const hasI = dataParts.has(fp.i);
      const hasJ = dataParts.has(fp.j);
      if ((hasI && hasJ) || (!hasI && !hasJ)) continue;

      const knownKey = hasI ? fp.i : fp.j;
      const missingKey = hasI ? fp.j : fp.i;

      const xorBytes = b64ToBytes(fp.data);
      const knownBytes = new TextEncoder().encode(dataParts.get(knownKey)!);
      const recovered = new Uint8Array(xorBytes.length);
      for (let k = 0; k < xorBytes.length; k++) {
        recovered[k] = (xorBytes[k] || 0) ^ (knownBytes[k] || 0);
      }
      // Trim trailing zeros (base64 chars are never 0x00)
      let end = recovered.length;
      while (end > 0 && recovered[end - 1] === 0) end--;
      dataParts.set(missingKey, new TextDecoder().decode(recovered.slice(0, end)));
      progress = true;
    }
  }
  return dataParts.size >= total;
}
