<script lang="ts">
  import { ShieldCheck, ShieldAlert, ShieldQuestion } from "lucide-svelte";
  import { TRUST_COLORS } from "$lib/types";
  import * as m from "$lib/paraglide/messages.js";

  interface Props {
    level: number;
  }
  let { level }: Props = $props();

  const TRUST_LABEL_FNS: Record<number, () => string> = {
    0: () => m.trust_unknown(),
    1: () => m.trust_imported(),
    2: () => m.trust_verified(),
  };

  const label = $derived((TRUST_LABEL_FNS[level] ?? TRUST_LABEL_FNS[0])());
  const colorClass = $derived(TRUST_COLORS[level] ?? "text-gray-400");
  const Icon = $derived(level >= 2 ? ShieldCheck : level === 1 ? ShieldAlert : ShieldQuestion);
</script>

<span class="inline-flex items-center gap-1 text-xs font-medium {colorClass}">
  <Icon size={14} />
  {label}
</span>
