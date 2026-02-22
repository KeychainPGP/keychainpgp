<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { generateQrSvg, writeClipboard } from "$lib/tauri";
  import { isDesktop } from "$lib/platform";
  import { Heart, Copy, Check } from "lucide-svelte";
  import * as m from "$lib/paraglide/messages.js";

  const WALLETS = [
    { id: "btc", name: () => m.donate_btc(), prefix: "bitcoin:", address: "bc1qed0rtdgxq5t9g5lrsztxqvw9gnz4s2wd3gner3" },
    { id: "eth", name: () => m.donate_eth(), prefix: "ethereum:", address: "0xD2Ca6c40f03Db5A0493012C7D2792e69C8C493D8" },
    { id: "xmr", name: () => m.donate_xmr(), prefix: "monero:", address: "898XvFtvBMmes31kNwF4AJiNHsaWZA5aGLqf1jZT5ZZXPGCL4AgNAgZUgw9o6d3tg17fks3q1i4tWJ69tHAhh9MpQLLFGrf" },
  ] as const;

  let qrSvgs: Record<string, string> = $state({});
  let copiedId: string | null = $state(null);

  $effect(() => {
    for (const w of WALLETS) {
      generateQrSvg(`${w.prefix}${w.address}`)
        .then(svg => { qrSvgs[w.id] = svg; })
        .catch(() => {});
    }
  });

  async function copyAddress(id: string, address: string) {
    try {
      if (isDesktop()) {
        await writeClipboard(address);
      } else {
        await navigator.clipboard.writeText(address);
      }
      copiedId = id;
      setTimeout(() => { copiedId = null; }, 2000);
    } catch {
      // Silently fail
    }
  }
</script>

<ModalContainer title={m.donate_title()}>
  <div class="space-y-5">
    <div class="flex items-start gap-3">
      <div class="p-2 rounded-lg bg-[var(--color-primary)]/10 text-[var(--color-primary)]">
        <Heart size={20} />
      </div>
      <p class="text-sm text-[var(--color-text-secondary)]">
        {m.donate_desc()}
      </p>
    </div>

    {#each WALLETS as wallet}
      <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4 space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-semibold">{wallet.name()}</h3>
          <button
            class="inline-flex items-center gap-1 px-2.5 py-1 text-xs rounded-md
                   border border-[var(--color-border)] font-medium
                   hover:bg-[var(--color-bg)] transition-colors"
            onclick={() => copyAddress(wallet.id, wallet.address)}
          >
            {#if copiedId === wallet.id}
              <Check size={12} class="text-green-500" />
              {m.donate_copied()}
            {:else}
              <Copy size={12} />
              {m.donate_copy()}
            {/if}
          </button>
        </div>

        <p class="text-xs font-mono break-all text-[var(--color-text-secondary)] select-all">
          {wallet.address}
        </p>

        {#if qrSvgs[wallet.id]}
          <div class="flex justify-center p-3 bg-white rounded-lg">
            {@html qrSvgs[wallet.id]}
          </div>
        {:else}
          <p class="text-xs text-center text-[var(--color-text-secondary)]">
            {m.donate_qr_loading()}
          </p>
        {/if}
      </div>
    {/each}

    <p class="text-xs text-center text-[var(--color-text-secondary)]">
      {m.donate_thanks()}
    </p>

    <div class="flex justify-end">
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        {m.qr_close()}
      </button>
    </div>
  </div>
</ModalContainer>
