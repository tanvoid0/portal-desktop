<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { cn } from "$lib/utils";
  import type { CatalogModel } from "../../types/index.js";
  import {
    getModelDisplayParts,
    type ModelDisplayBadge,
  } from "../../utils/catalog.js";

  interface Props {
    model: CatalogModel;
    compact?: boolean;
    class?: string;
  }

  let { model, compact = false, class: className = "" }: Props = $props();

  const parts = $derived(getModelDisplayParts(model));
</script>

{#snippet metadataBadge(badge: ModelDisplayBadge, showTitle = true)}
  <Badge
    variant="outline"
    title="{badge.title}: {badge.label}"
    class={cn(
      "inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium leading-4 shadow-none",
      badge.className,
    )}
  >
    {#if showTitle}
      <span class="text-[9px] font-normal uppercase tracking-wide text-muted-foreground/80">
        {badge.title}
      </span>
    {/if}
    <span class={cn("font-semibold", badge.valueClassName)}>{badge.label}</span>
  </Badge>
{/snippet}

<div class={cn("flex min-w-0 flex-col", compact ? "gap-0" : "gap-1.5", className)}>
  <div class="flex min-w-0 items-start justify-between gap-2">
    <div class="min-w-0">
      <span class="block truncate text-sm font-semibold text-foreground">
        {parts.id}
      </span>
      {#if !compact && parts.displayName}
        <span class="mt-0.5 block truncate text-xs text-muted-foreground">
          {parts.displayName}
        </span>
      {/if}
    </div>

    {#if !compact && parts.statusBadges.length > 0}
      <div class="flex shrink-0 flex-wrap justify-end gap-1">
        {#each parts.statusBadges as badge (badge.key)}
          {@render metadataBadge(badge, false)}
        {/each}
      </div>
    {/if}
  </div>

  {#if !compact && parts.backendId}
    <p class="text-[11px] text-muted-foreground">
      <span class="text-muted-foreground/70">Resolves to</span>
      <span class="ms-1 font-mono text-xs text-foreground/80">{parts.backendId}</span>
    </p>
  {/if}

  {#if !compact && parts.specBadges.length > 0}
    <div class="flex flex-wrap items-center gap-1">
      {#each parts.specBadges as badge (badge.key)}
        {@render metadataBadge(badge, true)}
      {/each}
    </div>
  {/if}
</div>
