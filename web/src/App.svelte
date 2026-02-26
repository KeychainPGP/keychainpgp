<script lang="ts">
  import { initWasm } from "./lib/wasm";
  import { getTheme, setTheme, applyTheme, type ThemeMode } from "./lib/theme";
  import { hasCompletedOnboarding } from "./lib/preferences";
  import { t, getLocale, setLocale, LOCALES, type Locale } from "./lib/i18n.svelte";
  import Onboarding from "./components/Onboarding.svelte";
  import EncryptView from "./components/EncryptView.svelte";
  import DecryptView from "./components/DecryptView.svelte";
  import SignView from "./components/SignView.svelte";
  import VerifyView from "./components/VerifyView.svelte";
  import KeyManager from "./components/KeyManager.svelte";

  let ready = $state(false);
  let error = $state("");
  let tab: "encrypt" | "decrypt" | "sign" | "verify" | "keys" = $state("encrypt");
  let theme: ThemeMode = $state(getTheme());
  let showOnboarding = $state(!hasCompletedOnboarding());

  $effect(() => {
    applyTheme(theme);
  });

  function toggleTheme() {
    theme = theme === "dark" ? "light" : "dark";
    setTheme(theme);
  }

  function handleLocaleChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    setLocale(select.value as Locale);
  }

  async function load() {
    try {
      await initWasm();
      ready = true;
    } catch (e) {
      error = t("loading_error", { error: String(e) });
    }
  }

  load();
</script>

<header style="text-align: center; margin-bottom: 2rem; position: relative;">
  <div class="header-controls">
    <a href="https://keychainpgp.org" target="_blank" rel="noopener noreferrer" class="header-link" title="keychainpgp.org">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
      </svg>
    </a>
    <a href="https://github.com/KeychainPGP/keychainpgp" target="_blank" rel="noopener noreferrer" class="header-link" title="GitHub">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
      </svg>
    </a>

    <select
      class="locale-select"
      value={getLocale()}
      onchange={handleLocaleChange}
    >
      {#each LOCALES as locale (locale.code)}
        <option value={locale.code}>{locale.name}</option>
      {/each}
    </select>

    <button
      class="theme-toggle"
      onclick={toggleTheme}
      title={theme === "light" ? t("theme_light") : t("theme_dark")}
    >
      {#if theme === "light"}
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
        </svg>
      {:else}
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
        </svg>
      {/if}
    </button>
  </div>

  <h1 style="font-size: 1.5rem; font-weight: 700; display: inline-flex; align-items: center; gap: 0.5rem; justify-content: center;">
    <img src="./icon.png" alt="" style="width: 32px; height: 32px; border-radius: 6px;" />
    KeychainPGP <span style="color: var(--color-text-secondary); font-weight: 400; font-size: 0.875rem;">Web</span>
  </h1>
  <p style="color: var(--color-text-secondary); font-size: 0.875rem; margin-top: 0.25rem;">
    {t("app_subtitle")}
  </p>
</header>

{#if error}
  <div class="card" style="text-align: center;">
    <p class="error">{error}</p>
  </div>
{:else if !ready}
  <div class="card" style="text-align: center;">
    <p style="color: var(--color-text-secondary);">{t("loading")}</p>
  </div>
{:else}
  {#if showOnboarding}
    <Onboarding onDismiss={() => showOnboarding = false} />
  {/if}

  <div class="tabs">
    <button class="tab" class:active={tab === "encrypt"} onclick={() => tab = "encrypt"}>{t("tab_encrypt")}</button>
    <button class="tab" class:active={tab === "decrypt"} onclick={() => tab = "decrypt"}>{t("tab_decrypt")}</button>
    <button class="tab" class:active={tab === "sign"} onclick={() => tab = "sign"}>{t("tab_sign")}</button>
    <button class="tab" class:active={tab === "verify"} onclick={() => tab = "verify"}>{t("tab_verify")}</button>
    <button class="tab" class:active={tab === "keys"} onclick={() => tab = "keys"}>{t("tab_keys")}</button>
  </div>

  {#if tab === "encrypt"}
    <EncryptView />
  {:else if tab === "decrypt"}
    <DecryptView />
  {:else if tab === "sign"}
    <SignView />
  {:else if tab === "verify"}
    <VerifyView />
  {:else if tab === "keys"}
    <KeyManager />
  {/if}
{/if}

<footer style="text-align: center; margin-top: 3rem; color: var(--color-text-secondary); font-size: 0.75rem;">
  <p>{t("app_footer")}</p>
</footer>

<style>
  .header-controls {
    position: absolute;
    top: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .locale-select {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 0.3rem 0.5rem;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    cursor: pointer;
    outline: none;
  }
  .locale-select option {
    background: var(--color-bg-secondary);
    color: var(--color-text);
  }
  .locale-select:focus {
    border-color: var(--color-primary);
  }

  .theme-toggle {
    background: none;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 0.375rem;
    color: var(--color-text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s, border-color 0.15s;
  }
  .theme-toggle:hover {
    color: var(--color-text);
    border-color: var(--color-text-secondary);
  }

  .header-link {
    color: var(--color-text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.375rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    transition: color 0.15s, border-color 0.15s;
  }
  .header-link:hover {
    color: var(--color-text);
    border-color: var(--color-text-secondary);
  }
</style>
