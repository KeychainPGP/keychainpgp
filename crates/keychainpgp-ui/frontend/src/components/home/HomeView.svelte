<script lang="ts">
  import { Lock, Unlock, PenLine, ShieldCheck, Clipboard, MessageSquare } from "lucide-svelte";
  import ClipboardPreview from "./ClipboardPreview.svelte";
  import ComposeInput from "./ComposeInput.svelte";
  import Kbd from "../shared/Kbd.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { isPgpMessage } from "$lib/utils";
  import {
    decryptClipboard, signClipboard, verifyClipboard,
    decryptText, signText, verifyText, writeClipboard,
  } from "$lib/tauri";

  let isCompose = $derived(appStore.inputMode === "compose");

  /** Get the active text content depending on input mode. */
  function getContent(): string | null {
    if (isCompose) {
      return appStore.composeText || null;
    }
    return clipboardStore.content;
  }

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
    const content = getContent();
    if (!content) {
      appStore.setStatus(isCompose ? "Compose field is empty. Type a message first." : "Clipboard is empty. Copy some text first.");
      return;
    }
    if (keyStore.keys.length === 0) {
      appStore.setStatus("No keys available. Generate or import a key first.");
      return;
    }
    if (isCompose) {
      appStore.openModal("recipient-selector", { text: content });
    } else {
      appStore.openModal("recipient-selector");
    }
  }

  async function handleDecrypt() {
    const content = getContent();
    if (!content) {
      appStore.setStatus(isCompose ? "Compose field is empty." : "Clipboard is empty. Copy an encrypted message first.");
      return;
    }
    if (!isPgpMessage(content)) {
      appStore.setStatus("No PGP message detected.");
      return;
    }
    appStore.setStatus("Decrypting...", 0);
    try {
      const result = isCompose ? await decryptText(content) : await decryptClipboard();
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
              const result = isCompose ? await decryptText(content, passphrase) : await decryptClipboard(passphrase);
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
    const content = getContent();
    if (!content) {
      appStore.setStatus(isCompose ? "Compose field is empty." : "Clipboard is empty. Copy some text first.");
      return;
    }
    if (!keyStore.hasOwnKey) {
      appStore.setStatus("No private key available. Generate or import one first.");
      return;
    }
    appStore.setStatus("Signing...", 0);

    async function doSign(passphrase?: string) {
      if (isCompose) {
        const result = await signText(content, passphrase);
        if (result.success) {
          appStore.composeText = result.message;
          appStore.setStatus("Message signed.");
          appStore.closeModal();
        } else {
          appStore.openModal("error", { error: result.message });
        }
      } else {
        const result = await signClipboard(passphrase);
        if (result.success) {
          appStore.setStatus(result.message);
          appStore.closeModal();
          clipboardStore.refresh();
        } else {
          appStore.openModal("error", { error: result.message });
        }
      }
    }

    try {
      await doSign();
    } catch (e) {
      const msg = String(e);
      if (msg.includes("passphrase")) {
        appStore.openModal("passphrase", {
          onSubmit: async (passphrase: string) => {
            try {
              await doSign(passphrase);
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
    const content = getContent();
    if (!content) {
      appStore.setStatus(isCompose ? "Compose field is empty." : "Clipboard is empty. Copy a signed message first.");
      return;
    }
    appStore.setStatus("Verifying...", 0);
    try {
      const result = isCompose ? await verifyText(content) : await verifyClipboard();
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
      {isCompose ? "Type your message, then choose an action below." : "Copy text, then choose an action below."}
    </p>
  </div>

  <!-- Input mode toggle -->
  <div class="flex justify-center">
    <div class="inline-flex rounded-lg border border-[var(--color-border)] p-0.5">
      <button
        class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md transition-colors"
        class:bg-[var(--color-primary)]={!isCompose}
        class:text-white={!isCompose}
        onclick={() => appStore.inputMode = "clipboard"}
      >
        <Clipboard size={14} />
        Clipboard
      </button>
      <button
        class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md transition-colors"
        class:bg-[var(--color-primary)]={isCompose}
        class:text-white={isCompose}
        onclick={() => appStore.inputMode = "compose"}
      >
        <MessageSquare size={14} />
        Compose
      </button>
    </div>
  </div>

  {#if isCompose}
    <ComposeInput />
  {:else}
    <ClipboardPreview />
  {/if}

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
