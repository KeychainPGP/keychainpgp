<script lang="ts">
  import { Download, Trash2, Info, Share, Flame } from "lucide-svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import type { KeyInfo } from "$lib/tauri";
  import {
    exportKey,
    exportPrivateKey,
    publishRevocationCert,
    deleteKey,
    writeClipboard,
  } from "$lib/tauri";
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
      await writeClipboard(armored);
      appStore.setStatus(m.keys_export_success());
    } catch (e) {
      appStore.openModal("error", { error: String(e) });
    }
  }

  async function handleBackup() {
    appStore.openModal("confirm", {
      title: m.key_backup_warning_title(),
      message: m.key_backup_warning_message(),
      confirmLabel: m.key_backup_btn(),
      onConfirm: async () => {
        try {
          const path = await save({
            filters: [{ name: "PGP Private Key", extensions: ["asc", "key"] }],
            defaultPath: `private_${keyInfo.fingerprint.slice(-8)}.asc`,
          });
          if (path) {
            await exportPrivateKey(keyInfo.fingerprint, path);
            appStore.setStatus(m.keys_export_private_success());
            appStore.closeModal();
          }
        } catch (e) {
          appStore.openModal("error", { error: String(e) });
        }
      },
    });
  }

  function handleRevocationPublish() {
    appStore.openModal("key-revoke", {
      onConfirmRevoke: async (deleteLocal: boolean) => {
        try {
          await publishRevocationCert(keyInfo.fingerprint);
          if (deleteLocal) {
            await deleteKey(keyInfo.fingerprint);
          }
          await keyStore.refresh();
          appStore.setStatus(m.key_revocation_publish_btn());
          appStore.closeModal();
        } catch (e) {
          appStore.openModal("error", { error: String(e) });
        }
      },
    });
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
  class:opacity-60={keyInfo.is_revoked}
  style={keyInfo.is_revoked ? "background-color: rgba(239, 68, 68, 0.05)" : ""}
>
  <div class="flex items-start justify-between gap-3">
    <div class="min-w-0 flex-1">
      <div class="mb-1 flex items-center gap-2">
        <span class="truncate font-medium">
          {keyInfo.name ?? m.unnamed()}
        </span>
        {#if keyInfo.is_revoked}
          <span
            class="rounded border border-red-500/20 bg-red-500/10 px-1.5 py-0.5 text-[10px] font-bold tracking-wider text-red-500 uppercase"
          >
            {m.key_details_revoked()}
          </span>
        {:else}
          <TrustBadge level={keyInfo.trust_level} />
        {/if}
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
        <Share size={16} class="text-[var(--color-text-secondary)]" />
      </button>

      {#if keyInfo.is_own_key}
        <button
          class="rounded p-1.5 transition-colors hover:bg-[var(--color-border)]"
          onclick={handleBackup}
          title={m.key_backup_btn()}
        >
          <Download size={16} class="text-blue-500" />
        </button>
        <button
          class="rounded p-1.5 transition-colors hover:bg-[var(--color-border)]"
          onclick={handleRevocationPublish}
          title={m.key_revocation_publish_btn()}
        >
          <Flame size={16} class="text-orange-500" />
        </button>
      {/if}

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
