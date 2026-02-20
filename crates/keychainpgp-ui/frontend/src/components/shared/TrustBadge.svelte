<script lang="ts">
  import { ShieldCheck, ShieldAlert, ShieldQuestion } from "lucide-svelte";
  import { TRUST_LABELS, TRUST_COLORS } from "$lib/types";

  interface Props {
    level: number;
  }
  let { level }: Props = $props();

  const label = $derived(TRUST_LABELS[level] ?? "Unknown");
  const colorClass = $derived(TRUST_COLORS[level] ?? "text-gray-400");
  const Icon = $derived(level >= 2 ? ShieldCheck : level === 1 ? ShieldAlert : ShieldQuestion);
</script>

<span class="inline-flex items-center gap-1 text-xs font-medium {colorClass}">
  <Icon size={14} />
  {label}
</span>
