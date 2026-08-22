<!--
	The summary row every scan result opens with. One component so "Reclaimable"
	means the same thing, and sits in the same place, on every tab.
-->

<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { DISK_STAT_GRID } from '../../utils';

  export type Tile = {
    label: string;
    /** Pre-formatted — the caller decides bytes vs count. */
    value: string;
    sub?: string;
    /** Tailwind text colour class, e.g. `text-status-success`. */
    accent?: string;
  };

  let { tiles, grid = DISK_STAT_GRID }: { tiles: Tile[]; grid?: string } = $props();
</script>

<div class="{grid} mb-4">
  {#each tiles as t (t.label)}
    <Card class="gap-0 py-4">
      <CardContent class="px-4">
        <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">{t.label}</div>
        <div
          class="text-2xl font-semibold tabular-nums tracking-tight {t.accent ?? 'text-foreground'}"
        >
          {t.value}
        </div>
        {#if t.sub}
          <div class="mt-1 text-xs text-muted-foreground">{t.sub}</div>
        {/if}
      </CardContent>
    </Card>
  {/each}
</div>
