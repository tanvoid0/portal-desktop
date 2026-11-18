<script lang="ts">
  import type { ScanSummary } from '../types';
  import { fmtBytes, DISK_STAT_GRID_FOUR } from '../utils';
  import StatTiles from './shared/StatTiles.svelte';

  let { proposals }: { proposals: ScanSummary['proposals'] } = $props();

  const stats = $derived.by(() => {
    const byRisk: Record<string, { n: number; b: number }> = {
      Safe: { n: 0, b: 0 },
      Review: { n: 0, b: 0 },
      Danger: { n: 0, b: 0 },
    };
    let total = 0;
    for (const p of proposals) {
      const rr = byRisk[p.risk] ?? (byRisk[p.risk] = { n: 0, b: 0 });
      rr.n++;
      rr.b += p.sizeBytes;
      total += p.sizeBytes;
    }
    return { total, byRisk, count: proposals.length };
  });
</script>

<StatTiles
  grid={DISK_STAT_GRID_FOUR}
  tiles={[
    {
      label: 'Reclaimable',
      value: fmtBytes(stats.total),
      sub: `${stats.count} proposal${stats.count === 1 ? '' : 's'}`,
    },
    {
      label: 'Safe',
      value: fmtBytes(stats.byRisk.Safe.b),
      sub: `${stats.byRisk.Safe.n} items — rebuilt or re-downloadable`,
      accent: 'text-status-success',
    },
    {
      label: 'Review',
      value: fmtBytes(stats.byRisk.Review.b),
      sub: `${stats.byRisk.Review.n} items — check before ticking`,
      accent: 'text-status-warning',
    },
    {
      label: 'Danger',
      value: fmtBytes(stats.byRisk.Danger.b),
      sub: `${stats.byRisk.Danger.n} items — could be the only copy`,
      accent: 'text-status-error',
    },
  ]}
/>

{#if stats.total > 0}
  <div class="mb-6 flex h-2 w-full overflow-hidden rounded-full bg-muted">
    {#each [['Safe', stats.byRisk.Safe.b, 'bg-status-success'], ['Review', stats.byRisk.Review.b, 'bg-status-warning'], ['Danger', stats.byRisk.Danger.b, 'bg-status-error']] as [risk, bytes, color]}
      {#if (bytes as number) > 0}
        <div
          class={color as string}
          style="width: {((bytes as number) / stats.total) * 100}%"
          title={String(risk)}
        ></div>
      {/if}
    {/each}
  </div>
{/if}
