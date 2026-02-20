<script lang="ts">
  import { Search } from "lucide-svelte";

  interface Props {
    value: string;
    placeholder?: string;
    oninput: (value: string) => void;
  }
  let { value, placeholder = "Search...", oninput }: Props = $props();
</script>

<div class="relative">
  <Search
    size={16}
    class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-secondary)]"
  />
  <input
    type="text"
    {placeholder}
    {value}
    oninput={(e) => oninput(e.currentTarget.value)}
    class="w-full pl-9 pr-3 py-2 text-sm rounded-lg border border-[var(--color-border)]
           bg-[var(--color-bg)] text-[var(--color-text)]
           focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent"
  />
</div>
