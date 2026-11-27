<!-- CPU / RAM / GPU usage, sitting between the footer buttons; links to the System page. Renders
     nothing until a reading lands (never, outside the desktop shell). -->
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
  <!-- Collapsed rail: the same readings stacked, still the way in to /system. -->
  <a
    href="/system"
    class="flex min-w-0 items-center gap-2.5 rounded-md px-1.5 py-0.5 transition-colors group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:gap-1 group-data-[collapsible=icon]:px-0 hover:bg-sidebar-accent focus-visible:ring-2 focus-visible:ring-sidebar-ring focus-visible:outline-none"
  >
    <span class="sr-only">Open System page.</span>
    {#each meters as m (m.label)}
      <span class="flex flex-col items-center leading-tight tabular-nums" title={m.title}>
        <span class="text-[9px] text-muted-foreground">{m.label}</span>
        <span class="text-[11px] font-medium {pressureTextClass(m.pct)}">{Math.round(m.pct)}%</span>
      </span>
    {/each}
  </a>
{/if}
