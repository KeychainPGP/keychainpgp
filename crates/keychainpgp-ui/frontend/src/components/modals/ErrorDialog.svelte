<script lang="ts">
  import { TriangleAlert } from "lucide-svelte";
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import * as m from "$lib/paraglide/messages.js";
</script>

<ModalContainer title={m.error_title()}>
  <div class="space-y-3">
    <div class="flex items-start gap-3">
      <TriangleAlert size={20} class="text-[var(--color-danger)] shrink-0 mt-0.5" />
      <div>
        <p class="text-sm">{appStore.modalProps.error ?? m.error_fallback()}</p>
        {#if appStore.modalProps.suggestion}
          <p class="mt-1 text-xs text-[var(--color-text-secondary)]">
            {appStore.modalProps.suggestion}
          </p>
        {/if}
      </div>
    </div>
    <div class="flex justify-end">
      <button
        class="rounded-lg bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white
               transition-colors hover:bg-[var(--color-primary-hover)]"
        onclick={() => appStore.closeModal()}
      >
        {m.error_ok()}
      </button>
    </div>
  </div>
</ModalContainer>
