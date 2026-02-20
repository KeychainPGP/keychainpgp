<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
</script>

<ModalContainer title={appStore.modalProps.title ?? "Confirm"}>
  <div class="space-y-4">
    <p class="text-sm">{appStore.modalProps.message ?? "Are you sure?"}</p>
    <div class="flex justify-end gap-2">
      <button
        class="px-4 py-2 text-sm rounded-lg border border-[var(--color-border)]
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        Cancel
      </button>
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-danger)] text-white font-medium
               hover:opacity-90 transition-opacity"
        onclick={() => appStore.modalProps.onConfirm?.()}
      >
        Delete
      </button>
    </div>
  </div>
</ModalContainer>
