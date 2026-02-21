/**
 * Platform detection for conditional UI rendering.
 *
 * Uses @tauri-apps/plugin-os to determine the current platform at runtime.
 */
import { platform as osPlatform } from "@tauri-apps/plugin-os";

export type Platform = "android" | "ios" | "desktop";

let _platform: Platform = "desktop";
let _initialized = false;

/** Initialize platform detection. Call once during app startup. */
export async function initPlatform(): Promise<void> {
  if (_initialized) return;
  try {
    const os = osPlatform();
    if (os === "android") _platform = "android";
    else if (os === "ios") _platform = "ios";
    else _platform = "desktop";
  } catch {
    _platform = "desktop";
  }
  _initialized = true;
}

export function getPlatform(): Platform {
  return _platform;
}

export function isMobile(): boolean {
  return _platform === "android" || _platform === "ios";
}

export function isDesktop(): boolean {
  return _platform === "desktop";
}
