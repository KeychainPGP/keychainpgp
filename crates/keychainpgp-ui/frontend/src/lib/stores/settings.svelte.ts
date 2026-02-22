import type { Settings } from "$lib/tauri";
import { getSettings, updateSettings } from "$lib/tauri";

const defaults: Settings = {
  auto_clear_enabled: true,
  auto_clear_delay_secs: 30,
  auto_clear_after_encrypt: false,
  clipboard_monitoring: true,
  encrypt_to_self: true,
  encrypt_to_self_keys: [],
  theme: "system",
  passphrase_cache_secs: 600,
  keyserver_url: "https://keys.openpgp.org",
  include_armor_headers: true,
  locale: "auto",
  proxy_url: "socks5://127.0.0.1:9050",
  proxy_enabled: false,
  proxy_preset: "tor",
  close_to_tray: false,
  opsec_mode: false,
  opsec_window_title: "Notes",
  opsec_view_timeout_secs: 30,
};

let settings: Settings = $state({ ...defaults });
let loaded: boolean = $state(false);

export const settingsStore = {
  get settings() { return settings; },
  get loaded() { return loaded; },

  async load() {
    try {
      settings = await getSettings();
    } catch {
      settings = { ...defaults };
    }
    loaded = true;
  },

  async save(partial: Partial<Settings>) {
    settings = { ...settings, ...partial };
    try {
      await updateSettings(settings);
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  },
};
