<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { AuditEntry } from "../types";
  import { fmtBytes, fmtDate } from "../utils";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";

  let { active }: { active: boolean } = $props();

  let entries = $state<AuditEntry[]>([]);
  let status = $state("Loading…");

  async function load() {
    status = "Loading…";
    try {
      const log = await invoke<AuditEntry[]>("get_audit_log");
      entries = log;
      status = log.length === 0 ? "No actions recorded yet." : `${log.length} action(s) recorded.`;
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
    const moved = entries.filter((e) => e.status === "moved");
    return {
      reclaimed: moved.reduce((a, e) => a + e.sizeBytes, 0),
      movedN: moved.length,
      failedN: entries.length - moved.length,
    };
  });

  async function openBin() {
    try {
      await invoke("open_recycle_bin");
    } catch (e) {
      status = String(e);
    }
  }
</script>

<div class="mb-4 grid grid-cols-3 gap-3">
  <Card class="gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Total reclaimed</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight text-status-success">{fmtBytes(hstats.reclaimed)}</div>
    </CardContent>
  </Card>
  <Card class="gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Moved</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight text-foreground">{hstats.movedN}</div>
      <div class="mt-1 text-xs text-muted-foreground">to Recycle Bin</div>
    </CardContent>
  </Card>
  <Card class="gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Failed</div>
      <div class="text-2xl font-semibold tabular-nums tracking-tight {hstats.failedN ? 'text-status-error' : 'text-foreground'}">{hstats.failedN}</div>
    </CardContent>
  </Card>
</div>

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
              class={e.status === "moved"
                ? "bg-status-success-bg text-status-success border-status-success/30"
                : "bg-status-error-bg text-status-error border-status-error/30"}
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
