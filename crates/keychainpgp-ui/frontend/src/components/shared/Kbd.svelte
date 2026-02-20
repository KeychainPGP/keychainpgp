<script lang="ts">
  let { keys, variant = "default" }: { keys: string[]; variant?: "default" | "light" } = $props();
</script>

<span class="inline-flex items-center gap-0.5">
  {#each keys as key, i}
    {#if i > 0}
      <span class="text-[10px] opacity-50">+</span>
    {/if}
    <kbd
      class="inline-block min-w-[1.25rem] text-center text-[10px] font-mono font-medium leading-none
             px-1 py-0.5 rounded border shadow-[0_1px_0]
             {variant === 'light'
               ? 'bg-white/20 border-white/30 text-white shadow-white/10'
               : 'bg-[var(--color-bg)] border-[var(--color-border)] text-[var(--color-text-secondary)] shadow-[var(--color-border)]'}"
    >
      {key}
    </kbd>
  {/each}
</span>
