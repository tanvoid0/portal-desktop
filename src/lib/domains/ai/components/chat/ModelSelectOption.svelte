<script lang="ts">
  import { Eye, Wrench, Binary } from '@lucide/svelte';
  import { cn } from '$lib/utils';
  import type { CatalogModel } from '../../types/index.js';
  import { getModelRow } from '../../utils/catalog.js';

  interface Props {
    model: CatalogModel;
    /** Trigger view: name plus quant only, on one line. */
    compact?: boolean;
    class?: string;
  }

  let { model, compact = false, class: className = '' }: Props = $props();

  const row = $derived(getModelRow(model));
</script>

<!--
LM Studio-style row: identity on the left, then aligned metadata columns so
models line up against each other. Columns collapse for providers that report
no metadata (most cloud backends).
-->
<div class={cn('flex min-w-0 items-center gap-2 text-xs', compact ? '' : 'w-full', className)}>
  <div class="flex min-w-0 flex-1 items-center gap-1.5">
    <span class="truncate text-sm font-medium text-foreground">{row.name}</span>
    {#if row.quant}
      <span
        class="shrink-0 rounded border border-border bg-muted/60 px-1 text-[9px] font-medium uppercase tracking-wide text-muted-foreground"
        title="Quantization"
      >
        {row.quant}
      </span>
    {/if}
    {#if !compact}
      {#if row.vision}
        <Eye class="h-3.5 w-3.5 shrink-0 text-amber-600 dark:text-amber-400" />
      {/if}
      {#if row.tools}
        <Wrench class="h-3.5 w-3.5 shrink-0 text-sky-600 dark:text-sky-400" />
      {/if}
      {#if row.embeddings}
        <Binary class="h-3.5 w-3.5 shrink-0 text-emerald-600 dark:text-emerald-400" />
      {/if}
    {/if}
  </div>

  {#if !compact}
    <span class="w-20 shrink-0 truncate text-muted-foreground" title="Publisher">
      {row.publisher ?? ''}
    </span>
    <span class="w-16 shrink-0 truncate tabular-nums text-muted-foreground" title="Parameters">
      {row.params ?? ''}
    </span>
    <span class="w-24 shrink-0">
      {#if row.family}
        <span
          class="block truncate rounded border border-border px-1 py-px text-[10px] text-muted-foreground"
          title="Architecture"
        >
          {row.family}
        </span>
      {/if}
    </span>
    <span class="w-14 shrink-0">
      {#if row.source}
        <span
          class={cn(
            'block rounded px-1 py-px text-center text-[10px] font-semibold uppercase text-white',
            row.source === 'live' ? 'bg-indigo-600' : 'bg-orange-500'
          )}
          title={row.source === 'live' ? 'Discovered live' : 'Config alias'}
        >
          {row.source}
        </span>
      {/if}
    </span>
    <span class="w-20 shrink-0 text-right tabular-nums text-muted-foreground" title="On-disk size">
      {row.sizeLabel ?? ''}
    </span>
  {/if}
</div>

{#if !compact && row.backendId}
  <p class="mt-0.5 text-[10px] text-muted-foreground/70">
    Resolves to <span class="font-mono">{row.backendId}</span>
  </p>
{/if}
