<script lang="ts">
  import { Shield, KeyRound, Settings } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { isMobile } from "$lib/platform";
  import * as m from "$lib/paraglide/messages.js";
  import type { View } from "$lib/types";

  const tabs: { id: View; label: () => string; icon: typeof Shield }[] = [
    { id: "home", label: () => m.nav_home(), icon: Shield },
    { id: "keys", label: () => m.nav_keys(), icon: KeyRound },
    { id: "settings", label: () => m.nav_settings(), icon: Settings },
  ];

  const mobile = isMobile();
</script>

{#if mobile}
  <!-- Mobile: bottom tab bar -->
  <nav class="fixed bottom-0 left-0 right-0 z-40 flex border-t border-[var(--color-border)] bg-[var(--color-bg)] safe-area-bottom">
    {#each tabs as tab}
      <button
        class="flex-1 flex flex-col items-center justify-center gap-0.5 py-3 text-xs font-medium transition-colors
               min-h-[56px]"
        class:text-[var(--color-primary)]={appStore.currentView === tab.id}
        onclick={() => (appStore.currentView = tab.id)}
      >
        <tab.icon size={20} />
        {tab.label()}
      </button>
    {/each}
  </nav>
{:else}
  <!-- Desktop: top tab bar -->
  <nav class="flex border-b border-[var(--color-border)] px-2 shrink-0">
    {#each tabs as tab}
      <button
        class="flex items-center gap-1.5 px-4 py-3 text-sm font-medium transition-colors
               hover:text-[var(--color-primary)]"
        class:text-[var(--color-primary)]={appStore.currentView === tab.id}
        class:border-b-2={appStore.currentView === tab.id}
        class:border-[var(--color-primary)]={appStore.currentView === tab.id}
        onclick={() => (appStore.currentView = tab.id)}
      >
        <tab.icon size={16} />
        {tab.label()}
      </button>
    {/each}
  </nav>
{/if}
