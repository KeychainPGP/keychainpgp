<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import {
    clearPassphraseCache,
    enableOpsecMode,
    disableOpsecMode,
    testProxyConnection,
    isPortable as checkPortable,
  } from "$lib/tauri";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { shortFingerprint } from "$lib/utils";
  import { isDesktop } from "$lib/platform";
  import { changeLocale, localeStore } from "$lib/stores/locale.svelte";
  import { RefreshCw, Shield, Globe, Heart, HardDrive } from "lucide-svelte";
  import * as m from "$lib/paraglide/messages.js";

  const desktop = isDesktop();
  let portable = $state(false);
  if (desktop) {
    checkPortable()
      .then((v) => {
        portable = v;
      })
      .catch(() => {});
  }

  let proxyTesting = $state(false);
  let proxyTestResult: string | null = $state(null);
  let proxyTestSuccess = $state(false);

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
      current = keyStore.ownKeys.map((k) => k.fingerprint);
    }
    const next = current.includes(fp) ? current.filter((k) => k !== fp) : [...current, fp];
    const allOwn = keyStore.ownKeys.map((k) => k.fingerprint);
    const isAll = allOwn.length > 0 && allOwn.every((f) => next.includes(f));
    settingsStore.save({ encrypt_to_self_keys: isAll ? [] : next });
  }

  function toggle(
    key:
      | "auto_clear_enabled"
      | "clipboard_monitoring"
      | "encrypt_to_self"
      | "auto_clear_after_encrypt"
      | "include_armor_headers",
  ) {
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

  async function toggleOpsecMode() {
    const newValue = !settingsStore.settings.opsec_mode;
    await settingsStore.save({ opsec_mode: newValue });
    if (newValue) {
      await enableOpsecMode(settingsStore.settings.opsec_window_title || undefined);
      appStore.setStatus(m.opsec_enabled());
    } else {
      await disableOpsecMode();
      appStore.setStatus(m.opsec_disabled());
    }
  }

  const PROXY_PRESETS: Record<string, string> = {
    tor: "socks5://127.0.0.1:9050",
    lokinet: "socks5://127.0.0.1:1080",
  };

  function toggleProxy() {
    settingsStore.save({ proxy_enabled: !settingsStore.settings.proxy_enabled });
    proxyTestResult = null;
  }

  function selectProxyPreset(preset: string) {
    proxyTestResult = null;
    if (preset === "custom") {
      settingsStore.save({ proxy_preset: "custom" });
    } else {
      settingsStore.save({
        proxy_preset: preset,
        proxy_url: PROXY_PRESETS[preset] ?? PROXY_PRESETS.tor,
      });
    }
  }

  function effectiveProxyUrl(): string {
    const s = settingsStore.settings;
    if (s.proxy_preset === "custom") return s.proxy_url;
    return PROXY_PRESETS[s.proxy_preset] ?? s.proxy_url;
  }

  async function handleProxyTest() {
    proxyTesting = true;
    proxyTestResult = null;
    proxyTestSuccess = false;
    try {
      await testProxyConnection(effectiveProxyUrl());
      proxyTestResult = m.settings_proxy_success();
      proxyTestSuccess = true;
    } catch (e) {
      proxyTestResult = m.settings_proxy_failed({ error: String(e) });
      proxyTestSuccess = false;
    } finally {
      proxyTesting = false;
    }
  }

  const themeLabels: Record<string, () => string> = {
    system: () => m.settings_theme_system(),
    light: () => m.settings_theme_light(),
    dark: () => m.settings_theme_dark(),
  };
</script>

<div class="mx-auto max-w-2xl space-y-6">
  <h2 class="text-xl font-bold">{m.settings_title()}</h2>

  <!-- Theme -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase">
      {m.settings_appearance()}
    </h3>
    <div class="flex gap-2">
      {#each ["system", "light", "dark"] as theme}
        <button
          class="rounded-lg border px-4 py-2 text-sm transition-colors"
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

    {#if desktop}
      <label
        class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
      >
        <div>
          <p class="text-sm font-medium">{m.settings_close_to_tray_label()}</p>
          <p class="text-xs text-[var(--color-text-secondary)]">
            {m.settings_close_to_tray_desc()}
          </p>
        </div>
        <input
          type="checkbox"
          checked={settingsStore.settings.close_to_tray}
          onchange={() =>
            settingsStore.save({ close_to_tray: !settingsStore.settings.close_to_tray })}
          class="h-4 w-4 accent-[var(--color-primary)]"
        />
      </label>
    {/if}
  </section>

  <!-- Language -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase">
      {m.settings_language()}
    </h3>

    <label
      class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
    >
      <div>
        <p class="text-sm font-medium">{m.settings_language_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_language_desc()}</p>
      </div>
      <select
        value={settingsStore.settings.locale}
        onchange={handleLocaleChange}
        class="rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-sm
               focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
      >
        <option value="auto">{m.settings_locale_auto()}</option>
        {#each localeStore.locales as loc}
          <option value={loc}>{LOCALE_LABELS[loc] ?? loc}</option>
        {/each}
      </select>
    </label>
  </section>

  <!-- Clipboard (desktop only) -->
  {#if desktop}
    <section class="space-y-3">
      <h3 class="text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase">
        {m.settings_clipboard()}
      </h3>

      <label
        class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
      >
        <div>
          <p class="text-sm font-medium">{m.settings_auto_clear_label()}</p>
          <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_auto_clear_desc()}</p>
        </div>
        <input
          type="checkbox"
          checked={settingsStore.settings.auto_clear_enabled}
          onchange={() => toggle("auto_clear_enabled")}
          class="h-4 w-4 accent-[var(--color-primary)]"
        />
      </label>

      {#if settingsStore.settings.auto_clear_enabled}
        <label
          class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
        >
          <div>
            <p class="text-sm font-medium">{m.settings_auto_clear_delay_label()}</p>
            <p class="text-xs text-[var(--color-text-secondary)]">
              {m.settings_auto_clear_delay_desc()}
            </p>
          </div>
          <input
            type="number"
            min="5"
            max="300"
            value={settingsStore.settings.auto_clear_delay_secs}
            onchange={(e) =>
              settingsStore.save({ auto_clear_delay_secs: parseInt(e.currentTarget.value) || 30 })}
            class="w-20 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-sm
                 focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
          />
        </label>
      {/if}
    </section>
  {/if}

  <!-- Encryption -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase">
      {m.settings_encryption()}
    </h3>

    <label
      class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
    >
      <div>
        <p class="text-sm font-medium">{m.settings_encrypt_to_self_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">
          {m.settings_encrypt_to_self_desc()}
        </p>
      </div>
      <input
        type="checkbox"
        checked={settingsStore.settings.encrypt_to_self}
        onchange={() => toggle("encrypt_to_self")}
        class="h-4 w-4 accent-[var(--color-primary)]"
      />
    </label>

    {#if settingsStore.settings.encrypt_to_self && keyStore.ownKeys.length > 0}
      <div class="space-y-2 rounded-lg border border-[var(--color-border)] p-3">
        <p class="text-xs text-[var(--color-text-secondary)]">
          {#if settingsStore.settings.encrypt_to_self_keys.length === 0}
            {m.settings_self_keys_all()}
          {:else if settingsStore.settings.encrypt_to_self_keys.length === 1}
            {m.settings_self_keys_count_one()}
          {:else}
            {m.settings_self_keys_count_other({
              count: settingsStore.settings.encrypt_to_self_keys.length,
            })}
          {/if}
        </p>
        <div class="space-y-1">
          {#each keyStore.ownKeys as k (k.fingerprint)}
            <label
              class="flex cursor-pointer items-center gap-2 rounded p-2 hover:bg-[var(--color-bg-secondary)]"
            >
              <input
                type="checkbox"
                checked={settingsStore.settings.encrypt_to_self_keys.length === 0 ||
                  settingsStore.settings.encrypt_to_self_keys.includes(k.fingerprint)}
                onchange={() => toggleSelfKey(k.fingerprint)}
                class="h-3.5 w-3.5 accent-[var(--color-primary)]"
              />
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm">{k.name ?? m.unnamed()}</p>
                <p class="truncate text-xs text-[var(--color-text-secondary)]">
                  {k.email ?? shortFingerprint(k.fingerprint)}
                </p>
              </div>
            </label>
          {/each}
        </div>
      </div>
    {/if}

    <label
      class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
    >
      <div>
        <p class="text-sm font-medium">{m.settings_include_armor_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_include_armor_desc()}</p>
      </div>
      <input
        type="checkbox"
        checked={settingsStore.settings.include_armor_headers}
        onchange={() => toggle("include_armor_headers")}
        class="h-4 w-4 accent-[var(--color-primary)]"
      />
    </label>
  </section>

  <!-- Security -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase">
      {m.settings_security()}
    </h3>

    <label
      class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
    >
      <div>
        <p class="text-sm font-medium">{m.settings_passphrase_cache_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">
          {m.settings_passphrase_cache_desc()}
        </p>
      </div>
      <input
        type="number"
        min="0"
        max="3600"
        value={settingsStore.settings.passphrase_cache_secs}
        onchange={(e) =>
          settingsStore.save({ passphrase_cache_secs: parseInt(e.currentTarget.value) || 0 })}
        class="w-20 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-sm
               focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
      />
    </label>

    <button
      class="rounded-lg border border-red-300 px-4 py-2 text-sm text-red-600 transition-colors hover:bg-red-50"
      onclick={handleClearCache}
    >
      {m.settings_clear_cache()}
    </button>
  </section>

  <!-- OPSEC Mode -->
  {#if desktop}
    <section class="space-y-3">
      <h3
        class="flex items-center gap-1.5 text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase"
      >
        <Shield size={14} />
        {m.settings_opsec()}
      </h3>
      <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_opsec_desc()}</p>

      <label
        class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
      >
        <div>
          <p class="text-sm font-medium">{m.settings_opsec_enable()}</p>
          <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_opsec_enable_desc()}</p>
        </div>
        <input
          type="checkbox"
          checked={settingsStore.settings.opsec_mode}
          onchange={toggleOpsecMode}
          class="h-4 w-4 accent-[var(--color-primary)]"
        />
      </label>

      {#if settingsStore.settings.opsec_mode}
        <label
          class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
        >
          <div>
            <p class="text-sm font-medium">{m.settings_opsec_title_label()}</p>
            <p class="text-xs text-[var(--color-text-secondary)]">
              {m.settings_opsec_title_desc()}
            </p>
          </div>
          <input
            type="text"
            value={settingsStore.settings.opsec_window_title}
            onchange={(e) =>
              settingsStore.save({
                opsec_window_title: e.currentTarget.value || m.settings_opsec_title_placeholder(),
              })}
            placeholder={m.settings_opsec_title_placeholder()}
            class="w-32 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-sm
                 focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
          />
        </label>

        <label
          class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
        >
          <div>
            <p class="text-sm font-medium">{m.settings_opsec_timeout_label()}</p>
            <p class="text-xs text-[var(--color-text-secondary)]">
              {m.settings_opsec_timeout_desc()}
            </p>
          </div>
          <input
            type="number"
            min="0"
            max="300"
            value={settingsStore.settings.opsec_view_timeout_secs}
            onchange={(e) =>
              settingsStore.save({
                opsec_view_timeout_secs: parseInt(e.currentTarget.value) || 30,
              })}
            class="w-20 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-sm
                 focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
          />
        </label>
      {/if}
    </section>
  {/if}

  <!-- Key Discovery -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase">
      {m.settings_key_discovery()}
    </h3>

    <label
      class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
    >
      <div>
        <p class="text-sm font-medium">{m.settings_keyserver_label()}</p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_keyserver_desc()}</p>
      </div>
      <input
        type="text"
        value={settingsStore.settings.keyserver_url}
        onchange={(e) =>
          settingsStore.save({
            keyserver_url: e.currentTarget.value || "https://keys.openpgp.org",
          })}
        class="w-56 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-sm
               focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
      />
    </label>

    <!-- Proxy (Tor/Lokinet) -->
    <label
      class="flex items-center justify-between rounded-lg border border-[var(--color-border)] p-3"
    >
      <div>
        <p class="flex items-center gap-1.5 text-sm font-medium">
          <Globe size={14} />
          {m.settings_proxy()}
        </p>
        <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_proxy_desc()}</p>
      </div>
      <input
        type="checkbox"
        checked={settingsStore.settings.proxy_enabled}
        onchange={toggleProxy}
        class="h-4 w-4 accent-[var(--color-primary)]"
      />
    </label>

    {#if settingsStore.settings.proxy_enabled}
      <div class="space-y-3 rounded-lg border border-[var(--color-border)] p-3">
        <div class="flex gap-2">
          {#each ["tor", "lokinet", "custom"] as preset}
            <button
              class="rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors"
              class:bg-[var(--color-primary)]={settingsStore.settings.proxy_preset === preset}
              class:text-white={settingsStore.settings.proxy_preset === preset}
              class:border-[var(--color-primary)]={settingsStore.settings.proxy_preset === preset}
              class:border-[var(--color-border)]={settingsStore.settings.proxy_preset !== preset}
              onclick={() => selectProxyPreset(preset)}
            >
              {preset === "tor"
                ? "Tor"
                : preset === "lokinet"
                  ? "Lokinet"
                  : m.settings_proxy_custom()}
            </button>
          {/each}
        </div>

        {#if settingsStore.settings.proxy_preset !== "custom"}
          <p class="font-mono text-xs text-[var(--color-text-secondary)]">
            {PROXY_PRESETS[settingsStore.settings.proxy_preset]}
          </p>
        {:else}
          <input
            type="text"
            value={settingsStore.settings.proxy_url}
            onchange={(e) =>
              settingsStore.save({ proxy_url: e.currentTarget.value || "socks5://127.0.0.1:9050" })}
            placeholder={m.settings_proxy_placeholder()}
            class="w-full rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 font-mono
                   text-sm text-xs focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
          />
        {/if}

        <div class="flex items-center gap-2">
          <button
            class="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium
                   transition-colors hover:bg-[var(--color-bg-secondary)] disabled:opacity-50"
            onclick={handleProxyTest}
            disabled={proxyTesting}
          >
            {proxyTesting ? m.settings_proxy_testing() : m.settings_proxy_test()}
          </button>
          {#if proxyTestResult}
            <p
              class="text-xs"
              class:text-green-600={proxyTestSuccess}
              class:text-red-600={!proxyTestSuccess}
            >
              {proxyTestResult}
            </p>
          {/if}
        </div>
      </div>
    {/if}
  </section>

  <!-- Key Sync -->
  <section class="space-y-3">
    <h3 class="text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase">
      {m.settings_sync()}
    </h3>
    <p class="text-xs text-[var(--color-text-secondary)]">{m.settings_sync_desc()}</p>
    <div class="flex gap-2">
      <button
        class="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] px-4
               py-2 text-sm font-medium
               transition-colors hover:bg-[var(--color-bg-secondary)]"
        onclick={() => appStore.openModal("key-sync-export")}
      >
        <RefreshCw size={16} />
        {m.settings_sync_export()}
      </button>
      <button
        class="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] px-4
               py-2 text-sm font-medium
               transition-colors hover:bg-[var(--color-bg-secondary)]"
        onclick={() => appStore.openModal("key-sync-import")}
      >
        <RefreshCw size={16} />
        {m.settings_sync_import()}
      </button>
    </div>
  </section>

  <!-- About -->
  <section class="space-y-3 border-t border-[var(--color-border)] pt-4">
    <h3 class="text-sm font-semibold tracking-wide text-[var(--color-text-secondary)] uppercase">
      {m.settings_about_title()}
    </h3>
    <p class="text-sm text-[var(--color-text-secondary)]">
      {m.settings_about({ version: `v${__APP_VERSION__}` })}
    </p>
    {#if portable}
      <div
        class="inline-flex items-center gap-1.5 rounded-full bg-amber-100 px-3 py-1.5 text-xs
                  font-medium text-amber-800 dark:bg-amber-900/30 dark:text-amber-300"
      >
        <HardDrive size={12} />
        {m.settings_portable_badge()}
      </div>
    {/if}
    <button
      class="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-primary)] px-4 py-2
             text-sm font-medium text-white
             transition-colors hover:bg-[var(--color-primary-hover)]"
      onclick={() => appStore.openModal("donate")}
    >
      <Heart size={16} />
      {m.donate_button()}
    </button>
  </section>
</div>
