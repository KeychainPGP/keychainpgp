<script lang="ts">
  import { Lock, Unlock, PenLine, ShieldCheck } from "lucide-svelte";
  import ClipboardPreview from "./ClipboardPreview.svelte";
  import Kbd from "../shared/Kbd.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { decryptClipboard, signClipboard, verifyClipboard } from "$lib/tauri";

  // React to external actions (hotkeys, tray)
  $effect(() => {
    const action = appStore.pendingAction;
    if (!action) return;
    appStore.clearAction();
    switch (action) {
      case "encrypt": handleEncrypt(); break;
      case "decrypt": handleDecrypt(); break;
      case "sign": handleSign(); break;
      case "verify": handleVerify(); break;
    }
  });

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

  async function handleSign() {
    if (!clipboardStore.content) {
      appStore.setStatus("Clipboard is empty. Copy some text first.");
      return;
    }
    if (!keyStore.hasOwnKey) {
      appStore.setStatus("No private key available. Generate or import one first.");
      return;
    }
    appStore.setStatus("Signing...", 0);
    try {
      const result = await signClipboard();
      if (result.success) {
        appStore.setStatus(result.message);
        clipboardStore.refresh();
      } else {
        appStore.setStatus(result.message);
      }
    } catch (e) {
      const msg = String(e);
      if (msg.includes("passphrase")) {
        appStore.openModal("passphrase", {
          onSubmit: async (passphrase: string) => {
            try {
              const result = await signClipboard(passphrase);
              if (result.success) {
                appStore.setStatus(result.message);
                appStore.closeModal();
                clipboardStore.refresh();
              } else {
                appStore.openModal("error", { error: result.message });
              }
            } catch (e2) {
              appStore.openModal("error", { error: String(e2) });
            }
          },
        });
      } else {
        appStore.openModal("error", { error: msg });
      }
    }
  }

  async function handleVerify() {
    if (!clipboardStore.content) {
      appStore.setStatus("Clipboard is empty. Copy a signed message first.");
      return;
    }
    appStore.setStatus("Verifying...", 0);
    try {
      const result = await verifyClipboard();
      appStore.openModal("verify-result", { verifyResult: result });
      appStore.setStatus(result.valid ? "Signature verified." : "Verification failed.");
    } catch (e) {
      appStore.openModal("error", { error: String(e) });
    }
  }
</script>

<div class="max-w-2xl mx-auto space-y-6">
  <div class="text-center space-y-2">
    <h1 class="text-2xl font-bold">KeychainPGP</h1>
    <p class="text-[var(--color-text-secondary)]">
      Copy text, then choose an action below.
    </p>
  </div>

  <ClipboardPreview />

  <div class="grid grid-cols-2 gap-3 max-w-md mx-auto">
    <button
      class="py-4 rounded-lg bg-[var(--color-primary)] text-white font-semibold
             hover:bg-[var(--color-primary-hover)] transition-colors
             flex flex-col items-center gap-1"
      onclick={handleEncrypt}
    >
      <Lock size={20} />
      ENCRYPT
      <Kbd keys={["Ctrl", "Shift", "E"]} variant="light" />
    </button>
    <button
      class="py-4 rounded-lg bg-[var(--color-primary)] text-white font-semibold
             hover:bg-[var(--color-primary-hover)] transition-colors
             flex flex-col items-center gap-1"
      onclick={handleDecrypt}
    >
      <Unlock size={20} />
      DECRYPT
      <Kbd keys={["Ctrl", "Shift", "D"]} variant="light" />
    </button>
    <button
      class="py-4 rounded-lg border-2 border-[var(--color-primary)] text-[var(--color-primary)] font-semibold
             hover:bg-[var(--color-primary)] hover:text-white transition-colors
             flex flex-col items-center gap-1"
      onclick={handleSign}
    >
      <PenLine size={20} />
      SIGN
      <Kbd keys={["Ctrl", "Shift", "S"]} />
    </button>
    <button
      class="py-4 rounded-lg border-2 border-[var(--color-primary)] text-[var(--color-primary)] font-semibold
             hover:bg-[var(--color-primary)] hover:text-white transition-colors
             flex flex-col items-center gap-1"
      onclick={handleVerify}
    >
      <ShieldCheck size={20} />
      VERIFY
      <Kbd keys={["Ctrl", "Shift", "V"]} />
    </button>
  </div>
</div>
