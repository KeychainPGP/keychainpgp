<script lang="ts">
  import { Eye, EyeOff } from "lucide-svelte";
  import ModalContainer from "./ModalContainer.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import * as m from "$lib/paraglide/messages.js";

  let passphrase = $state("");
  let showPassword = $state(false);

  function handleSubmit() {
    if (appStore.modalProps.onSubmit) {
      appStore.modalProps.onSubmit(passphrase);
    }
  }
</script>

<ModalContainer title={m.passphrase_title()}>
  <div class="space-y-4">
    <p class="text-sm text-[var(--color-text-secondary)]">
      {m.passphrase_desc()}
    </p>
    <div class="relative">
      <input
        type={showPassword ? "text" : "password"}
        placeholder={m.passphrase_placeholder()}
        bind:value={passphrase}
        class="w-full px-3 py-2.5 text-sm rounded-lg border border-[var(--color-border)]
               bg-[var(--color-bg)] pr-10 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
        onkeydown={(e) => e.key === "Enter" && handleSubmit()}
      />
      <button
        class="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => (showPassword = !showPassword)}
      >
        {#if showPassword}
          <EyeOff size={16} class="text-[var(--color-text-secondary)]" />
        {:else}
          <Eye size={16} class="text-[var(--color-text-secondary)]" />
        {/if}
      </button>
    </div>
    <div class="flex justify-end gap-2">
      <button
        class="px-4 py-2 text-sm rounded-lg border border-[var(--color-border)]
               hover:bg-[var(--color-bg-secondary)] transition-colors"
        onclick={() => appStore.closeModal()}
      >
        {m.passphrase_cancel()}
      </button>
      <button
        class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors disabled:opacity-50"
        onclick={handleSubmit}
        disabled={!passphrase}
      >
        {m.passphrase_submit()}
      </button>
    </div>
  </div>
</ModalContainer>
