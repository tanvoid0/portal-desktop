<!--
	Developer leftovers — build output, dependency folders, and container junk.

	This absorbed the old "Projects" tab: both called the same Rust cleaner
	(`ProjectDevCleaner`), one through `scan_projects` and one through
	`scan_dev_cleaners`, so the app showed one scan twice under two names. Project
	roots are now a scope you add to this scan, not a separate page.
-->

<script module lang="ts">
  import type { DevCleanerCleanProgress, DevCleanerScan, ScanProgress } from '../types';

  // Module level: scan, selection and scope survive leaving /utilities/disk.
  let busy = $state(false);
  let cleanProgress = $state<DevCleanerCleanProgress | null>(null);
  let scanProgress = $state<ScanProgress | null>(null);
  let scanResult = $state<DevCleanerScan | null>(null);
  let selected = $state<Set<string>>(new Set());
  let collapsed = $state<Set<string>>(new Set());
  let subCollapsed = $state<Set<string>>(new Set());
  let status = $state('');
  let scanRoots = $state<Set<string>>(new Set());
  let permanentDelete = $state(false);
  let rootsLoaded = false;
  let jumpSeen = 0;
  // Latest scan wins: a deep link can start one while another is still running.
  let latestRun = 0;
</script>

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { DevCleanerCleanResult, Location } from '../types';
  import { fmtBytes } from '../utils';
  import { confirmAction } from '$lib/utils/confirm';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import ScanProgressCard from './shared/ScanProgressCard.svelte';
  import TaskProgressCard from './shared/TaskProgressCard.svelte';
  import StatTiles from './shared/StatTiles.svelte';
  import SelectionBar from './shared/SelectionBar.svelte';
  import PermanentDeleteToggle from './shared/PermanentDeleteToggle.svelte';
  import DevToolsScopeCard from './DevToolsScopeCard.svelte';
  import DevToolsGroupCard from './DevToolsGroupCard.svelte';
  import RelocationCard from './RelocationCard.svelte';

  let { pending }: { pending: { tab: string; path?: string; seq: number } | null } = $props();

  let locations = $state<Location[]>([]);

  onMount(() => {
    invoke<Location[]>('list_locations')
      .then((l) => (locations = l))
      .catch(() => (locations = []));
    // Once: merging again on return would re-add roots a deep link had replaced.
    if (!rootsLoaded) {
      invoke<string[]>('list_dev_scan_roots')
        .then((roots) => {
          scanRoots = new Set([...scanRoots, ...roots]);
          rootsLoaded = true;
        })
        .catch(() => {});
    }
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
    [...scanRoots].filter((p) => !suggestedRoots.some((l) => l.path === p)).sort()
  );

  // Deep link from the project audit: "clean this project's build output".
  $effect(() => {
    if (pending && pending.tab === 'devtools' && pending.seq !== jumpSeen) {
      jumpSeen = pending.seq;
      if (pending.path) scanRoots = new Set([pending.path]);
      void runScan();
    }
  });

  function toggleRoot(path: string) {
    const next = new Set(scanRoots);
    if (next.has(path)) {
      next.delete(path);
      void invoke('remove_dev_scan_root', { path }).catch(() => {});
    } else {
      next.add(path);
      void invoke('add_dev_scan_root', { path }).catch(() => {});
    }
    scanRoots = next;
  }

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir !== 'string') return;
    scanRoots = new Set([...scanRoots, dir]);
    void invoke('add_dev_scan_root', { path: dir }).catch(() => {});
  }

  async function runScan() {
    const run = ++latestRun;
    busy = true;
    status = 'Scanning…';
    scanProgress = null;
    selected = new Set();
    const roots = [...scanRoots];
    try {
      const res = await invoke<DevCleanerScan>('scan_dev_cleaners', {
        roots: roots.length ? roots : null,
      });
      if (run !== latestRun) return;
      scanResult = res;
      collapsed = new Set();
      subCollapsed = new Set();
      const available = res.groups.filter((g) => g.available).length;
      status =
        res.itemCount === 0
          ? `Nothing to reclaim · ${available} source${available === 1 ? '' : 's'} checked`
          : `${res.itemCount} item${res.itemCount === 1 ? '' : 's'} found · ${fmtBytes(res.totalBytes)} reclaimable`;
    } catch (e) {
      if (run !== latestRun) return;
      status = String(e) === 'cancelled' ? 'Scan cancelled' : `Scan failed: ${String(e)}`;
    } finally {
      if (run === latestRun) {
        busy = false;
        scanProgress = null;
      }
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

  function toggled(set: Set<string>, v: string) {
    const next = new Set(set);
    next.has(v) ? next.delete(v) : next.add(v);
    return next;
  }

  function toggleCollapse(groupId: string) {
    collapsed = toggled(collapsed, groupId);
  }

  function selectAll() {
    selected = new Set(allItems.map((i) => i.id));
  }

  /** Safe items only — the risk column is the whole point of showing it. */
  function selectSafe() {
    selected = new Set(allItems.filter((i) => i.risk === 'Safe').map((i) => i.id));
  }

  /** Containers are removed by their own tooling, not moved to the Recycle Bin. */
  const containerSelected = $derived(
    allItems.some((i) => selected.has(i.id) && i.cleanerId !== 'projects')
  );

  async function clean() {
    if (!scanResult || selected.size === 0) return;
    const chosen = allItems.filter((i) => selected.has(i.id));
    const items = chosen.map((i) => ({
      cleanerId: i.cleanerId,
      id: i.id,
      path: i.path,
      kind: i.kind,
      permanent: permanentDelete && i.cleanerId === 'projects' && i.risk === 'Safe',
    }));
    const permCount = items.filter((i) => i.permanent).length;
    const ok = await confirmAction(
      `Remove ${items.length} selected item(s) (${fmtBytes(selectedBytes)})?\n\n${
        permCount > 0
          ? `${permCount} Safe project item${permCount === 1 ? '' : 's'} will be permanently deleted — not recoverable. `
          : ''
      }Other project folders go to the Recycle Bin and can be restored. Container resources are removed by Docker/Podman and cannot.`,
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

<DevToolsScopeCard
  {suggestedRoots}
  {customRoots}
  {scanRoots}
  {busy}
  ontoggleRoot={toggleRoot}
  onpick={pickFolder}
  onscan={runScan}
  oncancel={cancelScan}
/>

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
      actionLabel={containerSelected || permanentDelete ? 'Remove selected' : 'Move to Recycle Bin'}
      onaction={clean}
    >
      {#snippet tools()}
        <Button variant="outline" size="sm" onclick={selectSafe}>Select all Safe</Button>
        <Button variant="outline" size="sm" onclick={selectAll}>Select all</Button>
        <Button variant="outline" size="sm" onclick={() => (selected = new Set())}>Clear</Button>
        <PermanentDeleteToggle
          bind:checked={permanentDelete}
          id="devtools-permanent"
          label="Delete Safe project folders permanently"
        />
      {/snippet}
    </SelectionBar>
  {/if}

  <div class="space-y-3">
    {#each scanResult.groups as g (g.cleanerId)}
      <DevToolsGroupCard
        group={g}
        {selected}
        {collapsed}
        {subCollapsed}
        ontoggleSel={toggleSel}
        ontoggleCollapse={toggleCollapse}
        ontoggleSub={(label) => (subCollapsed = toggled(subCollapsed, label))}
      />
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

<div class="mt-4">
  <RelocationCard />
</div>
