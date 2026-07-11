<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton/index.js";
  import { Plus, Search, Loader2 } from "@lucide/svelte";

  interface Props {
    searchPlaceholder?: string;
    searchValue?: string;
    onCreateNew?: () => void;
    createLabel?: string;
    createTitle?: string;
    loading?: boolean;
    showLoadingSkeleton?: boolean;
    isEmpty?: boolean;
    /** When false, body manages its own scroll (e.g. nested workspace list). */
    internalScroll?: boolean;
    filters?: Snippet;
    toolbar?: Snippet;
    meta?: Snippet;
    footer?: Snippet;
    empty?: Snippet;
    children?: Snippet;
  }

  let {
    searchPlaceholder = "Search…",
    searchValue = $bindable(""),
    onCreateNew,
    createLabel,
    createTitle = "New",
    loading = false,
    showLoadingSkeleton = false,
    isEmpty = false,
    internalScroll = true,
    filters,
    toolbar,
    meta,
    footer,
    empty,
    children,
  }: Props = $props();
</script>

<div class="flex h-full flex-col">
  <div
    class="divider-edge-b divider-edge-full space-y-2 px-3 py-2.5 transition-opacity {showLoadingSkeleton
      ? 'pointer-events-none opacity-50'
      : ''}"
  >
    <div class="flex items-center gap-2">
      <div class="relative min-w-0 flex-1">
        <Search
          class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground"
        />
        <Input
          placeholder={searchPlaceholder}
          bind:value={searchValue}
          class="h-8 pl-8 text-sm"
        />
      </div>
      {#if toolbar}
        {@render toolbar()}
      {:else if onCreateNew}
        <Button onclick={onCreateNew} size="sm" class="h-8 shrink-0" title={createTitle}>
          <Plus class="h-3.5 w-3.5" />
          {#if createLabel}
            <span class="text-xs">{createLabel}</span>
          {/if}
        </Button>
      {/if}
    </div>

    {#if filters}
      {@render filters()}
    {/if}

    {#if meta}
      {@render meta()}
    {/if}
  </div>

  <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
    {#if showLoadingSkeleton}
      <div class="flex flex-1 flex-col gap-2 p-2.5">
        <div class="flex items-center justify-center gap-2 py-4 text-muted-foreground">
          <Loader2 class="h-4 w-4 animate-spin" />
          <p class="text-sm">Loading…</p>
        </div>
        {#each Array(4) as _, i (i)}
          <Skeleton class="h-8 w-full" />
        {/each}
      </div>
    {:else if isEmpty && empty}
      <div class="flex flex-1 flex-col">
        {@render empty()}
      </div>
    {:else if internalScroll}
      <ScrollArea class="flex-1">
        <div class="space-y-2 p-2">
          {#if children}
            {@render children()}
          {/if}
        </div>
      </ScrollArea>
    {:else if children}
      {@render children()}
    {/if}
  </div>

  {#if footer}
    {@render footer()}
  {/if}
</div>
