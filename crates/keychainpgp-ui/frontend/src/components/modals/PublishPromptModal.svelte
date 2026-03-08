<script lang="ts">
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { keyserverUpload } from "$lib/tauri";
  import { TriangleAlert, CircleCheck, CircleX, LoaderCircle } from "lucide-svelte";
  import * as m from "$lib/paraglide/messages.js";

  const { fingerprint } = appStore.modalProps as { fingerprint: string };

  let selectedServers = $state<string[]>([]);
  let publishing = $state(false);
  let results = $state<Record<string, { success: boolean; message: string }>>(
    {},
  );

  const verifiedServers = $derived(
    (settingsStore.settings.keyserver_url || "")
      .split(",")
      .map((s: string) => s.trim())
      .filter(Boolean),
  );
  const unverifiedServers = $derived(
    (settingsStore.settings.unverified_keyserver_url || "")
      .split(",")
      .map((s: string) => s.trim())
      .filter(Boolean),
  );
  const allServers = $derived([...verifiedServers, ...unverifiedServers]);

  /** Normalizes a URL by removing trailing slashes for reliable comparison. */
  function normalizeUrl(url: string) {
    return url.replace(/\/$/, "");
  }

  // Hosts where email verification is required to complete publication.
  const emailVerificationHosts = new Set(["keys.openpgp.org"]);

  function requiresEmailVerification(url: string): boolean {
    try {
      return emailVerificationHosts.has(new URL(url).hostname.toLowerCase());
    } catch {
      const normalized = normalizeUrl(url).toLowerCase();
      return Array.from(emailVerificationHosts).some((host) =>
        normalized.includes(host),
      );
    }
  }

  const unverifiedNormalized = $derived(unverifiedServers.map(normalizeUrl));
  const showEmailVerificationNotice = $derived(
    selectedServers.some(requiresEmailVerification),
  );

  function isVerificationStillPending(message: string): boolean {
    return /verification email already pending/i.test(message);
  }

  // Initialize selected servers on mount
  $effect(() => {
    selectedServers = [...allServers];
  });

  async function handlePublish() {
    if (selectedServers.length === 0) return;
    publishing = true;
    results = {};

    const uploads = selectedServers.map(async (url) => {
      try {
        const msg = await keyserverUpload(fingerprint, url);
        results[url] = { success: true, message: msg };
      } catch (e) {
        results[url] = { success: false, message: String(e) };
      }
    });

    await Promise.all(uploads);
    publishing = false;

    const failCount = Object.values(results).filter((r) => !r.success).length;
    if (failCount === 0) {
      const pendingCount = selectedServers.filter((url) => {
        const result = results[url];
        return result?.success && isVerificationStillPending(result.message);
      }).length;
      if (pendingCount > 0) {
        appStore.openModal("notice", {
          title: "Email Verification Pending",
          message:
            pendingCount === 1
              ? "Your key upload is still awaiting email verification. Please click the verification link in your inbox to finish publishing."
              : "Some key uploads are still awaiting email verification. Please click the verification links in your inbox to finish publishing.",
        });
        return;
      }
      appStore.setStatus(m.publish_success({ count: selectedServers.length }));
      setTimeout(() => appStore.closeModal(), 2000);
    }
  }

  function toggleServer(url: string) {
    if (selectedServers.includes(url)) {
      selectedServers = selectedServers.filter((s) => s !== url);
    } else {
      selectedServers = [...selectedServers, url];
    }
  }
</script>

<ModalContainer title={m.publish_prompt_title()}>
  <div class="space-y-6">
    <p class="text-sm text-[var(--color-text)]">
      {m.publish_prompt_ask()}
    </p>

    <div class="space-y-3">
      {#each allServers as url}
        <div class="flex flex-col gap-1">
          <label
            class="flex items-center gap-3 p-3 rounded-lg border border-[var(--color-border)] hover:bg-[var(--color-bg-secondary)] cursor-pointer transition-colors"
          >
            <input
              type="checkbox"
              checked={selectedServers.includes(url)}
              onchange={() => toggleServer(url)}
              disabled={publishing}
              class="w-4 h-4 accent-[var(--color-primary)]"
            />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">{url}</p>
              {#if unverifiedNormalized.includes(normalizeUrl(url))}
                <p
                  class="text-xs text-[var(--color-danger)] flex items-center gap-1 mt-0.5 font-medium"
                >
                  {m.settings_unverified_keyservers_warning()}
                </p>
              {/if}
            </div>
            {#if results[url]}
              {#if results[url].success}
                <CircleCheck size={16} class="text-green-500" />
              {:else}
                <CircleX size={16} class="text-red-500" />
              {/if}
            {/if}
          </label>
          {#if results[url] && !results[url].success}
            <p class="text-xs text-red-600 px-1">{results[url].message}</p>
          {/if}
        </div>
      {/each}
    </div>

    {#if showEmailVerificationNotice}
      <div
        class="rounded-lg border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900"
      >
        Key upload is not complete until you click the verification link sent to
        your email inbox.
      </div>
    {/if}

    <div class="flex gap-3 justify-end pt-2">
      <button
        class="px-4 py-2 text-sm font-medium rounded-lg border border-[var(--color-border)] hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.closeModal()}
        disabled={publishing}
      >
        {m.publish_not_now()}
      </button>
      <button
        class="px-6 py-2 text-sm font-semibold rounded-lg bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50 flex items-center gap-2"
        onclick={handlePublish}
        disabled={publishing || selectedServers.length === 0}
      >
        {#if publishing}
          <LoaderCircle size={16} class="animate-spin" />
        {/if}
        {m.publish_btn()}
      </button>
    </div>
  </div>
</ModalContainer>
