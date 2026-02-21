<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { importKeyBundle } from "$lib/tauri";
  import { parseKcpgpPart, cancelScan } from "$lib/qr-scan";
  import { isMobile } from "$lib/platform";
  import { Upload, Camera } from "lucide-svelte";
  import QrScanOverlay from "../shared/QrScanOverlay.svelte";
  import * as m from "$lib/paraglide/messages.js";

  const mobile = isMobile();

  let encryptedData = $state("");
  let passphrase = $state("");
  let error: string | null = $state(null);
  let importing = $state(false);
  let importedCount: number | null = $state(null);

  // Multi-part QR scanning state
  let scanning = $state(false);
  let scannedParts = $state<Map<number, string>>(new Map());
  let totalParts = $state(0);
  let passphraseFromQr = $state(false);

  async function handleFileLoad(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    encryptedData = await file.text();
  }

  function handleScanResult(content: string): boolean {
    // Auto-detect passphrase QR
    if (content.startsWith("KCPGP-PASS:")) {
      passphrase = content.slice(11);
      passphraseFromQr = true;
      return false; // keep scanning for data parts
    }

    const part = parseKcpgpPart(content);
    if (!part) {
      error = "Not a sync QR code. Use the Import Key function for single keys.";
      return true; // stop
    }

    totalParts = part.total;
    scannedParts.set(part.part, part.data);
    scannedParts = new Map(scannedParts); // reactivity

    if (scannedParts.size >= part.total) {
      // All parts collected — reassemble
      const sorted = Array.from(scannedParts.entries())
        .sort(([a], [b]) => a - b)
        .map(([, data]) => data);
      encryptedData = sorted.join("");
      return true; // stop
    }
    return false; // continue scanning
  }

  function startScan() {
    error = null;
    scannedParts = new Map();
    totalParts = 0;
    passphraseFromQr = false;
    scanning = true;
  }

  function handleCancelScan() {
    cancelScan();
    scanning = false;
  }

  async function handleImport() {
    if (!encryptedData.trim()) {
      error = m.sync_error_no_data();
      return;
    }
    if (!passphrase.trim()) {
      error = m.sync_error_no_passphrase();
      return;
    }
    error = null;
    importing = true;
    try {
      const count = await importKeyBundle(encryptedData.trim(), passphrase.trim());
      importedCount = count;
      await keyStore.refresh();
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
    }
  }

  // Auto-import when QR scan captured both data and passphrase
  $effect(() => {
    if (passphraseFromQr && encryptedData && passphrase && !importing && importedCount === null && !scanning) {
      handleImport();
    }
  });

  const scanProgress = $derived(
    totalParts > 0
      ? m.sync_qr_progress({ current: scannedParts.size, total: totalParts })
      : undefined
  );
</script>

{#if scanning}
  <QrScanOverlay
    onscan={(content) => {
      const done = handleScanResult(content);
      if (done) scanning = false;
      return done;
    }}
    oncancel={handleCancelScan}
    progress={scanProgress}
  />
{/if}

<ModalContainer title={m.sync_import_title()}>
  <div class="space-y-4">
    {#if importedCount !== null}
      <!-- Success -->
      <div class="p-4 rounded-lg bg-green-50 border border-green-200 text-green-800">
        <p class="text-sm font-medium">
          {importedCount === 1
            ? m.sync_success_count_one()
            : m.sync_success_count_other({ count: importedCount })}
        </p>
      </div>
      <div class="flex justify-end">
        <button
          class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
                 hover:bg-[var(--color-primary-hover)] transition-colors"
          onclick={() => appStore.closeModal()}
        >
          {m.done()}
        </button>
      </div>
    {:else}
      <!-- Data input -->
      <div class="space-y-2">
        {#if mobile}
          <button
            class="w-full flex items-center justify-center gap-2 px-4 py-3 text-sm rounded-lg
                   bg-[var(--color-primary)] text-white font-medium
                   hover:bg-[var(--color-primary-hover)] transition-colors"
            onclick={startScan}
          >
            <Camera size={16} />
            {m.onboarding_scan_qr()}
          </button>
          {#if scannedParts.size > 0 && scannedParts.size < totalParts}
            <p class="text-xs text-center text-[var(--color-warning)]">
              {m.sync_qr_progress({ current: scannedParts.size, total: totalParts })}
            </p>
          {:else if encryptedData && totalParts > 0}
            <p class="text-xs text-center text-[var(--color-success)]">
              {m.sync_qr_progress({ current: totalParts, total: totalParts })}
            </p>
          {/if}
          <p class="text-xs text-center text-[var(--color-text-secondary)]">{m.import_or()}</p>
        {/if}
        <p class="text-sm font-medium">{m.sync_file_load()}</p>
        <div class="flex gap-2">
          <label
            class="flex-1 flex items-center justify-center gap-2 px-4 py-3 text-sm rounded-lg
                   border-2 border-dashed border-[var(--color-border)] cursor-pointer
                   hover:border-[var(--color-primary)] transition-colors"
          >
            <Upload size={16} />
            {encryptedData ? m.sync_file_loaded() : m.sync_file_choose()}
            <input type="file" accept=".enc,.txt" class="hidden" onchange={handleFileLoad} />
          </label>
        </div>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.sync_or_paste()}</p>
        <textarea
          bind:value={encryptedData}
          rows={3}
          class="w-full px-3 py-2 text-sm rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)]
                 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] resize-none font-mono"
          placeholder={m.sync_paste_placeholder()}
        ></textarea>
      </div>

      <!-- Passphrase input -->
      <div class="space-y-1">
        <label class="block text-sm font-medium" for="sync-passphrase">{m.sync_passphrase_label()}</label>
        <input
          id="sync-passphrase"
          type="text"
          bind:value={passphrase}
          class="w-full px-3 py-2 text-sm rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)]
                 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] font-mono tracking-wider"
          placeholder="1234-5678-9012-3456-7890-1234"
        />
      </div>

      {#if error}
        <p class="text-sm text-red-600">{error}</p>
      {/if}

      <div class="flex justify-end gap-2">
        <button
          class="px-4 py-2 text-sm rounded-lg border border-[var(--color-border)] font-medium
                 hover:bg-[var(--color-bg-secondary)] transition-colors"
          onclick={() => appStore.closeModal()}
        >
          {m.cancel()}
        </button>
        <button
          class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
                 hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50"
          onclick={handleImport}
          disabled={importing}
        >
          {importing ? m.sync_importing() : m.sync_import_btn()}
        </button>
      </div>
    {/if}
  </div>
</ModalContainer>
