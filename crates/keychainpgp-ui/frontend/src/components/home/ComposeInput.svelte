<script lang="ts">
  import { Lock, PenLine, X } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { isPgpMessage, isPgpSignedMessage } from "$lib/utils";
  import * as m from "$lib/paraglide/messages.js";

  let pgpMessage = $derived(appStore.composeText ? isPgpMessage(appStore.composeText) : false);
  let signedMessage = $derived(appStore.composeText ? isPgpSignedMessage(appStore.composeText) : false);
</script>

<div
  class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4 min-h-32 relative"
>
  <div class="flex items-center justify-between mb-2">
    <span class="text-xs font-medium text-[var(--color-text-secondary)] uppercase tracking-wide">
      {m.compose_label()}
    </span>
    <div class="flex items-center gap-2">
      {#if signedMessage}
        <span class="inline-flex items-center gap-1 text-xs font-medium text-green-600">
          <PenLine size={12} />
          {m.clipboard_signed_message()}
        </span>
      {:else if pgpMessage}
        <span class="inline-flex items-center gap-1 text-xs font-medium text-[var(--color-primary)]">
          <Lock size={12} />
          {m.clipboard_pgp_message()}
        </span>
      {/if}
      {#if appStore.composeText}
        <button
          class="p-1 rounded hover:bg-[var(--color-border)] transition-colors"
          onclick={() => appStore.composeText = ""}
          title={m.compose_clear()}
        >
          <X size={14} class="text-[var(--color-text-secondary)]" />
        </button>
      {/if}
    </div>
  </div>

  <textarea
    class="w-full min-h-24 max-h-60 text-sm font-mono bg-transparent resize-y
           text-[var(--color-text)] placeholder-[var(--color-text-secondary)]
           focus:outline-none"
    placeholder={m.compose_placeholder()}
    bind:value={appStore.composeText}
  ></textarea>
</div>
