<script lang="ts">
  import { AlertTriangle } from "lucide-svelte";
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
</script>

<ModalContainer title="Error">
  <div class="space-y-3">
    <div class="flex items-start gap-3">
      <AlertTriangle size={20} class="text-[var(--color-danger)] shrink-0 mt-0.5" />
      <div>
        <p class="text-sm">{appStore.modalProps.error ?? "An unexpected error occurred."}</p>
        {#if appStore.modalProps.suggestion}
          <p class="text-xs text-[var(--color-text-secondary)] mt-1">
            {appStore.modalProps.suggestion}
          </p>
        {/if}
      </div>
    </div>
    <div class="flex justify-end">
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        OK
      </button>
    </div>
  </div>
</ModalContainer>
