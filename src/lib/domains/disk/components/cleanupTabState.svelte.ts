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
import { buildTree, flattenTree, folderPaths, fmtBytes, fmtDate, verdictMap } from '../utils';
import { confirmAction } from '$lib/utils/confirm';
import { backgroundJobs } from '$lib/domains/shared/jobs/backgroundJobs.svelte';

type Pending = { tab: string; path?: string; seq: number } | null;

/**
 * Scan / select / verify / quarantine state behind CleanupTab. Module-level
 * singleton: it all survives navigating away from /utilities/disk.
 */
class CleanupTabState {
  root = $state('');
  busy = $state(false);
  progress = $state<ScanProgress | null>(null);
  qProgress = $state<QuarantineProgress | null>(null);
  summary = $state<ScanSummary | null>(null);
  selected = $state<Set<string>>(new Set());
  expanded = $state<Set<string>>(new Set());
  status = $state('');
  cached = $state<CachedScan | null>(null);
  permanentDelete = $state(false);
  locations = $state<Location[]>([]);
  verifying = $state(false);
  verifyProgress = $state<VerifyProgress | null>(null);
  verification = $state<VerificationResult | null>(null);
  verifyErr = $state('');

  verdictByPath = $derived(
    this.verification ? verdictMap(this.verification.verdicts) : new Map<string, ItemVerdict>()
  );

  selectedBytes = $derived.by(() => {
    if (!this.summary) return 0;
    return this.summary.proposals
      .filter((p) => this.selected.has(p.id))
      .reduce((a, p) => a + p.sizeBytes, 0);
  });

  tree = $derived(this.summary ? buildTree(this.summary.proposals, this.summary.root) : []);
  rows = $derived(flattenTree(this.tree, this.expanded));

  /** What the effects below already handled, so a remount doesn't redo it. */
  private expandedFor: ScanSummary | null = null;
  private jumpSeen = 0;
  /** Latest scan wins: a deep link can start one while another is still running. */
  private run = 0;

  /** The `$effect`s live and die with the component: call during its init. */
  watch(getPending: () => Pending) {
    $effect(() => {
      if (this.summary !== this.expandedFor) {
        this.expandedFor = this.summary;
        this.expanded = new Set(folderPaths(this.tree));
      }
    });

    $effect(() => {
      const dir = this.root;
      if (!dir) {
        this.cached = null;
        return;
      }
      let stale = false;
      invoke<CachedScan | null>('get_cached_scan', { root: dir })
        .then((c) => !stale && (this.cached = c))
        .catch(() => !stale && (this.cached = null));
      return () => {
        stale = true;
      };
    });

    $effect(() => {
      const pending = getPending();
      if (pending && pending.tab === 'cleanup' && pending.path && pending.seq !== this.jumpSeen) {
        this.jumpSeen = pending.seq;
        this.root = pending.path;
        void this.scan(pending.path);
      }
    });
  }

  /** Load locations and subscribe to progress events; returns the teardown. */
  init = () => {
    invoke<Location[]>('list_locations')
      .then((l) => (this.locations = l))
      .catch(() => (this.locations = []));
    const offs = [
      listen<ScanProgress>('scan://progress', (e) => (this.progress = e.payload)),
      listen<QuarantineProgress>('quarantine://progress', (e) => (this.qProgress = e.payload)),
      listen<VerifyProgress>('verify://progress', (e) => (this.verifyProgress = e.payload)),
    ];
    return () => offs.forEach((p) => p.then((off) => off()));
  };

  private async refreshCached(target?: string) {
    const dir = target ?? this.root;
    if (!dir) return (this.cached = null);
    try {
      this.cached = await invoke<CachedScan | null>('get_cached_scan', { root: dir });
    } catch {
      this.cached = null;
    }
  }

  loadCached = () => {
    const { cached } = this;
    if (!cached) return;
    this.summary = cached.summary;
    this.selected = new Set();
    this.status =
      cached.status === 'partial'
        ? `Loaded interrupted scan from ${fmtDate(cached.ts)} — partial results. Resume for a full scan.`
        : `Loaded cached scan from ${fmtDate(cached.ts)} — rescan for fresh results.`;
  };

  discardCached = async () => {
    if (!this.root) return;
    try {
      await invoke('remove_cached_scan', { root: this.root });
      this.cached = null;
      this.status = 'Discarded saved scan.';
    } catch (e) {
      this.status = `Discard failed: ${String(e)}`;
    }
  };

  scan = async (target?: string) => {
    const dir = target ?? this.root;
    if (!dir) return;
    if (target) this.root = target;
    const run = ++this.run;
    this.busy = true;
    this.status = 'Scanning…';
    this.progress = null;
    this.selected = new Set();
    backgroundJobs.upsert({
      id: 'scan',
      kind: 'scan',
      title: `Scanning ${dir.split(/[\\/]/).filter(Boolean).pop() ?? dir}`,
      detail: 'Starting…',
      percent: null,
      href: '/utilities/disk',
      cancel: () => void this.cancelScan(),
    });
    try {
      const res = await invoke<ScanSummary>('scan_directory', { root: dir });
      if (run !== this.run) return;
      this.summary = res;
      this.status = `Scanned ${res.scannedFiles.toLocaleString()} files in ${res.elapsedMs} ms · ${res.proposals.length} proposals`;
      void this.refreshCached(dir);
    } catch (e) {
      if (run !== this.run) return;
      this.status = String(e) === 'cancelled' ? 'Scan cancelled' : `Scan failed: ${String(e)}`;
      if (String(e) === 'cancelled') void this.refreshCached(dir);
    } finally {
      if (run === this.run) {
        this.busy = false;
        this.progress = null;
        backgroundJobs.remove('scan');
      }
    }
  };

  cancelScan = async () => {
    await invoke('cancel_scan');
    this.status = 'Cancelling…';
  };

  toggleSel = (ids: string[], on: boolean) => {
    const next = new Set(this.selected);
    for (const id of ids) on ? next.add(id) : next.delete(id);
    this.selected = next;
  };

  toggleExp = (path: string) => {
    const next = new Set(this.expanded);
    next.has(path) ? next.delete(path) : next.add(path);
    this.expanded = next;
  };

  selectSafe = () => {
    if (!this.summary) return;
    this.selected = new Set(
      this.summary.proposals.filter((p) => p.risk === 'Safe').map((p) => p.id)
    );
  };

  clearSelection = () => (this.selected = new Set());
  expandAll = () => (this.expanded = new Set(folderPaths(this.tree)));
  collapseAll = () => (this.expanded = new Set());

  dismissVerification = () => {
    this.verification = null;
    this.verifyErr = '';
  };

  verifyWithAi = async () => {
    const { summary } = this;
    if (!summary) return;
    const toVerify =
      this.selected.size > 0
        ? summary.proposals.filter((p) => this.selected.has(p.id))
        : summary.proposals;
    if (toVerify.length === 0) return;
    this.verifying = true;
    this.verifyErr = '';
    this.verification = null;
    this.verifyProgress = null;
    try {
      this.verification = await invoke<VerificationResult>('verify_proposals', {
        root: summary.root,
        proposals: toVerify,
      });
    } catch (e) {
      const msg = String(e);
      if (/cancelled/i.test(msg)) this.verifyErr = '';
      else
        this.verifyErr = /scope/i.test(msg)
          ? `${msg} — mint a token with process:read + process:write (Settings).`
          : msg;
    } finally {
      this.verifying = false;
      this.verifyProgress = null;
    }
  };

  cancelVerify = () => {
    invoke('cancel_verify').catch(() => {});
  };

  deselectFlagged = () => {
    const { verification, summary } = this;
    if (!verification || !summary) return;
    const flagged = new Set(
      verification.verdicts.filter((v) => v.verdict !== 'safe').map((v) => v.path)
    );
    if (flagged.size === 0) return;
    const ids = new Set(summary.proposals.filter((p) => flagged.has(p.path)).map((p) => p.id));
    const next = new Set(this.selected);
    ids.forEach((id) => next.delete(id));
    this.selected = next;
  };

  quarantine = async () => {
    const { summary } = this;
    if (!summary || this.selected.size === 0) return;
    const proposals = summary.proposals.filter((p) => this.selected.has(p.id));
    const items = proposals.map((p) => ({
      path: p.path,
      kind: p.kind,
      permanent: this.permanentDelete && p.risk === 'Safe',
    }));
    const permCount = items.filter((i) => i.permanent).length;
    const binCount = items.length - permCount;
    const ok = await confirmAction(
      permCount > 0
        ? `Remove ${items.length} item(s) (${fmtBytes(this.selectedBytes)})?\n\n${permCount} Safe item${permCount === 1 ? '' : 's'} will be permanently deleted — not recoverable.${binCount ? ` ${binCount} other item${binCount === 1 ? '' : 's'} go to the Recycle Bin.` : ''}`
        : `Move ${items.length} item(s) (${fmtBytes(this.selectedBytes)}) to the Recycle Bin?\n\nNothing is permanently deleted — you can restore from the Recycle Bin.`,
      permCount > 0 ? 'Remove selected' : 'Move to Recycle Bin',
      { confirmLabel: permCount > 0 ? 'Delete' : 'Move' }
    );
    if (!ok) return;
    this.busy = true;
    this.qProgress = { done: 0, total: items.length, currentPath: '' };
    this.status = permCount > 0 ? 'Removing…' : 'Moving to Recycle Bin…';
    try {
      const res = await invoke<QuarantineResult>('quarantine_paths', { items });
      this.status = `Reclaimed ${fmtBytes(res.reclaimedBytes)} · ${res.moved.length} moved · ${res.failed.length} failed`;
      this.summary = {
        ...summary,
        proposals: summary.proposals.filter((p) => !this.selected.has(p.id)),
      };
      this.selected = new Set();
    } catch (e) {
      this.status = `Quarantine failed: ${String(e)}`;
    } finally {
      this.busy = false;
      this.qProgress = null;
    }
  };
}

export const cleanupTab = new CleanupTabState();
