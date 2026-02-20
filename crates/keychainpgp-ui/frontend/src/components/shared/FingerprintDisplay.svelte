<script lang="ts">
  import { Copy, Check } from "lucide-svelte";
  import { formatFingerprint } from "$lib/utils";

  interface Props {
    fingerprint: string;
    short?: boolean;
  }
  let { fingerprint, short = false }: Props = $props();

  let copied = $state(false);

  const display = $derived(
    short ? fingerprint.slice(-8).toUpperCase() : formatFingerprint(fingerprint)
  );

  async function copyToClipboard() {
    await navigator.clipboard.writeText(fingerprint);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<span class="inline-flex items-center gap-1 font-mono text-xs">
  <span class="select-all">{display}</span>
  <button
    class="p-0.5 rounded hover:bg-[var(--color-bg-secondary)] transition-colors"
    onclick={copyToClipboard}
    title="Copy fingerprint"
  >
    {#if copied}
      <Check size={12} class="text-[var(--color-success)]" />
    {:else}
      <Copy size={12} class="text-[var(--color-text-secondary)]" />
    {/if}
  </button>
</span>
