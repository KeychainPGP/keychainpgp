<script lang="ts">
  import { Download, Trash2, Info } from "lucide-svelte";
  import type { KeyInfo } from "$lib/tauri";
  import { exportKey, deleteKey } from "$lib/tauri";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import TrustBadge from "../shared/TrustBadge.svelte";
  import FingerprintDisplay from "../shared/FingerprintDisplay.svelte";
  import { formatDate } from "$lib/utils";
  import * as m from "$lib/paraglide/messages.js";

  interface Props {
    key: KeyInfo;
  }
  let { key: keyInfo }: Props = $props();

  async function handleExport() {
    try {
      const armored = await exportKey(keyInfo.fingerprint);
      await navigator.clipboard.writeText(armored);
      appStore.setStatus(m.keys_export_success());
    } catch (e) {
      appStore.openModal("error", { error: String(e) });
    }
  }

  function handleDelete() {
    appStore.openModal("confirm", {
      title: m.key_delete_title(),
      message: m.key_delete_message({
        name: keyInfo.name ?? keyInfo.email ?? keyInfo.fingerprint.slice(-8),
      }),
      onConfirm: async () => {
        try {
          await deleteKey(keyInfo.fingerprint);
          await keyStore.refresh();
          appStore.setStatus(m.keys_deleted());
          appStore.closeModal();
        } catch (e) {
          appStore.openModal("error", { error: String(e) });
        }
      },
    });
  }

  function handleDetails() {
    appStore.openModal("key-details", { fingerprint: keyInfo.fingerprint });
  }
</script>

<div
  class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4
         transition-colors hover:border-[var(--color-primary)]/30"
>
  <div class="flex items-start justify-between gap-3">
    <div class="min-w-0 flex-1">
      <div class="mb-1 flex items-center gap-2">
        <span class="truncate font-medium">
          {keyInfo.name ?? m.unnamed()}
        </span>
        <TrustBadge level={keyInfo.trust_level} />
      </div>
      {#if keyInfo.email}
        <p class="truncate text-sm text-[var(--color-text-secondary)]">{keyInfo.email}</p>
      {/if}
      <div class="mt-2 flex items-center gap-3 text-xs text-[var(--color-text-secondary)]">
        <FingerprintDisplay fingerprint={keyInfo.fingerprint} short />
        <span>{keyInfo.algorithm}</span>
        <span>{formatDate(keyInfo.created_at)}</span>
      </div>
    </div>

    <div class="flex shrink-0 items-center gap-1">
      <button
        class="rounded p-1.5 transition-colors hover:bg-[var(--color-border)]"
        onclick={handleDetails}
        title={m.key_details_btn()}
      >
        <Info size={16} class="text-[var(--color-text-secondary)]" />
      </button>
      <button
        class="rounded p-1.5 transition-colors hover:bg-[var(--color-border)]"
        onclick={handleExport}
        title={m.key_export_btn()}
      >
        <Download size={16} class="text-[var(--color-text-secondary)]" />
      </button>
      <button
        class="rounded p-1.5 transition-colors hover:bg-[var(--color-danger)]/10"
        onclick={handleDelete}
        title={m.key_delete_btn()}
      >
        <Trash2 size={16} class="text-[var(--color-danger)]" />
      </button>
    </div>
  </div>
</div>
