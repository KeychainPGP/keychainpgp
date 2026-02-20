<script lang="ts">
  import { Plus, Upload } from "lucide-svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { searchKeys } from "$lib/tauri";
  import type { KeyInfo } from "$lib/tauri";
  import SearchBar from "../shared/SearchBar.svelte";
  import LoadingSpinner from "../shared/LoadingSpinner.svelte";
  import KeyCard from "./KeyCard.svelte";
  import KeyGenerateForm from "./KeyGenerateForm.svelte";

  let query = $state("");
  let filteredKeys: KeyInfo[] = $state([]);
  let searching = $state(false);
  let showGenerateForm = $state(false);

  // Derive filtered keys from query
  $effect(() => {
    if (!query.trim()) {
      filteredKeys = keyStore.keys;
      return;
    }
    searching = true;
    searchKeys(query)
      .then((results) => (filteredKeys = results))
      .catch(() => (filteredKeys = keyStore.keys))
      .finally(() => (searching = false));
  });

  const ownKeys = $derived(filteredKeys.filter(k => k.is_own_key));
  const contactKeys = $derived(filteredKeys.filter(k => !k.is_own_key));
</script>

<div class="max-w-2xl mx-auto space-y-4">
  <div class="flex items-center justify-between gap-3">
    <h2 class="text-xl font-bold">Key Manager</h2>
    <div class="flex items-center gap-2">
      <button
        class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg
               bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors"
        onclick={() => (showGenerateForm = !showGenerateForm)}
      >
        <Plus size={16} />
        Generate
      </button>
      <button
        class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg
               border border-[var(--color-border)] font-medium
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.openModal("key-import")}
      >
        <Upload size={16} />
        Import
      </button>
    </div>
  </div>

  {#if showGenerateForm}
    <KeyGenerateForm onDone={() => (showGenerateForm = false)} />
  {/if}

  <SearchBar value={query} placeholder="Search by name, email, or fingerprint..." oninput={(v) => (query = v)} />

  {#if keyStore.loading}
    <LoadingSpinner />
  {:else if filteredKeys.length === 0}
    <p class="text-center text-[var(--color-text-secondary)] py-8">
      {query ? "No keys match your search." : "No keys yet. Generate or import one to get started."}
    </p>
  {:else}
    {#if ownKeys.length > 0}
      <div>
        <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide mb-2">
          Your Keys
        </h3>
        <div class="space-y-2">
          {#each ownKeys as k (k.fingerprint)}
            <KeyCard key={k} />
          {/each}
        </div>
      </div>
    {/if}

    {#if contactKeys.length > 0}
      <div>
        <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide mb-2">
          Contacts
        </h3>
        <div class="space-y-2">
          {#each contactKeys as k (k.fingerprint)}
            <KeyCard key={k} />
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>
