<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import type {
    CachedScan,
    ItemVerdict,
    Location,
    Proposal,
    QuarantineProgress,
    QuarantineResult,
    ScanProgress,
    ScanSummary,
    VerificationResult,
    VerifyProgress,
  } from "../types";
  import {
    buildTree,
    flattenTree,
    folderPaths,
    fmtBytes,
    fmtDate,
    fmtDuration,
    fmtAgo,
    leafIds,
    loadAiConfig,
    RISK_BADGE,
    VERDICT_BADGE,
    verdictMap,
    type TreeNode,
  } from "../utils";

  let { pending }: { pending: { tab: string; path?: string; seq: number } | null } = $props();

  let root = $state("");
  let busy = $state(false);
  let progress = $state<ScanProgress | null>(null);
  let qProgress = $state<QuarantineProgress | null>(null);
  let summary = $state<ScanSummary | null>(null);
  let selected = $state<Set<string>>(new Set());
  let expanded = $state<Set<string>>(new Set());
  let status = $state("");
  let cached = $state<CachedScan | null>(null);
  let locations = $state<Location[]>([]);
  let verifying = $state(false);
  let verifyProgress = $state<VerifyProgress | null>(null);
  let verification = $state<VerificationResult | null>(null);
  let verifyErr = $state("");

  const btnPrimary =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-white text-black text-sm font-medium transition-colors hover:bg-neutral-200 disabled:opacity-40 disabled:pointer-events-none";
  const btnSecondary =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-neutral-900 text-neutral-200 text-sm font-medium border border-neutral-800 transition-colors hover:bg-neutral-800 hover:border-neutral-700 disabled:opacity-40 disabled:pointer-events-none";
  const btnDanger =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-red-600 text-white text-sm font-medium transition-colors hover:bg-red-500 disabled:opacity-40 disabled:pointer-events-none";
  const btnGhostSm =
    "inline-flex items-center justify-center h-7 px-2.5 rounded-md text-xs font-medium text-neutral-300 border border-neutral-800 bg-neutral-950 transition-colors hover:bg-neutral-900 hover:text-white disabled:opacity-40 disabled:pointer-events-none";
  const inputCls =
    "h-9 px-3 rounded-md bg-neutral-950 border border-neutral-800 text-sm text-neutral-100 placeholder:text-neutral-600 transition-colors focus:outline-none focus:border-neutral-600";

  onMount(() => {
    invoke<Location[]>("list_locations").then((l) => (locations = l)).catch(() => (locations = []));
    const offs = [
      listen<ScanProgress>("scan://progress", (e) => (progress = e.payload)),
      listen<QuarantineProgress>("quarantine://progress", (e) => (qProgress = e.payload)),
      listen<VerifyProgress>("verify://progress", (e) => (verifyProgress = e.payload)),
    ];
    return () => offs.forEach((p) => p.then((off) => off()));
  });

  const verdictByPath = $derived(verification ? verdictMap(verification.verdicts) : new Map<string, ItemVerdict>());

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

  // Expand everything whenever a fresh tree arrives.
  let lastTreeRef: TreeNode[] | null = null;
  $effect(() => {
    if (tree !== lastTreeRef) {
      lastTreeRef = tree;
      expanded = new Set(folderPaths(tree));
    }
  });

  // Restore card: surface any cached scan for the chosen root.
  $effect(() => {
    const dir = root;
    if (!dir) {
      cached = null;
      return;
    }
    let stale = false;
    invoke<CachedScan | null>("get_cached_scan", { root: dir })
      .then((c) => !stale && (cached = c))
      .catch(() => !stale && (cached = null));
    return () => {
      stale = true;
    };
  });

  // Deep-link auto-scan from the dashboard.
  let jumpSeen = 0;
  $effect(() => {
    if (pending && pending.tab === "cleanup" && pending.path && pending.seq !== jumpSeen) {
      jumpSeen = pending.seq;
      root = pending.path;
      void scan(pending.path);
    }
  });

  async function refreshCached(target?: string) {
    const dir = target ?? root;
    if (!dir) return (cached = null);
    try {
      cached = await invoke<CachedScan | null>("get_cached_scan", { root: dir });
    } catch {
      cached = null;
    }
  }

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === "string") root = dir;
  }

  function loadCached() {
    if (!cached) return;
    summary = cached.summary;
    selected = new Set();
    status =
      cached.status === "partial"
        ? `Loaded interrupted scan from ${fmtDate(cached.ts)} — partial results. Resume for a full scan.`
        : `Loaded cached scan from ${fmtDate(cached.ts)} — rescan for fresh results.`;
  }

  async function discardCached() {
    if (!root) return;
    try {
      await invoke("remove_cached_scan", { root });
      cached = null;
      status = "Discarded saved scan.";
    } catch (e) {
      status = `Discard failed: ${String(e)}`;
    }
  }

  async function scan(target?: string) {
    const dir = target ?? root;
    if (!dir) return;
    if (target) root = target;
    busy = true;
    status = "Scanning…";
    progress = null;
    selected = new Set();
    try {
      const res = await invoke<ScanSummary>("scan_directory", { root: dir });
      summary = res;
      status = `Scanned ${res.scannedFiles.toLocaleString()} files in ${res.elapsedMs} ms · ${res.proposals.length} proposals`;
      void refreshCached(dir);
    } catch (e) {
      status = String(e) === "cancelled" ? "Scan cancelled" : `Scan failed: ${String(e)}`;
      if (String(e) === "cancelled") void refreshCached(dir);
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

  function toggleExp(path: string) {
    const next = new Set(expanded);
    next.has(path) ? next.delete(path) : next.add(path);
    expanded = next;
  }

  function selectSafe() {
    if (!summary) return;
    selected = new Set(summary.proposals.filter((p) => p.risk === "Safe").map((p) => p.id));
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
    verifyErr = "";
    verification = null;
    verifyProgress = null;
    try {
      verification = await invoke<VerificationResult>("verify_proposals", {
        root: summary.root,
        proposals: toVerify,
        config: loadAiConfig(),
      });
    } catch (e) {
      const msg = String(e);
      if (/cancelled/i.test(msg)) verifyErr = "";
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
    invoke("cancel_verify").catch(() => {});
  }

  function deselectFlagged() {
    if (!verification || !summary) return;
    const flagged = new Set(verification.verdicts.filter((v) => v.verdict !== "safe").map((v) => v.path));
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
    const ok = window.confirm(
      `Move ${items.length} item(s) (${fmtBytes(selectedBytes)}) to the Recycle Bin?\n\nNothing is permanently deleted — you can restore from the Recycle Bin.`,
    );
    if (!ok) return;
    busy = true;
    qProgress = { done: 0, total: items.length, currentPath: "" };
    status = "Moving to Recycle Bin…";
    try {
      const res = await invoke<QuarantineResult>("quarantine_paths", { items });
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

  const scanPct = $derived(
    progress && progress.phase !== "counting" && progress.totalFiles > 0
      ? Math.min(100, (progress.scannedFiles / progress.totalFiles) * 100)
      : 0,
  );
  const restorePct = $derived(
    cached && cached.status === "partial" && cached.totalFiles > 0
      ? Math.min(100, Math.round((cached.scannedFiles / cached.totalFiles) * 100))
      : 100,
  );
  const restoreReclaimable = $derived(cached ? cached.summary.proposals.reduce((a, p) => a + p.sizeBytes, 0) : 0);
  const verifyCounts = $derived.by(() => {
    const c = { safe: 0, review: 0, dangerous: 0 };
    verification?.verdicts.forEach((v) => c[v.verdict]++);
    return c;
  });
</script>

<div class="mb-4 flex items-center gap-2">
  <button onclick={pickFolder} class={btnSecondary}>Choose folder</button>
  <input bind:value={root} placeholder="C:\Users\you\Downloads" class="flex-1 {inputCls}" />
  <button onclick={() => scan()} disabled={busy || !root} class={btnPrimary}>{busy ? "Working…" : "Scan"}</button>
  {#if busy}
    <button onclick={cancelScan} class={btnDanger}>Stop</button>
  {/if}
</div>

{#if !summary && locations.length > 0}
  <div class="mb-6">
    <div class="mb-2 text-xs uppercase tracking-wide text-neutral-600">Suggested — drives &amp; common folders</div>
    <div class="flex flex-wrap gap-2">
      {#each locations as loc (loc.path)}
        <button
          onclick={() => scan(loc.path)}
          disabled={busy}
          title={loc.path}
          class="inline-flex h-9 items-center gap-2 rounded-md border border-neutral-800 bg-neutral-950 px-3 text-sm text-neutral-200 transition-colors hover:border-neutral-700 hover:bg-neutral-900 disabled:pointer-events-none disabled:opacity-40"
        >
          <span class="text-neutral-500">{loc.kind === "drive" ? "▮" : "▸"}</span>
          {loc.label}
        </button>
      {/each}
    </div>
  </div>
{/if}

{#if cached && !summary && !busy}
  {@const partial = cached.status === "partial"}
  <div class="mb-4 overflow-hidden rounded-xl border bg-neutral-950/60 {partial ? 'border-amber-500/30' : 'border-neutral-800'}">
    <div class="flex items-start gap-3 px-4 py-3.5">
      <div class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-sm {partial ? 'bg-amber-500/10 text-amber-400' : 'bg-emerald-500/10 text-emerald-400'}">
        {partial ? "◴" : "↻"}
      </div>
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <span class="text-sm font-medium text-neutral-100">{partial ? "Interrupted scan can be restored" : "Saved scan available"}</span>
          <span class="inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium {partial ? 'border-amber-500/20 bg-amber-500/10 text-amber-400' : 'border-emerald-500/20 bg-emerald-500/10 text-emerald-400'}">
            {partial ? `stopped at ${restorePct}%` : "complete"}
          </span>
        </div>
        <div class="mt-1 text-xs text-neutral-500">
          {fmtAgo(cached.ts)} · {fmtDate(cached.ts)} · {cached.scannedFiles.toLocaleString()}{partial && cached.totalFiles > 0 ? ` / ${cached.totalFiles.toLocaleString()}` : ""} files · {fmtBytes(restoreReclaimable)} reclaimable
        </div>
        {#if partial}
          <div class="mt-2 h-1.5 w-full overflow-hidden rounded-full bg-neutral-900">
            <div class="h-full rounded-full bg-amber-500" style="width: {restorePct}%"></div>
          </div>
        {/if}
        <div class="mt-3 flex flex-wrap items-center gap-2">
          {#if partial}
            <button onclick={() => scan()} class="{btnPrimary} h-8 text-xs">Resume — full scan</button>
            <button onclick={loadCached} class={btnGhostSm}>Load partial results</button>
          {:else}
            <button onclick={loadCached} class="{btnPrimary} h-8 text-xs">Load saved scan</button>
            <button onclick={() => scan()} class={btnGhostSm}>Rescan fresh</button>
          {/if}
          <button onclick={discardCached} class="{btnGhostSm} hover:!border-red-600 hover:!bg-red-600 hover:!text-white">Discard</button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if busy && progress}
  {@const counting = progress.phase === "counting"}
  <div class="mb-4 rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-2 flex items-center justify-between text-sm">
      <span class="text-neutral-300">
        {counting ? "Counting files…" : "Scanning"}
        {#if !counting}
          <span class="text-neutral-500"> · {progress.scannedFiles.toLocaleString()} / {progress.totalFiles.toLocaleString()} · {fmtBytes(progress.totalBytes)}</span>
        {:else}
          <span class="text-neutral-500"> · {progress.scannedFiles.toLocaleString()} found</span>
        {/if}
      </span>
      <span class="tabular-nums text-neutral-400">
        {counting ? fmtDuration(progress.elapsedMs) : `${scanPct.toFixed(0)}% · ETA ${fmtDuration(progress.etaMs)}`}
      </span>
    </div>
    <div class="h-2 w-full overflow-hidden rounded-full bg-neutral-900">
      {#if counting}
        <div class="h-full w-1/3 animate-pulse rounded-full bg-white/60"></div>
      {:else}
        <div class="h-full rounded-full bg-white transition-[width] duration-150 ease-linear" style="width: {scanPct}%"></div>
      {/if}
    </div>
    <div class="mt-1.5 truncate font-mono text-xs text-neutral-600" title={progress.currentPath}>{progress.currentPath}</div>
  </div>
{/if}

{#if qProgress}
  {@const qpct = qProgress.total > 0 ? Math.min(100, (qProgress.done / qProgress.total) * 100) : 0}
  <div class="mb-4 rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-2 flex items-center justify-between text-sm">
      <span class="text-neutral-300">Moving to Recycle Bin<span class="text-neutral-500"> · {qProgress.done} / {qProgress.total}</span></span>
      <span class="tabular-nums text-neutral-400">{qpct.toFixed(0)}%</span>
    </div>
    <div class="h-2 w-full overflow-hidden rounded-full bg-neutral-900">
      <div class="h-full rounded-full bg-white transition-[width] duration-150 ease-linear" style="width: {qpct}%"></div>
    </div>
    <div class="mt-1.5 truncate font-mono text-xs text-neutral-600" title={qProgress.currentPath}>{qProgress.currentPath}</div>
  </div>
{/if}

{#if status && !(busy && progress) && !qProgress}
  <div class="mb-4 text-sm text-neutral-500">{status}</div>
{/if}

{#if summary && stats}
  <div class="mb-4 grid grid-cols-2 gap-3 md:grid-cols-4">
    {#each [["Reclaimable", stats.total, `${stats.count} proposal${stats.count === 1 ? "" : "s"}`, "text-white"], ["Safe", stats.byRisk.Safe.b, `${stats.byRisk.Safe.n} items`, "text-emerald-400"], ["Review", stats.byRisk.Review.b, `${stats.byRisk.Review.n} items`, "text-amber-400"], ["Danger", stats.byRisk.Danger.b, `${stats.byRisk.Danger.n} items`, "text-red-400"]] as [label, bytes, sub, accent]}
      <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
        <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">{label}</div>
        <div class="text-2xl font-semibold tabular-nums tracking-tight {accent}">{fmtBytes(bytes as number)}</div>
        <div class="mt-1 text-xs text-neutral-600">{sub}</div>
      </div>
    {/each}
  </div>

  {#if stats.total > 0}
    <div class="mb-6 flex h-2 w-full overflow-hidden rounded-full bg-neutral-900">
      {#each [["Safe", stats.byRisk.Safe.b, "bg-emerald-500"], ["Review", stats.byRisk.Review.b, "bg-amber-500"], ["Danger", stats.byRisk.Danger.b, "bg-red-500"]] as [risk, bytes, color]}
        {#if (bytes as number) > 0}
          <div class={color as string} style="width: {((bytes as number) / stats.total) * 100}%" title="{risk}"></div>
        {/if}
      {/each}
    </div>
  {/if}

  <div class="mb-3 flex items-center gap-2">
    <button onclick={selectSafe} class={btnGhostSm}>Select all Safe</button>
    <button onclick={() => (selected = new Set())} class={btnGhostSm}>Clear</button>
    <button onclick={() => (expanded = new Set(folderPaths(tree)))} class={btnGhostSm}>Expand all</button>
    <button onclick={() => (expanded = new Set())} class={btnGhostSm}>Collapse all</button>
    <button onclick={verifyWithAi} disabled={verifying} class={btnGhostSm}>
      {verifying ? "Verifying…" : selected.size > 0 ? `Verify ${selected.size} selected with AI` : "Verify with AI"}
    </button>
    {#if verifying}
      <button onclick={cancelVerify} class={btnGhostSm}>Stop</button>
    {/if}
    <div class="ml-auto text-sm text-neutral-400">
      {selected.size} selected · <span class="font-semibold text-white">{fmtBytes(selectedBytes)}</span>
    </div>
    <button onclick={quarantine} disabled={busy || selected.size === 0} class="h-7 px-3 {btnDanger} text-xs">Move to Recycle Bin</button>
  </div>

  {#if verifying && verifyProgress}
    <div class="mb-4 overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
      <div class="flex items-center gap-2 border-b border-neutral-900 bg-neutral-900/40 px-4 py-2.5">
        <span class="text-sm font-medium text-neutral-200">AI verification</span>
        <span class="text-xs text-neutral-500">process #{verifyProgress.processId} · {verifyProgress.status.replace(/_/g, " ")}</span>
        <button onclick={cancelVerify} class="ml-auto {btnGhostSm}">Stop</button>
      </div>
      <div class="px-4 py-3">
        {#if verifyProgress.tasks.length === 0}
          <div class="flex items-center gap-2 text-sm text-neutral-500"><span class="h-1.5 w-1.5 animate-pulse rounded-full bg-white"></span>Planning the review…</div>
        {:else}
          <ul class="space-y-1.5">
            {#each verifyProgress.tasks as t, i (i)}
              <li class="flex items-center gap-2 text-sm text-neutral-300">
                <span class="h-1.5 w-1.5 rounded-full {t.status === 'completed' ? 'bg-emerald-400' : t.status === 'failed' ? 'bg-red-400' : t.status.includes('progress') || t.status === 'running' ? 'bg-white animate-pulse' : 'bg-neutral-600'}"></span>
                <span>{t.role}</span>
                <span class="text-xs text-neutral-600">{t.status.replace(/_/g, " ")}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>
  {/if}

  {#if verifyErr || verification}
    <div class="mb-4 overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
      <div class="flex items-center gap-2 border-b border-neutral-900 bg-neutral-900/40 px-4 py-2.5">
        <span class="text-sm font-medium text-neutral-200">AI verification</span>
        {#if verification}
          <span class="text-xs text-neutral-500">process #{verification.processId} · {verification.status}</span>
        {/if}
        {#if verification?.gated}
          <span class="inline-flex items-center rounded-full border border-amber-500/20 bg-amber-500/10 px-2 py-0.5 text-xs font-medium text-amber-400">paused at a review gate</span>
        {/if}
        <button onclick={() => { verification = null; verifyErr = ""; }} class="ml-auto {btnGhostSm}">Dismiss</button>
      </div>
      <div class="px-4 py-3">
        <p class="mb-3 text-xs text-neutral-500">Advisory only — the agents review deletion candidates but never delete or change risk. You still tick every item.</p>
        {#if verification && verification.verdicts.length > 0}
          <div class="mb-3 flex flex-wrap items-center gap-3">
            <span class="text-sm text-neutral-300">
              <span class="text-emerald-400">{verifyCounts.safe} safe</span> · <span class="text-amber-400">{verifyCounts.review} review</span> · <span class="text-red-400">{verifyCounts.dangerous} dangerous</span>
            </span>
            {#if verifyCounts.review + verifyCounts.dangerous > 0}
              <button onclick={deselectFlagged} class={btnGhostSm}>Deselect {verifyCounts.review + verifyCounts.dangerous} flagged</button>
            {/if}
          </div>
        {/if}
        {#if verifyErr}
          <div class="whitespace-pre-wrap text-sm text-red-400">{verifyErr}</div>
        {/if}
        {#if verification}
          {#if verification.notes.length === 0}
            <div class="text-sm text-neutral-500">No agent notes returned{verification.gated ? " before the gate." : "."}</div>
          {:else}
            <div class="space-y-3">
              {#each verification.notes as n (n.taskId)}
                <div class="rounded-lg border border-neutral-800 bg-neutral-950/60 p-3">
                  <div class="mb-1.5 flex items-center gap-2">
                    <span class="text-xs font-medium text-neutral-300">{n.role}</span>
                    <span class="text-xs text-neutral-600">#{n.taskId} · {n.status}</span>
                  </div>
                  <div class="whitespace-pre-wrap break-words text-sm text-neutral-400">{n.output}</div>
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </div>
    </div>
  {/if}

  <div class="overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
    <table class="w-full text-sm">
      <thead class="bg-neutral-900/50 text-left text-xs uppercase tracking-wide text-neutral-500">
        <tr>
          <th class="w-8 px-3 py-2.5"></th>
          <th class="px-3 py-2.5 font-medium">Path</th>
          <th class="px-3 py-2.5 font-medium">Kind</th>
          <th class="px-3 py-2.5 font-medium">Reason</th>
          <th class="px-3 py-2.5 text-right font-medium">Size</th>
          <th class="px-3 py-2.5 font-medium">Risk</th>
          {#if verdictByPath.size > 0}<th class="px-3 py-2.5 font-medium">AI</th>{/if}
        </tr>
      </thead>
      <tbody>
        {#each rows as { node, depth, isFolder } (node.fullPath + (node.proposal?.id ?? ""))}
          {@const sel = rowSel(node)}
          {@const p = node.proposal}
          <tr class="border-t border-neutral-900 transition-colors hover:bg-neutral-900/40">
            <td class="px-3 py-2 align-top">
              <input type="checkbox" class="accent-white" checked={sel.all} indeterminate={sel.some} onchange={() => toggleSel(sel.ids, !sel.all)} />
            </td>
            <td class="break-all px-3 py-2 font-mono text-xs text-neutral-300">
              <div class="flex items-start" style="padding-left: {depth * 16}px">
                {#if isFolder}
                  <button onclick={() => toggleExp(node.fullPath)} class="mr-1.5 mt-px w-4 shrink-0 text-neutral-500 hover:text-white">{expanded.has(node.fullPath) ? "▾" : "▸"}</button>
                {:else}
                  <span class="mr-1.5 w-4 shrink-0 text-neutral-700">·</span>
                {/if}
                <span class={isFolder ? "text-neutral-200" : ""}>
                  {node.name}
                  {#if isFolder}<span class="ml-2 text-neutral-600">{node.count} item{node.count === 1 ? "" : "s"}</span>{/if}
                </span>
              </div>
            </td>
            <td class="px-3 py-2 align-top text-neutral-400">{p ? p.kind : ""}</td>
            <td class="px-3 py-2 align-top text-neutral-500">{p ? p.reason : ""}</td>
            <td class="whitespace-nowrap px-3 py-2 text-right align-top text-neutral-300">{fmtBytes(node.sizeBytes)}</td>
            <td class="px-3 py-2 align-top">
              {#if p}<span class="inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium {RISK_BADGE[p.risk]}">{p.risk}</span>{/if}
            </td>
            {#if verdictByPath.size > 0}
              <td class="px-3 py-2 align-top">
                {#if p && verdictByPath.has(p.path)}
                  {@const v = verdictByPath.get(p.path)!}
                  <span title={v.reason} class="inline-flex cursor-help items-center rounded-full border px-2 py-0.5 text-xs font-medium {VERDICT_BADGE[v.verdict]}">{v.verdict}</span>
                {/if}
              </td>
            {/if}
          </tr>
        {/each}
        {#if summary.proposals.length === 0}
          <tr><td colspan={verdictByPath.size > 0 ? 7 : 6} class="px-3 py-10 text-center text-neutral-600">No cleanup proposals. Disk looks tidy.</td></tr>
        {/if}
      </tbody>
    </table>
  </div>
{/if}
