<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { appStore, type AppAction } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { registerHotkeys, unregisterHotkeys } from "$lib/hotkeys";
  import { initLocale, localeStore } from "$lib/stores/locale.svelte";
  import { initPlatform, isDesktop, isMobile } from "$lib/platform";
  import { panicWipe } from "$lib/tauri";
  import * as m from "$lib/paraglide/messages.js";

  import NavBar from "./components/layout/NavBar.svelte";
  import StatusBar from "./components/layout/StatusBar.svelte";

  import OnboardingView from "./components/onboarding/OnboardingView.svelte";
  import HomeView from "./components/home/HomeView.svelte";
  import KeysView from "./components/keys/KeysView.svelte";
  import SettingsView from "./components/settings/SettingsView.svelte";

  import RecipientSelector from "./components/modals/RecipientSelector.svelte";
  import DecryptedViewer from "./components/modals/DecryptedViewer.svelte";
  import PassphraseDialog from "./components/modals/PassphraseDialog.svelte";
  import KeyImportDialog from "./components/modals/KeyImportDialog.svelte";
  import KeyDetailsModal from "./components/modals/KeyDetailsModal.svelte";
  import ErrorDialog from "./components/modals/ErrorDialog.svelte";
  import ConfirmDialog from "./components/modals/ConfirmDialog.svelte";
  import VerifyResultModal from "./components/modals/VerifyResultModal.svelte";
  import QrExportModal from "./components/modals/QrExportModal.svelte";
  import KeyDiscoveryModal from "./components/modals/KeyDiscoveryModal.svelte";
  import KeySyncExportModal from "./components/modals/KeySyncExportModal.svelte";
  import KeySyncImportModal from "./components/modals/KeySyncImportModal.svelte";

  let initialized = $state(false);
  let unlistenTray: UnlistenFn | null = null;

  onMount(async () => {
    await initPlatform();

    await Promise.all([
      keyStore.refresh(),
      settingsStore.load(),
    ]);
    initLocale(settingsStore.settings.locale);

    if (isDesktop()) {
      clipboardStore.startPolling();

      // Register global hotkeys (desktop only)
      await registerHotkeys({
        onEncrypt: () => appStore.dispatchAction("encrypt"),
        onDecrypt: () => appStore.dispatchAction("decrypt"),
        onSign: () => appStore.dispatchAction("sign"),
        onVerify: () => appStore.dispatchAction("verify"),
        onPanic: async () => {
          if (settingsStore.settings.opsec_mode) {
            await panicWipe();
          }
        },
      });

      // Listen for tray menu actions (desktop only)
      unlistenTray = await listen<string>("tray-action", (event) => {
        const action = event.payload as AppAction;
        if (action) appStore.dispatchAction(action);
      });
    } else {
      // On mobile, default to compose mode (no system clipboard monitoring)
      appStore.inputMode = "compose";
    }

    initialized = true;
  });

  onDestroy(() => {
    if (isDesktop()) {
      unregisterHotkeys();
      unlistenTray?.();
    }
  });

  const showOnboarding = $derived(initialized && !keyStore.hasOwnKey);
</script>

<main class="flex flex-col h-screen" class:safe-area-top={isMobile()}>
  {#if !initialized}
    <div class="flex items-center justify-center h-full">
      <p class="text-[var(--color-text-secondary)]">{m.loading()}</p>
    </div>
  {:else}
    {#key localeStore.current}
      {#if showOnboarding}
        <OnboardingView />
      {:else}
        <NavBar />

        <div class="flex-1 overflow-auto p-6" class:pb-24={isMobile()}>
          {#if appStore.currentView === "home"}
            <HomeView />
          {:else if appStore.currentView === "keys"}
            <KeysView />
          {:else if appStore.currentView === "settings"}
            <SettingsView />
          {/if}
        </div>

        <StatusBar />
      {/if}
    {/key}
  {/if}

  <!-- Modal layer -->
  {#if appStore.activeModal === "recipient-selector"}
    <RecipientSelector />
  {:else if appStore.activeModal === "decrypted-viewer"}
    <DecryptedViewer />
  {:else if appStore.activeModal === "passphrase"}
    <PassphraseDialog />
  {:else if appStore.activeModal === "key-import"}
    <KeyImportDialog />
  {:else if appStore.activeModal === "key-details"}
    <KeyDetailsModal />
  {:else if appStore.activeModal === "error"}
    <ErrorDialog />
  {:else if appStore.activeModal === "confirm"}
    <ConfirmDialog />
  {:else if appStore.activeModal === "verify-result"}
    <VerifyResultModal />
  {:else if appStore.activeModal === "qr-export"}
    <QrExportModal />
  {:else if appStore.activeModal === "key-discovery"}
    <KeyDiscoveryModal />
  {:else if appStore.activeModal === "key-sync-export"}
    <KeySyncExportModal />
  {:else if appStore.activeModal === "key-sync-import"}
    <KeySyncImportModal />
  {/if}
</main>
