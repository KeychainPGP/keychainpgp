<script lang="ts">
  import type { Snippet } from "svelte";
  import { appStore } from "$lib/stores/app.svelte";

  interface Props {
    title?: string;
    children: Snippet;
  }
  let { title, children }: Props = $props();

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") appStore.closeModal();
  }

  function onBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) appStore.closeModal();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  role="button"
  tabindex="-1"
  onclick={onBackdropClick}
>
  <div
    class="mx-4 flex max-h-[80vh] w-full max-w-lg
           flex-col rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] shadow-xl"
  >
    {#if title}
      <div
        class="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-4"
      >
        <h2 class="text-lg font-semibold">{title}</h2>
        <button
          class="rounded p-1 text-[var(--color-text-secondary)] transition-colors hover:bg-[var(--color-bg-secondary)]"
          onclick={() => appStore.closeModal()}
        >
          &times;
        </button>
      </div>
    {/if}
    <div class="overflow-auto p-5">
      {@render children()}
    </div>
  </div>
</div>
