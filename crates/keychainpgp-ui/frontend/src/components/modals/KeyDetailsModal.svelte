<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import FingerprintDisplay from "../shared/FingerprintDisplay.svelte";
  import TrustBadge from "../shared/TrustBadge.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { setKeyTrust, inspectKeyDetailed, type KeyDetailedInfo } from "$lib/tauri";
  import { Globe, User, Shield, Key as KeyIcon, Mail, Calendar, Hash, MoreHorizontal } from "lucide-svelte";
  import { formatDate } from "$lib/utils";
  import * as m from "$lib/paraglide/messages.js";

  const fp = appStore.modalProps.fingerprint ?? "";
  const keyInfo = $derived(keyStore.keys.find((k) => k.fingerprint === fp));

  let detailed: KeyDetailedInfo | null = $state(null);
  let updating = $state(false);
  let showSubkeys = $state(false);

  // Load detailed info on mount
  $effect(() => {
    if (fp) {
      inspectKeyDetailed(fp)
        .then((d) => {
          detailed = d;
        })
        .catch(() => {});
    }
  });

  async function toggleTrust() {
    if (!keyInfo || updating) return;
    updating = true;
    try {
      const newLevel = keyInfo.trust_level >= 2 ? 1 : 2;
      await setKeyTrust(keyInfo.fingerprint, newLevel);
      await keyStore.refresh();
    } catch (e) {
      appStore.setStatus(m.key_trust_update_failed({ error: String(e) }));
    } finally {
      updating = false;
    }
  }
</script>

<ModalContainer title={m.key_details_title()}>
  {#if keyInfo}
    <div class="space-y-4">
      <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
        <span class="text-[var(--color-text-secondary)]">{m.key_details_name()}</span>
        <span class="font-medium">{keyInfo.name ?? m.unnamed()}</span>

        <span class="text-[var(--color-text-secondary)]">{m.key_details_email()}</span>
        <span>{keyInfo.email ?? m.none_value()}</span>

        <span class="text-[var(--color-text-secondary)]">{m.key_details_fingerprint()}</span>
        <FingerprintDisplay fingerprint={keyInfo.fingerprint} />

        <span class="text-[var(--color-text-secondary)]">{m.key_details_algorithm()}</span>
        <span>{keyInfo.algorithm}</span>

        <span class="text-[var(--color-text-secondary)]">{m.key_details_created()}</span>
        <span>{formatDate(keyInfo.created_at)}</span>

        <span class="text-[var(--color-text-secondary)]">{m.key_details_expires()}</span>
        <span
          >{keyInfo.expires_at
            ? formatDate(keyInfo.expires_at)
            : m.key_details_expires_never()}</span
        >

        <span class="text-[var(--color-text-secondary)]">{m.key_details_trust()}</span>
        <TrustBadge level={keyInfo.trust_level} />

        <span class="text-[var(--color-text-secondary)]">{m.key_details_type()}</span>
        <span>{keyInfo.is_own_key ? m.key_details_own_key() : m.key_details_public_key()}</span>
      </div>

      {#if detailed && detailed.user_ids.length > 1}
        <div
          class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-3"
        >
          <p
            class="mb-2 text-xs font-medium tracking-wide text-[var(--color-text-secondary)] uppercase"
          >
            {m.key_details_user_ids()}
          </p>
          {#each detailed.user_ids as uid}
            <div class="py-0.5 text-sm">
              {uid.name ?? ""}{uid.email ? ` <${uid.email}>` : ""}
            </div>
          {/each}
        </div>
      {/if}

      {#if detailed && detailed.subkeys.length > 0}
        <div
          class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-3"
        >
          <button
            class="flex w-full items-center justify-between text-xs font-medium tracking-wide text-[var(--color-text-secondary)] uppercase"
            onclick={() => (showSubkeys = !showSubkeys)}
          >
            <span>{m.key_details_subkeys({ count: detailed.subkeys.length })}</span>
            <span class="text-base">{showSubkeys ? "\u2212" : "+"}</span>
          </button>
          {#if showSubkeys}
            <div class="mt-2 space-y-2">
              {#each detailed.subkeys as subkey}
                <div class="border-t border-[var(--color-border)] pt-2 text-sm">
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-xs text-[var(--color-text-secondary)]">
                      {subkey.fingerprint.slice(-16)}
                    </span>
                    {#if subkey.is_revoked}
                      <span class="rounded bg-red-100 px-1.5 py-0.5 text-xs text-red-600"
                        >{m.key_details_revoked()}</span
                      >
                    {/if}
                  </div>
                  <div class="mt-1 flex flex-wrap gap-2">
                    {#each subkey.capabilities as cap}
                      <span class="rounded bg-blue-100 px-1.5 py-0.5 text-xs text-blue-700"
                        >{cap}</span
                      >
                    {/each}
                  </div>
                  <div class="mt-1 text-xs text-[var(--color-text-secondary)]">
                    {subkey.algorithm} · {m.key_details_created_prefix()}
                    {formatDate(subkey.created_at)}
                    {#if subkey.expires_at}
                      · {m.key_details_expires_prefix()} {formatDate(subkey.expires_at)}
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <div class="flex items-center justify-between pt-2">
        <div class="flex gap-2">
          {#if !keyInfo.is_own_key}
            <button
              class="rounded-lg border px-3 py-1.5 text-sm transition-colors
                     {keyInfo.trust_level >= 2
                ? 'border-red-300 text-red-600 hover:bg-red-50'
                : 'border-green-300 text-green-600 hover:bg-green-50'}"
              onclick={toggleTrust}
              disabled={updating}
            >
              {keyInfo.trust_level >= 2 ? m.key_details_revoke_btn() : m.key_details_verify_btn()}
            </button>
          {/if}
          <button
            class="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-sm transition-colors hover:bg-[var(--color-bg-secondary)]"
            onclick={() => appStore.openModal("qr-export", { fingerprint: keyInfo.fingerprint })}
          >
            {m.key_details_qr_btn()}
          </button>
          {#if keyInfo.is_own_key}
            <button
              class="px-3 py-1.5 text-sm rounded-lg border border-[var(--color-border)] hover:bg-[var(--color-bg-secondary)] transition-colors flex items-center gap-1.5"
              onclick={() => appStore.openModal("publish-prompt", { fingerprint: keyInfo.fingerprint })}
            >
              <Globe size={14} />
              {m.publish_prompt_title()}
            </button>
          {/if}
        </div>
        <button
          class="rounded-lg bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white
                 transition-colors hover:bg-[var(--color-primary-hover)]"
          onclick={() => appStore.closeModal()}
        >
          {m.qr_close()}
        </button>
      </div>
    </div>
  {:else}
    <p class="text-sm text-[var(--color-text-secondary)]">{m.key_details_not_found()}</p>
  {/if}
</ModalContainer>
