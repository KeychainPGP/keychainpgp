<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { clearPassphraseCache } from "$lib/tauri";
  import { appStore } from "$lib/stores/app.svelte";

  function toggle(key: "auto_clear_enabled" | "clipboard_monitoring" | "encrypt_to_self" | "auto_clear_after_encrypt") {
    settingsStore.save({ [key]: !settingsStore.settings[key] });
  }

  async function handleClearCache() {
    try {
      await clearPassphraseCache();
      appStore.setStatus("Passphrase cache cleared.");
    } catch (e) {
      appStore.setStatus(`Failed to clear cache: ${e}`);
    }
  }

  function setTheme(theme: string) {
    settingsStore.save({ theme });
    applyTheme(theme);
  }

  function applyTheme(theme: string) {
    if (theme === "dark") {
      document.documentElement.setAttribute("data-theme", "dark");
    } else if (theme === "light") {
      document.documentElement.setAttribute("data-theme", "light");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
  }
</script>

<div class="max-w-2xl mx-auto space-y-6">
  <h2 class="text-xl font-bold">Settings</h2>

  <!-- Theme -->
  <section class="space-y-2">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">Appearance</h3>
    <div class="flex gap-2">
      {#each ["system", "light", "dark"] as theme}
        <button
          class="px-4 py-2 text-sm rounded-lg border transition-colors capitalize"
          class:bg-[var(--color-primary)]={settingsStore.settings.theme === theme}
          class:text-white={settingsStore.settings.theme === theme}
          class:border-[var(--color-primary)]={settingsStore.settings.theme === theme}
          class:border-[var(--color-border)]={settingsStore.settings.theme !== theme}
          onclick={() => setTheme(theme)}
        >
          {theme}
        </button>
      {/each}
    </div>
  </section>

  <!-- Clipboard -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">Clipboard</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">Auto-clear clipboard</p>
        <p class="text-xs text-[var(--color-text-secondary)]">Clear sensitive data from clipboard after decryption</p>
      </div>
      <input type="checkbox" checked={settingsStore.settings.auto_clear_enabled} onchange={() => toggle("auto_clear_enabled")}
        class="w-4 h-4 accent-[var(--color-primary)]" />
    </label>

    {#if settingsStore.settings.auto_clear_enabled}
      <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
        <div>
          <p class="text-sm font-medium">Auto-clear delay</p>
          <p class="text-xs text-[var(--color-text-secondary)]">Seconds before clipboard is cleared</p>
        </div>
        <input
          type="number"
          min="5"
          max="300"
          value={settingsStore.settings.auto_clear_delay_secs}
          onchange={(e) => settingsStore.save({ auto_clear_delay_secs: parseInt(e.currentTarget.value) || 30 })}
          class="w-20 px-2 py-1 text-sm rounded border border-[var(--color-border)] bg-[var(--color-bg)]
                 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
        />
      </label>
    {/if}
  </section>

  <!-- Encryption -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">Encryption</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">Encrypt to self</p>
        <p class="text-xs text-[var(--color-text-secondary)]">Always include your own key as a recipient</p>
      </div>
      <input type="checkbox" checked={settingsStore.settings.encrypt_to_self} onchange={() => toggle("encrypt_to_self")}
        class="w-4 h-4 accent-[var(--color-primary)]" />
    </label>
  </section>

  <!-- Security -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">Security</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">Passphrase cache duration</p>
        <p class="text-xs text-[var(--color-text-secondary)]">Seconds to remember passphrases (0 = disabled)</p>
      </div>
      <input
        type="number"
        min="0"
        max="3600"
        value={settingsStore.settings.passphrase_cache_secs}
        onchange={(e) => settingsStore.save({ passphrase_cache_secs: parseInt(e.currentTarget.value) || 0 })}
        class="w-20 px-2 py-1 text-sm rounded border border-[var(--color-border)] bg-[var(--color-bg)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      />
    </label>

    <button
      class="px-4 py-2 text-sm rounded-lg border border-red-300 text-red-600 hover:bg-red-50 transition-colors"
      onclick={handleClearCache}
    >
      Clear cached passphrases
    </button>
  </section>

  <!-- Key Discovery -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">Key Discovery</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">Keyserver URL</p>
        <p class="text-xs text-[var(--color-text-secondary)]">Used for key search and upload</p>
      </div>
      <input
        type="text"
        value={settingsStore.settings.keyserver_url}
        onchange={(e) => settingsStore.save({ keyserver_url: e.currentTarget.value || "https://keys.openpgp.org" })}
        class="w-56 px-2 py-1 text-sm rounded border border-[var(--color-border)] bg-[var(--color-bg)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      />
    </label>
  </section>

  <!-- About -->
  <section class="space-y-2 pt-4 border-t border-[var(--color-border)]">
    <p class="text-sm text-[var(--color-text-secondary)]">
      KeychainPGP v0.1.0 &mdash; Clipboard-first PGP encryption for desktop.
    </p>
  </section>
</div>
