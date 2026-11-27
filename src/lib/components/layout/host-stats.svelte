<!-- CPU / RAM / GPU usage, sitting between the footer buttons. Renders nothing until a reading
     lands (never, outside the desktop shell). -->
<script lang="ts">
  import { createHostStatsQuery } from '$lib/domains/disk/queries/hostStats';
  import { fmtBytes, pressureTextClass, usedPct } from '$lib/domains/disk/utils';

  const hostQ = createHostStatsQuery();

  const meters = $derived.by(() => {
    const h = hostQ.data;
    if (!h) return [];
    const list = [
      { label: 'CPU', pct: h.cpuPercent, title: 'Host CPU load' },
      {
        label: 'RAM',
        pct: usedPct(h.memoryUsedBytes, h.memoryTotalBytes),
        title: `${fmtBytes(h.memoryUsedBytes)} of ${fmtBytes(h.memoryTotalBytes)} used`,
      },
    ];
    if (h.gpuPercent != null) list.push({ label: 'GPU', pct: h.gpuPercent, title: 'GPU load' });
    return list;
  });
</script>

{#if meters.length}
  <div class="flex min-w-0 items-center gap-2.5 group-data-[collapsible=icon]:hidden">
    {#each meters as m (m.label)}
      <div class="flex flex-col items-center leading-tight tabular-nums" title={m.title}>
        <span class="text-[9px] text-muted-foreground">{m.label}</span>
        <span class="text-[11px] font-medium {pressureTextClass(m.pct)}">{Math.round(m.pct)}%</span>
      </div>
    {/each}
  </div>
{/if}
