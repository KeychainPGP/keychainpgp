<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import FingerprintDisplay from "../shared/FingerprintDisplay.svelte";
  import TrustBadge from "../shared/TrustBadge.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { setKeyTrust, inspectKeyDetailed, type KeyDetailedInfo } from "$lib/tauri";
  import { formatDate } from "$lib/utils";

  const fp = appStore.modalProps.fingerprint ?? "";
  const keyInfo = $derived(keyStore.keys.find(k => k.fingerprint === fp));

  let detailed: KeyDetailedInfo | null = $state(null);
  let updating = $state(false);
  let showSubkeys = $state(false);

  // Load detailed info on mount
  $effect(() => {
    if (fp) {
      inspectKeyDetailed(fp).then(d => { detailed = d; }).catch(() => {});
    }
  });

  async function toggleTrust() {
    if (!keyInfo || updating) return;
    updating = true;
    try {
      const newLevel = keyInfo.trust_level >= 2 ? 1 : 2;
      await setKeyTrust(keyInfo.fingerprint, newLevel);
      await keyStore.refresh();
    } catch (e) {
      appStore.setStatus(`Failed to update trust: ${e}`);
    } finally {
      updating = false;
    }
  }
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

      {#if detailed && detailed.user_ids.length > 1}
        <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-3">
          <p class="text-xs font-medium text-[var(--color-text-secondary)] uppercase tracking-wide mb-2">
            All User IDs
          </p>
          {#each detailed.user_ids as uid}
            <div class="text-sm py-0.5">
              {uid.name ?? ""}{uid.email ? ` <${uid.email}>` : ""}
            </div>
          {/each}
        </div>
      {/if}

      {#if detailed && detailed.subkeys.length > 0}
        <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-3">
          <button
            class="w-full flex items-center justify-between text-xs font-medium text-[var(--color-text-secondary)] uppercase tracking-wide"
            onclick={() => showSubkeys = !showSubkeys}
          >
            <span>Subkeys ({detailed.subkeys.length})</span>
            <span class="text-base">{showSubkeys ? "−" : "+"}</span>
          </button>
          {#if showSubkeys}
            <div class="mt-2 space-y-2">
              {#each detailed.subkeys as subkey}
                <div class="text-sm border-t border-[var(--color-border)] pt-2">
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-xs text-[var(--color-text-secondary)]">
                      {subkey.fingerprint.slice(-16)}
                    </span>
                    {#if subkey.is_revoked}
                      <span class="text-xs px-1.5 py-0.5 rounded bg-red-100 text-red-600">Revoked</span>
                    {/if}
                  </div>
                  <div class="flex gap-2 mt-1 flex-wrap">
                    {#each subkey.capabilities as cap}
                      <span class="text-xs px-1.5 py-0.5 rounded bg-blue-100 text-blue-700">{cap}</span>
                    {/each}
                  </div>
                  <div class="text-xs text-[var(--color-text-secondary)] mt-1">
                    {subkey.algorithm} · Created {formatDate(subkey.created_at)}
                    {#if subkey.expires_at}
                      · Expires {formatDate(subkey.expires_at)}
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <div class="flex justify-between items-center pt-2">
        <div class="flex gap-2">
          {#if !keyInfo.is_own_key}
            <button
              class="px-3 py-1.5 text-sm rounded-lg border transition-colors
                     {keyInfo.trust_level >= 2
                       ? 'border-red-300 text-red-600 hover:bg-red-50'
                       : 'border-green-300 text-green-600 hover:bg-green-50'}"
              onclick={toggleTrust}
              disabled={updating}
            >
              {keyInfo.trust_level >= 2 ? "Revoke Verification" : "Mark as Verified"}
            </button>
          {/if}
          <button
            class="px-3 py-1.5 text-sm rounded-lg border border-[var(--color-border)] hover:bg-[var(--color-bg-secondary)] transition-colors"
            onclick={() => appStore.openModal("qr-export", { fingerprint: keyInfo.fingerprint })}
          >
            QR Code
          </button>
        </div>
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
