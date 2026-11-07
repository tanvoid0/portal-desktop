<script lang="ts">
  import type { Component } from 'svelte';

  export interface PageStat {
    label: string;
    value: string | number;
    icon?: Component;
    description?: string;
  }

  interface Props {
    stats: PageStat[];
    /** Kept for API compatibility — the strip is a single wrapping row. */
    columns?: 2 | 3 | 4;
  }

  let { stats }: Props = $props();
</script>

<div class="flex flex-wrap items-center gap-y-1 rounded-md border bg-card px-3 py-1.5">
  {#each stats as stat}
    <div
      class="flex items-center gap-1.5 border-l px-4 first:border-l-0 first:pl-0"
      title={stat.description}
    >
      {#if stat.icon}
        {@const Icon = stat.icon}
        <Icon class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
      {/if}
      <span class="text-xs text-muted-foreground">{stat.label}</span>
      <span class="text-sm font-semibold tabular-nums">{stat.value}</span>
    </div>
  {/each}
</div>
