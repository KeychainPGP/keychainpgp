<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { importKey } from "$lib/tauri";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";

  let keyData = $state("");
  let importing = $state(false);
  let error = $state("");

  async function handleImport() {
    if (!keyData.trim()) {
      error = "Paste a PGP key (ASCII-armored).";
      return;
    }
    error = "";
    importing = true;
    try {
      const info = await importKey(keyData.trim());
      await keyStore.refresh();
      appStore.setStatus(`Imported key for ${info.name ?? info.fingerprint.slice(-8)}.`);
      appStore.closeModal();
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
    }
  }

  async function handleFileInput(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    keyData = await file.text();
  }
</script>

<ModalContainer title="Import Key">
  <div class="space-y-3">
    <textarea
      placeholder="Paste ASCII-armored PGP key here..."
      bind:value={keyData}
      rows={8}
      class="w-full px-3 py-2.5 text-sm rounded-lg border border-[var(--color-border)]
             bg-[var(--color-bg)] font-mono resize-none
             focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
    ></textarea>

    <div class="text-center text-xs text-[var(--color-text-secondary)]">
      or
    </div>

    <label
      class="block w-full py-3 text-center text-sm rounded-lg border-2 border-dashed
             border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors cursor-pointer"
    >
      Choose file...
      <input type="file" accept=".asc,.gpg,.pgp,.pub,.key" class="hidden" onchange={handleFileInput} />
    </label>

    {#if error}
      <p class="text-sm text-[var(--color-danger)]">{error}</p>
    {/if}

    <div class="flex justify-end gap-2 pt-2">
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
        onclick={handleImport}
        disabled={!keyData.trim() || importing}
      >
        {importing ? "Importing..." : "Import"}
      </button>
    </div>
  </div>
</ModalContainer>
