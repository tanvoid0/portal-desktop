<script lang="ts">
  import type { DockerContainer } from "../types";
  import {
    fmtBytes,
    fmtPercent,
    isContainerRunning,
    shortImageName,
  } from "../utils/format";
  import DonutChart from "./DonutChart.svelte";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import { Cpu, MemoryStick, Network } from "@lucide/svelte";

  let { containers }: { containers: DockerContainer[] } = $props();

  const PALETTE = [
    "#34d399",
    "#60a5fa",
    "#f59e0b",
    "#a78bfa",
    "#f472b6",
    "#22d3ee",
    "#fb7185",
    "#a3e635",
  ];

  const stats = $derived.by(() => {
    const running = containers.filter((c) => isContainerRunning(c.status));
    const stopped = containers.filter(
      (c) =>
        !isContainerRunning(c.status) &&
        (c.status?.toLowerCase().includes("exited") ||
          c.status?.toLowerCase().includes("stopped") ||
          c.status?.toLowerCase().includes("created")),
    );
    const other = containers.length - running.length - stopped.length;

    const totalCpu = running.reduce(
      (sum, c) => sum + (c.resourceStats?.cpuPercent ?? 0),
      0,
    );
    const totalMem = running.reduce(
      (sum, c) => sum + (c.resourceStats?.memoryBytes ?? 0),
      0,
    );
    const totalNetRx = running.reduce(
      (sum, c) => sum + (c.resourceStats?.networkRxBytes ?? 0),
      0,
    );
    const totalNetTx = running.reduce(
      (sum, c) => sum + (c.resourceStats?.networkTxBytes ?? 0),
      0,
    );

    const imageCounts = new Map<string, number>();
    for (const c of containers) {
      const img = shortImageName(c.image);
      imageCounts.set(img, (imageCounts.get(img) ?? 0) + 1);
    }
    const topImages = [...imageCounts.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 6)
      .map(([label, value], i) => ({
        label,
        value,
        color: PALETTE[i % PALETTE.length],
      }));

    const topMemory = running
      .filter((c) => c.resourceStats)
      .sort(
        (a, b) =>
          (b.resourceStats?.memoryBytes ?? 0) -
          (a.resourceStats?.memoryBytes ?? 0),
      )
      .slice(0, 5);

    const topCpu = running
      .filter((c) => c.resourceStats)
      .sort(
        (a, b) =>
          (b.resourceStats?.cpuPercent ?? 0) -
          (a.resourceStats?.cpuPercent ?? 0),
      )
      .slice(0, 5);

    return {
      total: containers.length,
      running: running.length,
      stopped: stopped.length,
      other,
      totalCpu,
      totalMem,
      totalNetRx,
      totalNetTx,
      topImages,
      topMemory,
      topCpu,
      statusSegments: [
        { label: "Running", value: running.length, color: "#34d399" },
        { label: "Stopped", value: stopped.length, color: "#94a3b8" },
        ...(other > 0
          ? [{ label: "Other", value: other, color: "#f59e0b" }]
          : []),
      ],
    };
  });

  const maxMem = $derived(
    Math.max(...stats.topMemory.map((c) => c.resourceStats?.memoryBytes ?? 0), 1),
  );
  const maxCpu = $derived(
    Math.max(...stats.topCpu.map((c) => c.resourceStats?.cpuPercent ?? 0), 1),
  );
</script>

{#if containers.length > 0}
  <!-- Summary stat cards -->
  <div class="grid gap-4 md:grid-cols-4">
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Total</CardTitle>
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">{stats.total}</div>
        <p class="text-xs text-muted-foreground">containers on host</p>
      </CardContent>
    </Card>
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">CPU (running)</CardTitle>
        <Cpu class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold tabular-nums">
          {fmtPercent(stats.totalCpu)}
        </div>
        <p class="text-xs text-muted-foreground">aggregate usage</p>
      </CardContent>
    </Card>
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Memory (running)</CardTitle>
        <MemoryStick class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold tabular-nums">
          {fmtBytes(stats.totalMem)}
        </div>
        <p class="text-xs text-muted-foreground">in use</p>
      </CardContent>
    </Card>
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Network I/O</CardTitle>
        <Network class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-sm font-semibold tabular-nums">
          ↓ {fmtBytes(stats.totalNetRx)} · ↑ {fmtBytes(stats.totalNetTx)}
        </div>
        <p class="text-xs text-muted-foreground">running containers</p>
      </CardContent>
    </Card>
  </div>

  <!-- Charts row -->
  <div class="grid gap-4 lg:grid-cols-3">
    <Card>
      <CardHeader class="pb-2">
        <CardTitle class="text-sm font-medium">Status breakdown</CardTitle>
      </CardHeader>
      <CardContent>
        <DonutChart
          segments={stats.statusSegments}
          total={stats.total}
          centerLabel={String(stats.total)}
          centerSubLabel="CONTAINERS"
        />
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="pb-2">
        <CardTitle class="text-sm font-medium">Images in use</CardTitle>
      </CardHeader>
      <CardContent>
        {#if stats.topImages.length > 0}
          <DonutChart
            segments={stats.topImages}
            total={stats.total}
            centerLabel={String(stats.topImages.length)}
            centerSubLabel="IMAGES"
          />
        {:else}
          <p class="py-8 text-center text-sm text-muted-foreground">
            No images to show
          </p>
        {/if}
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="pb-2">
        <CardTitle class="text-sm font-medium">Top resource consumers</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        {#if stats.topMemory.length === 0 && stats.topCpu.length === 0}
          <p class="py-4 text-center text-sm text-muted-foreground">
            Start containers to see live resource usage
          </p>
        {:else}
          <div class="space-y-3">
            <div class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              Memory
            </div>
            {#each stats.topMemory as c (c.id)}
              {@const mem = c.resourceStats?.memoryBytes ?? 0}
              {@const pct = c.resourceStats?.memoryPercent ?? 0}
              <div class="space-y-1">
                <div class="flex items-center justify-between text-xs">
                  <span class="truncate font-medium" title={c.name}
                    >{c.name}</span
                  >
                  <span class="tabular-nums text-muted-foreground"
                    >{fmtBytes(mem)} · {fmtPercent(pct)}</span
                  >
                </div>
                <Progress
                  value={(mem / maxMem) * 100}
                  class="h-1.5 [&>[data-slot=progress-indicator]]:bg-blue-500"
                />
              </div>
            {/each}
          </div>

          {#if stats.topCpu.length > 0}
            <div class="space-y-3">
              <div class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                CPU
              </div>
              {#each stats.topCpu as c (c.id)}
                {@const cpu = c.resourceStats?.cpuPercent ?? 0}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="truncate font-medium" title={c.name}
                      >{c.name}</span
                    >
                    <span class="tabular-nums text-muted-foreground"
                      >{fmtPercent(cpu)}</span
                    >
                  </div>
                  <Progress
                    value={(cpu / maxCpu) * 100}
                    class="h-1.5 [&>[data-slot=progress-indicator]]:bg-emerald-500"
                  />
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </CardContent>
    </Card>
  </div>
{/if}
