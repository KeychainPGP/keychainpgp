<script lang="ts">
  import { KeyRound, Upload, Camera, RefreshCw } from "lucide-svelte";
  import { generateKeyPair, importKey } from "$lib/tauri";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { isMobile } from "$lib/platform";
  import { cancelScan } from "$lib/qr-scan";
  import QrScanOverlay from "../shared/QrScanOverlay.svelte";
  import * as m from "$lib/paraglide/messages.js";

  const mobile = isMobile();

  let name = $state("");
  let email = $state("");
  let passphrase = $state("");
  let generating = $state(false);
  let error = $state("");
  let scanning = $state(false);

  async function handleGenerate() {
    if (!name.trim() || !email.trim()) {
      error = m.keygen_required();
      return;
    }
    error = "";
    generating = true;
    try {
      const info = await generateKeyPair(name.trim(), email.trim(), passphrase || undefined);
      await keyStore.refresh();
      appStore.setStatus(m.keygen_success());
      setTimeout(() => {
        appStore.openModal("publish-prompt", { fingerprint: info.fingerprint });
      }, 100);
    } catch (e) {
      error = String(e);
    } finally {
      generating = false;
    }
  }

  function handleScanResult(content: string): boolean {
    if (content.startsWith("KCPGP:")) {
      error = m.error_sync_qr_use_sync();
      return true;
    }
    importKey(content)
      .then(async () => {
        await keyStore.refresh();
      })
      .catch((e) => {
        error = String(e);
      });
    return true;
  }

  function handleCancelScan() {
    cancelScan();
    scanning = false;
  }
</script>

{#if scanning}
  <QrScanOverlay
    onscan={(content) => {
      const done = handleScanResult(content);
      if (done) scanning = false;
      return done;
    }}
    oncancel={handleCancelScan}
  />
{/if}

<div class="flex h-full flex-col items-center justify-center px-6">
  <div class="w-full max-w-md space-y-6">
    <div class="space-y-2 text-center">
      <div class="mb-2 inline-flex rounded-full bg-[var(--color-primary)]/10 p-3">
        <KeyRound size={32} class="text-[var(--color-primary)]" />
      </div>
      <h1 class="text-2xl font-bold">{m.onboarding_title()}</h1>
      <p class="text-sm text-[var(--color-text-secondary)]">
        {m.onboarding_subtitle()}
      </p>
    </div>

    <div class="space-y-3">
      <input
        type="text"
        placeholder={m.onboarding_name_placeholder()}
        bind:value={name}
        class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2.5
               text-sm text-[var(--color-text)]
               focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
      />
      <input
        type="email"
        placeholder={m.onboarding_email_placeholder()}
        bind:value={email}
        class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2.5
               text-sm text-[var(--color-text)]
               focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
      />
      <input
        type="password"
        placeholder={m.onboarding_passphrase_placeholder()}
        bind:value={passphrase}
        class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2.5
               text-sm text-[var(--color-text)]
               focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
      />
    </div>

    {#if error}
      <p class="text-sm text-[var(--color-danger)]">{error}</p>
    {/if}

    <div class="space-y-2">
      <button
        class="w-full rounded-lg bg-[var(--color-primary)] py-3 font-semibold text-white
               transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
        onclick={handleGenerate}
        disabled={generating}
      >
        {generating ? m.onboarding_creating() : m.onboarding_create()}
      </button>
      <button
        class="flex w-full items-center justify-center gap-2
               rounded-lg border
               border-[var(--color-border)] py-3
               font-medium text-[var(--color-text)] transition-colors hover:bg-[var(--color-bg-secondary)]"
        onclick={() => appStore.openModal("key-import")}
      >
        <Upload size={16} />
        {m.onboarding_import()}
      </button>
      {#if mobile}
        <button
          class="flex w-full items-center justify-center gap-2
                 rounded-lg border
                 border-[var(--color-border)] py-3
                 font-medium text-[var(--color-text)] transition-colors hover:bg-[var(--color-bg-secondary)]"
          onclick={() => {
            error = "";
            scanning = true;
          }}
        >
          <Camera size={16} />
          {m.onboarding_scan_qr()}
        </button>
      {/if}
      <button
        class="flex w-full items-center justify-center gap-2
               rounded-lg border
               border-[var(--color-border)] py-3
               font-medium text-[var(--color-text)] transition-colors hover:bg-[var(--color-bg-secondary)]"
        onclick={() => appStore.openModal("key-sync-import")}
      >
        <RefreshCw size={16} />
        {m.onboarding_sync()}
      </button>
    </div>
  </div>
</div>
