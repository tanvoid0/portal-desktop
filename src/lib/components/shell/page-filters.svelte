<script lang="ts">
  import type { Snippet } from "svelte";
  import { Input } from "$lib/components/ui/input";

  interface Props {
    searchQuery?: string;
    searchPlaceholder?: string;
    onSearchChange?: (value: string) => void;
    filters?: Snippet;
    actions?: Snippet;
  }

  let {
    searchQuery = $bindable(""),
    searchPlaceholder = "Search...",
    onSearchChange,
    filters,
    actions,
  }: Props = $props();

  function handleSearchInput(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value;
    searchQuery = value;
    onSearchChange?.(value);
  }
</script>

<div class="flex flex-wrap items-center gap-4">
  <Input
    placeholder={searchPlaceholder}
    value={searchQuery}
    oninput={handleSearchInput}
    class="max-w-sm"
  />
  {#if filters}
    {@render filters()}
  {/if}
  {#if actions}
    <div class="ml-auto flex items-center gap-2">
      {@render actions()}
    </div>
  {/if}
</div>
