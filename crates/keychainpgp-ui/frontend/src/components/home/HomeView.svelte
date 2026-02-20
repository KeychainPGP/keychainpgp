<script lang="ts">
  import { Lock, Unlock } from "lucide-svelte";
  import ClipboardPreview from "./ClipboardPreview.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { decryptClipboard } from "$lib/tauri";

  function handleEncrypt() {
    if (!clipboardStore.content) {
      appStore.setStatus("Clipboard is empty. Copy some text first.");
      return;
    }
    if (keyStore.keys.length === 0) {
      appStore.setStatus("No keys available. Generate or import a key first.");
      return;
    }
    appStore.openModal("recipient-selector");
  }

  async function handleDecrypt() {
    if (!clipboardStore.content) {
      appStore.setStatus("Clipboard is empty. Copy an encrypted message first.");
      return;
    }
    if (!clipboardStore.isPgpMessage) {
      appStore.setStatus("Clipboard doesn't contain a PGP message.");
      return;
    }
    appStore.setStatus("Decrypting...", 0);
    try {
      const result = await decryptClipboard();
      if (result.success) {
        appStore.openModal("decrypted-viewer", { plaintext: result.plaintext });
        appStore.setStatus("Decrypted successfully.");
      } else {
        appStore.setStatus(result.message);
      }
    } catch (e) {
      const msg = String(e);
      if (msg.includes("passphrase") || msg.includes("private key")) {
        appStore.openModal("passphrase", {
          onSubmit: async (passphrase: string) => {
            try {
              const result = await decryptClipboard(passphrase);
              if (result.success) {
                appStore.openModal("decrypted-viewer", { plaintext: result.plaintext });
                appStore.setStatus("Decrypted successfully.");
              } else {
                appStore.openModal("error", { error: result.message });
              }
            } catch (e2) {
              appStore.openModal("error", { error: String(e2) });
            }
          },
        });
      } else {
        appStore.openModal("error", { error: msg, suggestion: "Make sure you have the correct private key imported." });
      }
    }
  }
</script>

<div class="max-w-2xl mx-auto space-y-6">
  <div class="text-center space-y-2">
    <h1 class="text-2xl font-bold">KeychainPGP</h1>
    <p class="text-[var(--color-text-secondary)]">
      Copy text, then click Encrypt or Decrypt.
    </p>
  </div>

  <ClipboardPreview />

  <div class="flex gap-4 justify-center">
    <button
      class="flex-1 max-w-48 py-4 rounded-lg bg-[var(--color-primary)] text-white font-semibold
             hover:bg-[var(--color-primary-hover)] transition-colors
             flex flex-col items-center gap-1"
      onclick={handleEncrypt}
    >
      <Lock size={20} />
      ENCRYPT
      <span class="text-xs font-normal opacity-75">Ctrl+Shift+E</span>
    </button>
    <button
      class="flex-1 max-w-48 py-4 rounded-lg bg-[var(--color-primary)] text-white font-semibold
             hover:bg-[var(--color-primary-hover)] transition-colors
             flex flex-col items-center gap-1"
      onclick={handleDecrypt}
    >
      <Unlock size={20} />
      DECRYPT
      <span class="text-xs font-normal opacity-75">Ctrl+Shift+D</span>
    </button>
  </div>
</div>
