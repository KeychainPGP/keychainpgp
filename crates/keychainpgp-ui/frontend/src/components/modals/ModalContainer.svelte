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

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={onBackdropClick}
>
  <div
    class="bg-[var(--color-bg)] border border-[var(--color-border)] rounded-xl shadow-xl
           w-full max-w-lg mx-4 max-h-[80vh] flex flex-col"
  >
    {#if title}
      <div class="flex items-center justify-between px-5 py-4 border-b border-[var(--color-border)]">
        <h2 class="text-lg font-semibold">{title}</h2>
        <button
          class="p-1 rounded hover:bg-[var(--color-bg-secondary)] transition-colors text-[var(--color-text-secondary)]"
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
