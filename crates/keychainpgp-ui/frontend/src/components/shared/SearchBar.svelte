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
    class="absolute top-1/2 left-3 -translate-y-1/2 text-[var(--color-text-secondary)]"
  />
  <input
    type="text"
    {placeholder}
    {value}
    oninput={(e) => oninput(e.currentTarget.value)}
    class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] py-2 pr-3 pl-9
           text-sm text-[var(--color-text)]
           focus:border-transparent focus:ring-2 focus:ring-[var(--color-primary)] focus:outline-none"
  />
</div>
