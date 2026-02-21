<script lang="ts">
  /**
   * Full-screen QR scan overlay with live camera feed via <video> element.
   * Uses qr-scanner (JS/WebWorker) for continuous scanning — no native plugin,
   * no camera restart, no blinking, no refocus between scans.
   */
  import { X } from "lucide-svelte";
  import { onMount } from "svelte";
  import { startContinuousScan, cancelScan } from "$lib/qr-scan";
  import * as m from "$lib/paraglide/messages.js";

  interface Props {
    /** Called with each scanned QR content. Return true to stop scanning. */
    onscan: (content: string) => boolean;
    /** Called when user taps cancel. */
    oncancel: () => void;
    /** Optional progress text, e.g. "2 / 5" */
    progress?: string;
  }
  let { onscan, oncancel, progress }: Props = $props();

  let videoEl: HTMLVideoElement | undefined = $state();
  let videoReady = $state(false);
  let error: string | null = $state(null);
  let cleanup: (() => void) | null = null;

  onMount(() => {
    if (!videoEl) return;
    // Show the video once the camera stream actually renders a frame
    videoEl.addEventListener("playing", () => { videoReady = true; }, { once: true });
    cleanup = startContinuousScan(
      videoEl,
      onscan,
      (err) => { error = err; },
    );
    return () => {
      if (cleanup) cleanup();
    };
  });

  function handleCancel() {
    if (cleanup) cleanup();
    cancelScan();
    oncancel();
  }
</script>

<div class="fixed inset-0 z-[9999] bg-black">
  <!-- Camera video feed (full-screen, hidden until stream starts) -->
  <!-- svelte-ignore element_invalid_self_closing_tag -->
  <video
    bind:this={videoEl}
    class="absolute inset-0 w-full h-full object-cover transition-opacity duration-200"
    class:opacity-0={!videoReady}
    playsinline
  />

  <!-- Dark overlay with viewfinder cutout (box-shadow trick) -->
  <div
    class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 rounded-2xl border-4 border-white/80"
    style="box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.6);"
  ></div>

  <!-- Controls layer -->
  <div class="relative z-10 flex flex-col items-center justify-between h-full py-16 pointer-events-none">
    <!-- Top: progress / error -->
    <div class="pointer-events-auto">
      {#if error}
        <div class="px-4 py-2 rounded-full bg-red-600/90 text-white text-sm font-medium">
          {error}
        </div>
      {:else if progress}
        <div class="px-4 py-2 rounded-full bg-black/80 text-white text-sm font-medium">
          {progress}
        </div>
      {/if}
    </div>

    <!-- Spacer -->
    <div></div>

    <!-- Bottom: cancel button -->
    <button
      class="pointer-events-auto flex items-center gap-2 px-8 py-3 rounded-full bg-black/80 text-white font-medium text-base active:bg-black/90"
      onclick={handleCancel}
    >
      <X size={18} />
      {m.cancel()}
    </button>
  </div>
</div>
