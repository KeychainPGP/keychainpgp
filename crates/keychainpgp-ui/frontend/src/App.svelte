<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { appStore, type AppAction } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { registerPanicHotkey, unregisterPanicHotkey } from "$lib/hotkeys";
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
  import RevokeKeyModal from "./components/modals/RevokeKeyModal.svelte";
  import DonateModal from "./components/modals/DonateModal.svelte";
  import NoticeDialog from "./components/modals/NoticeDialog.svelte";
  import PublishPromptModal from "./components/modals/PublishPromptModal.svelte";

  let initialized = $state(false);
  let mobile = $state(false);
  let unlistenTray: UnlistenFn | null = null;
  let keydownHandler: ((e: KeyboardEvent) => void) | null = null;
  let unlistenUpload: UnlistenFn | null = null;

  onMount(async () => {
    await initPlatform();
    mobile = isMobile();

    await Promise.all([keyStore.refresh(), settingsStore.load()]);
    initLocale(settingsStore.settings.locale);

    if (isDesktop()) {
      clipboardStore.startPolling();

      // Register panic wipe as global hotkey (works even when app is in background)
      await registerPanicHotkey(async () => {
        if (settingsStore.settings.opsec_mode) {
          await panicWipe();
        }
      });

      // Window-scoped keyboard shortcuts (only active when app has focus)
      keydownHandler = (e: KeyboardEvent) => {
        if (e.ctrlKey && e.shiftKey) {
          switch (e.key.toUpperCase()) {
            case "E":
              e.preventDefault();
              appStore.dispatchAction("encrypt");
              break;
            case "D":
              e.preventDefault();
              appStore.dispatchAction("decrypt");
              break;
            case "S":
              e.preventDefault();
              appStore.dispatchAction("sign");
              break;
            case "V":
              e.preventDefault();
              appStore.dispatchAction("verify");
              break;
          }
        }
      };
      window.addEventListener("keydown", keydownHandler);

      // Listen for tray menu actions (desktop only)
      unlistenTray = await listen<string>("tray-action", (event) => {
        const action = event.payload as AppAction;
        if (action) appStore.dispatchAction(action);
      });

      // Listen for background auto-upload results
      unlistenUpload = await listen<string>("auto-upload-result", (event) => {
        appStore.setStatus(event.payload);
      });
    } else {
      // On mobile, default to compose mode (no system clipboard monitoring)
      appStore.inputMode = "compose";
    }

    initialized = true;
  });

  onDestroy(() => {
    if (isDesktop()) {
      unregisterPanicHotkey();
      unlistenTray?.();
      if (keydownHandler) window.removeEventListener("keydown", keydownHandler);
      unlistenUpload?.();
    }
  });

  const showOnboarding = $derived(initialized && !keyStore.hasOwnKey);
</script>

<main class="flex h-screen flex-col" class:safe-area-top={mobile}>
  {#if !initialized}
    <div class="flex h-full items-center justify-center">
      <p class="text-[var(--color-text-secondary)]">{m.loading()}</p>
    </div>
  {:else}
    {#key localeStore.current}
      {#if showOnboarding}
        <OnboardingView />
      {:else}
        <NavBar />

        <div class="flex-1 overflow-auto p-6" class:pb-14={mobile}>
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
  {:else if appStore.activeModal === "notice"}
    <NoticeDialog />
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
  {:else if appStore.activeModal === "key-revoke"}
    <RevokeKeyModal onConfirmRevoke={appStore.modalProps.onConfirmRevoke!} />
  {:else if appStore.activeModal === "donate"}
    <DonateModal />
  {:else if appStore.activeModal === "publish-prompt"}
    <PublishPromptModal />
  {/if}
</main>
