<script lang="ts">
  import { KeyRound, Upload } from "lucide-svelte";
  import { generateKeyPair } from "$lib/tauri";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import * as m from "$lib/paraglide/messages.js";

  let name = $state("");
  let email = $state("");
  let passphrase = $state("");
  let generating = $state(false);
  let error = $state("");

  async function handleGenerate() {
    if (!name.trim() || !email.trim()) {
      error = m.keygen_required();
      return;
    }
    error = "";
    generating = true;
    try {
      await generateKeyPair(name.trim(), email.trim(), passphrase || undefined);
      await keyStore.refresh();
      appStore.setStatus(m.keygen_success());
    } catch (e) {
      error = String(e);
    } finally {
      generating = false;
    }
  }
</script>

<div class="flex flex-col items-center justify-center h-full px-6">
  <div class="max-w-md w-full space-y-6">
    <div class="text-center space-y-2">
      <div class="inline-flex p-3 rounded-full bg-[var(--color-primary)]/10 mb-2">
        <KeyRound size={32} class="text-[var(--color-primary)]" />
      </div>
      <h1 class="text-2xl font-bold">{m.onboarding_title()}</h1>
      <p class="text-[var(--color-text-secondary)] text-sm">
        {m.onboarding_subtitle()}
      </p>
    </div>

    <div class="space-y-3">
      <input
        type="text"
        placeholder={m.onboarding_name_placeholder()}
        bind:value={name}
        class="w-full px-3 py-2.5 text-sm rounded-lg border border-[var(--color-border)]
               bg-[var(--color-bg)] text-[var(--color-text)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      />
      <input
        type="email"
        placeholder={m.onboarding_email_placeholder()}
        bind:value={email}
        class="w-full px-3 py-2.5 text-sm rounded-lg border border-[var(--color-border)]
               bg-[var(--color-bg)] text-[var(--color-text)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      />
      <input
        type="password"
        placeholder={m.onboarding_passphrase_placeholder()}
        bind:value={passphrase}
        class="w-full px-3 py-2.5 text-sm rounded-lg border border-[var(--color-border)]
               bg-[var(--color-bg)] text-[var(--color-text)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      />
    </div>

    {#if error}
      <p class="text-sm text-[var(--color-danger)]">{error}</p>
    {/if}

    <div class="space-y-2">
      <button
        class="w-full py-3 rounded-lg bg-[var(--color-primary)] text-white font-semibold
               hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50"
        onclick={handleGenerate}
        disabled={generating}
      >
        {generating ? m.onboarding_creating() : m.onboarding_create()}
      </button>
      <button
        class="w-full py-3 rounded-lg border border-[var(--color-border)]
               text-[var(--color-text)] font-medium
               hover:bg-[var(--color-bg-secondary)] transition-colors
               flex items-center justify-center gap-2"
        onclick={() => appStore.openModal("key-import")}
      >
        <Upload size={16} />
        {m.onboarding_import()}
      </button>
    </div>
  </div>
</div>
