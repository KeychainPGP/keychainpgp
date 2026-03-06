<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { exportKeyQr } from "$lib/tauri";
  import * as m from "$lib/paraglide/messages.js";

  const fp = appStore.modalProps.fingerprint ?? "";

  let svgData: string | null = $state(null);
  let error: string | null = $state(null);

  $effect(() => {
    if (fp) {
      exportKeyQr(fp)
        .then((svg) => {
          svgData = svg;
        })
        .catch((e) => {
          error = String(e);
        });
    }
  });
</script>

<ModalContainer title={m.qr_title()}>
  <div class="space-y-4">
    {#if error}
      <p class="text-sm text-red-600">{error}</p>
    {:else if svgData}
      <div class="flex justify-center rounded-lg bg-white p-4">
        <img src="data:image/svg+xml;base64,{btoa(svgData)}" alt={m.qr_code_alt()} />
      </div>
      <p class="text-center text-xs text-[var(--color-text-secondary)]">
        {m.qr_desc()}
      </p>
    {:else}
      <p class="text-sm text-[var(--color-text-secondary)]">{m.qr_generating()}</p>
    {/if}

    <div class="flex justify-end">
      <button
        class="rounded-lg bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white
               transition-colors hover:bg-[var(--color-primary-hover)]"
        onclick={() => appStore.closeModal()}
      >
        {m.qr_close()}
      </button>
    </div>
  </div>
</ModalContainer>
