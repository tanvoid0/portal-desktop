<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { AuditEntry, DiskUsage, Location } from "../types";
  import { fmtBytes, fmtDate } from "../utils";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { HardDrive, Usb, RefreshCw, Download, Recycle, Boxes, Trash2, Container } from "@lucide/svelte";

  let { active, go }: {
    active: boolean;
    go: (tab: "cleanup" | "projects" | "devtools", path?: string) => void;
  } = $props();

  const PALETTE = ["#34d399", "#60a5fa", "#f59e0b", "#a78bfa", "#f472b6", "#22d3ee", "#fb7185", "#a3e635"];

  let disks = $state<DiskUsage[]>([]);
  let audit = $state<AuditEntry[]>([]);
  let locations = $state<Location[]>([]);
  let loading = $state(false);

  async function load() {
    loading = true;
    try {
      const [d, a, l] = await Promise.all([
        invoke<DiskUsage[]>("disk_usage").catch(() => [] as DiskUsage[]),
        invoke<AuditEntry[]>("get_audit_log").catch(() => [] as AuditEntry[]),
        invoke<Location[]>("list_locations").catch(() => [] as Location[]),
      ]);
      disks = d;
      audit = a;
      locations = l;
    } finally {
      loading = false;
    }
  }

  let lastActive = false;
  $effect(() => {
    if (active && !lastActive) void load();
    lastActive = active;
  });

  const reclaimed = $derived.by(() => {
    const moved = audit.filter((e) => e.status === "moved");
    const total = moved.reduce((a, e) => a + e.sizeBytes, 0);
    const byKind = new Map<string, number>();
    for (const e of moved) byKind.set(e.kind, (byKind.get(e.kind) ?? 0) + e.sizeBytes);
    const segments = [...byKind.entries()]
      .sort((a, b) => b[1] - a[1])
      .map(([label, value], i) => ({ label, value, color: PALETTE[i % PALETTE.length] }));
    return { total, movedN: moved.length, segments };
  });

  const quick = $derived.by(() => {
    const byLabel = (l: string) => locations.find((x) => x.label.toLowerCase() === l.toLowerCase());
    return {
      downloads: byLabel("Downloads"),
      temp: byLabel("Temp") ?? locations.find((x) => /temp/i.test(x.path)),
      home: locations.find((x) => x.kind === "folder"),
    };
  });

  const recent = $derived(audit.slice(0, 6));
  const freeTotal = $derived(disks.reduce((a, d) => a + d.availableBytes, 0));

  // Donut geometry.
  const size = 160;
  const stroke = 22;
  const r = (size - stroke) / 2;
  const circ = 2 * Math.PI * r;
  function dashArray(value: number, total: number) {
    const len = (total > 0 ? value / total : 0) * circ;
    return `${len} ${circ - len}`;
  }
  function dashOffset(idx: number) {
    let off = 0;
    for (let i = 0; i < idx; i++) off += (reclaimed.total > 0 ? reclaimed.segments[i].value / reclaimed.total : 0) * circ;
    return -off;
  }

  function usePct(d: DiskUsage) {
    const used = Math.max(0, d.totalBytes - d.availableBytes);
    return d.totalBytes > 0 ? Math.min(100, (used / d.totalBytes) * 100) : 0;
  }
</script>

<div class="mb-4 flex items-center gap-3">
  <p class="max-w-2xl text-sm text-muted-foreground">
    Overview of your drives and cleanup history. Jump straight into a scan from any drive or quick
    action below.
  </p>
  <Button variant="outline" size="sm" onclick={() => void load()} disabled={loading} class="ml-auto">
    <RefreshCw class="size-3.5 {loading ? 'animate-spin' : ''}" />
    {loading ? "Refreshing…" : "Refresh"}
  </Button>
</div>

<div class="mb-2 text-xs uppercase tracking-wide text-muted-foreground">Disk storage</div>
<div class="mb-6 grid grid-cols-1 gap-3 md:grid-cols-2">
  {#each disks as d (d.mountPoint)}
    {@const pct = usePct(d)}
    {@const used = Math.max(0, d.totalBytes - d.availableBytes)}
    <Card class="gap-0 py-4">
      <CardContent class="px-4">
        <div class="mb-2 flex items-center gap-2">
          {#if d.isRemovable}
            <Usb class="size-4 text-muted-foreground" />
          {:else}
            <HardDrive class="size-4 text-muted-foreground" />
          {/if}
          <span class="truncate text-sm font-medium text-foreground" title={d.mountPoint}>{d.mountPoint}</span>
          <span class="truncate text-xs text-muted-foreground">{d.name?.trim() || d.mountPoint}</span>
          <span class="ml-auto text-xs uppercase text-muted-foreground">{d.fsKind}</span>
        </div>
        <Progress
          value={pct}
          class="mb-2 h-2 {pct >= 90 ? '[&>[data-slot=progress-indicator]]:bg-status-error' : pct >= 75 ? '[&>[data-slot=progress-indicator]]:bg-status-warning' : ''}"
        />
        <div class="flex items-center justify-between text-xs">
          <span class="tabular-nums text-muted-foreground">{fmtBytes(used)} used · {fmtBytes(d.availableBytes)} free</span>
          <span class="tabular-nums text-muted-foreground/70">{pct.toFixed(0)}% of {fmtBytes(d.totalBytes)}</span>
        </div>
        <div class="mt-3">
          <Button variant="outline" size="sm" onclick={() => go("cleanup", d.mountPoint)}>Scan this drive</Button>
        </div>
      </CardContent>
    </Card>
  {/each}
  {#if disks.length === 0}
    <Card class="md:col-span-2">
      <CardContent class="py-8 text-center text-sm text-muted-foreground">
        {loading ? "Reading drives…" : "No drives detected."}
      </CardContent>
    </Card>
  {/if}
</div>

<div class="mb-2 text-xs uppercase tracking-wide text-muted-foreground">Quick actions</div>
<div class="mb-6 flex flex-wrap gap-2">
  {#if quick.downloads}
    <Button variant="secondary" onclick={() => go("cleanup", quick.downloads!.path)}>
      <Download class="size-4" /> Clean Downloads
    </Button>
  {/if}
  {#if quick.temp}
    <Button variant="secondary" onclick={() => go("cleanup", quick.temp!.path)}>
      <Recycle class="size-4" /> Clear Temp files
    </Button>
  {/if}
  <Button variant="secondary" onclick={() => go("projects", quick.home?.path)}>
    <Boxes class="size-4" /> Find project junk
  </Button>
  <Button variant="secondary" onclick={() => go("devtools")}>
    <Container class="size-4" /> Docker / Podman
  </Button>
  <Button variant="secondary" onclick={() => void invoke("open_recycle_bin").catch(() => {})}>
    <Trash2 class="size-4" /> Open Recycle Bin
  </Button>
</div>

<div class="mb-2 text-xs uppercase tracking-wide text-muted-foreground">Cleanup summary</div>
<div class="mb-4 grid grid-cols-1 gap-3 md:grid-cols-3">
  <Card class="gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Lifetime reclaimed</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight text-status-success">{fmtBytes(reclaimed.total)}</div>
      <div class="mt-1 text-xs text-muted-foreground">{reclaimed.movedN} item{reclaimed.movedN === 1 ? "" : "s"} moved</div>
    </CardContent>
  </Card>
  <Card class="gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Actions logged</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">{audit.length}</div>
      <div class="mt-1 text-xs text-muted-foreground">{audit.length ? `last ${fmtDate(audit[0].ts)}` : "none yet"}</div>
    </CardContent>
  </Card>
  <Card class="gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Drives</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">{disks.length}</div>
      <div class="mt-1 text-xs text-muted-foreground">{fmtBytes(freeTotal)} free total</div>
    </CardContent>
  </Card>
</div>

<div class="mb-6 grid grid-cols-1 gap-3 md:grid-cols-2">
  <Card class="gap-0 overflow-hidden py-0">
    <CardHeader class="border-b bg-muted/40 px-4 py-3 [.border-b]:pb-3">
      <CardTitle class="text-sm font-medium">Reclaimed by type</CardTitle>
    </CardHeader>
    <CardContent class="p-4">
      {#if reclaimed.segments.length === 0}
        <div class="py-8 text-center text-sm text-muted-foreground">No cleanups yet. Reclaimed space will chart here.</div>
      {:else}
        <div class="flex items-center gap-5">
          <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
            <g transform={`rotate(-90 ${size / 2} ${size / 2})`}>
              <circle cx={size / 2} cy={size / 2} {r} fill="none" class="stroke-muted" stroke-width={stroke} />
              {#each reclaimed.segments as s, i (s.label)}
                <circle
                  cx={size / 2}
                  cy={size / 2}
                  {r}
                  fill="none"
                  stroke={s.color}
                  stroke-width={stroke}
                  stroke-dasharray={dashArray(s.value, reclaimed.total)}
                  stroke-dashoffset={dashOffset(i)}
                />
              {/each}
            </g>
            <text x="50%" y="47%" text-anchor="middle" class="fill-foreground" style="font-size:18px;font-weight:600">
              {fmtBytes(reclaimed.total)}
            </text>
            <text x="50%" y="60%" text-anchor="middle" class="fill-muted-foreground" style="font-size:10px;letter-spacing:0.05em">
              RECLAIMED
            </text>
          </svg>
          <div class="flex-1 space-y-1.5">
            {#each reclaimed.segments.slice(0, 8) as s (s.label)}
              <div class="flex items-center gap-2 text-xs">
                <span class="h-2.5 w-2.5 shrink-0 rounded-sm" style="background-color:{s.color}"></span>
                <span class="truncate text-foreground">{s.label}</span>
                <span class="ml-auto tabular-nums text-muted-foreground">{fmtBytes(s.value)}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </CardContent>
  </Card>

  <Card class="gap-0 overflow-hidden py-0">
    <CardHeader class="border-b bg-muted/40 px-4 py-3 [.border-b]:pb-3">
      <CardTitle class="text-sm font-medium">Recent activity</CardTitle>
    </CardHeader>
    <CardContent class="p-0">
      <div class="divide-y divide-border">
        {#each recent as e (e.id)}
          <div class="flex items-center gap-3 px-4 py-2.5 text-xs">
            <span class="h-1.5 w-1.5 shrink-0 rounded-full {e.status === 'moved' ? 'bg-status-success' : 'bg-status-error'}"></span>
            <span class="truncate font-mono text-muted-foreground" title={e.path}>{e.path}</span>
            <span class="ml-auto whitespace-nowrap tabular-nums text-muted-foreground">{fmtBytes(e.sizeBytes)}</span>
          </div>
        {/each}
        {#if recent.length === 0}
          <div class="px-4 py-8 text-center text-sm text-muted-foreground">Nothing here yet. Quarantined items show up here.</div>
        {/if}
      </div>
    </CardContent>
  </Card>
</div>
