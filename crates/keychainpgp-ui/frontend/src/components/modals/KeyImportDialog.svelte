<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { importKey, importBackup } from "$lib/tauri";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";

  let keyData = $state("");
  let importing = $state(false);
  let error = $state("");
  let transferCode = $state("");

  // Detect OpenKeychain backup format
  const isBackup = $derived(
    keyData.includes("Passphrase-Format: numeric9x4")
  );

  // Extract Passphrase-Begin hint
  const passphraseBegin = $derived.by(() => {
    const match = keyData.match(/Passphrase-Begin:\s*(\d+)/);
    return match ? match[1] : null;
  });

  async function handleImport() {
    if (!keyData.trim()) {
      error = "Paste a PGP key or backup file.";
      return;
    }

    if (isBackup) {
      if (!transferCode.trim()) {
        error = "Enter the transfer code shown during OpenKeychain backup.";
        return;
      }
      error = "";
      importing = true;
      try {
        const result = await importBackup(keyData.trim(), transferCode.trim());
        await keyStore.refresh();
        const parts = [];
        if (result.imported_count > 0) {
          parts.push(`Imported ${result.imported_count} key${result.imported_count !== 1 ? "s" : ""}`);
        }
        if (result.skipped_count > 0) {
          parts.push(`${result.skipped_count} already in keyring`);
        }
        appStore.setStatus(parts.join(", ") + ".");
        appStore.closeModal();
      } catch (e) {
        error = String(e);
      } finally {
        importing = false;
      }
    } else {
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
      placeholder="Paste ASCII-armored PGP key or backup file..."
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
      <input type="file" accept=".asc,.gpg,.pgp,.pub,.key,.sec.pgp" class="hidden" onchange={handleFileInput} />
    </label>

    {#if isBackup}
      <div class="p-3 rounded-lg bg-[var(--color-bg-secondary)] border border-[var(--color-border)] space-y-2">
        <p class="text-sm font-medium">OpenKeychain Backup Detected</p>
        <p class="text-xs text-[var(--color-text-secondary)]">
          Enter the transfer code shown when you created this backup.
          {#if passphraseBegin}
            It starts with <strong>{passphraseBegin}</strong>.
          {/if}
        </p>
        <input
          type="text"
          placeholder="1234-5678-9012-3456-7890-1234-5678-9012-3456"
          bind:value={transferCode}
          class="w-full px-3 py-2.5 text-sm rounded-lg border border-[var(--color-border)]
                 bg-[var(--color-bg)] font-mono tracking-wider text-center
                 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
        />
      </div>
    {/if}

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
        disabled={!keyData.trim() || (isBackup && !transferCode.trim()) || importing}
      >
        {importing ? "Importing..." : isBackup ? "Import Backup" : "Import"}
      </button>
    </div>
  </div>
</ModalContainer>
