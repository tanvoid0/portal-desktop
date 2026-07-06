<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { AuditEntry, DiskUsage, Location } from "../types";
  import { fmtBytes, fmtDate } from "../utils";

  let { active, go }: { active: boolean; go: (tab: "cleanup" | "projects", path?: string) => void } =
    $props();

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

  const btnSecondary =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-neutral-900 text-neutral-200 text-sm font-medium border border-neutral-800 transition-colors hover:bg-neutral-800 hover:border-neutral-700 disabled:opacity-40 disabled:pointer-events-none";
  const btnGhostSm =
    "inline-flex items-center justify-center h-7 px-2.5 rounded-md text-xs font-medium text-neutral-300 border border-neutral-800 bg-neutral-950 transition-colors hover:bg-neutral-900 hover:text-white disabled:opacity-40 disabled:pointer-events-none";

  function usePct(d: DiskUsage) {
    const used = Math.max(0, d.totalBytes - d.availableBytes);
    return d.totalBytes > 0 ? Math.min(100, (used / d.totalBytes) * 100) : 0;
  }
</script>

<div class="mb-4 flex items-center gap-3">
  <p class="max-w-2xl text-sm text-neutral-500">
    Overview of your drives and cleanup history. Jump straight into a scan from any drive or quick
    action below.
  </p>
  <button onclick={() => void load()} disabled={loading} class="ml-auto {btnGhostSm}">
    {loading ? "Refreshing…" : "Refresh"}
  </button>
</div>

<div class="mb-2 text-xs uppercase tracking-wide text-neutral-600">Disk storage</div>
<div class="mb-6 grid grid-cols-1 gap-3 md:grid-cols-2">
  {#each disks as d (d.mountPoint)}
    {@const pct = usePct(d)}
    {@const used = Math.max(0, d.totalBytes - d.availableBytes)}
    <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
      <div class="mb-2 flex items-center gap-2">
        <span class="text-neutral-500">{d.isRemovable ? "▭" : "▮"}</span>
        <span class="truncate text-sm font-medium text-neutral-100" title={d.mountPoint}>{d.mountPoint}</span>
        <span class="truncate text-xs text-neutral-600">{d.name?.trim() || d.mountPoint}</span>
        <span class="ml-auto text-xs uppercase text-neutral-500">{d.fsKind}</span>
      </div>
      <div class="mb-2 h-2 w-full overflow-hidden rounded-full bg-neutral-900">
        <div
          class="h-full rounded-full {pct >= 90 ? 'bg-red-500' : pct >= 75 ? 'bg-amber-500' : 'bg-white'}"
          style="width: {pct}%"
        ></div>
      </div>
      <div class="flex items-center justify-between text-xs">
        <span class="tabular-nums text-neutral-400">{fmtBytes(used)} used · {fmtBytes(d.availableBytes)} free</span>
        <span class="tabular-nums text-neutral-600">{pct.toFixed(0)}% of {fmtBytes(d.totalBytes)}</span>
      </div>
      <div class="mt-3">
        <button onclick={() => go("cleanup", d.mountPoint)} class={btnGhostSm}>Scan this drive</button>
      </div>
    </div>
  {/each}
  {#if disks.length === 0}
    <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-3 py-8 text-center text-sm text-neutral-600">
      {loading ? "Reading drives…" : "No drives detected."}
    </div>
  {/if}
</div>

<div class="mb-2 text-xs uppercase tracking-wide text-neutral-600">Quick actions</div>
<div class="mb-6 flex flex-wrap gap-2">
  {#if quick.downloads}
    <button onclick={() => go("cleanup", quick.downloads!.path)} class={btnSecondary}>⬇ Clean Downloads</button>
  {/if}
  {#if quick.temp}
    <button onclick={() => go("cleanup", quick.temp!.path)} class={btnSecondary}>♻ Clear Temp files</button>
  {/if}
  <button onclick={() => go("projects", quick.home?.path)} class={btnSecondary}>⌘ Find project junk</button>
  <button onclick={() => void invoke("open_recycle_bin").catch(() => {})} class={btnSecondary}>🗑 Open Recycle Bin</button>
</div>

<div class="mb-2 text-xs uppercase tracking-wide text-neutral-600">Cleanup summary</div>
<div class="mb-4 grid grid-cols-1 gap-3 md:grid-cols-3">
  <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Lifetime reclaimed</div>
    <div class="text-2xl font-semibold tabular-nums tracking-tight text-emerald-400">{fmtBytes(reclaimed.total)}</div>
    <div class="mt-1 text-xs text-neutral-600">{reclaimed.movedN} item{reclaimed.movedN === 1 ? "" : "s"} moved</div>
  </div>
  <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Actions logged</div>
    <div class="text-2xl font-semibold tabular-nums tracking-tight text-white">{audit.length}</div>
    <div class="mt-1 text-xs text-neutral-600">{audit.length ? `last ${fmtDate(audit[0].ts)}` : "none yet"}</div>
  </div>
  <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Drives</div>
    <div class="text-2xl font-semibold tabular-nums tracking-tight text-white">{disks.length}</div>
    <div class="mt-1 text-xs text-neutral-600">{fmtBytes(freeTotal)} free total</div>
  </div>
</div>

<div class="mb-6 grid grid-cols-1 gap-3 md:grid-cols-2">
  <div class="overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
    <div class="border-b border-neutral-900 bg-neutral-900/40 px-4 py-3 text-sm font-medium text-neutral-200">
      Reclaimed by type
    </div>
    <div class="p-4">
      {#if reclaimed.segments.length === 0}
        <div class="py-8 text-center text-sm text-neutral-600">No cleanups yet. Reclaimed space will chart here.</div>
      {:else}
        <div class="flex items-center gap-5">
          <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
            <g transform={`rotate(-90 ${size / 2} ${size / 2})`}>
              <circle cx={size / 2} cy={size / 2} {r} fill="none" stroke="#171717" stroke-width={stroke} />
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
            <text x="50%" y="47%" text-anchor="middle" class="fill-white" style="font-size:18px;font-weight:600">
              {fmtBytes(reclaimed.total)}
            </text>
            <text x="50%" y="60%" text-anchor="middle" class="fill-neutral-500" style="font-size:10px;letter-spacing:0.05em">
              RECLAIMED
            </text>
          </svg>
          <div class="flex-1 space-y-1.5">
            {#each reclaimed.segments.slice(0, 8) as s (s.label)}
              <div class="flex items-center gap-2 text-xs">
                <span class="h-2.5 w-2.5 shrink-0 rounded-sm" style="background-color:{s.color}"></span>
                <span class="truncate text-neutral-300">{s.label}</span>
                <span class="ml-auto tabular-nums text-neutral-500">{fmtBytes(s.value)}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
    <div class="border-b border-neutral-900 bg-neutral-900/40 px-4 py-3 text-sm font-medium text-neutral-200">
      Recent activity
    </div>
    <div class="divide-y divide-neutral-900">
      {#each recent as e (e.id)}
        <div class="flex items-center gap-3 px-4 py-2.5 text-xs">
          <span class="h-1.5 w-1.5 shrink-0 rounded-full {e.status === 'moved' ? 'bg-emerald-400' : 'bg-red-400'}"></span>
          <span class="truncate font-mono text-neutral-400" title={e.path}>{e.path}</span>
          <span class="ml-auto whitespace-nowrap tabular-nums text-neutral-500">{fmtBytes(e.sizeBytes)}</span>
        </div>
      {/each}
      {#if recent.length === 0}
        <div class="px-4 py-8 text-center text-sm text-neutral-600">Nothing here yet. Quarantined items show up here.</div>
      {/if}
    </div>
  </div>
</div>
