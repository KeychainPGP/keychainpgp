import { readClipboard } from "$lib/tauri";
import { isPgpMessage } from "$lib/utils";

let content: string | null = $state(null);
let pollInterval: ReturnType<typeof setInterval> | null = null;

export const clipboardStore = {
  get content() { return content; },

  get isPgpMessage() {
    return content ? isPgpMessage(content) : false;
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
};
