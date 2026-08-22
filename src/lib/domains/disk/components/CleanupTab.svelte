<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type {
    CachedScan,
    ItemVerdict,
    Location,
    QuarantineProgress,
    QuarantineResult,
    ScanProgress,
    ScanSummary,
    VerificationResult,
    VerifyProgress,
  } from '../types';
  import {
    buildTree,
    flattenTree,
    folderPaths,
    fmtBytes,
    fmtDate,
    fmtAgo,
    leafIds,
    RISK_BADGE,
    VERDICT_BADGE,
    verdictMap,
    DISK_STAT_GRID_FOUR,
    type TreeNode,
  } from '../utils';
  import ScanProgressCard from './shared/ScanProgressCard.svelte';
  import TaskProgressCard from './shared/TaskProgressCard.svelte';
  import StatTiles from './shared/StatTiles.svelte';
  import TargetPicker from './shared/TargetPicker.svelte';
  import SelectionBar from './shared/SelectionBar.svelte';
  import { confirmAction } from '$lib/utils/confirm';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Progress } from '$lib/components/ui/progress';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';

  let { pending }: { pending: { tab: string; path?: string; seq: number } | null } = $props();

  let root = $state('');
  let busy = $state(false);
  let progress = $state<ScanProgress | null>(null);
  let qProgress = $state<QuarantineProgress | null>(null);
  let summary = $state<ScanSummary | null>(null);
  let selected = $state<Set<string>>(new Set());
  let expanded = $state<Set<string>>(new Set());
  let status = $state('');
  let cached = $state<CachedScan | null>(null);
  let locations = $state<Location[]>([]);
  let verifying = $state(false);
  let verifyProgress = $state<VerifyProgress | null>(null);
  let verification = $state<VerificationResult | null>(null);
  let verifyErr = $state('');

  onMount(() => {
    invoke<Location[]>('list_locations')
      .then((l) => (locations = l))
      .catch(() => (locations = []));
    const offs = [
      listen<ScanProgress>('scan://progress', (e) => (progress = e.payload)),
      listen<QuarantineProgress>('quarantine://progress', (e) => (qProgress = e.payload)),
      listen<VerifyProgress>('verify://progress', (e) => (verifyProgress = e.payload)),
    ];
    return () => offs.forEach((p) => p.then((off) => off()));
  });

  const verdictByPath = $derived(
    verification ? verdictMap(verification.verdicts) : new Map<string, ItemVerdict>()
  );

  const selectedBytes = $derived.by(() => {
    if (!summary) return 0;
    return summary.proposals.filter((p) => selected.has(p.id)).reduce((a, p) => a + p.sizeBytes, 0);
  });

  const stats = $derived.by(() => {
    if (!summary) return null;
    const byRisk: Record<string, { n: number; b: number }> = {
      Safe: { n: 0, b: 0 },
      Review: { n: 0, b: 0 },
      Danger: { n: 0, b: 0 },
    };
    let total = 0;
    for (const p of summary.proposals) {
      const rr = byRisk[p.risk] ?? (byRisk[p.risk] = { n: 0, b: 0 });
      rr.n++;
      rr.b += p.sizeBytes;
      total += p.sizeBytes;
    }
    return { total, byRisk, count: summary.proposals.length };
  });

  const tree = $derived(summary ? buildTree(summary.proposals, summary.root) : []);
  const rows = $derived(flattenTree(tree, expanded));

  let lastTreeRef: TreeNode[] | null = null;
  $effect(() => {
    if (tree !== lastTreeRef) {
      lastTreeRef = tree;
      expanded = new Set(folderPaths(tree));
    }
  });

  $effect(() => {
    const dir = root;
    if (!dir) {
      cached = null;
      return;
    }
    let stale = false;
    invoke<CachedScan | null>('get_cached_scan', { root: dir })
      .then((c) => !stale && (cached = c))
      .catch(() => !stale && (cached = null));
    return () => {
      stale = true;
    };
  });

  let jumpSeen = 0;
  $effect(() => {
    if (pending && pending.tab === 'cleanup' && pending.path && pending.seq !== jumpSeen) {
      jumpSeen = pending.seq;
      root = pending.path;
      void scan(pending.path);
    }
  });

  async function refreshCached(target?: string) {
    const dir = target ?? root;
    if (!dir) return (cached = null);
    try {
      cached = await invoke<CachedScan | null>('get_cached_scan', { root: dir });
    } catch {
      cached = null;
    }
  }

  function loadCached() {
    if (!cached) return;
    summary = cached.summary;
    selected = new Set();
    status =
      cached.status === 'partial'
        ? `Loaded interrupted scan from ${fmtDate(cached.ts)} — partial results. Resume for a full scan.`
        : `Loaded cached scan from ${fmtDate(cached.ts)} — rescan for fresh results.`;
  }

  async function discardCached() {
    if (!root) return;
    try {
      await invoke('remove_cached_scan', { root });
      cached = null;
      status = 'Discarded saved scan.';
    } catch (e) {
      status = `Discard failed: ${String(e)}`;
    }
  }

  async function scan(target?: string) {
    const dir = target ?? root;
    if (!dir) return;
    if (target) root = target;
    busy = true;
    status = 'Scanning…';
    progress = null;
    selected = new Set();
    try {
      const res = await invoke<ScanSummary>('scan_directory', { root: dir });
      summary = res;
      status = `Scanned ${res.scannedFiles.toLocaleString()} files in ${res.elapsedMs} ms · ${res.proposals.length} proposals`;
      void refreshCached(dir);
    } catch (e) {
      status = String(e) === 'cancelled' ? 'Scan cancelled' : `Scan failed: ${String(e)}`;
      if (String(e) === 'cancelled') void refreshCached(dir);
    } finally {
      busy = false;
      progress = null;
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

  function toggleExp(path: string) {
    const next = new Set(expanded);
    next.has(path) ? next.delete(path) : next.add(path);
    expanded = next;
  }

  function selectSafe() {
    if (!summary) return;
    selected = new Set(summary.proposals.filter((p) => p.risk === 'Safe').map((p) => p.id));
  }

  function rowSel(node: TreeNode) {
    const ids = leafIds(node);
    const sel = ids.filter((id) => selected.has(id)).length;
    return { ids, all: sel === ids.length && ids.length > 0, some: sel > 0 && sel !== ids.length };
  }

  async function verifyWithAi() {
    if (!summary) return;
    const toVerify =
      selected.size > 0 ? summary.proposals.filter((p) => selected.has(p.id)) : summary.proposals;
    if (toVerify.length === 0) return;
    verifying = true;
    verifyErr = '';
    verification = null;
    verifyProgress = null;
    try {
      verification = await invoke<VerificationResult>('verify_proposals', {
        root: summary.root,
        proposals: toVerify,
      });
    } catch (e) {
      const msg = String(e);
      if (/cancelled/i.test(msg)) verifyErr = '';
      else
        verifyErr = /scope/i.test(msg)
          ? `${msg} — mint a token with process:read + process:write (Settings).`
          : msg;
    } finally {
      verifying = false;
      verifyProgress = null;
    }
  }

  function cancelVerify() {
    invoke('cancel_verify').catch(() => {});
  }

  function deselectFlagged() {
    if (!verification || !summary) return;
    const flagged = new Set(
      verification.verdicts.filter((v) => v.verdict !== 'safe').map((v) => v.path)
    );
    if (flagged.size === 0) return;
    const ids = new Set(summary.proposals.filter((p) => flagged.has(p.path)).map((p) => p.id));
    const next = new Set(selected);
    ids.forEach((id) => next.delete(id));
    selected = next;
  }

  async function quarantine() {
    if (!summary || selected.size === 0) return;
    const items = summary.proposals
      .filter((p) => selected.has(p.id))
      .map((p) => ({ path: p.path, kind: p.kind }));
    const ok = await confirmAction(
      `Move ${items.length} item(s) (${fmtBytes(selectedBytes)}) to the Recycle Bin?\n\nNothing is permanently deleted — you can restore from the Recycle Bin.`,
      'Move to Recycle Bin',
      { confirmLabel: 'Move' }
    );
    if (!ok) return;
    busy = true;
    qProgress = { done: 0, total: items.length, currentPath: '' };
    status = 'Moving to Recycle Bin…';
    try {
      const res = await invoke<QuarantineResult>('quarantine_paths', { items });
      status = `Reclaimed ${fmtBytes(res.reclaimedBytes)} · ${res.moved.length} moved · ${res.failed.length} failed`;
      summary = { ...summary, proposals: summary.proposals.filter((p) => !selected.has(p.id)) };
      selected = new Set();
    } catch (e) {
      status = `Quarantine failed: ${String(e)}`;
    } finally {
      busy = false;
      qProgress = null;
    }
  }

  const restorePct = $derived(
    cached && cached.status === 'partial' && cached.totalFiles > 0
      ? Math.min(100, Math.round((cached.scannedFiles / cached.totalFiles) * 100))
      : 100
  );
  const restoreReclaimable = $derived(
    cached ? cached.summary.proposals.reduce((a, p) => a + p.sizeBytes, 0) : 0
  );
  const verifyCounts = $derived.by(() => {
    const c = { safe: 0, review: 0, dangerous: 0 };
    verification?.verdicts.forEach((v) => {
      // `verdict` is a plain String on the Rust side — the synthesizer's output
      // is model text, so the parser stays tolerant and normalizes anything it
      // does not recognise to "review". The type cannot say that, so do it here.
      if (v.verdict in c) c[v.verdict as keyof typeof c]++;
      else c.review++;
    });
    return c;
  });
</script>

<TargetPicker
  bind:value={root}
  {locations}
  {busy}
  placeholder="C:\Users\you\Downloads"
  showSuggestions={!summary}
  onscan={(p) => scan(p)}
  oncancel={cancelScan}
/>

{#if cached && !summary && !busy}
  {@const partial = cached.status === 'partial'}
  <Card class="mb-4 gap-0 {partial ? 'border-status-warning/30' : ''}">
    <CardContent class="flex items-start gap-3 px-4 py-3.5">
      <div
        class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-sm {partial
          ? 'bg-status-warning-bg text-status-warning'
          : 'bg-status-success-bg text-status-success'}"
      >
        {partial ? '◴' : '↻'}
      </div>
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <span class="text-sm font-medium text-foreground"
            >{partial ? 'Interrupted scan can be restored' : 'Saved scan available'}</span
          >
          <Badge
            variant="outline"
            class={partial
              ? 'bg-status-warning-bg text-status-warning border-status-warning/30'
              : 'bg-status-success-bg text-status-success border-status-success/30'}
          >
            {partial ? `stopped at ${restorePct}%` : 'complete'}
          </Badge>
        </div>
        <div class="mt-1 text-xs text-muted-foreground">
          {fmtAgo(cached.ts)} · {fmtDate(cached.ts)} · {cached.scannedFiles.toLocaleString()}{partial &&
          cached.totalFiles > 0
            ? ` / ${cached.totalFiles.toLocaleString()}`
            : ''} files · {fmtBytes(restoreReclaimable)} reclaimable
        </div>
        {#if partial}
          <Progress
            value={restorePct}
            class="mt-2 h-1.5 [&>[data-slot=progress-indicator]]:bg-status-warning"
          />
        {/if}
        <div class="mt-3 flex flex-wrap items-center gap-2">
          {#if partial}
            <Button size="sm" onclick={() => scan()}>Resume — full scan</Button>
            <Button variant="outline" size="sm" onclick={loadCached}>Load partial results</Button>
          {:else}
            <Button size="sm" onclick={loadCached}>Load saved scan</Button>
            <Button variant="outline" size="sm" onclick={() => scan()}>Rescan fresh</Button>
          {/if}
          <Button
            variant="outline"
            size="sm"
            onclick={discardCached}
            class="text-destructive hover:text-destructive">Discard</Button
          >
        </div>
      </div>
    </CardContent>
  </Card>
{/if}

<ScanProgressCard progress={busy ? progress : null} />

<TaskProgressCard progress={qProgress} />

{#if status && !(busy && progress) && !qProgress}
  <div class="mb-4 text-sm text-muted-foreground">{status}</div>
{/if}

{#if summary && stats}
  <StatTiles
    grid={DISK_STAT_GRID_FOUR}
    tiles={[
      {
        label: 'Reclaimable',
        value: fmtBytes(stats.total),
        sub: `${stats.count} proposal${stats.count === 1 ? '' : 's'}`,
      },
      {
        label: 'Safe',
        value: fmtBytes(stats.byRisk.Safe.b),
        sub: `${stats.byRisk.Safe.n} items — rebuilt or re-downloadable`,
        accent: 'text-status-success',
      },
      {
        label: 'Review',
        value: fmtBytes(stats.byRisk.Review.b),
        sub: `${stats.byRisk.Review.n} items — check before ticking`,
        accent: 'text-status-warning',
      },
      {
        label: 'Danger',
        value: fmtBytes(stats.byRisk.Danger.b),
        sub: `${stats.byRisk.Danger.n} items — could be the only copy`,
        accent: 'text-status-error',
      },
    ]}
  />

  {#if stats.total > 0}
    <div class="mb-6 flex h-2 w-full overflow-hidden rounded-full bg-muted">
      {#each [['Safe', stats.byRisk.Safe.b, 'bg-status-success'], ['Review', stats.byRisk.Review.b, 'bg-status-warning'], ['Danger', stats.byRisk.Danger.b, 'bg-status-error']] as [risk, bytes, color]}
        {#if (bytes as number) > 0}
          <div
            class={color as string}
            style="width: {((bytes as number) / stats.total) * 100}%"
            title={String(risk)}
          ></div>
        {/if}
      {/each}
    </div>
  {/if}

  <SelectionBar count={selected.size} bytes={selectedBytes} disabled={busy} onaction={quarantine}>
    {#snippet tools()}
      <Button variant="outline" size="sm" onclick={selectSafe}>Select all Safe</Button>
      <Button variant="outline" size="sm" onclick={() => (selected = new Set())}>Clear</Button>
      <Button variant="outline" size="sm" onclick={() => (expanded = new Set(folderPaths(tree)))}>
        Expand all
      </Button>
      <Button variant="outline" size="sm" onclick={() => (expanded = new Set())}>
        Collapse all
      </Button>
      <Button variant="outline" size="sm" onclick={verifyWithAi} disabled={verifying}>
        {verifying
          ? 'Verifying…'
          : selected.size > 0
            ? `Ask AI about ${selected.size} selected`
            : 'Ask AI to double-check'}
      </Button>
      {#if verifying}
        <Button variant="outline" size="sm" onclick={cancelVerify}>Stop</Button>
      {/if}
    {/snippet}
  </SelectionBar>

  {#if verifying && verifyProgress}
    <Card class="mb-4 gap-0 overflow-hidden py-0">
      <CardHeader
        class="divider-edge-b divider-edge-full bg-muted/40 px-4 py-2.5 [.border-b]:pb-2.5"
      >
        <div class="flex items-center gap-2">
          <CardTitle class="text-sm font-medium">AI verification</CardTitle>
          <span class="text-xs text-muted-foreground"
            >process #{verifyProgress.processId} · {verifyProgress.status.replace(/_/g, ' ')}</span
          >
          <Button variant="outline" size="sm" onclick={cancelVerify} class="ml-auto">Stop</Button>
        </div>
      </CardHeader>
      <CardContent class="px-4 py-3">
        {#if verifyProgress.tasks.length === 0}
          <div class="flex items-center gap-2 text-sm text-muted-foreground">
            <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-primary"></span>Planning the
            review…
          </div>
        {:else}
          <ul class="space-y-1.5">
            {#each verifyProgress.tasks as t, i (i)}
              <li class="flex items-center gap-2 text-sm text-foreground">
                <span
                  class="h-1.5 w-1.5 rounded-full {t.status === 'completed'
                    ? 'bg-status-success'
                    : t.status === 'failed'
                      ? 'bg-status-error'
                      : t.status.includes('progress') || t.status === 'running'
                        ? 'bg-primary animate-pulse'
                        : 'bg-muted-foreground'}"
                ></span>
                <span>{t.role}</span>
                <span class="text-xs text-muted-foreground">{t.status.replace(/_/g, ' ')}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </CardContent>
    </Card>
  {/if}

  {#if verifyErr || verification}
    <Card class="mb-4 gap-0 overflow-hidden py-0">
      <CardHeader
        class="divider-edge-b divider-edge-full bg-muted/40 px-4 py-2.5 [.border-b]:pb-2.5"
      >
        <div class="flex items-center gap-2">
          <CardTitle class="text-sm font-medium">AI verification</CardTitle>
          {#if verification}
            <span class="text-xs text-muted-foreground"
              >process #{verification.processId} · {verification.status}</span
            >
          {/if}
          {#if verification?.gated}
            <Badge
              variant="outline"
              class="bg-status-warning-bg text-status-warning border-status-warning/30"
              >paused at a review gate</Badge
            >
          {/if}
          <Button
            variant="outline"
            size="sm"
            onclick={() => {
              verification = null;
              verifyErr = '';
            }}
            class="ml-auto">Dismiss</Button
          >
        </div>
      </CardHeader>
      <CardContent class="px-4 py-3">
        <p class="mb-3 text-xs text-muted-foreground">
          Advisory only — the agents review deletion candidates but never delete or change risk. You
          still tick every item.
        </p>
        {#if verification && verification.verdicts.length > 0}
          <div class="mb-3 flex flex-wrap items-center gap-3">
            <span class="text-sm text-foreground">
              <span class="text-status-success">{verifyCounts.safe} safe</span> ·
              <span class="text-status-warning">{verifyCounts.review} review</span>
              · <span class="text-status-error">{verifyCounts.dangerous} dangerous</span>
            </span>
            {#if verifyCounts.review + verifyCounts.dangerous > 0}
              <Button variant="outline" size="sm" onclick={deselectFlagged}
                >Deselect {verifyCounts.review + verifyCounts.dangerous} flagged</Button
              >
            {/if}
          </div>
        {/if}
        {#if verifyErr}
          <Alert variant="destructive" class="mb-3">
            <AlertDescription class="whitespace-pre-wrap">{verifyErr}</AlertDescription>
          </Alert>
        {/if}
        {#if verification}
          {#if verification.notes.length === 0}
            <div class="text-sm text-muted-foreground">
              No agent notes returned{verification.gated ? ' before the gate.' : '.'}
            </div>
          {:else}
            <div class="space-y-3">
              {#each verification.notes as n (n.taskId)}
                <Card class="gap-0 py-3">
                  <CardContent class="px-3">
                    <div class="mb-1.5 flex items-center gap-2">
                      <span class="text-xs font-medium text-foreground">{n.role}</span>
                      <span class="text-xs text-muted-foreground">#{n.taskId} · {n.status}</span>
                    </div>
                    <div class="whitespace-pre-wrap break-words text-sm text-muted-foreground">
                      {n.output}
                    </div>
                  </CardContent>
                </Card>
              {/each}
            </div>
          {/if}
        {/if}
      </CardContent>
    </Card>
  {/if}

  <Card class="gap-0 overflow-hidden py-0">
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead class="w-8"></TableHead>
          <TableHead>Path</TableHead>
          <TableHead>Kind</TableHead>
          <TableHead>Reason</TableHead>
          <TableHead class="text-right">Size</TableHead>
          <TableHead>Risk</TableHead>
          {#if verdictByPath.size > 0}<TableHead>AI</TableHead>{/if}
        </TableRow>
      </TableHeader>
      <TableBody>
        {#each rows as { node, depth, isFolder } (node.fullPath + (node.proposal?.id ?? ''))}
          {@const sel = rowSel(node)}
          {@const p = node.proposal}
          <TableRow>
            <TableCell class="align-top">
              <Checkbox
                checked={sel.all}
                indeterminate={sel.some}
                onCheckedChange={() => toggleSel(sel.ids, !sel.all)}
              />
            </TableCell>
            <TableCell class="break-all align-top font-mono text-xs">
              <div class="flex items-start" style="padding-left: {depth * 16}px">
                {#if isFolder}
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    class="mr-1.5 mt-px h-4 w-4 shrink-0 p-0 text-muted-foreground"
                    onclick={() => toggleExp(node.fullPath)}
                    >{expanded.has(node.fullPath) ? '▾' : '▸'}</Button
                  >
                {:else}
                  <span class="mr-1.5 w-4 shrink-0 text-muted-foreground/50">·</span>
                {/if}
                <span class={isFolder ? 'text-foreground' : ''}>
                  {node.name}
                  {#if isFolder}<span class="ml-2 text-muted-foreground"
                      >{node.count} item{node.count === 1 ? '' : 's'}</span
                    >{/if}
                </span>
              </div>
            </TableCell>
            <TableCell class="align-top text-muted-foreground">{p ? p.kind : ''}</TableCell>
            <TableCell class="align-top text-muted-foreground">{p ? p.reason : ''}</TableCell>
            <TableCell class="whitespace-nowrap text-right align-top"
              >{fmtBytes(node.sizeBytes)}</TableCell
            >
            <TableCell class="align-top">
              {#if p}<Badge variant="outline" class={RISK_BADGE[p.risk]}>{p.risk}</Badge>{/if}
            </TableCell>
            {#if verdictByPath.size > 0}
              <TableCell class="align-top">
                {#if p && verdictByPath.has(p.path)}
                  {@const v = verdictByPath.get(p.path)!}
                  <Badge variant="outline" class={VERDICT_BADGE[v.verdict]} title={v.reason}
                    >{v.verdict}</Badge
                  >
                {/if}
              </TableCell>
            {/if}
          </TableRow>
        {/each}
        {#if summary.proposals.length === 0}
          <TableRow>
            <TableCell
              colspan={verdictByPath.size > 0 ? 7 : 6}
              class="py-10 text-center text-muted-foreground"
            >
              No cleanup proposals. Disk looks tidy.
            </TableCell>
          </TableRow>
        {/if}
      </TableBody>
    </Table>
  </Card>
{/if}
