<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { exportKeyQr } from "$lib/tauri";

  const fp = appStore.modalProps.fingerprint ?? "";

  let svgData: string | null = $state(null);
  let error: string | null = $state(null);

  $effect(() => {
    if (fp) {
      exportKeyQr(fp)
        .then(svg => { svgData = svg; })
        .catch(e => { error = String(e); });
    }
  });
</script>

<ModalContainer title="QR Code Export">
  <div class="space-y-4">
    {#if error}
      <p class="text-sm text-red-600">{error}</p>
    {:else if svgData}
      <div class="flex justify-center p-4 bg-white rounded-lg">
        {@html svgData}
      </div>
      <p class="text-xs text-center text-[var(--color-text-secondary)]">
        Scan this QR code to import the public key.
      </p>
    {:else}
      <p class="text-sm text-[var(--color-text-secondary)]">Generating QR code...</p>
    {/if}

    <div class="flex justify-end">
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        Close
      </button>
    </div>
  </div>
</ModalContainer>
