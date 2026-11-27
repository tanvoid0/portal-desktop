<!--
	System Page
	Live CPU, memory, GPU and per-volume usage of the host, over `host_stats` + `disk_usage`.
	Same page on the desktop and the thin client — both commands are served over RPC.
-->
<script lang="ts">
  import {
    PageContainer,
    PageEmpty,
    PageError,
    PageHeader,
    PageLoading,
  } from '$lib/components/shell';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { createDiskUsageQuery, createHostStatsQuery } from '$lib/domains/disk/queries/hostStats';
  import { fmtBytes, pressureClass, usedPct } from '$lib/domains/disk/utils';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import Cpu from '~icons/fa6-solid/microchip';
  import Cube from '~icons/fa6-solid/cube';
  import HardDrive from '~icons/fa6-solid/hard-drive';
  import MemoryStick from '~icons/fa6-solid/memory';

  const hostQ = createHostStatsQuery();
  const diskQ = createDiskUsageQuery();

  const volumes = $derived(diskQ.data ?? []);
  // `pct` stays null until the first reading lands: CPU % is a delta between polls, so there is
  // nothing honest to draw before that.
  const meters = $derived.by(() => {
    const h = hostQ.data;
    return [
      {
        label: 'CPU',
        icon: Cpu,
        pct: h ? usedPct(h.cpuPercent, 100) : null,
        detail: h ? 'Across all cores' : 'Measuring…',
      },
      {
        label: 'Memory',
        icon: MemoryStick,
        pct: h ? usedPct(h.memoryUsedBytes, h.memoryTotalBytes) : null,
        detail: h
          ? `${fmtBytes(h.memoryUsedBytes)} of ${fmtBytes(h.memoryTotalBytes)}`
          : 'Reading…',
      },
      // Only once a reading says there is a GPU to report.
      ...(h?.gpuPercent != null
        ? [{ label: 'GPU', icon: Cube, pct: usedPct(h.gpuPercent, 100), detail: 'Compute load' }]
        : []),
    ];
  });
</script>

<svelte:head>
  <title>System - Portal Desktop</title>
</svelte:head>

<!-- px-0: the app shell's own PageContainer already sets the gutter. -->
<PageContainer variant="readable" class="space-y-6 px-0">
  <PageHeader title="System" description="Live resource and storage usage of the host machine" />

  {#if hostQ.error && !hostQ.data}
    <PageError
      title="Could not read host stats"
      message={getErrorMessage(hostQ.error, 'Failed to read host stats')}
      onRetry={() => hostQ.refetch()}
    />
  {:else}
    <div class="grid grid-cols-1 gap-4 {meters.length > 2 ? 'lg:grid-cols-3' : 'md:grid-cols-2'}">
      {#each meters as m (m.label)}
        {@const Icon = m.icon}
        <Card class="gap-0 py-4">
          <CardContent class="px-4">
            <div
              class="mb-2 flex items-center gap-2 text-xs tracking-wide text-muted-foreground uppercase"
            >
              <Icon class="size-3.5" />
              {m.label}
            </div>
            {#if m.pct === null}
              <Skeleton class="h-8 w-24" />
              <Skeleton class="mt-3 h-2 w-full rounded-full" />
            {:else}
              <div class="text-2xl font-semibold tracking-tight tabular-nums">
                {m.pct.toFixed(1)}%
              </div>
              <Progress
                value={m.pct}
                aria-label="{m.label} usage"
                class="mt-3 {pressureClass(m.pct)}"
              />
            {/if}
            <div class="mt-2 text-xs text-muted-foreground tabular-nums">{m.detail}</div>
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}

  <section class="space-y-3">
    <h2 class="text-xs tracking-wide text-muted-foreground uppercase">Volumes</h2>
    {#if diskQ.isPending}
      <PageLoading message="Reading volumes..." />
    {:else if diskQ.error && !diskQ.data}
      <PageError
        title="Could not read volumes"
        message={getErrorMessage(diskQ.error, 'Failed to read volumes')}
        onRetry={() => diskQ.refetch()}
      />
    {:else if volumes.length === 0}
      <PageEmpty
        title="No volumes"
        description="The host reported no mounted volumes."
        icon={HardDrive}
      />
    {:else}
      <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
        {#each volumes as d (d.mountPoint)}
          {@const used = Math.max(0, d.totalBytes - d.availableBytes)}
          {@const pct = usedPct(used, d.totalBytes)}
          <Card class="gap-0 py-4">
            <CardContent class="px-4">
              <div class="mb-2 flex items-center gap-2">
                <span class="truncate text-sm font-medium" title={d.mountPoint}>
                  {d.mountPoint}
                </span>
                {#if d.name.trim()}
                  <span class="truncate text-xs text-muted-foreground">{d.name}</span>
                {/if}
                <span class="ml-auto shrink-0 text-xs text-muted-foreground uppercase">
                  {d.fsKind}
                </span>
              </div>
              <Progress value={pct} aria-label="{d.mountPoint} usage" class={pressureClass(pct)} />
              <div
                class="mt-2 flex items-center justify-between gap-2 text-xs text-muted-foreground tabular-nums"
              >
                <span>{fmtBytes(used)} of {fmtBytes(d.totalBytes)}</span>
                <span>{pct.toFixed(0)}%</span>
              </div>
            </CardContent>
          </Card>
        {/each}
      </div>
    {/if}
  </section>
</PageContainer>
