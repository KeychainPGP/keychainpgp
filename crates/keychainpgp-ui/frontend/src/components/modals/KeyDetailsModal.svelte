<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import FingerprintDisplay from "../shared/FingerprintDisplay.svelte";
  import TrustBadge from "../shared/TrustBadge.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { formatDate } from "$lib/utils";

  const fp = appStore.modalProps.fingerprint ?? "";
  const keyInfo = $derived(keyStore.keys.find(k => k.fingerprint === fp));
</script>

<ModalContainer title="Key Details">
  {#if keyInfo}
    <div class="space-y-4">
      <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
        <span class="text-[var(--color-text-secondary)]">Name</span>
        <span class="font-medium">{keyInfo.name ?? "(unnamed)"}</span>

        <span class="text-[var(--color-text-secondary)]">Email</span>
        <span>{keyInfo.email ?? "(none)"}</span>

        <span class="text-[var(--color-text-secondary)]">Fingerprint</span>
        <FingerprintDisplay fingerprint={keyInfo.fingerprint} />

        <span class="text-[var(--color-text-secondary)]">Algorithm</span>
        <span>{keyInfo.algorithm}</span>

        <span class="text-[var(--color-text-secondary)]">Created</span>
        <span>{formatDate(keyInfo.created_at)}</span>

        <span class="text-[var(--color-text-secondary)]">Expires</span>
        <span>{keyInfo.expires_at ? formatDate(keyInfo.expires_at) : "Never"}</span>

        <span class="text-[var(--color-text-secondary)]">Trust</span>
        <TrustBadge level={keyInfo.trust_level} />

        <span class="text-[var(--color-text-secondary)]">Type</span>
        <span>{keyInfo.is_own_key ? "Own key (has secret key)" : "Public key only"}</span>
      </div>

      <div class="flex justify-end pt-2">
        <button
          class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
                 hover:bg-[var(--color-primary-hover)] transition-colors"
          onclick={() => appStore.closeModal()}
        >
          Close
        </button>
      </div>
    </div>
  {:else}
    <p class="text-sm text-[var(--color-text-secondary)]">Key not found.</p>
  {/if}
</ModalContainer>
