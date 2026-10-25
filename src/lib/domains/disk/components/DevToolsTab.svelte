<!--
	Developer leftovers — build output, dependency folders, and container junk.

	This absorbed the old "Projects" tab: both called the same Rust cleaner
	(`ProjectDevCleaner`), one through `scan_projects` and one through
	`scan_dev_cleaners`, so the app showed one scan twice under two names. Project
	roots are now a scope you add to this scan, not a separate page.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import type {
    DevCleanerCleanProgress,
    DevCleanerCleanResult,
    DevCleanerItem,
    DevCleanerScan,
    Location,
    ScanProgress,
  } from '../types';
  import { fmtBytes, KIND_BADGE, RISK_BADGE } from '../utils';
  import { confirmAction } from '$lib/utils/confirm';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Table, TableBody, TableCell, TableRow } from '$lib/components/ui/table';
  import { Boxes, Container, FolderOpen, Search, Square, X } from '@lucide/svelte';
  import ScanProgressCard from './shared/ScanProgressCard.svelte';
  import TaskProgressCard from './shared/TaskProgressCard.svelte';
  import StatTiles from './shared/StatTiles.svelte';
  import SelectionBar from './shared/SelectionBar.svelte';

  let { pending }: { pending: { tab: string; path?: string; seq: number } | null } = $props();

  let busy = $state(false);
  let cleanProgress = $state<DevCleanerCleanProgress | null>(null);
  let scanProgress = $state<ScanProgress | null>(null);
  let scanResult = $state<DevCleanerScan | null>(null);
  let selected = $state<Set<string>>(new Set());
  let collapsed = $state<Set<string>>(new Set());
  let subCollapsed = $state<Set<string>>(new Set());
  let status = $state('');
  let locations = $state<Location[]>([]);
  let scanRoots = $state<Set<string>>(new Set());

  onMount(() => {
    invoke<Location[]>('list_locations')
      .then((l) => (locations = l))
      .catch(() => (locations = []));
    const offs = [
      listen<DevCleanerCleanProgress>('dev-clean://progress', (e) => (cleanProgress = e.payload)),
      listen<ScanProgress>('scan://progress', (e) => (scanProgress = e.payload)),
    ];
    return () => offs.forEach((p) => p.then((off) => off()));
  });

  const allItems = $derived(scanResult ? scanResult.groups.flatMap((g) => g.items) : []);
  const selectedBytes = $derived(
    allItems.filter((i) => selected.has(i.id)).reduce((a, i) => a + i.sizeBytes, 0)
  );

  /** Drives and Home are offered as one-click scopes; anything else is picked. */
  const suggestedRoots = $derived(
    locations.filter((l) => l.kind === 'drive' || l.label === 'Home')
  );
  const customRoots = $derived(
    [...scanRoots].filter((p) => !suggestedRoots.some((l) => l.path === p))
  );

  // Deep link from the project audit: "clean this project's build output".
  let jumpSeen = 0;
  $effect(() => {
    if (pending && pending.tab === 'devtools' && pending.seq !== jumpSeen) {
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
    if (typeof dir === 'string') scanRoots = new Set([...scanRoots, dir]);
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
    status = 'Scanning…';
    scanProgress = null;
    selected = new Set();
    const roots = [...scanRoots];
    try {
      const res = await invoke<DevCleanerScan>('scan_dev_cleaners', {
        roots: roots.length ? roots : null,
      });
      scanResult = res;
      collapsed = new Set();
      subCollapsed = new Set();
      const available = res.groups.filter((g) => g.available).length;
      status =
        res.itemCount === 0
          ? `Nothing to reclaim · ${available} source${available === 1 ? '' : 's'} checked`
          : `${res.itemCount} item${res.itemCount === 1 ? '' : 's'} found · ${fmtBytes(res.totalBytes)} reclaimable`;
    } catch (e) {
      status = String(e) === 'cancelled' ? 'Scan cancelled' : `Scan failed: ${String(e)}`;
    } finally {
      busy = false;
      scanProgress = null;
    }
  }

  async function cancelScan() {
    await invoke('cancel_scan');
    status = 'Cancelling…';
  }

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

  /** Safe items only — the risk column is the whole point of showing it. */
  function selectSafe() {
    selected = new Set(allItems.filter((i) => i.risk === 'Safe').map((i) => i.id));
  }

  function groupSel(items: DevCleanerItem[]) {
    const ids = items.map((i) => i.id);
    const sel = ids.filter((id) => selected.has(id)).length;
    return { ids, all: sel === ids.length && ids.length > 0, some: sel > 0 && sel !== ids.length };
  }

  function kindLabel(kind: string): string {
    return kind.replace(/-/g, ' ');
  }

  /** Containers are removed by their own tooling, not moved to the Recycle Bin. */
  const containerSelected = $derived(
    allItems.some((i) => selected.has(i.id) && i.cleanerId !== 'projects')
  );

  async function clean() {
    if (!scanResult || selected.size === 0) return;
    const items = allItems
      .filter((i) => selected.has(i.id))
      .map((i) => ({ cleanerId: i.cleanerId, id: i.id, path: i.path, kind: i.kind }));
    const ok = await confirmAction(
      `Remove ${items.length} selected item(s) (${fmtBytes(selectedBytes)})?\n\nProject folders go to the Recycle Bin and can be restored. Container resources are removed by Docker/Podman and cannot.`,
      'Remove selected',
      { confirmLabel: 'Remove' }
    );
    if (!ok) return;
    busy = true;
    cleanProgress = { done: 0, total: items.length, currentPath: '' };
    status = 'Removing…';
    try {
      const res = await invoke<DevCleanerCleanResult>('clean_dev_items', { items });
      const cleanedPaths = new Set(res.cleaned.map((c) => c.path));
      scanResult = {
        ...scanResult,
        groups: scanResult.groups.map((g) => {
          const kept = g.items.filter((i) => !cleanedPaths.has(i.path));
          return { ...g, items: kept, totalBytes: kept.reduce((a, i) => a + i.sizeBytes, 0) };
        }),
        itemCount: scanResult.groups.reduce(
          (n, g) => n + g.items.filter((i) => !cleanedPaths.has(i.path)).length,
          0
        ),
        totalBytes: scanResult.totalBytes - res.reclaimedBytes,
      };
      selected = new Set();
      status = `Reclaimed ${fmtBytes(res.reclaimedBytes)} · ${res.cleaned.length} removed · ${res.failed.length} failed`;
    } catch (e) {
      status = `Removal failed: ${String(e)}`;
    } finally {
      busy = false;
      cleanProgress = null;
    }
  }
</script>

<Card class="mb-4 gap-0 py-4">
  <CardContent class="space-y-3 px-4">
    <div class="flex flex-wrap items-center gap-2 text-sm">
      <Container class="size-4 text-muted-foreground" />
      <span class="font-medium text-foreground">Container tools</span>
      <span class="text-muted-foreground">
        Docker and Podman — stopped containers, dangling images, unused volumes and build cache.
        Always included; skipped when the tool is not installed.
      </span>
    </div>

    <div class="flex flex-wrap items-center gap-2 text-sm">
      <Boxes class="size-4 text-muted-foreground" />
      <span class="font-medium text-foreground">Project build output</span>
      <span class="text-muted-foreground">
        {scanRoots.size
          ? 'node_modules, target, dist and friends under the folders below.'
          : 'Pick a folder to include node_modules, target, dist and friends.'}
      </span>
    </div>

    <div class="flex flex-wrap gap-2">
      {#each suggestedRoots as loc (loc.path)}
        <Button
          variant={scanRoots.has(loc.path) ? 'default' : 'outline'}
          size="sm"
          onclick={() => toggleRoot(loc.path)}
          disabled={busy}
          title={loc.path}
        >
          {loc.label}
        </Button>
      {/each}
      {#each customRoots as path (path)}
        <Badge variant="secondary" class="gap-1 font-mono text-xs">
          {path}
          <Button
            variant="ghost"
            class="h-auto p-0"
            aria-label={`Remove ${path}`}
            onclick={() => toggleRoot(path)}
          >
            <X class="size-3" />
          </Button>
        </Badge>
      {/each}
      <Button variant="outline" size="sm" onclick={pickFolder} disabled={busy}>
        <FolderOpen class="size-4" /> Add folder…
      </Button>
    </div>

    <div class="flex items-center gap-2">
      <Button onclick={() => runScan()} disabled={busy}>
        <Search class="size-4" />
        {busy ? 'Working…' : 'Scan'}
      </Button>
      {#if busy && scanRoots.size > 0}
        <!-- Only the project walk is cancellable; container enumeration is not. -->
        <Button variant="destructive" onclick={cancelScan}>
          <Square class="size-4" /> Stop
        </Button>
      {/if}
      <span class="text-xs text-muted-foreground">
        {scanRoots.size
          ? `Containers + projects under ${scanRoots.size} folder${scanRoots.size === 1 ? '' : 's'}`
          : 'Containers only — add a folder to include projects'}
      </span>
    </div>
  </CardContent>
</Card>

<ScanProgressCard progress={busy ? scanProgress : null} label="Scanning projects" />
<TaskProgressCard progress={cleanProgress} label="Removing" />

{#if status && !cleanProgress && !(busy && scanProgress)}
  <div class="mb-4 text-sm text-muted-foreground">{status}</div>
{/if}

{#if scanResult}
  <StatTiles
    tiles={[
      { label: 'Reclaimable', value: fmtBytes(scanResult.totalBytes) },
      { label: 'Items', value: String(scanResult.itemCount) },
      {
        label: 'Selected',
        value: fmtBytes(selectedBytes),
        sub: `${selected.size} item${selected.size === 1 ? '' : 's'}`,
      },
    ]}
  />

  {#if allItems.length > 0}
    <SelectionBar
      count={selected.size}
      bytes={selectedBytes}
      disabled={busy}
      actionLabel={containerSelected ? 'Remove selected' : 'Move to Recycle Bin'}
      onaction={clean}
    >
      {#snippet tools()}
        <Button variant="outline" size="sm" onclick={selectSafe}>Select all Safe</Button>
        <Button variant="outline" size="sm" onclick={selectAll}>Select all</Button>
        <Button variant="outline" size="sm" onclick={() => (selected = new Set())}>Clear</Button>
      {/snippet}
    </SelectionBar>
  {/if}

  <div class="space-y-3">
    {#each scanResult.groups as g (g.cleanerId)}
      <Card class="gap-0 overflow-hidden py-0">
        <div
          class="divider-edge-b divider-edge-full flex items-center gap-2.5 bg-muted/40 px-3 py-2.5"
        >
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
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            class="h-4 w-4 shrink-0 p-0 text-muted-foreground"
            onclick={() => toggleCollapse(g.cleanerId)}
          >
            {collapsed.has(g.cleanerId) ? '▸' : '▾'}
          </Button>
          <Badge variant="outline" class={KIND_BADGE[g.cleanerId] ?? ''}>{g.label}</Badge>
          {#if !g.available}
            <span class="text-xs text-muted-foreground"
              >{g.unavailableReason ?? 'not available'}</span
            >
          {:else}
            <div class="ml-auto flex items-center gap-3 whitespace-nowrap">
              <span class="text-xs text-muted-foreground"
                >{g.items.length} item{g.items.length === 1 ? '' : 's'}</span
              >
              <span class="text-sm font-semibold text-foreground tabular-nums"
                >{fmtBytes(g.totalBytes)}</span
              >
            </div>
          {/if}
        </div>
        {#if !collapsed.has(g.cleanerId) && g.available}
          {#if g.items.length === 0}
            <CardContent class="py-6 text-center text-sm text-muted-foreground">
              Nothing to reclaim.
            </CardContent>
          {:else if g.cleanerId === 'projects'}
            {#each projectSubgroups(g.items) as sub (sub.label)}
              {@const sel = groupSel(sub.items)}
              <div class="divider-edge-b divider-edge-full last:[&::after]:hidden">
                <div class="flex items-center gap-2 bg-muted/20 px-3 py-2">
                  <Checkbox
                    checked={sel.all}
                    indeterminate={sel.some}
                    onCheckedChange={() => toggleSel(sel.ids, !sel.all)}
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    class="h-4 w-4 shrink-0 p-0 text-muted-foreground"
                    onclick={() => {
                      const next = new Set(subCollapsed);
                      next.has(sub.label) ? next.delete(sub.label) : next.add(sub.label);
                      subCollapsed = next;
                    }}
                  >
                    {subCollapsed.has(sub.label) ? '▸' : '▾'}
                  </Button>
                  <span class="font-mono text-xs break-all text-foreground" title={sub.label}>
                    {sub.label}
                  </span>
                  <span class="ml-auto flex items-center gap-3 whitespace-nowrap">
                    <span class="text-xs text-muted-foreground">
                      {sub.items.length} dir{sub.items.length === 1 ? '' : 's'}
                    </span>
                    <span class="text-sm font-semibold tabular-nums">{fmtBytes(sub.bytes)}</span>
                  </span>
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
                          <TableCell class="font-mono text-xs">
                            <span class="text-muted-foreground">…\</span>{item.kind}
                          </TableCell>
                          <TableCell class="text-xs whitespace-nowrap text-muted-foreground">
                            {item.fileCount.toLocaleString()} files
                          </TableCell>
                          <TableCell>
                            <Badge variant="outline" class={RISK_BADGE[item.risk] ?? ''}>
                              {item.risk}
                            </Badge>
                          </TableCell>
                          <TableCell
                            class="max-w-xs truncate text-xs text-muted-foreground"
                            title={item.reason}
                          >
                            {item.reason}
                          </TableCell>
                          <TableCell class="text-right whitespace-nowrap">
                            {fmtBytes(item.sizeBytes)}
                          </TableCell>
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
                    <TableCell class="font-mono text-xs break-all text-foreground">
                      {item.path}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" class={KIND_BADGE[item.kind] ?? ''}>
                        {kindLabel(item.kind)}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" class={RISK_BADGE[item.risk] ?? ''}>
                        {item.risk}
                      </Badge>
                    </TableCell>
                    <TableCell
                      class="max-w-xs truncate text-xs text-muted-foreground"
                      title={item.reason}
                    >
                      {item.reason}
                    </TableCell>
                    <TableCell class="text-right whitespace-nowrap">
                      {fmtBytes(item.sizeBytes)}
                    </TableCell>
                  </TableRow>
                {/each}
              </TableBody>
            </Table>
          {/if}
        {/if}
      </Card>
    {/each}
  </div>
{:else if !busy}
  <Card>
    <CardContent class="py-10 text-center text-sm text-muted-foreground">
      Nothing scanned yet. <span class="text-foreground">Scan</span> checks Docker and Podman; add a folder
      above to include project build output.
    </CardContent>
  </Card>
{/if}
