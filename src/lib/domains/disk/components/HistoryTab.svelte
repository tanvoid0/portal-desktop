<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { AuditEntry } from '../types';
  import { fmtBytes, fmtDate } from '../utils';
  import StatTiles from './shared/StatTiles.svelte';
  import { Card } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';

  let { active }: { active: boolean } = $props();

  let entries = $state<AuditEntry[]>([]);
  let status = $state('Loading…');

  async function load() {
    status = 'Loading…';
    try {
      const log = await invoke<AuditEntry[]>('get_audit_log');
      entries = log;
      status = log.length === 0 ? 'No actions recorded yet.' : `${log.length} action(s) recorded.`;
    } catch (e) {
      status = `Failed to load history: ${String(e)}`;
    }
  }

  let lastActive = false;
  $effect(() => {
    if (active && !lastActive) void load();
    lastActive = active;
  });

  const hstats = $derived.by(() => {
    const moved = entries.filter((e) => e.status === 'moved');
    return {
      reclaimed: moved.reduce((a, e) => a + e.sizeBytes, 0),
      movedN: moved.length,
      failedN: entries.length - moved.length,
    };
  });

  async function openBin() {
    try {
      await invoke('open_recycle_bin');
    } catch (e) {
      status = String(e);
    }
  }
</script>

<StatTiles
  tiles={[
    {
      label: 'Total reclaimed',
      value: fmtBytes(hstats.reclaimed),
      sub: 'across every scan',
      accent: 'text-status-success',
    },
    { label: 'Moved', value: String(hstats.movedN), sub: 'to the Recycle Bin' },
    {
      label: 'Failed',
      value: String(hstats.failedN),
      sub: hstats.failedN ? 'in use or permission denied' : 'none',
      accent: hstats.failedN ? 'text-status-error' : undefined,
    },
  ]}
/>

<div class="mb-4 flex items-center gap-3">
  <div class="text-sm text-muted-foreground">{status}</div>
  <Button variant="outline" size="sm" onclick={load} class="ml-auto">Refresh</Button>
  <Button variant="outline" size="sm" onclick={openBin}>Open Recycle Bin to undo</Button>
</div>

<Card class="gap-0 overflow-hidden py-0">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>When</TableHead>
        <TableHead>Path</TableHead>
        <TableHead>Kind</TableHead>
        <TableHead class="text-right">Size</TableHead>
        <TableHead>Status</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {#each entries as e (e.id)}
        <TableRow>
          <TableCell class="whitespace-nowrap text-muted-foreground">{fmtDate(e.ts)}</TableCell>
          <TableCell class="break-all font-mono text-xs">{e.path}</TableCell>
          <TableCell class="text-muted-foreground">{e.kind}</TableCell>
          <TableCell class="whitespace-nowrap text-right">{fmtBytes(e.sizeBytes)}</TableCell>
          <TableCell>
            <Badge
              variant="outline"
              class={e.status === 'moved'
                ? 'bg-status-success-bg text-status-success border-status-success/30'
                : 'bg-status-error-bg text-status-error border-status-error/30'}
            >
              {e.status}
            </Badge>
          </TableCell>
        </TableRow>
      {/each}
      {#if entries.length === 0}
        <TableRow>
          <TableCell colspan={5} class="py-10 text-center text-muted-foreground">
            Nothing here yet. Quarantined items will be logged.
          </TableCell>
        </TableRow>
      {/if}
    </TableBody>
  </Table>
</Card>
