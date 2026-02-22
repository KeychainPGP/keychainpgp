<script lang="ts">
  import { Lock, Unlock, PenLine, ShieldCheck, Clipboard, MessageSquare } from "lucide-svelte";
  import ClipboardPreview from "./ClipboardPreview.svelte";
  import ComposeInput from "./ComposeInput.svelte";
  import Kbd from "../shared/Kbd.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { isPgpMessage } from "$lib/utils";
  import { isDesktop } from "$lib/platform";
  import {
    decryptClipboard, signClipboard, verifyClipboard,
    decryptText, signText, verifyText, writeClipboard,
  } from "$lib/tauri";
  import * as m from "$lib/paraglide/messages.js";

  let isCompose = $derived(appStore.inputMode === "compose");
  const desktop = isDesktop();

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
      appStore.setStatus(isCompose ? m.encrypt_empty_compose() : m.encrypt_empty_clipboard());
      return;
    }
    if (keyStore.keys.length === 0) {
      appStore.setStatus(m.encrypt_no_keys());
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
      appStore.setStatus(isCompose ? m.decrypt_empty_compose() : m.decrypt_empty_clipboard());
      return;
    }
    if (!isPgpMessage(content)) {
      appStore.setStatus(m.decrypt_no_pgp());
      return;
    }
    appStore.setStatus(m.decrypt_in_progress(), 0);
    try {
      const result = isCompose ? await decryptText(content) : await decryptClipboard();
      if (result.success) {
        appStore.openModal("decrypted-viewer", { plaintext: result.plaintext });
        appStore.setStatus(m.decrypt_success());
        clipboardStore.scheduleAutoClear();
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
                appStore.setStatus(m.decrypt_success());
                clipboardStore.scheduleAutoClear();
              } else {
                appStore.openModal("error", { error: result.message });
              }
            } catch (e2) {
              appStore.openModal("error", { error: String(e2) });
            }
          },
        });
      } else {
        appStore.openModal("error", { error: msg, suggestion: m.decrypt_wrong_key_hint() });
      }
    }
  }

  async function handleSign() {
    const content = getContent();
    if (!content) {
      appStore.setStatus(isCompose ? m.sign_empty_compose() : m.sign_empty_clipboard());
      return;
    }
    if (!keyStore.hasOwnKey) {
      appStore.setStatus(m.sign_no_key());
      return;
    }
    appStore.setStatus(m.sign_in_progress(), 0);

    async function doSign(passphrase?: string) {
      if (isCompose) {
        const result = await signText(content!, passphrase);
        if (result.success) {
          appStore.composeText = result.message;
          appStore.setStatus(m.sign_success_compose());
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
      appStore.setStatus(isCompose ? m.verify_empty_compose() : m.verify_empty_clipboard());
      return;
    }
    appStore.setStatus(m.verify_in_progress(), 0);
    try {
      const result = isCompose ? await verifyText(content) : await verifyClipboard();
      appStore.openModal("verify-result", { verifyResult: result });
      appStore.setStatus(result.valid ? m.verify_success() : m.verify_failed());
    } catch (e) {
      appStore.openModal("error", { error: String(e) });
    }
  }
</script>

<div class="max-w-2xl mx-auto space-y-6">
  <div class="text-center space-y-2">
    <h1 class="text-2xl font-bold">{m.home_title()}</h1>
    <p class="text-[var(--color-text-secondary)]">
      {isCompose ? m.home_tagline_compose() : m.home_tagline_clipboard()}
    </p>
  </div>

  <!-- Input mode toggle (desktop only — mobile always uses compose) -->
  {#if desktop}
    <div class="flex justify-center">
      <div class="inline-flex rounded-lg border border-[var(--color-border)] p-0.5">
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md transition-colors"
          class:bg-[var(--color-primary)]={!isCompose}
          class:text-white={!isCompose}
          onclick={() => appStore.inputMode = "clipboard"}
        >
          <Clipboard size={14} />
          {m.mode_clipboard()}
        </button>
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md transition-colors"
          class:bg-[var(--color-primary)]={isCompose}
          class:text-white={isCompose}
          onclick={() => appStore.inputMode = "compose"}
        >
          <MessageSquare size={14} />
          {m.mode_compose()}
        </button>
      </div>
    </div>
  {/if}

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
      {m.action_encrypt()}
      {#if desktop}<Kbd keys={[m.kbd_ctrl(), m.kbd_shift(), "E"]} variant="light" />{/if}
    </button>
    <button
      class="py-4 rounded-lg bg-[var(--color-primary)] text-white font-semibold
             hover:bg-[var(--color-primary-hover)] transition-colors
             flex flex-col items-center gap-1"
      onclick={handleDecrypt}
    >
      <Unlock size={20} />
      {m.action_decrypt()}
      {#if desktop}<Kbd keys={[m.kbd_ctrl(), m.kbd_shift(), "D"]} variant="light" />{/if}
    </button>
    <button
      class="py-4 rounded-lg border-2 border-[var(--color-primary)] text-[var(--color-primary)] font-semibold
             hover:bg-[var(--color-primary)] hover:text-white transition-colors
             flex flex-col items-center gap-1"
      onclick={handleSign}
    >
      <PenLine size={20} />
      {m.action_sign()}
      {#if desktop}<Kbd keys={[m.kbd_ctrl(), m.kbd_shift(), "S"]} />{/if}
    </button>
    <button
      class="py-4 rounded-lg border-2 border-[var(--color-primary)] text-[var(--color-primary)] font-semibold
             hover:bg-[var(--color-primary)] hover:text-white transition-colors
             flex flex-col items-center gap-1"
      onclick={handleVerify}
    >
      <ShieldCheck size={20} />
      {m.action_verify()}
      {#if desktop}<Kbd keys={[m.kbd_ctrl(), m.kbd_shift(), "V"]} />{/if}
    </button>
  </div>
</div>
