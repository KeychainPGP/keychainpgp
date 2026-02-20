<script lang="ts">
  import { Copy, Check } from "lucide-svelte";
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";

  let copied = $state(false);
  const plaintext = appStore.modalProps.plaintext ?? "";

  async function copyPlaintext() {
    await navigator.clipboard.writeText(plaintext);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<ModalContainer title="Decrypted Message">
  <div class="space-y-3">
    <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4 max-h-64 overflow-auto">
      <pre class="text-sm whitespace-pre-wrap break-words font-mono">{plaintext}</pre>
    </div>

    <div class="flex justify-end gap-2">
      <button
        class="inline-flex items-center gap-1.5 px-4 py-2 text-sm rounded-lg border border-[var(--color-border)]
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={copyPlaintext}
      >
        {#if copied}
          <Check size={14} class="text-[var(--color-success)]" />
          Copied
        {:else}
          <Copy size={14} />
          Copy
        {/if}
      </button>
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        Close
      </button>
    </div>
  </div>
</ModalContainer>
