<script lang="ts">
  import { Check, Search, ChevronDown, ChevronRight } from "lucide-svelte";
  import ModalContainer from "./ModalContainer.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { encryptClipboard, encryptText, writeClipboard } from "$lib/tauri";
  import { shortFingerprint } from "$lib/utils";
  import type { KeyInfo } from "$lib/tauri";
  import * as m from "$lib/paraglide/messages.js";

  let encrypting = $state(false);
  let searchQuery = $state("");
  let showMyKeys = $state(localStorage.getItem("recipient-show-my-keys") === "true");
  let showContacts = $state(localStorage.getItem("recipient-show-contacts") !== "false");

  // Persist collapse state
  $effect(() => { localStorage.setItem("recipient-show-my-keys", String(showMyKeys)); });
  $effect(() => { localStorage.setItem("recipient-show-contacts", String(showContacts)); });

  // If text is provided via modal props, encrypt that instead of clipboard
  let textToEncrypt = appStore.modalProps.text ?? null;

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

  function matchesSearch(k: KeyInfo): boolean {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return (k.name?.toLowerCase().includes(q) ?? false)
      || (k.email?.toLowerCase().includes(q) ?? false)
      || k.fingerprint.toLowerCase().includes(q);
  }

  let filteredOwnKeys = $derived(keyStore.ownKeys.filter(matchesSearch));
  let filteredContactKeys = $derived(keyStore.contactKeys.filter(matchesSearch));

  function toggleKey(fp: string) {
    if (selected.has(fp)) {
      selected.delete(fp);
    } else {
      selected.add(fp);
    }
    selected = new Set(selected);
  }

  function selectedInGroup(keys: KeyInfo[]): number {
    return keys.filter(k => selected.has(k.fingerprint)).length;
  }

  function encryptButtonLabel(count: number): string {
    if (count === 1) return m.recipient_encrypt_btn_one();
    return m.recipient_encrypt_btn_other({ count });
  }

  async function handleEncrypt() {
    if (selected.size === 0) return;
    encrypting = true;
    try {
      if (textToEncrypt) {
        const result = await encryptText(textToEncrypt, [...selected]);
        if (result.success) {
          await writeClipboard(result.message);
          appStore.setStatus(m.recipient_encrypt_success());
          appStore.closeModal();
        } else {
          appStore.openModal("error", { error: result.message });
        }
      } else {
        const result = await encryptClipboard([...selected]);
        if (result.success) {
          appStore.setStatus(result.message);
          appStore.closeModal();
          clipboardStore.refresh();
        } else {
          appStore.openModal("error", { error: result.message });
        }
      }
    } catch (e) {
      appStore.openModal("error", { error: String(e) });
    } finally {
      encrypting = false;
    }
  }
</script>

<ModalContainer title={m.recipient_title()}>
  <div class="space-y-3">
    <!-- Search bar -->
    <div class="relative">
      <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-secondary)]" />
      <input
        type="text"
        placeholder={m.recipient_search_placeholder()}
        bind:value={searchQuery}
        class="w-full pl-9 pr-3 py-2 text-sm rounded-lg border border-[var(--color-border)]
               bg-[var(--color-bg)] text-[var(--color-text)]
               placeholder-[var(--color-text-secondary)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      />
    </div>

    {#if keyStore.keys.length === 0}
      <p class="text-sm text-[var(--color-text-secondary)]">{m.recipient_no_keys()}</p>
    {:else}
      <div class="max-h-72 overflow-auto space-y-2">
        <!-- My Keys -->
        {#if keyStore.ownKeys.length > 0}
          <div>
            <button
              class="flex items-center gap-1.5 w-full py-1 text-xs font-semibold
                     text-[var(--color-text-secondary)] uppercase tracking-wide hover:text-[var(--color-text)]"
              onclick={() => showMyKeys = !showMyKeys}
            >
              {#if showMyKeys}
                <ChevronDown size={14} />
              {:else}
                <ChevronRight size={14} />
              {/if}
              {m.recipient_my_keys()}
              {#if selectedInGroup(keyStore.ownKeys) > 0}
                <span class="text-[var(--color-primary)] normal-case tracking-normal">
                  ({selectedInGroup(keyStore.ownKeys)})
                </span>
              {/if}
            </button>
            {#if showMyKeys}
              <div class="space-y-1 mt-1">
                {#each filteredOwnKeys as k (k.fingerprint)}
                  <button
                    class="w-full flex items-center gap-3 p-2.5 rounded-lg border transition-colors text-left"
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
                      <p class="text-sm font-medium truncate">{k.name ?? m.unnamed()}</p>
                      <p class="text-xs text-[var(--color-text-secondary)] truncate">
                        {k.email ?? shortFingerprint(k.fingerprint)}
                      </p>
                    </div>
                  </button>
                {/each}
                {#if filteredOwnKeys.length === 0 && searchQuery}
                  <p class="text-xs text-[var(--color-text-secondary)] px-2 py-1 italic">{m.recipient_no_match()}</p>
                {/if}
              </div>
            {/if}
          </div>
        {/if}

        <!-- Contacts -->
        {#if keyStore.contactKeys.length > 0}
          <div>
            <button
              class="flex items-center gap-1.5 w-full py-1 text-xs font-semibold
                     text-[var(--color-text-secondary)] uppercase tracking-wide hover:text-[var(--color-text)]"
              onclick={() => showContacts = !showContacts}
            >
              {#if showContacts}
                <ChevronDown size={14} />
              {:else}
                <ChevronRight size={14} />
              {/if}
              {m.recipient_contacts()}
              {#if selectedInGroup(keyStore.contactKeys) > 0}
                <span class="text-[var(--color-primary)] normal-case tracking-normal">
                  ({selectedInGroup(keyStore.contactKeys)})
                </span>
              {/if}
            </button>
            {#if showContacts}
              <div class="space-y-1 mt-1">
                {#each filteredContactKeys as k (k.fingerprint)}
                  <button
                    class="w-full flex items-center gap-3 p-2.5 rounded-lg border transition-colors text-left"
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
                      <p class="text-sm font-medium truncate">{k.name ?? m.unnamed()}</p>
                      <p class="text-xs text-[var(--color-text-secondary)] truncate">
                        {k.email ?? shortFingerprint(k.fingerprint)}
                      </p>
                    </div>
                  </button>
                {/each}
                {#if filteredContactKeys.length === 0 && searchQuery}
                  <p class="text-xs text-[var(--color-text-secondary)] px-2 py-1 italic">{m.recipient_no_match()}</p>
                {/if}
              </div>
            {/if}
          </div>
        {/if}

        {#if filteredOwnKeys.length === 0 && filteredContactKeys.length === 0 && searchQuery}
          <p class="text-sm text-[var(--color-text-secondary)] text-center py-2">{m.recipient_no_match_global({ query: searchQuery })}</p>
        {/if}
      </div>
    {/if}

    <div class="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
      <button
        class="px-4 py-2 text-sm rounded-lg border border-[var(--color-border)]
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        {m.recipient_cancel()}
      </button>
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50"
        onclick={handleEncrypt}
        disabled={selected.size === 0 || encrypting}
      >
        {encrypting ? m.recipient_encrypting() : encryptButtonLabel(selected.size)}
      </button>
    </div>
  </div>
</ModalContainer>
