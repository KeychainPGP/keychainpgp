/**
 * Hotkey management.
 *
 * - Panic wipe (Ctrl+Shift+P): registered as a GLOBAL shortcut via
 *   tauri-plugin-global-shortcut so it works even when the app is in background.
 * - Encrypt/Decrypt/Sign/Verify: handled as window-scoped keydown events
 *   in App.svelte so they don't intercept system shortcuts.
 */
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";

export async function registerPanicHotkey(handler: () => void) {
  try {
    await register("CmdOrCtrl+Shift+P", (event) => {
      if (event.state === "Pressed") handler();
    });
  } catch (e) {
    console.warn("Failed to register panic hotkey:", e);
  }
}

export async function unregisterPanicHotkey() {
  try {
    await unregisterAll();
  } catch {
    // ignore
  }
}
