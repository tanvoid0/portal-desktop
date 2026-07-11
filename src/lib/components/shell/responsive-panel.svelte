<script lang="ts">
  import type { Snippet } from "svelte";
  import * as Sheet from "$lib/components/ui/sheet/index.js";
  import { IsMobile } from "$lib/hooks/is-mobile.svelte.js";
  import { cn } from "$lib/utils.js";

  interface Props {
    side?: "left" | "right";
    title?: string;
    open?: boolean;
    desktopClass?: string;
    borderClass?: string;
    children: Snippet<[]>;
    header?: Snippet<[]>;
  }

  let {
    side = "left",
    title,
    open = $bindable(false),
    desktopClass = "w-64",
    borderClass,
    children,
    header,
  }: Props = $props();

  const isMobile = new IsMobile();

  const resolvedBorder = $derived(
    borderClass ??
      (side === "left" ? "divider-edge-r" : "divider-edge-l"),
  );
</script>

{#if isMobile.current}
  <Sheet.Root bind:open>
    <Sheet.Content
      {side}
      class={cn(
        "flex w-[min(20rem,85vw)] max-w-sm flex-col gap-0 p-0",
        "[&>button]:hidden",
      )}
    >
      {#if header}
        <div class="divider-edge-b divider-edge-full shrink-0 bg-background">
          {@render header()}
        </div>
      {:else if title}
        <Sheet.Header class="divider-edge-b divider-edge-full shrink-0 px-4 py-3 text-left">
          <Sheet.Title class="text-sm font-semibold">{title}</Sheet.Title>
        </Sheet.Header>
      {/if}
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
        {@render children()}
      </div>
    </Sheet.Content>
  </Sheet.Root>
{:else}
  <aside
    class={cn(
      "flex shrink-0 flex-col overflow-hidden bg-background",
      resolvedBorder,
      desktopClass,
    )}
  >
    {#if header}
      <div class="divider-edge-b divider-edge-full shrink-0 bg-background">
        {@render header()}
      </div>
    {:else if title}
      <div class="divider-edge-b divider-edge-full shrink-0 px-4 py-3">
        <h2 class="text-sm font-semibold">{title}</h2>
      </div>
    {/if}
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      {@render children()}
    </div>
  </aside>
{/if}
