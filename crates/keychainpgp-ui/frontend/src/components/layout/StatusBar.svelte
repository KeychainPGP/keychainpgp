<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";
</script>

<footer
  class="flex shrink-0 justify-between border-t border-[var(--color-border)]
         px-4 py-2 text-xs text-[var(--color-text-secondary)]"
>
  <span>{appStore.statusMessage}</span>
</footer>
