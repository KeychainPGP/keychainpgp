<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import {
    wkdLookup,
    wkdFetchAndImport,
    keyserverSearch,
    fetchAndImportKey,
    type KeyInfo,
    type DiscoveryResult,
  } from "$lib/tauri";
  import * as m from "$lib/paraglide/messages.js";

  interface SearchResult {
    key: DiscoveryResult | KeyInfo;
    source: string;
  }

  let query = $state("");
  let results: SearchResult[] = $state([]);
  let searching = $state(false);
  let error: string | null = $state(null);
  let importedFps: Set<string> = $state(new Set());

  async function handleSearch() {
    if (!query.trim()) return;
    searching = true;
    error = null;
    results = [];

    const isEmail = query.includes("@");

    let ksError: string | null = null;

    try {
      const ksPromise = keyserverSearch(query.trim()).catch((e) => {
        ksError = String(e);
        return [] as DiscoveryResult[];
      });

      let wkdPromise: Promise<KeyInfo | null> | undefined;
      if (isEmail) {
        wkdPromise = wkdLookup(query.trim()).catch(() => null);
      }

      const [ksResults, wkdResult] = await Promise.all([ksPromise, wkdPromise]);

      let allResults: SearchResult[] = (ksResults || []).map((r) => ({
        key: r,
        source: r.source,
      }));

      if (wkdResult) {
        const exists = allResults.some((r) => r.key.fingerprint === wkdResult.fingerprint);
        if (!exists) {
          allResults.push({ key: wkdResult, source: "WKD" });
        }
      }

      results = allResults;

      if (results.length === 0) {
        error = ksError ?? m.discovery_not_found();
      }
    } catch (e) {
      error = String(e);
    } finally {
      searching = false;
    }
  }

  async function handleImport(result: SearchResult) {
    try {
      searching = true;
      appStore.setStatus(m.discovery_searching());

      let importedKey: KeyInfo;

      if (result.source === "WKD" && result.key.email) {
        importedKey = await wkdFetchAndImport(result.key.email);
      } else {
        const allUrls = [
          settingsStore.settings.keyserver_url,
          settingsStore.settings.unverified_keyserver_url,
        ]
          .filter(Boolean)
          .join(",");
        importedKey = await fetchAndImportKey(result.key.fingerprint, allUrls);
      }

      await keyStore.refresh();
      importedFps.add(result.key.fingerprint);
      appStore.setStatus(
        m.import_success_key({
          name: (importedKey.name ?? importedKey.email ?? importedKey.fingerprint) || "",
        }),
      );
      appStore.closeModal();
    } catch (e) {
      error = String(e);
      appStore.setStatus(`${e}`);
    } finally {
      searching = false;
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
        class="flex-1 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm
               focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
      />
      <button
        class="rounded-lg bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white
               transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
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
      <div class="max-h-64 space-y-2 overflow-auto">
        {#each results as result}
          <div
            class="rounded-lg border border-[var(--color-border)] p-3 {result.source === 'WKD'
              ? 'opacity-75'
              : ''}"
          >
            <div class="flex items-center justify-between">
              <div class="text-sm">
                <p class="font-medium">{result.key.name ?? m.unnamed()}</p>
                <p class="text-[var(--color-text-secondary)]">
                  {result.key.email ?? ""}
                </p>
                <div class="mt-0.5 flex flex-wrap items-center gap-1.5">
                  <p class="font-mono text-xs text-[var(--color-text-secondary)]">
                    {result.key.fingerprint.slice(-16)}
                  </p>
                  {#each result.source.split(", ") as src}
                    <span
                      class="rounded-full px-1.5 py-0.5 text-[10px] font-medium {src === 'WKD'
                        ? 'bg-blue-100 text-blue-700'
                        : 'bg-gray-100 text-gray-600'}"
                    >
                      {src}
                    </span>
                  {/each}
                </div>
              </div>
              {#if importedFps.has(result.key.fingerprint)}
                <span class="text-xs font-medium text-green-600">{m.discovery_found()}</span>
              {:else}
                <button
                  class="rounded-md bg-[var(--color-primary)] px-3 py-1 text-xs font-medium text-white
                         transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
                  onclick={() => handleImport(result)}
                  disabled={searching}
                >
                  {searching ? m.discovery_searching() : m.keys_import_btn()}
                </button>
              {/if}
            </div>
            {#if result.source === "WKD"}
              <p class="mt-2 text-xs text-amber-600">
                {m.discovery_wkd_warning()}
              </p>
            {/if}
          </div>
        {/each}
      </div>
      <p class="text-xs text-[var(--color-text-secondary)]">
        {m.discovery_import_hint()}
      </p>
    {/if}

    <div class="flex justify-end">
      <button
        class="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm
               transition-colors hover:bg-[var(--color-bg-secondary)]"
        onclick={() => appStore.closeModal()}
      >
        {m.discovery_close()}
      </button>
    </div>
  </div>
</ModalContainer>
