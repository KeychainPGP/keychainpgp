<script lang="ts">
  import { ShieldCheck, ShieldAlert, ShieldX } from "lucide-svelte";
  import ModalContainer from "./ModalContainer.svelte";
  import FingerprintDisplay from "../shared/FingerprintDisplay.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import * as m from "$lib/paraglide/messages.js";

  const result = appStore.modalProps.verifyResult;
  const valid = result?.valid ?? false;
  const trustLevel = result?.trust_level ?? 0;
</script>

<ModalContainer title={m.verify_modal_title()}>
  <div class="space-y-4">
    <div class="flex items-center gap-3">
      {#if valid && trustLevel >= 2}
        <div class="rounded-full bg-green-100 p-2">
          <ShieldCheck size={28} class="text-green-600" />
        </div>
        <div>
          <p class="font-semibold text-green-600">{m.verify_valid_verified()}</p>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {m.verify_valid_verified_desc()}
          </p>
        </div>
      {:else if valid}
        <div class="rounded-full bg-yellow-100 p-2">
          <ShieldAlert size={28} class="text-yellow-600" />
        </div>
        <div>
          <p class="font-semibold text-yellow-600">{m.verify_valid_unverified()}</p>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {m.verify_valid_unverified_desc()}
          </p>
        </div>
      {:else}
        <div class="rounded-full bg-red-100 p-2">
          <ShieldX size={28} class="text-red-600" />
        </div>
        <div>
          <p class="font-semibold text-red-600">{m.verify_invalid()}</p>
          <p class="text-sm text-[var(--color-text-secondary)]">
            {result?.message ?? ""}
          </p>
        </div>
      {/if}
    </div>

    {#if valid && result}
      <div
        class="space-y-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4"
      >
        {#if result.signer_name}
          <div class="flex justify-between text-sm">
            <span class="text-[var(--color-text-secondary)]">{m.verify_signer_label()}</span>
            <span class="font-medium">{result.signer_name}</span>
          </div>
        {/if}
        {#if result.signer_email}
          <div class="flex justify-between text-sm">
            <span class="text-[var(--color-text-secondary)]">{m.verify_email_label()}</span>
            <span>{result.signer_email}</span>
          </div>
        {/if}
        {#if result.signer_fingerprint}
          <div class="flex items-center justify-between text-sm">
            <span class="text-[var(--color-text-secondary)]">{m.verify_fingerprint_label()}</span>
            <FingerprintDisplay fingerprint={result.signer_fingerprint} short />
          </div>
        {/if}
      </div>
    {/if}

    <div class="flex justify-end">
      <button
        class="rounded-lg bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white
               transition-colors hover:bg-[var(--color-primary-hover)]"
        onclick={() => appStore.closeModal()}
      >
        {m.qr_close()}
      </button>
    </div>
  </div>
</ModalContainer>
