<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { exportKeyBundle, type SyncBundle } from "$lib/tauri";
  import { Copy, Download, Pause, Play, ChevronLeft, ChevronRight } from "lucide-svelte";
  import * as m from "$lib/paraglide/messages.js";

  let bundle: SyncBundle | null = $state(null);
  let error: string | null = $state(null);
  let loading = $state(true);
  let currentQrIndex = $state(0);
  let qrFixedSize = $state(0);
  let passphraseCopied = $state(false);
  let autoPlay = $state(true);
  let intervalId: ReturnType<typeof setInterval> | null = $state(null);
  /** Interval in ms — 200 (fast) … 2000 (slow). Default 600. */
  let speed = $state(600);

  $effect(() => {
    exportKeyBundle()
      .then((b) => {
        bundle = b;
        // Compute the largest SVG width across all QR parts to lock the container size
        let maxSize = 0;
        for (const svg of b.qr_parts) {
          const match = svg.match(/width="(\d+)"/);
          if (match) maxSize = Math.max(maxSize, Number(match[1]));
        }
        qrFixedSize = maxSize;
        loading = false;
        if (b.qr_parts.length > 1) startAutoPlay();
      })
      .catch((e) => {
        error = String(e);
        loading = false;
      });

    return () => {
      if (intervalId) clearInterval(intervalId);
    };
  });

  function startAutoPlay() {
    if (intervalId) clearInterval(intervalId);
    autoPlay = true;
    intervalId = setInterval(() => {
      if (!bundle) return;
      currentQrIndex = (currentQrIndex + 1) % bundle.qr_parts.length;
    }, speed);
  }

  function restartIfPlaying() {
    if (autoPlay) startAutoPlay();
  }

  function stopAutoPlay() {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    autoPlay = false;
  }

  function toggleAutoPlay() {
    if (autoPlay) stopAutoPlay();
    else startAutoPlay();
  }

  function goPrev() {
    if (!bundle) return;
    currentQrIndex = (currentQrIndex - 1 + bundle.qr_parts.length) % bundle.qr_parts.length;
  }

  function goNext() {
    if (!bundle) return;
    currentQrIndex = (currentQrIndex + 1) % bundle.qr_parts.length;
  }

  function handleSpeedChange(e: Event) {
    speed = Number((e.currentTarget as HTMLInputElement).value);
    restartIfPlaying();
  }

  function copyPassphrase() {
    if (bundle) {
      navigator.clipboard.writeText(bundle.passphrase);
      passphraseCopied = true;
      setTimeout(() => (passphraseCopied = false), 2000);
    }
  }

  function downloadFile() {
    if (!bundle) return;
    const blob = new Blob([bundle.file_data], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "keychainpgp-sync.enc";
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<ModalContainer title={m.sync_export_title()}>
  <div class="space-y-4">
    {#if loading}
      <p class="text-sm text-[var(--color-text-secondary)]">{m.sync_exporting()}</p>
    {:else if error}
      <p class="text-sm text-red-600">{error}</p>
    {:else if bundle}
      <!-- Passphrase display -->
      <div class="p-4 rounded-lg bg-[var(--color-bg-secondary)] border border-[var(--color-border)]">
        <p class="text-xs text-[var(--color-text-secondary)] mb-1">{m.sync_passphrase_label()}</p>
        <div class="flex items-center gap-2">
          <code class="text-lg font-mono font-bold tracking-wider flex-1">{bundle.passphrase}</code>
          <button
            class="p-1.5 rounded hover:bg-[var(--color-border)] transition-colors"
            onclick={copyPassphrase}
            title="Copy"
          >
            <Copy size={16} />
          </button>
        </div>
        <p class="text-xs text-[var(--color-text-secondary)] mt-1">
          {passphraseCopied ? m.sync_passphrase_copied() : m.sync_passphrase_desc()}
        </p>
      </div>

      <!-- QR code carousel with auto-play -->
      {#if bundle.qr_parts.length > 0}
        <div class="space-y-2">
          <div
            class="flex justify-center items-center p-4 bg-white rounded-lg"
            style={qrFixedSize ? `min-width:${qrFixedSize + 32}px;min-height:${qrFixedSize + 32}px` : ''}
          >
            <img src="data:image/svg+xml;base64,{btoa(bundle.qr_parts[currentQrIndex])}" alt="QR Code" />
          </div>
          {#if bundle.qr_parts.length > 1}
            <!-- Controls: arrows + play/pause + counter -->
            <div class="flex items-center justify-center gap-2">
              {#if !autoPlay}
                <button
                  class="p-1.5 rounded hover:bg-[var(--color-bg-secondary)] transition-colors"
                  onclick={goPrev}
                >
                  <ChevronLeft size={18} />
                </button>
              {/if}
              <button
                class="p-1.5 rounded hover:bg-[var(--color-bg-secondary)] transition-colors"
                onclick={toggleAutoPlay}
                title={autoPlay ? "Pause" : "Play"}
              >
                {#if autoPlay}
                  <Pause size={18} />
                {:else}
                  <Play size={18} />
                {/if}
              </button>
              {#if !autoPlay}
                <button
                  class="p-1.5 rounded hover:bg-[var(--color-bg-secondary)] transition-colors"
                  onclick={goNext}
                >
                  <ChevronRight size={18} />
                </button>
              {/if}
              <span class="text-sm font-medium tabular-nums">
                {currentQrIndex + 1}/{bundle.qr_parts.length}
              </span>
            </div>
            <!-- Speed slider -->
            <div class="flex items-center gap-2 px-2">
              <input
                type="range"
                min="200"
                max="2000"
                step="100"
                value={speed}
                oninput={handleSpeedChange}
                class="flex-1 h-1.5 accent-[var(--color-primary)] cursor-pointer"
                style="direction: rtl;"
              />
              <span class="text-xs text-[var(--color-text-secondary)] tabular-nums shrink-0">{speed}ms</span>
            </div>
          {/if}
        </div>
      {/if}

      <!-- File download fallback -->
      <button
        class="w-full flex items-center justify-center gap-2 px-4 py-2 text-sm rounded-lg
               border border-[var(--color-border)] font-medium
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={downloadFile}
      >
        <Download size={16} />
        {m.sync_file_save()}
      </button>
    {/if}

    <div class="flex justify-end">
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        {m.done()}
      </button>
    </div>
  </div>
</ModalContainer>
