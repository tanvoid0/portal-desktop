<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import type {
    DevCleanerCleanProgress,
    DevCleanerCleanResult,
    DevCleanerItem,
    DevCleanerScan,
    Location,
    ScanProgress,
  } from "../types";
  import { fmtBytes, fmtDuration, KIND_BADGE, RISK_BADGE } from "../utils";
  import { Button } from "$lib/components/ui/button";
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

  let busy = $state(false);
  let cleanProgress = $state<DevCleanerCleanProgress | null>(null);
  let scanProgress = $state<ScanProgress | null>(null);
  let scanResult = $state<DevCleanerScan | null>(null);
  let selected = $state<Set<string>>(new Set());
  let collapsed = $state<Set<string>>(new Set());
  let subCollapsed = $state<Set<string>>(new Set());
  let status = $state("");
  let locations = $state<Location[]>([]);
  let scanRoots = $state<Set<string>>(new Set());

  onMount(() => {
    invoke<Location[]>("list_locations").then((l) => (locations = l)).catch(() => (locations = []));
    const offs = [
      listen<DevCleanerCleanProgress>("dev-clean://progress", (e) => (cleanProgress = e.payload)),
      listen<ScanProgress>("scan://progress", (e) => (scanProgress = e.payload)),
    ];
    return () => offs.forEach((p) => p.then((off) => off()));
  });

  const allItems = $derived(scanResult ? scanResult.groups.flatMap((g) => g.items) : []);
  const selectedBytes = $derived(
    allItems.filter((i) => selected.has(i.id)).reduce((a, i) => a + i.sizeBytes, 0),
  );

  let jumpSeen = 0;
  $effect(() => {
    if (pending && pending.tab === "devtools" && pending.seq !== jumpSeen) {
      jumpSeen = pending.seq;
      if (pending.path) scanRoots = new Set([pending.path]);
      void runScan();
    }
  });

  function toggleRoot(path: string) {
    const next = new Set(scanRoots);
    next.has(path) ? next.delete(path) : next.add(path);
    scanRoots = next;
  }

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === "string") scanRoots = new Set([...scanRoots, dir]);
  }

  function projectSubgroups(items: DevCleanerItem[]) {
    const map = new Map<string, DevCleanerItem[]>();
    for (const item of items) {
      const key = item.groupLabel ?? item.path;
      const list = map.get(key) ?? [];
      list.push(item);
      map.set(key, list);
    }
    return [...map.entries()]
      .map(([label, subItems]) => ({
        label,
        items: subItems,
        bytes: subItems.reduce((a, i) => a + i.sizeBytes, 0),
      }))
      .sort((a, b) => b.bytes - a.bytes);
  }

  async function runScan() {
    busy = true;
    status = "Scanning dev tools…";
    scanProgress = null;
    selected = new Set();
    const roots = [...scanRoots];
    try {
      const res = await invoke<DevCleanerScan>("scan_dev_cleaners", { roots: roots.length ? roots : null });
      scanResult = res;
      collapsed = new Set();
      subCollapsed = new Set();
      const available = res.groups.filter((g) => g.available).length;
      status =
        res.itemCount === 0
          ? `No reclaimable items · ${available} cleaner${available === 1 ? "" : "s"} checked`
          : `Found ${res.itemCount} item${res.itemCount === 1 ? "" : "s"} · ${fmtBytes(res.totalBytes)} reclaimable`;
    } catch (e) {
      status = String(e) === "cancelled" ? "Scan cancelled" : `Scan failed: ${String(e)}`;
    } finally {
      busy = false;
      scanProgress = null;
    }
  }

  async function cancelScan() {
    await invoke("cancel_scan");
    status = "Cancelling…";
  }

  const scanPct = $derived(
    scanProgress && scanProgress.phase !== "counting" && scanProgress.totalFiles > 0
      ? Math.min(100, (scanProgress.scannedFiles / scanProgress.totalFiles) * 100)
      : 0,
  );

  function toggleSel(ids: string[], on: boolean) {
    const next = new Set(selected);
    for (const id of ids) on ? next.add(id) : next.delete(id);
    selected = next;
  }

  function toggleCollapse(groupId: string) {
    const next = new Set(collapsed);
    next.has(groupId) ? next.delete(groupId) : next.add(groupId);
    collapsed = next;
  }

  function selectAll() {
    selected = new Set(allItems.map((i) => i.id));
  }

  function groupSel(items: DevCleanerItem[]) {
    const ids = items.map((i) => i.id);
    const sel = ids.filter((id) => selected.has(id)).length;
    return { ids, all: sel === ids.length && ids.length > 0, some: sel > 0 && sel !== ids.length };
  }

  function kindLabel(kind: string): string {
    return kind.replace(/-/g, " ");
  }

  async function clean() {
    if (!scanResult || selected.size === 0) return;
    const items = allItems
      .filter((i) => selected.has(i.id))
      .map((i) => ({ cleanerId: i.cleanerId, id: i.id, path: i.path, kind: i.kind }));
    const ok = window.confirm(
      `Remove ${items.length} selected item(s) (${fmtBytes(selectedBytes)})?\n\nProject dirs go to the Recycle Bin. Container resources are removed via Docker/Podman. Review "Review" items carefully.`,
    );
    if (!ok) return;
    busy = true;
    cleanProgress = { done: 0, total: items.length, currentPath: "" };
    status = "Cleaning…";
    try {
      const res = await invoke<DevCleanerCleanResult>("clean_dev_items", { items });
      const cleanedPaths = new Set(res.cleaned.map((c) => c.path));
      scanResult = {
        ...scanResult,
        groups: scanResult.groups
          .map((g) => ({
            ...g,
            items: g.items.filter((i) => !cleanedPaths.has(i.path)),
            totalBytes: g.items
              .filter((i) => !cleanedPaths.has(i.path))
              .reduce((a, i) => a + i.sizeBytes, 0),
          }))
          .map((g) => ({ ...g, totalBytes: g.items.reduce((a, i) => a + i.sizeBytes, 0) })),
        itemCount: scanResult.groups.reduce(
          (n, g) => n + g.items.filter((i) => !cleanedPaths.has(i.path)).length,
          0,
        ),
        totalBytes: scanResult.totalBytes - res.reclaimedBytes,
      };
      selected = new Set();
      status = `Reclaimed ${fmtBytes(res.reclaimedBytes)} · ${res.cleaned.length} removed · ${res.failed.length} failed`;
    } catch (e) {
      status = `Clean failed: ${String(e)}`;
    } finally {
      busy = false;
      cleanProgress = null;
    }
  }
</script>

<p class="mb-4 max-w-2xl text-sm text-muted-foreground">
  Scans dev tools for reclaimable space — Docker, Podman, and optionally project build junk on selected
  drives or folders. Review every item before removal.
</p>

<div class="mb-4">
  <div class="mb-2 text-xs uppercase tracking-wide text-muted-foreground">Project scan roots (optional)</div>
  <div class="mb-2 flex flex-wrap gap-2">
    {#each locations.filter((l) => l.kind === "drive" || l.label === "Home") as loc (loc.path)}
      <Button
        variant={scanRoots.has(loc.path) ? "default" : "outline"}
        size="sm"
        onclick={() => toggleRoot(loc.path)}
        disabled={busy}
        title={loc.path}
      >
        {loc.label}
      </Button>
    {/each}
    <Button variant="outline" size="sm" onclick={pickFolder} disabled={busy}>Choose folder</Button>
  </div>
  {#if scanRoots.size > 0}
    <div class="text-xs text-muted-foreground">
      {scanRoots.size} root{scanRoots.size === 1 ? "" : "s"} selected for project scan
    </div>
  {/if}
</div>

<div class="mb-4 flex items-center gap-2">
  <Button onclick={() => runScan()} disabled={busy}>{busy ? "Working…" : "Scan dev tools"}</Button>
  {#if busy && scanRoots.size > 0}
    <Button variant="destructive" onclick={cancelScan}>Stop</Button>
  {/if}
</div>

{#if busy && scanProgress}
  {@const counting = scanProgress.phase === "counting"}
  <Card class="mb-4 gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-2 flex items-center justify-between text-sm">
        <span class="text-foreground">{counting ? "Counting files…" : "Scanning projects"}</span>
        <span class="tabular-nums text-muted-foreground">{counting ? fmtDuration(scanProgress.elapsedMs) : `${scanPct.toFixed(0)}%`}</span>
      </div>
      <Progress value={counting ? 33 : scanPct} class="h-2 {counting ? 'animate-pulse' : ''}" />
    </CardContent>
  </Card>
{/if}

{#if cleanProgress}
  {@const pct = cleanProgress.total > 0 ? Math.min(100, (cleanProgress.done / cleanProgress.total) * 100) : 0}
  <Card class="mb-4 gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-2 flex items-center justify-between text-sm">
        <span class="text-foreground">
          Cleaning<span class="text-muted-foreground"> · {cleanProgress.done} / {cleanProgress.total}</span>
        </span>
        <span class="tabular-nums text-muted-foreground">{pct.toFixed(0)}%</span>
      </div>
      <Progress value={pct} class="h-2" />
    </CardContent>
  </Card>
{/if}

{#if status && !cleanProgress && !(busy && scanProgress)}
  <div class="mb-4 text-sm text-muted-foreground">{status}</div>
{/if}

{#if scanResult}
  <div class="mb-4 grid grid-cols-2 gap-3 md:grid-cols-3">
    <Card class="gap-0 py-4">
      <CardContent class="px-4">
        <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Reclaimable</div>
        <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">
          {fmtBytes(scanResult.totalBytes)}
        </div>
      </CardContent>
    </Card>
    <Card class="gap-0 py-4">
      <CardContent class="px-4">
        <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Items</div>
        <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">{scanResult.itemCount}</div>
      </CardContent>
    </Card>
    <Card class="gap-0 py-4">
      <CardContent class="px-4">
        <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Selected</div>
        <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">{fmtBytes(selectedBytes)}</div>
        <div class="mt-1 text-xs text-muted-foreground">{selected.size} item{selected.size === 1 ? "" : "s"}</div>
      </CardContent>
    </Card>
  </div>

  {#if allItems.length > 0}
    <div class="mb-3 flex items-center gap-2">
      <Button variant="outline" size="sm" onclick={selectAll}>Select all</Button>
      <Button variant="outline" size="sm" onclick={() => (selected = new Set())}>Clear</Button>
      <div class="ml-auto"></div>
      <Button variant="destructive" size="sm" onclick={clean} disabled={busy || selected.size === 0}>
        Remove selected
      </Button>
    </div>
  {/if}

  <div class="space-y-3">
    {#each scanResult.groups as g (g.cleanerId)}
      <Card class="gap-0 overflow-hidden py-0">
        <div class="flex items-center gap-2.5 border-b bg-muted/40 px-3 py-2.5">
          {#if g.available && g.items.length > 0}
            {@const sel = groupSel(g.items)}
            <Checkbox
              checked={sel.all}
              indeterminate={sel.some}
              onCheckedChange={() => toggleSel(sel.ids, !sel.all)}
            />
          {:else}
            <span class="w-4"></span>
          {/if}
          <button
            onclick={() => toggleCollapse(g.cleanerId)}
            class="w-4 shrink-0 text-muted-foreground hover:text-foreground"
          >
            {collapsed.has(g.cleanerId) ? "▸" : "▾"}
          </button>
          <Badge variant="outline" class={KIND_BADGE[g.cleanerId] ?? ""}>{g.label}</Badge>
          {#if !g.available}
            <span class="text-xs text-muted-foreground">{g.unavailableReason ?? "not available"}</span>
          {:else}
            <div class="ml-auto flex items-center gap-3 whitespace-nowrap">
              <span class="text-xs text-muted-foreground">{g.items.length} item{g.items.length === 1 ? "" : "s"}</span>
              <span class="text-sm font-semibold tabular-nums text-foreground">{fmtBytes(g.totalBytes)}</span>
            </div>
          {/if}
        </div>
        {#if !collapsed.has(g.cleanerId) && g.available}
          {#if g.items.length === 0}
            <CardContent class="py-6 text-center text-sm text-muted-foreground">Nothing to reclaim.</CardContent>
          {:else if g.cleanerId === "projects"}
            {#each projectSubgroups(g.items) as sub (sub.label)}
              {@const sel = groupSel(sub.items)}
              <div class="border-b last:border-b-0">
                <div class="flex items-center gap-2 bg-muted/20 px-3 py-2">
                  <Checkbox
                    checked={sel.all}
                    indeterminate={sel.some}
                    onCheckedChange={() => toggleSel(sel.ids, !sel.all)}
                  />
                  <button
                    onclick={() => {
                      const next = new Set(subCollapsed);
                      next.has(sub.label) ? next.delete(sub.label) : next.add(sub.label);
                      subCollapsed = next;
                    }}
                    class="w-4 shrink-0 text-muted-foreground hover:text-foreground"
                  >
                    {subCollapsed.has(sub.label) ? "▸" : "▾"}
                  </button>
                  <span class="break-all font-mono text-xs text-foreground" title={sub.label}>{sub.label}</span>
                  <span class="ml-auto text-sm font-semibold tabular-nums">{fmtBytes(sub.bytes)}</span>
                </div>
                {#if !subCollapsed.has(sub.label)}
                  <Table>
                    <TableBody>
                      {#each sub.items as item (item.id)}
                        <TableRow>
                          <TableCell class="w-8">
                            <Checkbox
                              checked={selected.has(item.id)}
                              onCheckedChange={() => toggleSel([item.id], !selected.has(item.id))}
                            />
                          </TableCell>
                          <TableCell class="font-mono text-xs"><span class="text-muted-foreground">…\</span>{item.kind}</TableCell>
                          <TableCell>
                            <Badge variant="outline" class={RISK_BADGE[item.risk] ?? ""}>{item.risk}</Badge>
                          </TableCell>
                          <TableCell class="max-w-xs truncate text-xs text-muted-foreground" title={item.reason}>
                            {item.reason}
                          </TableCell>
                          <TableCell class="whitespace-nowrap text-right">{fmtBytes(item.sizeBytes)}</TableCell>
                        </TableRow>
                      {/each}
                    </TableBody>
                  </Table>
                {/if}
              </div>
            {/each}
          {:else}
            <Table>
              <TableBody>
                {#each g.items as item (item.id)}
                  <TableRow>
                    <TableCell class="w-8">
                      <Checkbox
                        checked={selected.has(item.id)}
                        onCheckedChange={() => toggleSel([item.id], !selected.has(item.id))}
                      />
                    </TableCell>
                    <TableCell class="break-all font-mono text-xs text-foreground">{item.path}</TableCell>
                    <TableCell>
                      <Badge variant="outline" class={KIND_BADGE[item.kind] ?? ""}>{kindLabel(item.kind)}</Badge>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" class={RISK_BADGE[item.risk] ?? ""}>{item.risk}</Badge>
                    </TableCell>
                    <TableCell class="max-w-xs truncate text-xs text-muted-foreground" title={item.reason}>
                      {item.reason}
                    </TableCell>
                    <TableCell class="whitespace-nowrap text-right">{fmtBytes(item.sizeBytes)}</TableCell>
                  </TableRow>
                {/each}
              </TableBody>
            </Table>
          {/if}
        {/if}
      </Card>
    {/each}
  </div>
{/if}
