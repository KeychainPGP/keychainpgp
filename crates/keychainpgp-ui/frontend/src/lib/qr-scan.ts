/**
 * QR code scanning utilities using qr-scanner (JS-based, WebWorker).
 *
 * Uses getUserMedia for camera access — the camera stays open continuously
 * with no refocus or blinking between scans. The video feed is rendered
 * in a <video> element managed by the scan overlay component.
 */
import QrScanner from "qr-scanner";
import { importKey, type KeyInfo } from "$lib/tauri";

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
    throw new Error(
      "This is a sync QR code. Use Settings → Key Sync → Import Keys to scan sync codes."
    );
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
