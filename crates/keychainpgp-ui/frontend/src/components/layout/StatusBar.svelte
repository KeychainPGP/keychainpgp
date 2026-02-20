<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";
</script>

<footer
  class="border-t border-[var(--color-border)] px-4 py-2 text-xs
         text-[var(--color-text-secondary)] flex justify-between shrink-0"
>
  <span>{appStore.statusMessage}</span>
</footer>
