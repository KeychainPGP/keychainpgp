/**
 * Global hotkey registration via tauri-plugin-global-shortcut.
 */
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";

export interface HotkeyHandlers {
  onEncrypt: () => void;
  onDecrypt: () => void;
  onSign: () => void;
}

export async function registerHotkeys(handlers: HotkeyHandlers) {
  try {
    await register("CmdOrCtrl+Shift+E", (event) => {
      if (event.state === "Pressed") handlers.onEncrypt();
    });
    await register("CmdOrCtrl+Shift+D", (event) => {
      if (event.state === "Pressed") handlers.onDecrypt();
    });
    await register("CmdOrCtrl+Shift+S", (event) => {
      if (event.state === "Pressed") handlers.onSign();
    });
  } catch (e) {
    console.warn("Failed to register global hotkeys:", e);
  }
}

export async function unregisterHotkeys() {
  try {
    await unregisterAll();
  } catch {
    // ignore
  }
}
