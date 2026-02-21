<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { clearPassphraseCache } from "$lib/tauri";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { shortFingerprint } from "$lib/utils";
  import { isDesktop } from "$lib/platform";
  import { changeLocale, localeStore } from "$lib/stores/locale.svelte";
  import { RefreshCw } from "lucide-svelte";
  import * as m from "$lib/paraglide/messages.js";

  const desktop = isDesktop();

  /** Native labels for each locale (always displayed in the locale's own language). */
  const LOCALE_LABELS: Record<string, string> = {
    en: "English",
    fr: "Français",
    de: "Deutsch",
    es: "Español",
    "pt-BR": "Português (Brasil)",
    "pt-PT": "Português (Portugal)",
    it: "Italiano",
    nl: "Nederlands",
    ru: "Русский",
    uk: "Українська",
    "zh-CN": "简体中文",
    "zh-TW": "繁體中文",
    ja: "日本語",
    ko: "한국어",
    ar: "العربية",
    he: "עברית",
    tr: "Türkçe",
    pl: "Polski",
    hi: "हिन्दी",
    th: "ไทย",
  };

  function toggleSelfKey(fp: string) {
    let current = settingsStore.settings.encrypt_to_self_keys;
    if (current.length === 0) {
      current = keyStore.ownKeys.map(k => k.fingerprint);
    }
    const next = current.includes(fp)
      ? current.filter(k => k !== fp)
      : [...current, fp];
    const allOwn = keyStore.ownKeys.map(k => k.fingerprint);
    const isAll = allOwn.length > 0 && allOwn.every(f => next.includes(f));
    settingsStore.save({ encrypt_to_self_keys: isAll ? [] : next });
  }

  function toggle(key: "auto_clear_enabled" | "clipboard_monitoring" | "encrypt_to_self" | "auto_clear_after_encrypt" | "include_armor_headers") {
    settingsStore.save({ [key]: !settingsStore.settings[key] });
  }

  async function handleClearCache() {
    try {
      await clearPassphraseCache();
      appStore.setStatus(m.settings_cache_cleared());
    } catch (e) {
      appStore.setStatus(m.settings_cache_clear_failed({ error: String(e) }));
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

  function handleLocaleChange(e: Event) {
    const tag = (e.currentTarget as HTMLSelectElement).value;
    settingsStore.save({ locale: tag });
    changeLocale(tag);
  }

  const themeLabels: Record<string, () => string> = {
    system: () => m.settings_theme_system(),
    light: () => m.settings_theme_light(),
    dark: () => m.settings_theme_dark(),
  };
</script>

<div class="max-w-2xl mx-auto space-y-6">
  <h2 class="text-xl font-bold">{m.settings_title()}</h2>

  <!-- Theme -->
  <section class="space-y-2">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{m.settings_appearance()}</h3>
    <div class="flex gap-2">
      {#each ["system", "light", "dark"] as theme}
        <button
          class="px-4 py-2 text-sm rounded-lg border transition-colors"
          class:bg-[var(--color-primary)]={settingsStore.settings.theme === theme}
          class:text-white={settingsStore.settings.theme === theme}
          class:border-[var(--color-primary)]={settingsStore.settings.theme === theme}
          class:border-[var(--color-border)]={settingsStore.settings.theme !== theme}
          onclick={() => setTheme(theme)}
        >
          {themeLabels[theme]()}
        </button>
      {/each}
    </div>
  </section>

  <!-- Language -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{m.settings_language()}</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">{m.settings_language_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_language_desc()}</p>
      </div>
      <select
        value={settingsStore.settings.locale}
        onchange={handleLocaleChange}
        class="px-2 py-1 text-sm rounded border border-[var(--color-border)] bg-[var(--color-bg)]
               focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
      >
        <option value="auto">Auto</option>
        {#each localeStore.locales as loc}
          <option value={loc}>{LOCALE_LABELS[loc] ?? loc}</option>
        {/each}
      </select>
    </label>
  </section>

  <!-- Clipboard (desktop only) -->
  {#if desktop}
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{m.settings_clipboard()}</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">{m.settings_auto_clear_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_auto_clear_desc()}</p>
      </div>
      <input type="checkbox" checked={settingsStore.settings.auto_clear_enabled} onchange={() => toggle("auto_clear_enabled")}
        class="w-4 h-4 accent-[var(--color-primary)]" />
    </label>

    {#if settingsStore.settings.auto_clear_enabled}
      <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
        <div>
          <p class="text-sm font-medium">{m.settings_auto_clear_delay_label()}</p>
          <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_auto_clear_delay_desc()}</p>
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
  {/if}

  <!-- Encryption -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{m.settings_encryption()}</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">{m.settings_encrypt_to_self_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_encrypt_to_self_desc()}</p>
      </div>
      <input type="checkbox" checked={settingsStore.settings.encrypt_to_self} onchange={() => toggle("encrypt_to_self")}
        class="w-4 h-4 accent-[var(--color-primary)]" />
    </label>

    {#if settingsStore.settings.encrypt_to_self && keyStore.ownKeys.length > 0}
      <div class="p-3 rounded-lg border border-[var(--color-border)] space-y-2">
        <p class="text-xs text-[var(--color-text-secondary)]">
          {#if settingsStore.settings.encrypt_to_self_keys.length === 0}
            {m.settings_self_keys_all()}
          {:else if settingsStore.settings.encrypt_to_self_keys.length === 1}
            {m.settings_self_keys_count_one()}
          {:else}
            {m.settings_self_keys_count_other({ count: settingsStore.settings.encrypt_to_self_keys.length })}
          {/if}
        </p>
        <div class="space-y-1">
          {#each keyStore.ownKeys as k (k.fingerprint)}
            <label class="flex items-center gap-2 p-2 rounded hover:bg-[var(--color-bg-secondary)] cursor-pointer">
              <input
                type="checkbox"
                checked={settingsStore.settings.encrypt_to_self_keys.length === 0 || settingsStore.settings.encrypt_to_self_keys.includes(k.fingerprint)}
                onchange={() => toggleSelfKey(k.fingerprint)}
                class="w-3.5 h-3.5 accent-[var(--color-primary)]"
              />
              <div class="min-w-0 flex-1">
                <p class="text-sm truncate">{k.name ?? m.unnamed()}</p>
                <p class="text-xs text-[var(--color-text-secondary)] truncate">
                  {k.email ?? shortFingerprint(k.fingerprint)}
                </p>
              </div>
            </label>
          {/each}
        </div>
      </div>
    {/if}

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">{m.settings_include_armor_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_include_armor_desc()}</p>
      </div>
      <input type="checkbox" checked={settingsStore.settings.include_armor_headers} onchange={() => toggle("include_armor_headers")}
        class="w-4 h-4 accent-[var(--color-primary)]" />
    </label>
  </section>

  <!-- Security -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{m.settings_security()}</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">{m.settings_passphrase_cache_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_passphrase_cache_desc()}</p>
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
      {m.settings_clear_cache()}
    </button>
  </section>

  <!-- Key Discovery -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{m.settings_key_discovery()}</h3>

    <label class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border)]">
      <div>
        <p class="text-sm font-medium">{m.settings_keyserver_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_keyserver_desc()}</p>
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

  <!-- Key Sync -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{m.settings_sync()}</h3>
    <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_sync_desc()}</p>
    <div class="flex gap-2">
      <button
        class="inline-flex items-center gap-1.5 px-4 py-2 text-sm rounded-lg
               border border-[var(--color-border)] font-medium
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.openModal("key-sync-export")}
      >
        <RefreshCw size={16} />
        {m.settings_sync_export()}
      </button>
      <button
        class="inline-flex items-center gap-1.5 px-4 py-2 text-sm rounded-lg
               border border-[var(--color-border)] font-medium
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.openModal("key-sync-import")}
      >
        <RefreshCw size={16} />
        {m.settings_sync_import()}
      </button>
    </div>
  </section>

  <!-- About -->
  <section class="space-y-2 pt-4 border-t border-[var(--color-border)]">
    <p class="text-sm text-[var(--color-text-secondary)]">
      {m.settings_about()}
    </p>
  </section>
</div>
