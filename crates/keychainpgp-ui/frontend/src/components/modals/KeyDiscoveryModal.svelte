<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { wkdLookup, keyserverSearch, importKey, exportKey, type KeyInfo } from "$lib/tauri";
  import * as m from "$lib/paraglide/messages.js";

  let query = $state("");
  let results: KeyInfo[] = $state([]);
  let searching = $state(false);
  let error: string | null = $state(null);
  let importedFps: Set<string> = $state(new Set());

  async function handleSearch() {
    if (!query.trim()) return;
    searching = true;
    error = null;
    results = [];

    try {
      // If it looks like an email, try WKD first
      if (query.includes("@")) {
        const wkdResult = await wkdLookup(query.trim());
        if (wkdResult) {
          results = [wkdResult];
          searching = false;
          return;
        }
      }

      // Fall back to keyserver search
      const ksResults = await keyserverSearch(query.trim());
      results = ksResults;

      if (results.length === 0) {
        error = m.discovery_not_found();
      }
    } catch (e) {
      error = String(e);
    } finally {
      searching = false;
    }
  }

  async function handleImport(key: KeyInfo) {
    try {
      const ksResults = await keyserverSearch(
        key.email ?? key.fingerprint
      );
      if (ksResults.length === 0) {
        appStore.setStatus(m.discovery_import_fail());
        return;
      }
      appStore.setStatus(m.discovery_import_hint());
    } catch (e) {
      appStore.setStatus(`${e}`);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleSearch();
  }
</script>

<ModalContainer title={m.discovery_title()}>
  <div class="space-y-4">
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={query}
        onkeydown={handleKeydown}
        placeholder={m.discovery_placeholder()}
        class="flex-1 px-3 py-2 text-sm rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      />
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50"
        onclick={handleSearch}
        disabled={searching || !query.trim()}
      >
        {searching ? m.discovery_searching() : m.discovery_search()}
      </button>
    </div>

    {#if error}
      <p class="text-sm text-red-600">{error}</p>
    {/if}

    {#if results.length > 0}
      <div class="space-y-2 max-h-64 overflow-auto">
        {#each results as key}
          <div class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
            <div class="text-sm">
              <p class="font-medium">{key.name ?? m.unnamed()}</p>
              <p class="text-[var(--color-text-secondary)]">{key.email ?? ""}</p>
              <p class="text-xs font-mono text-[var(--color-text-secondary)]">{key.fingerprint.slice(-16)}</p>
            </div>
            <span class="text-xs text-green-600 font-medium">{m.discovery_found()}</span>
          </div>
        {/each}
      </div>
      <p class="text-xs text-[var(--color-text-secondary)]">
        {m.discovery_import_hint()}
      </p>
    {/if}

    <div class="flex justify-end">
      <button
        class="px-4 py-2 text-sm rounded-lg border border-[var(--color-border)]
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        {m.discovery_close()}
      </button>
    </div>
  </div>
</ModalContainer>
