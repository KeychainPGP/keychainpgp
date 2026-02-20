<script lang="ts">
  import { Shield, KeyRound, Settings } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import type { View } from "$lib/types";

  const tabs: { id: View; label: string; icon: typeof Shield }[] = [
    { id: "home", label: "Encrypt / Decrypt", icon: Shield },
    { id: "keys", label: "Keys", icon: KeyRound },
    { id: "settings", label: "Settings", icon: Settings },
  ];
</script>

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
      {tab.label}
    </button>
  {/each}
</nav>
