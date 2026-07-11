<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { DevCleanerCleanProgress, DevCleanerCleanResult, Location, ProjectScan, ScanProgress } from "../types";
  import { fmtBytes, fmtDuration, KIND_BADGE } from "../utils";
  import { confirmAction } from "$lib/utils/confirm";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Progress } from "$lib/components/ui/progress";
  import {
    Table,
    TableBody,
    TableCell,
    TableRow,
  } from "$lib/components/ui/table";

  let { pending }: { pending: { tab: string; path?: string; seq: number } | null } = $props();

  let root = $state("");
  let busy = $state(false);
  let progress = $state<ScanProgress | null>(null);
  let qProgress = $state<DevCleanerCleanProgress | null>(null);
  let scanResult = $state<ProjectScan | null>(null);
  let selected = $state<Set<string>>(new Set());
  let collapsed = $state<Set<string>>(new Set());
  let status = $state("");
  let locations = $state<Location[]>([]);

  onMount(() => {
    invoke<Location[]>("list_locations").then((l) => (locations = l)).catch(() => (locations = []));
    const offs = [
      listen<ScanProgress>("scan://progress", (e) => (progress = e.payload)),
      listen<DevCleanerCleanProgress>("dev-clean://progress", (e) => (qProgress = e.payload)),
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
    const items = allTemps.filter((t) => selected.has(t.id)).map((t) => ({
      cleanerId: "projects",
      id: t.id,
      path: t.path,
      kind: t.tempKind,
    }));
    const ok = await confirmAction(
      `Move ${items.length} project temp dir(s) (${fmtBytes(selectedBytes)}) to the Recycle Bin?\n\nThese are regenerable (rebuilt by your toolchain). Nothing is permanently deleted — restore from the Recycle Bin.`,
      "Move to Recycle Bin",
      { confirmLabel: "Move" },
    );
    if (!ok) return;
    busy = true;
    qProgress = { done: 0, total: items.length, currentPath: "" };
    status = "Moving to Recycle Bin…";
    try {
      const res = await invoke<DevCleanerCleanResult>("clean_dev_items", { items });
      const movedPaths = new Set(res.cleaned.map((m) => m.path));
      scanResult = {
        ...scanResult,
        projects: scanResult.projects
          .map((p) => ({ ...p, temps: p.temps.filter((t) => !movedPaths.has(t.path)) }))
          .filter((p) => p.temps.length > 0),
      };
      selected = new Set();
      status = `Reclaimed ${fmtBytes(res.reclaimedBytes)} · ${res.cleaned.length} moved · ${res.failed.length} failed`;
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

<p class="mb-4 max-w-2xl text-sm text-muted-foreground">
  Finds project roots by their marker files (package.json, pom.xml, Cargo.toml, build.gradle,
  pyproject.toml, .csproj, go.mod, composer.json) and flags each project's regenerable build /
  dependency directories. All are safe — your toolchain rebuilds them.
</p>

<div class="mb-4 flex items-center gap-2">
  <Button variant="outline" onclick={pickFolder}>Choose folder</Button>
  <Input bind:value={root} placeholder="C:\Users\you\code" class="flex-1" />
  <Button onclick={() => runScan()} disabled={busy || !root}>{busy ? "Working…" : "Scan projects"}</Button>
  {#if busy}
    <Button variant="destructive" onclick={cancelScan}>Stop</Button>
  {/if}
</div>

{#if !scanResult && locations.length > 0}
  <div class="mb-6">
    <div class="mb-2 text-xs uppercase tracking-wide text-muted-foreground">Suggested — drives &amp; common folders</div>
    <div class="flex flex-wrap gap-2">
      {#each locations as loc (loc.path)}
        <Button variant="outline" onclick={() => runScan(loc.path)} disabled={busy} title={loc.path}>
          <span class="text-muted-foreground">{loc.kind === "drive" ? "▮" : "▸"}</span>
          {loc.label}
        </Button>
      {/each}
    </div>
  </div>
{/if}

{#if busy && progress}
  {@const counting = progress.phase === "counting"}
  <Card class="mb-4 gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-2 flex items-center justify-between text-sm">
        <span class="text-foreground">{counting ? "Counting files…" : "Scanning"}</span>
        <span class="tabular-nums text-muted-foreground">{counting ? fmtDuration(progress.elapsedMs) : `${scanPct.toFixed(0)}%`}</span>
      </div>
      {#if counting}
        <Progress value={33} class="h-2 animate-pulse" />
      {:else}
        <Progress value={scanPct} class="h-2" />
      {/if}
    </CardContent>
  </Card>
{/if}

{#if qProgress}
  {@const qpct = qProgress.total > 0 ? Math.min(100, (qProgress.done / qProgress.total) * 100) : 0}
  <Card class="mb-4 gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-2 flex items-center justify-between text-sm">
        <span class="text-foreground">Moving to Recycle Bin<span class="text-muted-foreground"> · {qProgress.done} / {qProgress.total}</span></span>
        <span class="tabular-nums text-muted-foreground">{qpct.toFixed(0)}%</span>
      </div>
      <Progress value={qpct} class="h-2" />
    </CardContent>
  </Card>
{/if}

{#if status && !(busy && progress) && !qProgress}
  <div class="mb-4 text-sm text-muted-foreground">{status}</div>
{/if}

{#if scanResult}
  <div class="mb-4 grid grid-cols-2 gap-3 md:grid-cols-3">
    <Card class="gap-0 py-4">
      <CardContent class="px-4">
        <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Reclaimable</div>
        <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">{fmtBytes(scanResult.totalBytes)}</div>
      </CardContent>
    </Card>
    <Card class="gap-0 py-4">
      <CardContent class="px-4">
        <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Projects</div>
        <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">{scanResult.projectCount}</div>
      </CardContent>
    </Card>
    <Card class="gap-0 py-4">
      <CardContent class="px-4">
        <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Selected</div>
        <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">{fmtBytes(selectedBytes)}</div>
        <div class="mt-1 text-xs text-muted-foreground">{selected.size} dir{selected.size === 1 ? "" : "s"}</div>
      </CardContent>
    </Card>
  </div>

  <div class="mb-3 flex items-center gap-2">
    <Button variant="outline" size="sm" onclick={selectAll}>Select all</Button>
    <Button variant="outline" size="sm" onclick={() => (selected = new Set())}>Clear</Button>
    <div class="ml-auto"></div>
    <Button variant="destructive" size="sm" onclick={quarantine} disabled={busy || selected.size === 0}>Move to Recycle Bin</Button>
  </div>

  <div class="space-y-3">
    {#each scanResult.projects as p (p.root)}
      {@const sel = projSel(p.temps)}
      <Card class="gap-0 overflow-hidden py-0">
        <div class="divider-edge-b divider-edge-full flex items-center gap-2.5 bg-muted/40 px-3 py-2.5">
          <Checkbox
            checked={sel.all}
            indeterminate={sel.some}
            onCheckedChange={() => toggleSel(sel.ids, !sel.all)}
          />
          <Button type="button" variant="ghost" size="icon-sm" class="h-4 w-4 shrink-0 p-0 text-muted-foreground" onclick={() => toggleCollapse(p.root)}>{collapsed.has(p.root) ? "▸" : "▾"}</Button>
          <span class="break-all font-mono text-xs text-foreground" title={p.root}>{rel(p.root, scanResult.root)}</span>
          <span class="inline-flex gap-1">
            {#each p.kind.split("+") as k (k)}
              <Badge variant="outline" class={KIND_BADGE[k] ?? ""}>{k}</Badge>
            {/each}
          </span>
          <div class="ml-auto flex items-center gap-3 whitespace-nowrap">
            <span class="text-xs text-muted-foreground">{p.temps.length} dir{p.temps.length === 1 ? "" : "s"}</span>
            <span class="text-sm font-semibold tabular-nums text-foreground">{fmtBytes(p.totalBytes)}</span>
          </div>
        </div>
        {#if !collapsed.has(p.root)}
          <Table>
            <TableBody>
              {#each p.temps as t (t.id)}
                <TableRow>
                  <TableCell class="w-8">
                    <Checkbox
                      checked={selected.has(t.id)}
                      onCheckedChange={() => toggleSel([t.id], !selected.has(t.id))}
                    />
                  </TableCell>
                  <TableCell class="break-all font-mono text-xs"><span class="text-muted-foreground">…\</span>{t.tempKind}</TableCell>
                  <TableCell class="whitespace-nowrap text-xs text-muted-foreground">{t.fileCount.toLocaleString()} files</TableCell>
                  <TableCell class="whitespace-nowrap text-right">{fmtBytes(t.sizeBytes)}</TableCell>
                </TableRow>
              {/each}
            </TableBody>
          </Table>
        {/if}
      </Card>
    {/each}
    {#if scanResult.projects.length === 0}
      <Card>
        <CardContent class="py-10 text-center text-muted-foreground">
          No projects with regenerable directories found here.
        </CardContent>
      </Card>
    {/if}
  </div>
{/if}
