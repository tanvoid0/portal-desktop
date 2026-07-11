<script lang="ts">
  import { ChevronRight } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import type { ActivitySummaryPart } from "../utils/feedBlocks.js";

  interface Props {
    parts: ActivitySummaryPart[];
    isRunning?: boolean;
    hasFailed?: boolean;
    /** When expanded, render child tool lines. */
    children?: import("svelte").Snippet;
  }

  let {
    parts,
    isRunning = false,
    hasFailed = false,
    children,
  }: Props = $props();

  let open = $state(false);
</script>

<div class="py-0.5">
  <Button
    type="button"
    variant="ghost"
    class="h-auto w-full justify-between gap-2 px-0 py-0.5 text-left text-[11px] font-normal hover:bg-transparent {hasFailed
      ? 'text-destructive hover:text-destructive'
      : isRunning
        ? 'text-foreground hover:text-foreground'
        : 'text-muted-foreground hover:text-foreground'}"
    onclick={() => (open = !open)}
  >
    <span class="min-w-0 flex-1 leading-snug">
      {#each parts as part, i}
        {#if i > 0}<span class="text-muted-foreground">, </span>{/if}
        {#if part.isActive}
          <span class="font-semibold text-foreground">{part.verb}</span
          ><span>{part.rest}</span>
        {:else}
          <span>{part.verb}{part.rest}</span>
        {/if}
      {/each}
    </span>
    <ChevronRight
      class="h-3 w-3 shrink-0 transition-transform {open ? 'rotate-90' : ''}"
    />
  </Button>

  {#if open && children}
    <div
      class="mt-1 space-y-0.5 rounded-lg border border-border/80 px-2.5 py-1.5"
    >
      {@render children()}
    </div>
  {/if}
</div>
