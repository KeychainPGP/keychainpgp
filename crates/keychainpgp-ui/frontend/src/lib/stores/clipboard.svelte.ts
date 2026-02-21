import { readClipboard, clearClipboard } from "$lib/tauri";
import { isPgpMessage, isPgpSignedMessage } from "$lib/utils";
import { settingsStore } from "./settings.svelte";

let content: string | null = $state(null);
let pollInterval: ReturnType<typeof setInterval> | null = null;
let autoClearTimer: ReturnType<typeof setTimeout> | null = null;

export const clipboardStore = {
  get content() { return content; },

  get isPgpMessage() {
    return content ? isPgpMessage(content) : false;
  },

  get isSignedMessage() {
    return content ? isPgpSignedMessage(content) : false;
  },

  async refresh() {
    try {
      content = await readClipboard();
    } catch {
      content = null;
    }
  },

  startPolling(intervalMs = 2000) {
    this.stopPolling();
    this.refresh();
    pollInterval = setInterval(() => this.refresh(), intervalMs);
  },

  stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  },

  /** Schedule clipboard auto-clear after decrypt/encrypt. OPSEC forces 10s. */
  scheduleAutoClear() {
    if (autoClearTimer) {
      clearTimeout(autoClearTimer);
      autoClearTimer = null;
    }

    const s = settingsStore.settings;
    const isOpsec = s.opsec_mode;

    // In OPSEC mode: always clear after 10s. Otherwise: use user setting.
    if (!isOpsec && !s.auto_clear_enabled) return;

    const delaySecs = isOpsec ? 10 : s.auto_clear_delay_secs;

    autoClearTimer = setTimeout(async () => {
      try {
        await clearClipboard();
      } catch {
        // ignore
      }
      autoClearTimer = null;
    }, delaySecs * 1000);
  },
};
