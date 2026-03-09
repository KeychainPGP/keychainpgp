<script lang="ts">
    import { Info } from "lucide-svelte";
    import ModalContainer from "./ModalContainer.svelte";
    import { appStore } from "$lib/stores/app.svelte";
    import * as m from "$lib/paraglide/messages.js";
</script>

<ModalContainer title={appStore.modalProps.title ?? m.confirm_default_title()}>
    <div class="space-y-4">
        <div class="flex items-start gap-3">
            <Info
                size={20}
                class="text-[var(--color-primary)] shrink-0 mt-0.5"
            />
            <p class="text-sm">
                {appStore.modalProps.message ?? ""}
            </p>
        </div>
        <div class="flex justify-end">
            <button
                class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white font-medium
               hover:bg-[var(--color-primary-hover)] transition-colors"
                onclick={() => appStore.closeModal()}
            >
                {m.error_ok()}
            </button>
        </div>
    </div>
</ModalContainer>
