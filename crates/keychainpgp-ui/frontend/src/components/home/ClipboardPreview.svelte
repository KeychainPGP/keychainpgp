<script lang="ts">
  import { Lock, PenLine, RefreshCw } from "lucide-svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { truncate } from "$lib/utils";
  import * as m from "$lib/paraglide/messages.js";
</script>

<div
  class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4 min-h-32 relative"
>
  <div class="flex items-center justify-between mb-2">
    <span class="text-xs font-medium text-[var(--color-text-secondary)] uppercase tracking-wide">
      {m.clipboard_label()}
    </span>
    <div class="flex items-center gap-2">
      {#if clipboardStore.isSignedMessage}
        <span class="inline-flex items-center gap-1 text-xs font-medium text-green-600">
          <PenLine size={12} />
          {m.clipboard_signed_message()}
        </span>
      {:else if clipboardStore.isPgpMessage}
        <span class="inline-flex items-center gap-1 text-xs font-medium text-[var(--color-primary)]">
          <Lock size={12} />
          {m.clipboard_pgp_message()}
        </span>
      {/if}
      <button
        class="p-1 rounded hover:bg-[var(--color-border)] transition-colors"
        onclick={() => clipboardStore.refresh()}
        title={m.clipboard_refresh()}
      >
        <RefreshCw size={14} class="text-[var(--color-text-secondary)]" />
      </button>
    </div>
  </div>

  {#if clipboardStore.content}
    <p class="text-sm font-mono whitespace-pre-wrap break-all text-[var(--color-text)]">
      {truncate(clipboardStore.content, 500)}
    </p>
  {:else}
    <p class="text-[var(--color-text-secondary)] text-sm italic">
      {m.clipboard_empty()}
    </p>
  {/if}
</div>
