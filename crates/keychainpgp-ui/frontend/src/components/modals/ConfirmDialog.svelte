<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import * as m from "$lib/paraglide/messages.js";
</script>

<ModalContainer title={appStore.modalProps.title ?? m.confirm_default_title()}>
  <div class="space-y-4">
    <p class="text-sm">
      {appStore.modalProps.message ?? m.confirm_default_message()}
    </p>
    <div class="flex justify-end gap-2">
      <button
        class="px-4 py-2 text-sm rounded-lg border border-[var(--color-border)]
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        {appStore.modalProps.cancelLabel ?? m.confirm_cancel()}
      </button>
      <button
        class="px-4 py-2 text-sm rounded-lg font-medium transition-opacity hover:opacity-90"
        class:bg-[var(--color-danger)]={!appStore.modalProps.confirmLabel ||
          appStore.modalProps.confirmLabel === m.confirm_delete()}
        class:bg-[var(--color-primary)]={appStore.modalProps.confirmLabel &&
          appStore.modalProps.confirmLabel !== m.confirm_delete()}
        class:text-white={true}
        onclick={() => appStore.modalProps.onConfirm?.()}
      >
        {appStore.modalProps.confirmLabel ?? m.confirm_delete()}
      </button>
    </div>
  </div>
</ModalContainer>
