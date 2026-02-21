<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { importKey, importBackup } from "$lib/tauri";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import * as m from "$lib/paraglide/messages.js";

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
      error = m.import_empty_error();
      return;
    }

    if (isBackup) {
      if (!transferCode.trim()) {
        error = m.import_backup_transfer_error();
        return;
      }
      error = "";
      importing = true;
      try {
        const result = await importBackup(keyData.trim(), transferCode.trim());
        await keyStore.refresh();
        const parts = [];
        if (result.imported_count > 0) {
          parts.push(result.imported_count === 1
            ? m.import_backup_success_one()
            : m.import_backup_success_other({ count: result.imported_count }));
        }
        if (result.skipped_count > 0) {
          parts.push(m.import_backup_skipped({ count: result.skipped_count }));
        }
        appStore.setStatus(parts.join("") + ".");
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
        appStore.setStatus(m.import_success_key({ name: info.name ?? info.fingerprint.slice(-8) }));
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

<ModalContainer title={m.import_title()}>
  <div class="space-y-3">
    <textarea
      placeholder={m.import_textarea_placeholder()}
      bind:value={keyData}
      rows={8}
      class="w-full px-3 py-2.5 text-sm rounded-lg border border-[var(--color-border)]
             bg-[var(--color-bg)] font-mono resize-none
             focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
    ></textarea>

    <div class="text-center text-xs text-[var(--color-text-secondary)]">
      {m.import_or()}
    </div>

    <label
      class="block w-full py-3 text-center text-sm rounded-lg border-2 border-dashed
             border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors cursor-pointer"
    >
      {m.import_choose_file()}
      <input type="file" accept=".asc,.gpg,.pgp,.pub,.key,.sec.pgp" class="hidden" onchange={handleFileInput} />
    </label>

    {#if isBackup}
      <div class="p-3 rounded-lg bg-[var(--color-bg-secondary)] border border-[var(--color-border)] space-y-2">
        <p class="text-sm font-medium">{m.import_backup_detected()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">
          {m.import_backup_desc()}
          {#if passphraseBegin}
            {m.import_backup_starts_with({ begin: passphraseBegin })}
          {/if}
        </p>
        <input
          type="text"
          placeholder={m.import_backup_placeholder()}
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
        {m.import_cancel()}
      </button>
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50"
        onclick={handleImport}
        disabled={!keyData.trim() || (isBackup && !transferCode.trim()) || importing}
      >
        {importing ? m.import_loading() : isBackup ? m.import_submit_backup() : m.import_submit()}
      </button>
    </div>
  </div>
</ModalContainer>
