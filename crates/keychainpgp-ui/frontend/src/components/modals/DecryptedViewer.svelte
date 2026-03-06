<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Copy, Check } from "lucide-svelte";
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import * as m from "$lib/paraglide/messages.js";

  let copied = $state(false);
  const plaintext = appStore.modalProps.plaintext ?? "";

  const isOpsec = settingsStore.settings.opsec_mode;
  const viewTimeout = settingsStore.settings.opsec_view_timeout_secs;

  let countdown = $state(isOpsec && viewTimeout > 0 ? viewTimeout : 0);
  let timerId: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    if (isOpsec && viewTimeout > 0) {
      timerId = setInterval(() => {
        countdown--;
        if (countdown <= 0) {
          if (timerId) clearInterval(timerId);
          appStore.closeModal();
        }
      }, 1000);
    }
  });

  onDestroy(() => {
    if (timerId) clearInterval(timerId);
  });

  async function copyPlaintext() {
    await navigator.clipboard.writeText(plaintext);
    copied = true;
    setTimeout(() => (copied = false), 2000);
    clipboardStore.scheduleAutoClear();
  }
</script>

<ModalContainer title={m.decrypted_title()}>
  <div class="space-y-3">
    <div
      class="max-h-64 overflow-auto rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4"
    >
      <pre
        class="font-mono text-sm break-words whitespace-pre-wrap"
        class:select-none={isOpsec}
        class:pointer-events-none={isOpsec}>{plaintext}</pre>
    </div>

    {#if isOpsec && countdown > 0}
      <p class="text-center text-xs text-[var(--color-text-secondary)]">
        {m.opsec_closing_in({ seconds: countdown })}
      </p>
    {/if}

    <div class="flex justify-end gap-2">
      {#if !isOpsec}
        <button
          class="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm
                 transition-colors hover:bg-[var(--color-bg-secondary)]"
          onclick={copyPlaintext}
        >
          {#if copied}
            <Check size={14} class="text-[var(--color-success)]" />
            {m.decrypted_copied()}
          {:else}
            <Copy size={14} />
            {m.decrypted_copy()}
          {/if}
        </button>
      {/if}
      <button
        class="rounded-lg bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white
               transition-colors hover:bg-[var(--color-primary-hover)]"
        onclick={() => appStore.closeModal()}
      >
        {m.decrypted_close()}
      </button>
    </div>
  </div>
</ModalContainer>
