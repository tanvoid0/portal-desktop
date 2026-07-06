<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Location, ProjectScan, QuarantineProgress, QuarantineResult, ScanProgress } from "../types";
  import { fmtBytes, fmtDuration, KIND_BADGE } from "../utils";

  let { pending }: { pending: { tab: string; path?: string; seq: number } | null } = $props();

  let root = $state("");
  let busy = $state(false);
  let progress = $state<ScanProgress | null>(null);
  let qProgress = $state<QuarantineProgress | null>(null);
  let scanResult = $state<ProjectScan | null>(null);
  let selected = $state<Set<string>>(new Set());
  let collapsed = $state<Set<string>>(new Set());
  let status = $state("");
  let locations = $state<Location[]>([]);

  const btnPrimary =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-white text-black text-sm font-medium transition-colors hover:bg-neutral-200 disabled:opacity-40 disabled:pointer-events-none";
  const btnSecondary =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-neutral-900 text-neutral-200 text-sm font-medium border border-neutral-800 transition-colors hover:bg-neutral-800 hover:border-neutral-700 disabled:opacity-40 disabled:pointer-events-none";
  const btnDanger =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-red-600 text-white text-sm font-medium transition-colors hover:bg-red-500 disabled:opacity-40 disabled:pointer-events-none";
  const btnGhostSm =
    "inline-flex items-center justify-center h-7 px-2.5 rounded-md text-xs font-medium text-neutral-300 border border-neutral-800 bg-neutral-950 transition-colors hover:bg-neutral-900 hover:text-white disabled:opacity-40 disabled:pointer-events-none";
  const inputCls =
    "h-9 px-3 rounded-md bg-neutral-950 border border-neutral-800 text-sm text-neutral-100 placeholder:text-neutral-600 transition-colors focus:outline-none focus:border-neutral-600";

  onMount(() => {
    invoke<Location[]>("list_locations").then((l) => (locations = l)).catch(() => (locations = []));
    const offs = [
      listen<ScanProgress>("scan://progress", (e) => (progress = e.payload)),
      listen<QuarantineProgress>("quarantine://progress", (e) => (qProgress = e.payload)),
    ];
    return () => offs.forEach((p) => p.then((off) => off()));
  });

  const allTemps = $derived(scanResult ? scanResult.projects.flatMap((p) => p.temps) : []);
  const selectedBytes = $derived(allTemps.filter((t) => selected.has(t.id)).reduce((a, t) => a + t.sizeBytes, 0));

  let jumpSeen = 0;
  $effect(() => {
    if (pending && pending.tab === "projects" && pending.path && pending.seq !== jumpSeen) {
      jumpSeen = pending.seq;
      root = pending.path;
      void runScan(pending.path);
    }
  });

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === "string") root = dir;
  }

  async function runScan(target?: string) {
    const dir = target ?? root;
    if (!dir) return;
    if (target) root = target;
    busy = true;
    status = "Scanning for projects…";
    progress = null;
    selected = new Set();
    try {
      const res = await invoke<ProjectScan>("scan_projects", { root: dir });
      scanResult = res;
      collapsed = new Set();
      status = `Found ${res.projectCount} project${res.projectCount === 1 ? "" : "s"} · ${fmtBytes(res.totalBytes)} reclaimable`;
    } catch (e) {
      status = String(e) === "cancelled" ? "Scan cancelled" : `Scan failed: ${String(e)}`;
    } finally {
      busy = false;
      progress = null;
    }
  }

  async function cancelScan() {
    await invoke("cancel_scan");
    status = "Cancelling…";
  }

  function toggleSel(ids: string[], on: boolean) {
    const next = new Set(selected);
    for (const id of ids) on ? next.add(id) : next.delete(id);
    selected = next;
  }
  function toggleCollapse(rootPath: string) {
    const next = new Set(collapsed);
    next.has(rootPath) ? next.delete(rootPath) : next.add(rootPath);
    collapsed = next;
  }
  function selectAll() {
    selected = new Set(allTemps.map((t) => t.id));
  }

  async function quarantine() {
    if (!scanResult || selected.size === 0) return;
    const items = allTemps.filter((t) => selected.has(t.id)).map((t) => ({ path: t.path, kind: t.tempKind }));
    const ok = window.confirm(
      `Move ${items.length} project temp dir(s) (${fmtBytes(selectedBytes)}) to the Recycle Bin?\n\nThese are regenerable (rebuilt by your toolchain). Nothing is permanently deleted — restore from the Recycle Bin.`,
    );
    if (!ok) return;
    busy = true;
    qProgress = { done: 0, total: items.length, currentPath: "" };
    status = "Moving to Recycle Bin…";
    try {
      const res = await invoke<QuarantineResult>("quarantine_paths", { items });
      const movedPaths = new Set(res.moved.map((m) => m.path));
      scanResult = {
        ...scanResult,
        projects: scanResult.projects
          .map((p) => ({ ...p, temps: p.temps.filter((t) => !movedPaths.has(t.path)) }))
          .filter((p) => p.temps.length > 0),
      };
      selected = new Set();
      status = `Reclaimed ${fmtBytes(res.reclaimedBytes)} · ${res.moved.length} moved · ${res.failed.length} failed`;
    } catch (e) {
      status = `Quarantine failed: ${String(e)}`;
    } finally {
      busy = false;
      qProgress = null;
    }
  }

  function projSel(temps: { id: string }[]) {
    const ids = temps.map((t) => t.id);
    const sel = ids.filter((id) => selected.has(id)).length;
    return { ids, all: sel === ids.length && ids.length > 0, some: sel > 0 && sel !== ids.length };
  }
  function rel(projectRoot: string, prefix: string) {
    return projectRoot.startsWith(prefix)
      ? projectRoot.slice(prefix.length).replace(/^[\\/]+/, "") || projectRoot
      : projectRoot;
  }
  const scanPct = $derived(
    progress && progress.phase !== "counting" && progress.totalFiles > 0
      ? Math.min(100, (progress.scannedFiles / progress.totalFiles) * 100)
      : 0,
  );
</script>

<p class="mb-4 max-w-2xl text-sm text-neutral-500">
  Finds project roots by their marker files (package.json, pom.xml, Cargo.toml, build.gradle,
  pyproject.toml, .csproj, go.mod, composer.json) and flags each project's regenerable build /
  dependency directories. All are safe — your toolchain rebuilds them.
</p>

<div class="mb-4 flex items-center gap-2">
  <button onclick={pickFolder} class={btnSecondary}>Choose folder</button>
  <input bind:value={root} placeholder="C:\Users\you\code" class="flex-1 {inputCls}" />
  <button onclick={() => runScan()} disabled={busy || !root} class={btnPrimary}>{busy ? "Working…" : "Scan projects"}</button>
  {#if busy}
    <button onclick={cancelScan} class={btnDanger}>Stop</button>
  {/if}
</div>

{#if !scanResult && locations.length > 0}
  <div class="mb-6">
    <div class="mb-2 text-xs uppercase tracking-wide text-neutral-600">Suggested — drives &amp; common folders</div>
    <div class="flex flex-wrap gap-2">
      {#each locations as loc (loc.path)}
        <button onclick={() => runScan(loc.path)} disabled={busy} title={loc.path}
          class="inline-flex h-9 items-center gap-2 rounded-md border border-neutral-800 bg-neutral-950 px-3 text-sm text-neutral-200 transition-colors hover:border-neutral-700 hover:bg-neutral-900 disabled:pointer-events-none disabled:opacity-40">
          <span class="text-neutral-500">{loc.kind === "drive" ? "▮" : "▸"}</span>{loc.label}
        </button>
      {/each}
    </div>
  </div>
{/if}

{#if busy && progress}
  {@const counting = progress.phase === "counting"}
  <div class="mb-4 rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-2 flex items-center justify-between text-sm">
      <span class="text-neutral-300">{counting ? "Counting files…" : "Scanning"}</span>
      <span class="tabular-nums text-neutral-400">{counting ? fmtDuration(progress.elapsedMs) : `${scanPct.toFixed(0)}%`}</span>
    </div>
    <div class="h-2 w-full overflow-hidden rounded-full bg-neutral-900">
      {#if counting}<div class="h-full w-1/3 animate-pulse rounded-full bg-white/60"></div>
      {:else}<div class="h-full rounded-full bg-white" style="width: {scanPct}%"></div>{/if}
    </div>
  </div>
{/if}

{#if qProgress}
  {@const qpct = qProgress.total > 0 ? Math.min(100, (qProgress.done / qProgress.total) * 100) : 0}
  <div class="mb-4 rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-2 flex items-center justify-between text-sm">
      <span class="text-neutral-300">Moving to Recycle Bin<span class="text-neutral-500"> · {qProgress.done} / {qProgress.total}</span></span>
      <span class="tabular-nums text-neutral-400">{qpct.toFixed(0)}%</span>
    </div>
    <div class="h-2 w-full overflow-hidden rounded-full bg-neutral-900"><div class="h-full rounded-full bg-white" style="width: {qpct}%"></div></div>
  </div>
{/if}

{#if status && !(busy && progress) && !qProgress}
  <div class="mb-4 text-sm text-neutral-500">{status}</div>
{/if}

{#if scanResult}
  <div class="mb-4 grid grid-cols-2 gap-3 md:grid-cols-3">
    <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Reclaimable</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight text-white">{fmtBytes(scanResult.totalBytes)}</div>
    </div>
    <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Projects</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight text-white">{scanResult.projectCount}</div>
    </div>
    <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Selected</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight text-white">{fmtBytes(selectedBytes)}</div>
      <div class="mt-1 text-xs text-neutral-600">{selected.size} dir{selected.size === 1 ? "" : "s"}</div>
    </div>
  </div>

  <div class="mb-3 flex items-center gap-2">
    <button onclick={selectAll} class={btnGhostSm}>Select all</button>
    <button onclick={() => (selected = new Set())} class={btnGhostSm}>Clear</button>
    <div class="ml-auto"></div>
    <button onclick={quarantine} disabled={busy || selected.size === 0} class="h-7 px-3 {btnDanger} text-xs">Move to Recycle Bin</button>
  </div>

  <div class="space-y-3">
    {#each scanResult.projects as p (p.root)}
      {@const sel = projSel(p.temps)}
      <div class="overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
        <div class="flex items-center gap-2.5 bg-neutral-900/40 px-3 py-2.5">
          <input type="checkbox" class="accent-white" checked={sel.all} indeterminate={sel.some} onchange={() => toggleSel(sel.ids, !sel.all)} />
          <button onclick={() => toggleCollapse(p.root)} class="w-4 shrink-0 text-neutral-500 hover:text-white">{collapsed.has(p.root) ? "▸" : "▾"}</button>
          <span class="break-all font-mono text-xs text-neutral-200" title={p.root}>{rel(p.root, scanResult.root)}</span>
          <span class="inline-flex gap-1">
            {#each p.kind.split("+") as k (k)}
              <span class="inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium {KIND_BADGE[k] ?? 'border-neutral-700 bg-neutral-800 text-neutral-300'}">{k}</span>
            {/each}
          </span>
          <div class="ml-auto flex items-center gap-3 whitespace-nowrap">
            <span class="text-xs text-neutral-500">{p.temps.length} dir{p.temps.length === 1 ? "" : "s"}</span>
            <span class="text-sm font-semibold tabular-nums text-white">{fmtBytes(p.totalBytes)}</span>
          </div>
        </div>
        {#if !collapsed.has(p.root)}
          <table class="w-full text-sm">
            <tbody>
              {#each p.temps as t (t.id)}
                <tr class="border-t border-neutral-900 transition-colors hover:bg-neutral-900/40">
                  <td class="w-8 px-3 py-2 align-middle">
                    <input type="checkbox" class="accent-white" checked={selected.has(t.id)} onchange={() => toggleSel([t.id], !selected.has(t.id))} />
                  </td>
                  <td class="break-all px-3 py-2 font-mono text-xs text-neutral-300"><span class="text-neutral-500">…\</span>{t.tempKind}</td>
                  <td class="whitespace-nowrap px-3 py-2 text-xs text-neutral-500">{t.fileCount.toLocaleString()} files</td>
                  <td class="whitespace-nowrap px-3 py-2 text-right text-neutral-300">{fmtBytes(t.sizeBytes)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {/each}
    {#if scanResult.projects.length === 0}
      <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-3 py-10 text-center text-neutral-600">No projects with regenerable directories found here.</div>
    {/if}
  </div>
{/if}
