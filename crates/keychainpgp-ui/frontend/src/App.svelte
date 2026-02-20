<script lang="ts">
  import { onMount } from "svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { keyStore } from "$lib/stores/keys.svelte";
  import { clipboardStore } from "$lib/stores/clipboard.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";

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

  let initialized = $state(false);

  onMount(async () => {
    await Promise.all([
      keyStore.refresh(),
      settingsStore.load(),
    ]);
    clipboardStore.startPolling();
    initialized = true;
  });

  const showOnboarding = $derived(initialized && !keyStore.hasOwnKey);
</script>

<main class="flex flex-col h-screen">
  {#if !initialized}
    <div class="flex items-center justify-center h-full">
      <p class="text-[var(--color-text-secondary)]">Loading...</p>
    </div>
  {:else if showOnboarding}
    <OnboardingView />
  {:else}
    <NavBar />

    <div class="flex-1 overflow-auto p-6">
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
  {/if}
</main>
