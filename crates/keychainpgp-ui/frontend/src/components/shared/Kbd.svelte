<script lang="ts">
  let { keys, variant = "default" }: { keys: string[]; variant?: "default" | "light" } = $props();
</script>

<span class="inline-flex items-center gap-0.5">
  {#each keys as key, i}
    {#if i > 0}
      <span class="text-[10px] opacity-50">+</span>
    {/if}
    <kbd
      class="inline-block min-w-[1.25rem] rounded border px-1 py-0.5 text-center
             font-mono text-[10px] leading-none font-medium shadow-[0_1px_0]
             {variant === 'light'
        ? 'border-white/30 bg-white/20 text-white shadow-white/10'
        : 'border-[var(--color-border)] bg-[var(--color-bg)] text-[var(--color-text-secondary)] shadow-[var(--color-border)]'}"
    >
      {key}
    </kbd>
  {/each}
</span>
