<!--
	One-click review: merges the dev-cleaner scan (containers + saved project
	roots + Home) with a junk scan of Downloads and Temp into a single
	reviewable list. Still nothing goes without a tick — this only saves the
	trip through three tabs to start the three scans.
-->

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type {
    DevCleanerCleanResult,
    DevCleanerScan,
    Location,
    QuarantineResult,
    Risk,
    ScanSummary,
  } from '../types';
  import { fmtBytes, RISK_BADGE, KIND_BADGE } from '../utils';
  import { confirmAction } from '$lib/utils/confirm';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Label } from '$lib/components/ui/label';
  import { Table, TableBody, TableCell, TableRow } from '$lib/components/ui/table';
  import StatTiles from './shared/StatTiles.svelte';

  interface SweepItem {
    id: string;
    itemId: string;
    source: 'dev' | 'file';
    cleanerId: string;
    path: string;
    kind: string;
    sizeBytes: number;
    risk: Risk;
  }

  let { open = $bindable(false) }: { open?: boolean } = $props();

  let phase = $state<'idle' | 'scanning' | 'reviewing' | 'cleaning'>('idle');
  let items = $state<SweepItem[]>([]);
  let selected = $state<Set<string>>(new Set());
  let status = $state('');
  let permanentDelete = $state(false);

  const totalBytes = $derived(items.reduce((a, i) => a + i.sizeBytes, 0));
  const selectedBytes = $derived(
    items.filter((i) => selected.has(i.id)).reduce((a, i) => a + i.sizeBytes, 0)
  );

  $effect(() => {
    if (open && phase === 'idle') void scan();
  });

  async function scan() {
    phase = 'scanning';
    status = 'Scanning projects, containers and common junk…';
    try {
      const [locations, devRoots] = await Promise.all([
        invoke<Location[]>('list_locations'),
        invoke<string[]>('list_dev_scan_roots'),
      ]);
      const home = locations.find((l) => l.label === 'Home');
      const roots = [...new Set([...devRoots, ...(home ? [home.path] : [])])];
      const downloads = locations.find((l) => l.label === 'Downloads');
      const temp =
        locations.find((l) => l.label === 'Temp') ?? locations.find((l) => /temp/i.test(l.path));
      const fileRoots = [downloads, temp].filter((l): l is Location => !!l);

      const [devScan, ...fileScans] = await Promise.all([
        invoke<DevCleanerScan>('scan_dev_cleaners', { roots: roots.length ? roots : null }),
        ...fileRoots.map((l) =>
          invoke<ScanSummary>('scan_directory', { root: l.path }).catch(() => null)
        ),
      ]);

      const devItems: SweepItem[] = devScan.groups.flatMap((g) =>
        g.items.map((i) => ({
          id: `dev:${i.id}`,
          itemId: i.id,
          source: 'dev' as const,
          cleanerId: i.cleanerId,
          path: i.path,
          kind: i.kind,
          sizeBytes: i.sizeBytes,
          risk: i.risk,
        }))
      );
      const fileItems: SweepItem[] = fileScans
        .filter((s): s is ScanSummary => !!s)
        .flatMap((s) => s.proposals)
        .map((p) => ({
          id: `file:${p.id}`,
          itemId: p.id,
          source: 'file' as const,
          cleanerId: 'files',
          path: p.path,
          kind: p.kind,
          sizeBytes: p.sizeBytes,
          risk: p.risk,
        }));

      items = [...devItems, ...fileItems].sort((a, b) => b.sizeBytes - a.sizeBytes);
      selected = new Set(items.filter((i) => i.risk === 'Safe').map((i) => i.id));
      status =
        items.length === 0
          ? 'Nothing to reclaim.'
          : `${items.length} item${items.length === 1 ? '' : 's'} found · ${fmtBytes(totalBytes)} reclaimable`;
      phase = 'reviewing';
    } catch (e) {
      status = `Scan failed: ${String(e)}`;
      phase = 'idle';
    }
  }

  function toggle(id: string) {
    const next = new Set(selected);
    next.has(id) ? next.delete(id) : next.add(id);
    selected = next;
  }

  async function clean() {
    const chosen = items.filter((i) => selected.has(i.id));
    if (chosen.length === 0) return;
    const containerSelected = chosen.some((i) => i.source === 'dev' && i.cleanerId !== 'projects');
    const isPermanent = (i: SweepItem) =>
      permanentDelete && i.risk === 'Safe' && i.cleanerId !== 'docker' && i.cleanerId !== 'podman';
    const permCount = chosen.filter(isPermanent).length;
    const ok = await confirmAction(
      `Remove ${chosen.length} item(s) (${fmtBytes(selectedBytes)})?\n\n${
        permCount > 0
          ? `${permCount} Safe item${permCount === 1 ? '' : 's'} will be permanently deleted — not recoverable. `
          : ''
      }Everything else goes to the Recycle Bin and can be restored.${containerSelected ? ' Container resources are removed by Docker/Podman and cannot.' : ''}`,
      'Remove selected',
      { confirmLabel: 'Remove' }
    );
    if (!ok) return;
    phase = 'cleaning';
    status = 'Removing…';
    try {
      const devChosen = chosen.filter((i) => i.source === 'dev');
      const fileChosen = chosen.filter((i) => i.source === 'file');
      const [devRes, fileRes] = await Promise.all([
        devChosen.length
          ? invoke<DevCleanerCleanResult>('clean_dev_items', {
              items: devChosen.map((i) => ({
                cleanerId: i.cleanerId,
                id: i.itemId,
                path: i.path,
                kind: i.kind,
                permanent: isPermanent(i),
              })),
            })
          : null,
        fileChosen.length
          ? invoke<QuarantineResult>('quarantine_paths', {
              items: fileChosen.map((i) => ({
                path: i.path,
                kind: i.kind,
                permanent: isPermanent(i),
              })),
            })
          : null,
      ]);
      const reclaimed = (devRes?.reclaimedBytes ?? 0) + (fileRes?.reclaimedBytes ?? 0);
      const removedN = (devRes?.cleaned.length ?? 0) + (fileRes?.moved.length ?? 0);
      const failedN = (devRes?.failed.length ?? 0) + (fileRes?.failed.length ?? 0);
      items = items.filter((i) => !selected.has(i.id));
      selected = new Set();
      status = `Reclaimed ${fmtBytes(reclaimed)} · ${removedN} removed${failedN ? ` · ${failedN} failed` : ''}`;
      phase = 'reviewing';
    } catch (e) {
      status = `Removal failed: ${String(e)}`;
      phase = 'reviewing';
    }
  }

  function close() {
    open = false;
    phase = 'idle';
    items = [];
    selected = new Set();
    status = '';
  }
</script>

<Dialog.Root bind:open onOpenChange={(v: boolean) => !v && close()}>
  <Dialog.Content class="max-h-[85vh] max-w-3xl overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Review & Clean Everything</Dialog.Title>
    </Dialog.Header>

    {#if phase === 'scanning'}
      <div class="py-10 text-center text-sm text-muted-foreground">{status}</div>
    {:else}
      {#if status}<div class="mb-3 text-sm text-muted-foreground">{status}</div>{/if}

      {#if items.length > 0}
        <StatTiles
          tiles={[
            { label: 'Found', value: fmtBytes(totalBytes), sub: `${items.length} items` },
            {
              label: 'Selected',
              value: fmtBytes(selectedBytes),
              sub: `${selected.size} item${selected.size === 1 ? '' : 's'}`,
            },
          ]}
        />

        <div class="my-3 flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onclick={() =>
              (selected = new Set(items.filter((i) => i.risk === 'Safe').map((i) => i.id)))}
          >
            Select all Safe
          </Button>
          <Button variant="outline" size="sm" onclick={() => (selected = new Set())}>Clear</Button>
          <div class="ml-2 flex items-center gap-1.5">
            <Checkbox bind:checked={permanentDelete} id="sweep-permanent" />
            <Label for="sweep-permanent" class="text-xs font-normal text-muted-foreground">
              Delete Safe items permanently
            </Label>
          </div>
        </div>

        <Table>
          <TableBody>
            {#each items as item (item.id)}
              <TableRow>
                <TableCell class="w-8">
                  <Checkbox
                    checked={selected.has(item.id)}
                    onCheckedChange={() => toggle(item.id)}
                    disabled={phase === 'cleaning'}
                  />
                </TableCell>
                <TableCell class="font-mono text-xs break-all text-foreground"
                  >{item.path}</TableCell
                >
                <TableCell>
                  <Badge variant="outline" class={KIND_BADGE[item.kind] ?? ''}>{item.kind}</Badge>
                </TableCell>
                <TableCell>
                  <Badge variant="outline" class={RISK_BADGE[item.risk] ?? ''}>{item.risk}</Badge>
                </TableCell>
                <TableCell class="text-right whitespace-nowrap"
                  >{fmtBytes(item.sizeBytes)}</TableCell
                >
              </TableRow>
            {/each}
          </TableBody>
        </Table>
      {:else if phase === 'reviewing'}
        <div class="py-10 text-center text-sm text-muted-foreground">Nothing to reclaim.</div>
      {/if}

      <div class="mt-4 flex justify-end gap-2">
        <Button variant="outline" onclick={close}>Close</Button>
        <Button onclick={clean} disabled={selected.size === 0 || phase === 'cleaning'}>
          {phase === 'cleaning' ? 'Removing…' : `Remove selected (${fmtBytes(selectedBytes)})`}
        </Button>
      </div>
    {/if}
  </Dialog.Content>
</Dialog.Root>
