<script lang="ts">
  import { Check } from "lucide-svelte";
  import ModalContainer from "./ModalContainer.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { encryptClipboard } from "$lib/tauri";
  import { shortFingerprint } from "$lib/utils";

  let encrypting = $state(false);

  // Compute initial selection from encrypt-to-self settings
  function getInitialSelection(): Set<string> {
    const set = new Set<string>();
    if (settingsStore.settings.encrypt_to_self) {
      const selfKeys = settingsStore.settings.encrypt_to_self_keys;
      if (selfKeys.length > 0) {
        for (const fp of selfKeys) set.add(fp);
      } else {
        for (const k of keyStore.ownKeys) set.add(k.fingerprint);
      }
    }
    return set;
  }

  let selected: Set<string> = $state(getInitialSelection());

  function toggleKey(fp: string) {
    if (selected.has(fp)) {
      selected.delete(fp);
    } else {
      selected.add(fp);
    }
    selected = new Set(selected); // trigger reactivity
  }

  async function handleEncrypt() {
    if (selected.size === 0) return;
    encrypting = true;
    try {
      const result = await encryptClipboard([...selected]);
      if (result.success) {
        appStore.setStatus(result.message);
        appStore.closeModal();
        clipboardStore.refresh();
      } else {
        appStore.openModal("error", { error: result.message });
      }
    } catch (e) {
      appStore.openModal("error", { error: String(e) });
    } finally {
      encrypting = false;
    }
  }
</script>

<ModalContainer title="Select Recipients">
  <div class="space-y-3">
    {#if keyStore.keys.length === 0}
      <p class="text-sm text-[var(--color-text-secondary)]">No keys available.</p>
    {:else}
      <div class="space-y-1 max-h-60 overflow-auto">
        {#each keyStore.keys as k (k.fingerprint)}
          <button
            class="w-full flex items-center gap-3 p-3 rounded-lg border transition-colors text-left"
            class:border-[var(--color-primary)]={selected.has(k.fingerprint)}
            class:bg-[var(--color-bg-secondary)]={selected.has(k.fingerprint)}
            class:border-[var(--color-border)]={!selected.has(k.fingerprint)}
            onclick={() => toggleKey(k.fingerprint)}
          >
            <div class="w-5 h-5 rounded border flex items-center justify-center shrink-0"
              class:bg-[var(--color-primary)]={selected.has(k.fingerprint)}
              class:border-[var(--color-primary)]={selected.has(k.fingerprint)}
              class:border-[var(--color-border)]={!selected.has(k.fingerprint)}
            >
              {#if selected.has(k.fingerprint)}
                <Check size={14} class="text-white" />
              {/if}
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-sm font-medium truncate">
                {k.name ?? "(unnamed)"}
                {#if k.is_own_key}
                  <span class="text-xs text-[var(--color-primary)]">(you)</span>
                {/if}
              </p>
              <p class="text-xs text-[var(--color-text-secondary)] truncate">
                {k.email ?? shortFingerprint(k.fingerprint)}
              </p>
            </div>
          </button>
        {/each}
      </div>
    {/if}

    <div class="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
      <button
        class="px-4 py-2 text-sm rounded-lg border border-[var(--color-border)]
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        Cancel
      </button>
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50"
        onclick={handleEncrypt}
        disabled={selected.size === 0 || encrypting}
      >
        {encrypting ? "Encrypting..." : `Encrypt for ${selected.size} recipient${selected.size !== 1 ? "s" : ""}`}
      </button>
    </div>
  </div>
</ModalContainer>
