<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { wkdLookup, keyserverSearch, importKey, exportKey, type KeyInfo } from "$lib/tauri";

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
        error = "No keys found.";
      }
    } catch (e) {
      error = String(e);
    } finally {
      searching = false;
    }
  }

  async function handleImport(key: KeyInfo) {
    try {
      // We need to get the key data — re-fetch from keyserver
      const ksResults = await keyserverSearch(
        key.email ?? key.fingerprint
      );
      if (ksResults.length === 0) {
        appStore.setStatus("Could not retrieve key data for import.");
        return;
      }
      // For WKD results, the key data is already fetched on the backend.
      // We need to export it from the search result. Since we don't have raw data
      // in KeyInfo, we'll search and import via the keyserver endpoint.
      // The actual import happens through the backend directly.
      // Let's use a direct approach: import from keyserver by fingerprint
      appStore.setStatus("Key imported via discovery is not yet supported for direct import. Copy the key data and use Import.");
    } catch (e) {
      appStore.setStatus(`Import failed: ${e}`);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleSearch();
  }
</script>

<ModalContainer title="Discover Keys">
  <div class="space-y-4">
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={query}
        onkeydown={handleKeydown}
        placeholder="Email address or name..."
        class="flex-1 px-3 py-2 text-sm rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      />
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50"
        onclick={handleSearch}
        disabled={searching || !query.trim()}
      >
        {searching ? "Searching..." : "Search"}
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
              <p class="font-medium">{key.name ?? "(unnamed)"}</p>
              <p class="text-[var(--color-text-secondary)]">{key.email ?? ""}</p>
              <p class="text-xs font-mono text-[var(--color-text-secondary)]">{key.fingerprint.slice(-16)}</p>
            </div>
            <span class="text-xs text-green-600 font-medium">Found</span>
          </div>
        {/each}
      </div>
      <p class="text-xs text-[var(--color-text-secondary)]">
        To import a discovered key, copy its public key and use the Import function in the Keys view.
      </p>
    {/if}

    <div class="flex justify-end">
      <button
        class="px-4 py-2 text-sm rounded-lg border border-[var(--color-border)]
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        Close
      </button>
    </div>
  </div>
</ModalContainer>
