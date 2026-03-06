<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { importKey, importBackup } from "$lib/tauri";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { isMobile } from "$lib/platform";
  import { cancelScan } from "$lib/qr-scan";
  import QrScanOverlay from "../shared/QrScanOverlay.svelte";
  import { Camera } from "lucide-svelte";
  import * as m from "$lib/paraglide/messages.js";

  const mobile = isMobile();
  let scanning = $state(false);

  let keyData = $state("");
  let importing = $state(false);
  let error = $state("");
  let transferCode = $state("");

  // Detect OpenKeychain backup format
  const isBackup = $derived(keyData.includes("Passphrase-Format: numeric9x4"));

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
          parts.push(
            result.imported_count === 1
              ? m.import_backup_success_one()
              : m.import_backup_success_other({ count: result.imported_count }),
          );
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

{#if scanning}
  <QrScanOverlay
    onscan={(content) => {
      if (content.startsWith("KCPGP:")) {
        error = m.error_sync_qr_wrong_context();
        scanning = false;
        return true;
      }
      importKey(content)
        .then(async (result) => {
          await keyStore.refresh();
          appStore.setStatus(
            m.import_success_key({ name: result.name ?? result.fingerprint.slice(-8) }),
          );
          appStore.closeModal();
        })
        .catch((e) => {
          error = String(e);
        });
      scanning = false;
      return true;
    }}
    oncancel={() => {
      cancelScan();
      scanning = false;
    }}
  />
{/if}

<ModalContainer title={m.import_title()}>
  <div class="space-y-3">
    {#if mobile}
      <button
        class="flex w-full items-center justify-center gap-2 rounded-lg bg-[var(--color-primary)] py-3
               text-sm font-medium text-white
               transition-colors hover:bg-[var(--color-primary-hover)]"
        onclick={() => {
          error = "";
          scanning = true;
        }}
      >
        <Camera size={16} />
        {m.onboarding_scan_qr()}
      </button>
      <div class="text-center text-xs text-[var(--color-text-secondary)]">
        {m.import_or()}
      </div>
    {/if}
    <textarea
      placeholder={m.import_textarea_placeholder()}
      bind:value={keyData}
      rows={8}
      class="w-full resize-none rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3
             py-2.5 font-mono text-sm
             focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
    ></textarea>

    <div class="text-center text-xs text-[var(--color-text-secondary)]">
      {m.import_or()}
    </div>

    <label
      class="block w-full cursor-pointer rounded-lg border-2 border-dashed border-[var(--color-border)] py-3
             text-center text-sm transition-colors hover:border-[var(--color-primary)]"
    >
      {m.import_choose_file()}
      <input
        type="file"
        accept=".asc,.gpg,.pgp,.pub,.key,.sec.pgp"
        class="hidden"
        onchange={handleFileInput}
      />
    </label>

    {#if isBackup}
      <div
        class="space-y-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-3"
      >
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
          class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2.5
                 text-center font-mono text-sm tracking-wider
                 focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
        />
      </div>
    {/if}

    {#if error}
      <p class="text-sm text-[var(--color-danger)]">{error}</p>
    {/if}

    <div class="flex justify-end gap-2 pt-2">
      <button
        class="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm
               transition-colors hover:bg-[var(--color-bg-secondary)]"
        onclick={() => appStore.closeModal()}
      >
        {m.import_cancel()}
      </button>
      <button
        class="rounded-lg bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white
               transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
        onclick={handleImport}
        disabled={!keyData.trim() || (isBackup && !transferCode.trim()) || importing}
      >
        {importing ? m.import_loading() : isBackup ? m.import_submit_backup() : m.import_submit()}
      </button>
    </div>
  </div>
</ModalContainer>
