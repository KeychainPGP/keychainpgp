import type { KeyInfo } from "$lib/tauri";
import { listKeys } from "$lib/tauri";

let keys: KeyInfo[] = $state([]);
let loading: boolean = $state(false);

export const keyStore = {
  get keys() { return keys; },
  get loading() { return loading; },

  get ownKeys() { return keys.filter(k => k.is_own_key); },
  get contactKeys() { return keys.filter(k => !k.is_own_key); },
  get hasKeys() { return keys.length > 0; },
  get hasOwnKey() { return keys.some(k => k.is_own_key); },

  async refresh() {
    loading = true;
    try {
      keys = await listKeys();
    } catch (e) {
      console.error("Failed to load keys:", e);
    } finally {
      loading = false;
    }
  },
};
