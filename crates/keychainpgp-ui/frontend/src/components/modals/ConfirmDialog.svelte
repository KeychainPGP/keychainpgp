<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import * as m from "$lib/paraglide/messages.js";
</script>

<ModalContainer title={appStore.modalProps.title ?? m.confirm_default_title()}>
  <div class="space-y-4">
    <p class="text-sm">{appStore.modalProps.message ?? m.confirm_default_message()}</p>
    <div class="flex justify-end gap-2">
      <button
        class="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm
               transition-colors hover:bg-[var(--color-bg-secondary)]"
        onclick={() => appStore.closeModal()}
      >
        {m.confirm_cancel()}
      </button>
      <button
        class="rounded-lg bg-[var(--color-danger)] px-4 py-2 text-sm font-medium text-white
               transition-opacity hover:opacity-90"
        onclick={() => appStore.modalProps.onConfirm?.()}
      >
        {m.confirm_delete()}
      </button>
    </div>
  </div>
</ModalContainer>
