<!--
	System Page
	Live state of the host over `host_stats`, `system_details` and `disk_usage`. Same page on the
	desktop and the thin client — all three are served over RPC. Any reading the host can't give
	is left out rather than shown empty.
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
  import * as Table from '$lib/components/ui/table';
  import Trace from '$lib/domains/disk/components/Trace.svelte';
  import {
    createDiskUsageQuery,
    createHostStatsQuery,
    createSystemDetailsQuery,
  } from '$lib/domains/disk/queries/hostStats';
  import { history } from '$lib/domains/disk/state/history.svelte';
  import { fmtBytes, pressureClass, pressureTextClass, usedPct } from '$lib/domains/disk/utils';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import HardDrive from '~icons/fa6-solid/hard-drive';

  const hostQ = createHostStatsQuery();
  const detailsQ = createSystemDetailsQuery();
  const diskQ = createDiskUsageQuery();

  const host = $derived(hostQ.data);
  const details = $derived(detailsQ.data);
  const gpu = $derived(host?.gpu);
  // Drive letter / mount point order; the backend lists biggest first.
  const volumes = $derived(
    [...(diskQ.data ?? [])].sort((a, b) => a.mountPoint.localeCompare(b.mountPoint))
  );
  const hasDriveTemps = $derived(volumes.some((v) => v.temperatureC != null));
  const peakCore = $derived(details?.measured ? Math.max(0, ...details.corePercents) : null);
  // An idle link would otherwise scale its background chatter to full height.
  const netMax = $derived(Math.max(100 * 1024, ...history.net.flatMap((s) => [s.a, s.b])));

  function fmtUptime(secs: number): string {
    const d = Math.floor(secs / 86400);
    const h = Math.floor((secs % 86400) / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return d ? `${d}d ${h}h` : h ? `${h}h ${m}m` : `${m}m`;
  }

  const rate = (bytes: number) => `${fmtBytes(Math.round(bytes))}/s`;

  // `pct` stays null until the first reading lands: CPU % is a delta between polls, so there is
  // nothing honest to draw before that.
  const meters = $derived.by(() => {
    const list: { label: string; pct: number | null; lines: string[] }[] = [
      {
        label: 'CPU',
        pct: host ? usedPct(host.cpuPercent, 100) : null,
        lines: [peakCore === null ? 'Across all cores' : `Peak core ${peakCore.toFixed(0)}%`],
      },
      {
        label: 'Memory',
        pct: host ? usedPct(host.memoryUsedBytes, host.memoryTotalBytes) : null,
        lines: [
          host
            ? `${fmtBytes(host.memoryUsedBytes)} of ${fmtBytes(host.memoryTotalBytes)}`
            : 'Reading…',
        ],
      },
    ];
    if (gpu) {
      const vram =
        gpu.memoryUsedBytes != null && gpu.memoryTotalBytes != null
          ? `VRAM ${fmtBytes(gpu.memoryUsedBytes)} of ${fmtBytes(gpu.memoryTotalBytes)}`
          : null;
      const sensors = [
        gpu.temperatureC != null ? `${gpu.temperatureC.toFixed(0)} °C` : null,
        gpu.powerWatts != null ? `${gpu.powerWatts.toFixed(0)} W` : null,
        gpu.fanPercent != null ? `Fan ${gpu.fanPercent.toFixed(0)}%` : null,
      ].filter(Boolean);
      list.push({
        label: 'GPU',
        pct: host?.gpuPercent != null ? usedPct(host.gpuPercent, 100) : null,
        lines: [gpu.name, vram, sensors.join(' · ')].filter(
          (l) => l != null && l !== ''
        ) as string[],
      });
    }
    return list;
  });

  const heading = 'text-xs font-medium tracking-wide text-muted-foreground uppercase';
</script>

<svelte:head>
  <title>System - Portal Desktop</title>
</svelte:head>

{#snippet legend(first: string, second: string)}
  <div class="flex flex-wrap gap-x-4 text-xs text-muted-foreground tabular-nums">
    <span class="flex items-center gap-1.5">
      <span class="size-2 rounded-full bg-primary"></span>{first}
    </span>
    <span class="flex items-center gap-1.5">
      <span class="size-2 rounded-full bg-status-warning"></span>{second}
    </span>
  </div>
{/snippet}

<!-- px-0: the app shell's own PageContainer already sets the gutter. -->
<PageContainer variant="full" class="space-y-4 px-0">
  <PageHeader title="System" description="Live state of the host machine" />

  {#if details}
    <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
      <span class="font-medium">{details.cpuName}</span>
      <span class="text-muted-foreground">{details.osName}</span>
      <span class="text-muted-foreground tabular-nums">Up {fmtUptime(details.uptimeSecs)}</span>
      <span class="ml-auto flex items-center gap-1.5 text-xs text-muted-foreground">
        <span
          class="size-2 rounded-full {detailsQ.isError ? 'bg-status-error' : 'bg-status-success'}"
        ></span>
        {detailsQ.isError ? 'Stale' : 'Live'}
      </span>
    </div>
  {/if}

  {#if hostQ.error && !host}
    <PageError
      title="Could not read host stats"
      message={getErrorMessage(hostQ.error, 'Failed to read host stats')}
      onRetry={() => hostQ.refetch()}
    />
  {:else}
    <div class="grid grid-cols-1 gap-4 {meters.length > 2 ? 'lg:grid-cols-3' : 'md:grid-cols-2'}">
      {#each meters as m (m.label)}
        <Card class="gap-0 py-4">
          <CardContent class="px-4">
            <h2 class="mb-2 {heading}">{m.label}</h2>
            {#if m.pct === null && !host}
              <Skeleton class="h-8 w-24" />
              <Skeleton class="mt-3 h-2 w-full rounded-full" />
            {:else if m.pct !== null}
              <div class="text-3xl font-semibold tracking-tight tabular-nums">
                {m.pct.toFixed(1)}%
              </div>
              <Progress
                value={m.pct}
                aria-label="{m.label} usage"
                class="mt-3 {pressureClass(m.pct)}"
              />
            {/if}
            <div class="mt-2 space-y-0.5 text-xs text-muted-foreground tabular-nums">
              {#each m.lines as line (line)}
                <div class="truncate" title={line}>{line}</div>
              {/each}
            </div>
          </CardContent>
        </Card>
      {/each}
    </div>

    <Card class="gap-0 py-4">
      <CardContent class="space-y-2 px-4">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <h2 class={heading}>Last 60 seconds</h2>
          {@render legend('CPU', 'Memory')}
        </div>
        <Trace samples={history.load} max={100} label="CPU and memory over the last minute" />
      </CardContent>
    </Card>
  {/if}

  {#if details?.measured}
    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <Card class="gap-0 py-4">
        <CardContent class="space-y-2 px-4">
          <h2 class={heading}>{details.corePercents.length} logical cores</h2>
          <div
            class="flex h-24 items-stretch gap-0.5"
            role="img"
            aria-label="Per-core CPU load, busiest {peakCore?.toFixed(0)}%"
          >
            {#each details.corePercents as pct, i (i)}
              <div
                class="flex min-w-0 flex-1 items-end overflow-hidden rounded-sm bg-muted"
                title="Core {i}: {pct.toFixed(0)}%"
              >
                <div
                  class="w-full {pct >= 90
                    ? 'bg-status-error'
                    : pct >= 75
                      ? 'bg-status-warning'
                      : 'bg-primary'}"
                  style:height="{Math.min(100, pct)}%"
                ></div>
              </div>
            {/each}
          </div>
        </CardContent>
      </Card>

      <Card class="gap-0 py-4">
        <CardContent class="space-y-2 px-4">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <h2 class={heading}>Network</h2>
            {@render legend(
              `Down ${rate(details.netDownBytesPerSec)}`,
              `Up ${rate(details.netUpBytesPerSec)}`
            )}
          </div>
          <Trace samples={history.net} max={netMax} label="Network traffic over the last minute" />
        </CardContent>
      </Card>
    </div>
  {:else if !detailsQ.error}
    <PageLoading message="Measuring cores, network and processes..." />
  {/if}

  <section class="space-y-2">
    <h2 class={heading}>Storage</h2>
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
      <Card class="gap-0 py-2">
        <CardContent class="px-2">
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.Head>Volume</Table.Head>
                <Table.Head class="hidden md:table-cell">Label</Table.Head>
                <Table.Head class="w-full">Usage</Table.Head>
                <Table.Head class="text-right">%</Table.Head>
                <Table.Head class="hidden text-right sm:table-cell">Used / total</Table.Head>
                <Table.Head class="text-right">Free</Table.Head>
                {#if hasDriveTemps}
                  <Table.Head class="hidden text-right sm:table-cell">Temp</Table.Head>
                {/if}
              </Table.Row>
            </Table.Header>
            <Table.Body class="tabular-nums">
              {#each volumes as v (v.mountPoint)}
                {@const used = Math.max(0, v.totalBytes - v.availableBytes)}
                {@const pct = usedPct(used, v.totalBytes)}
                <Table.Row>
                  <Table.Cell class="font-medium">{v.mountPoint}</Table.Cell>
                  <Table.Cell class="hidden text-muted-foreground md:table-cell">
                    <div class="max-w-48 truncate" title="{v.name} ({v.fsKind})">
                      {v.name.trim() || '—'}
                      <span class="text-xs uppercase">{v.fsKind}</span>
                    </div>
                  </Table.Cell>
                  <Table.Cell>
                    <Progress
                      value={pct}
                      aria-label="{v.mountPoint} usage"
                      class="min-w-16 {pressureClass(pct)}"
                    />
                  </Table.Cell>
                  <Table.Cell class="text-right {pressureTextClass(pct)}">
                    {pct.toFixed(0)}%
                  </Table.Cell>
                  <Table.Cell class="hidden text-right text-muted-foreground sm:table-cell">
                    {fmtBytes(used)} / {fmtBytes(v.totalBytes)}
                  </Table.Cell>
                  <Table.Cell class="text-right">{fmtBytes(v.availableBytes)}</Table.Cell>
                  {#if hasDriveTemps}
                    <Table.Cell class="hidden text-right text-muted-foreground sm:table-cell">
                      {v.temperatureC != null ? `${v.temperatureC} °C` : '—'}
                    </Table.Cell>
                  {/if}
                </Table.Row>
              {/each}
            </Table.Body>
          </Table.Root>
        </CardContent>
      </Card>
    {/if}
  </section>

  {#if details?.measured && details.processes.length}
    <section class="space-y-2">
      <h2 class={heading}>Top processes</h2>
      <Card class="gap-0 py-2">
        <CardContent class="px-2">
          <Table.Root class="table-fixed">
            <Table.Header>
              <Table.Row>
                <Table.Head>Process</Table.Head>
                <Table.Head class="w-28 text-right">Memory</Table.Head>
                <Table.Head class="w-20 text-right">CPU</Table.Head>
              </Table.Row>
            </Table.Header>
            <Table.Body class="tabular-nums">
              {#each details.processes as p, i (i)}
                <Table.Row>
                  <Table.Cell class="truncate" title={p.name}>{p.name}</Table.Cell>
                  <Table.Cell class="text-right text-muted-foreground">
                    {fmtBytes(p.memoryBytes)}
                  </Table.Cell>
                  <Table.Cell class="text-right">{p.cpuPercent.toFixed(1)}%</Table.Cell>
                </Table.Row>
              {/each}
            </Table.Body>
          </Table.Root>
        </CardContent>
      </Card>
    </section>
  {/if}
</PageContainer>
