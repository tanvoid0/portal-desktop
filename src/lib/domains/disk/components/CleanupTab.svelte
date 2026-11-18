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
    verdictMap,
    type TreeNode,
  } from '../utils';
  import ScanProgressCard from './shared/ScanProgressCard.svelte';
  import TaskProgressCard from './shared/TaskProgressCard.svelte';
  import CleanupStats from './CleanupStats.svelte';
  import TargetPicker from './shared/TargetPicker.svelte';
  import SelectionBar from './shared/SelectionBar.svelte';
  import PermanentDeleteToggle from './shared/PermanentDeleteToggle.svelte';
  import CleanupSavedScanCard from './CleanupSavedScanCard.svelte';
  import CleanupVerifyCards from './CleanupVerifyCards.svelte';
  import CleanupProposalTable from './CleanupProposalTable.svelte';
  import { confirmAction } from '$lib/utils/confirm';
  import { Button } from '$lib/components/ui/button';

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
  let permanentDelete = $state(false);
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
    const proposals = summary.proposals.filter((p) => selected.has(p.id));
    const items = proposals.map((p) => ({
      path: p.path,
      kind: p.kind,
      permanent: permanentDelete && p.risk === 'Safe',
    }));
    const permCount = items.filter((i) => i.permanent).length;
    const binCount = items.length - permCount;
    const ok = await confirmAction(
      permCount > 0
        ? `Remove ${items.length} item(s) (${fmtBytes(selectedBytes)})?\n\n${permCount} Safe item${permCount === 1 ? '' : 's'} will be permanently deleted — not recoverable.${binCount ? ` ${binCount} other item${binCount === 1 ? '' : 's'} go to the Recycle Bin.` : ''}`
        : `Move ${items.length} item(s) (${fmtBytes(selectedBytes)}) to the Recycle Bin?\n\nNothing is permanently deleted — you can restore from the Recycle Bin.`,
      permCount > 0 ? 'Remove selected' : 'Move to Recycle Bin',
      { confirmLabel: permCount > 0 ? 'Delete' : 'Move' }
    );
    if (!ok) return;
    busy = true;
    qProgress = { done: 0, total: items.length, currentPath: '' };
    status = permCount > 0 ? 'Removing…' : 'Moving to Recycle Bin…';
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
  <CleanupSavedScanCard
    {cached}
    onload={loadCached}
    onrescan={() => scan()}
    ondiscard={discardCached}
  />
{/if}

<ScanProgressCard progress={busy ? progress : null} />

<TaskProgressCard progress={qProgress} />

{#if status && !(busy && progress) && !qProgress}
  <div class="mb-4 text-sm text-muted-foreground">{status}</div>
{/if}

{#if summary}
  <CleanupStats proposals={summary.proposals} />

  <SelectionBar
    count={selected.size}
    bytes={selectedBytes}
    disabled={busy}
    onaction={quarantine}
    actionLabel={permanentDelete ? 'Remove selected' : 'Move to Recycle Bin'}
  >
    {#snippet tools()}
      <Button variant="outline" size="sm" onclick={selectSafe}>Select all Safe</Button>
      <Button variant="outline" size="sm" onclick={() => (selected = new Set())}>Clear</Button>
      <PermanentDeleteToggle
        bind:checked={permanentDelete}
        id="cleanup-permanent"
        label="Delete Safe items permanently"
      />
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

  <CleanupVerifyCards
    {verifying}
    progress={verifyProgress}
    {verification}
    error={verifyErr}
    oncancel={cancelVerify}
    ondismiss={() => {
      verification = null;
      verifyErr = '';
    }}
    ondeselectFlagged={deselectFlagged}
  />

  <CleanupProposalTable
    {rows}
    {selected}
    {expanded}
    {verdictByPath}
    empty={summary.proposals.length === 0}
    ontoggleSel={toggleSel}
    ontoggleExp={toggleExp}
  />
{/if}
