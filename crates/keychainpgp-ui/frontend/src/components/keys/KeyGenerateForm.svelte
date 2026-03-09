<script lang="ts">
  import { generateKeyPair } from "$lib/tauri";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import * as m from "$lib/paraglide/messages.js";

  interface Props {
    onDone: () => void;
  }
  let { onDone }: Props = $props();

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
      const info = await generateKeyPair(
        name.trim(),
        email.trim(),
        passphrase || undefined,
      );
      await keyStore.refresh();
      appStore.setStatus(m.keygen_success());

      onDone();
      setTimeout(() => {
        appStore.openModal("publish-prompt", { fingerprint: info.fingerprint });
      }, 100);
    } catch (e) {
      error = String(e);
    } finally {
      generating = false;
    }
  }
</script>

<div
  class="space-y-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4"
>
  <h3 class="font-medium">{m.keygen_title()}</h3>
  <div class="grid grid-cols-2 gap-3">
    <input
      type="text"
      placeholder={m.keygen_name_placeholder()}
      bind:value={name}
      class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2
             text-sm focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
    />
    <input
      type="email"
      placeholder={m.keygen_email_placeholder()}
      bind:value={email}
      class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2
             text-sm focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
    />
  </div>
  <input
    type="password"
    placeholder={m.keygen_passphrase_placeholder()}
    bind:value={passphrase}
    class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2
           text-sm focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
  />
  {#if error}
    <p class="text-sm text-[var(--color-danger)]">{error}</p>
  {/if}
  <div class="flex justify-end gap-2">
    <button
      class="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm
             transition-colors hover:bg-[var(--color-bg)]"
      onclick={onDone}
    >
      {m.keygen_cancel()}
    </button>
    <button
      class="rounded-lg bg-[var(--color-primary)] px-3 py-1.5 text-sm font-medium text-white
             transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
      onclick={handleGenerate}
      disabled={generating}
    >
      {generating ? m.keygen_loading() : m.keygen_submit()}
    </button>
  </div>
</div>
