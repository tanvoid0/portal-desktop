<!--
	Project Audit — bulk discovery and triage.

	The registry answers "what am I tracking"; this answers "what is actually on
	disk, is it alive, and is any of its work only here". Scanning writes into the
	same projects table, so a project triaged here is the same row the rest of the
	app already uses.
-->

<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Badge } from '$lib/components/ui/badge';
  import * as Tabs from '$lib/components/ui/tabs';
  import { PageStats, ResponsivePanel, type PageStat } from '$lib/components/shell';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import HardDrive from '~icons/fa6-solid/hard-drive';
  import Download from '~icons/fa6-solid/download';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { onDestroy, onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { logger } from '$lib/domains/shared/services/logger';
  import { confirmAction } from '$lib/utils/confirm';
  import { invokeClient } from '$lib/utils/invokeClient';
  // Per-row actions fail while the reader is deep in the list, where the scan
  // card's inline error is scrolled off. Only the scan reports inline.
  import { toast } from '$lib/utils/toast';
  // The audit reports reclaimable bytes; the Disk utility deletes them. Sharing
  // its formatter keeps one number reading the same on both pages.
  import { fmtBytes } from '$lib/domains/disk/utils';
  import {
    cancelDiscovery,
    defaultDiscoveryConfig,
    discoverAndImport,
    DISCOVERY_PROGRESS_EVENT,
    exportDiscoveryCsv,
    getDiscoveryState,
    saveDiscoveryConfig,
    setProjectTriage,
    setProjectTriageBulk,
    type TriageDecision,
  } from '$lib/domains/projects/api/discoveryApi';
  import { toCsv } from '$lib/domains/projects/utils/auditCsv';
  import { DECISIONS } from '$lib/domains/projects/utils/auditFormat';
  import type { DiscoveryProgress } from '$lib/domains/projects/generated/DiscoveryProgress';
  import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
  import type { DiscoveryReport } from '$lib/domains/projects/generated/DiscoveryReport';
  import type { DiscoveryChange } from '$lib/domains/projects/generated/DiscoveryChange';
  import AuditProjectCard from './AuditProjectCard.svelte';
  import AuditChangesList from './AuditChangesList.svelte';
  import AuditDuplicateSets from './AuditDuplicateSets.svelte';
  import AuditTree from './AuditTree.svelte';
  import AuditScanPanel from './AuditScanPanel.svelte';
  import AuditFocusedDetail from './AuditFocusedDetail.svelte';
  import AuditMarkedActions from './AuditMarkedActions.svelte';

  const log = logger.createScoped('ProjectAudit');

  type Tab = 'triage' | 'risk' | 'duplicates' | 'tree' | 'changes';

  let roots = $state<{ name: string; path: string }[]>([]);
  let newRootPath = $state('');
  let report = $state<DiscoveryReport | null>(null);
  let changes = $state<DiscoveryChange[]>([]);
  let measureSizes = $state(false);
  let markMissing = $state(false);
  let scanning = $state(false);
  let progress = $state<DiscoveryProgress | null>(null);
  let error = $state<string | null>(null);
  let tab = $state<Tab>('triage');
  let search = $state('');
  let sortBySize = $state(false);
  let selected = $state<DiscoveredProject | null>(null);
  let decisions = $state<Record<string, TriageDecision | null>>({});
  let idByPath = $state<Record<string, number>>({});
  let importSummary = $state<string | null>(null);
  let exportedTo = $state<string | null>(null);
  /** Saved roots arrive async; without this the card claims "no folders yet" first. */
  let loadingState = $state(true);
  /** Keyboard position in `filtered`. */
  let cursor = $state(0);
  /** Paths ticked for a bulk decision. */
  let marked = $state<Set<string>>(new Set());
  /**
   * The rail holds the scan config, the cursor row's facts and the bulk
   * actions. Below `md` ResponsivePanel renders it as a Sheet, so it needs an
   * open flag and a trigger; on desktop it is always on screen and this is
   * ignored.
   */
  let railOpen = $state(false);
  /**
   * Roots and scan options are set up once and then touched monthly, so the
   * form stays folded behind the Scan button. Opened on mount when there is
   * nothing to scan — folding an empty form would hide the only way out.
   */
  let scanOpen = $state(false);

  let unlisten: UnlistenFn | null = null;
  listen<DiscoveryProgress>(DISCOVERY_PROGRESS_EVENT, (e) => {
    progress = e.payload;
  })
    .then((fn) => (unlisten = fn))
    .catch((e) => log.warn('progress listener failed', e));

  onDestroy(() => unlisten?.());

  // The last scan is on disk, so the view opens populated instead of demanding
  // a fresh walk of every drive before it can show anything.
  onMount(async () => {
    try {
      const state = await getDiscoveryState();
      roots = state.config.roots ?? [];
      scanOpen = roots.length === 0;
      measureSizes = state.config.measureSizes;
      markMissing = state.config.markMissing;
      report = state.report;
      changes = state.changes;
      if (state.report) await loadRegistryIds();
    } catch (e) {
      log.warn('could not load saved audit state', e);
    } finally {
      loadingState = false;
    }
  });

  /** Roots and scan options outlive the session; the report is written by scans. */
  async function persistConfig() {
    try {
      await saveDiscoveryConfig(currentConfig());
    } catch (e) {
      log.warn('could not save audit config', e);
    }
  }

  function currentConfig() {
    return { ...defaultDiscoveryConfig(roots), measureSizes, markMissing };
  }

  const projects = $derived(report?.projects ?? []);

  const filtered = $derived.by(() => {
    const q = search.trim().toLowerCase();
    let list = projects;
    if (tab === 'risk') list = list.filter((p) => p.riskLevel === 'high' || p.riskLevel === 'med');
    if (tab === 'duplicates') list = list.filter((p) => p.duplicateOf.length > 0);
    if (q) {
      list = list.filter((p) =>
        `${p.path} ${p.name} ${p.langs.join(' ')} ${p.description ?? ''}`.toLowerCase().includes(q)
      );
    }
    // Backend order is newest-first; sorting by size answers the other question.
    return sortBySize ? [...list].sort((a, b) => b.regenerableBytes - a.regenerableBytes) : list;
  });

  // A filter change can leave the cursor past the end of the list.
  $effect(() => {
    if (cursor > filtered.length - 1) cursor = Math.max(0, filtered.length - 1);
  });

  const stats = $derived(report?.stats ?? null);
  const undecided = $derived(projects.filter((p) => !decisions[p.path]).length);
  const highRisk = $derived(projects.filter((p) => p.riskLevel === 'high'));

  /**
   * A cluster whose members no longer resolve to a scanned project draws
   * nothing. Dropping those here keeps the tab's empty state honest and stops
   * the "Duplicate sets" stat counting rows the tab will not show.
   */
  const duplicateSets = $derived(
    (report?.clusters ?? [])
      .map((cluster) => ({
        cluster,
        members: cluster.members
          .map((m) => projects.find((p) => p.path === m))
          .filter((p): p is DiscoveredProject => !!p)
          .sort((a, b) => b.activity - a.activity),
      }))
      .filter((set) => set.members.length > 1)
  );

  /**
   * What the rail describes. On the keyboard tabs that is the `j`/`k` cursor
   * row, so the facts follow the loop without a click; the other tabs have no
   * cursor and point `selected` with their own inspect buttons.
   */
  const focused = $derived(
    tab === 'triage' || tab === 'risk' ? (filtered[cursor] ?? null) : selected
  );

  /** `duplicateOf` holds cluster keys, not paths — resolve them to siblings. */
  const duplicateSiblings = $derived.by(() => {
    const p = focused;
    if (!p?.duplicateOf.length) return [];
    const paths = new Set<string>();
    for (const set of duplicateSets) {
      if (!set.members.some((m) => m.path === p.path)) continue;
      for (const m of set.members) if (m.path !== p.path) paths.add(m.path);
    }
    return [...paths];
  });

  /** The six headline numbers as one strip instead of six cards (UX-002). */
  const statStrip = $derived.by((): PageStat[] => {
    if (!stats) return [];
    return [
      { label: 'Projects', value: stats.projects, description: `${stats.roots} roots` },
      {
        label: 'Active',
        value: stats.active,
        description: `${stats.recent} recent · ${stats.dormant} dormant · ${stats.stale} stale`,
      },
      {
        label: 'At risk',
        value: highRisk.length,
        description: `${stats.unpushed} unpushed commits`,
      },
      {
        label: 'Duplicates',
        value: duplicateSets.length,
        description: 'sets sharing a git remote or a folder name',
      },
      { label: 'Undecided', value: undecided, description: `of ${stats.projects}` },
      {
        label: 'Reclaimable',
        value: stats.regenerableBytes < 0 ? '—' : fmtBytes(stats.regenerableBytes),
        description: stats.regenerableBytes < 0 ? 'not measured' : 'build output',
      },
    ];
  });

  /** Uses the same native picker the create-project flow uses. */
  async function addRoot() {
    try {
      const picked =
        newRootPath.trim() || (await invokeClient.post<string | null>('select_directory'));
      if (!picked) return;
      const base =
        picked
          .replace(/[\\/]+$/, '')
          .split(/[\\/]/)
          .pop() ?? 'root';
      if (roots.some((r) => r.path === picked)) return;
      // The name prefixes every project id from this root, and archiving
      // missing projects trusts that prefix to say which root a row came from.
      // Two folders called `production` on different drives would make one
      // root's rows look like the other's, so names are kept unique.
      let name = base;
      for (let n = 2; roots.some((r) => r.name === name); n++) name = `${base}-${n}`;
      roots = [...roots, { name, path: picked }];
      newRootPath = '';
      await persistConfig();
    } catch (e) {
      toast.error('Could not add that folder', e);
    }
  }

  async function removeRoot(path: string) {
    roots = roots.filter((r) => r.path !== path);
    await persistConfig();
  }

  async function scan() {
    if (!roots.length) {
      error = 'Add at least one folder to scan.';
      return;
    }
    scanning = true;
    error = null;
    importSummary = null;
    exportedTo = null;
    progress = null;
    // A selection made against the previous result set would silently apply to
    // whatever those paths mean now, or drop out of the batch without saying so.
    marked = new Set();
    cursor = 0;
    try {
      const result = await discoverAndImport(currentConfig(), markMissing);
      report = result.report;
      changes = [...result.changes, ...changes];
      importSummary =
        `${result.inserted.length} added · ${result.updated.length} refreshed · ` +
        `${result.adopted.length} linked to existing rows` +
        (result.missing.length ? ` · ${result.missing.length} archived as missing` : '') +
        (result.changes.length ? ` · ${result.changes.length} changes since last scan` : '');
      await loadRegistryIds();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error = msg === 'cancelled' ? 'Scan cancelled — nothing was imported.' : msg;
    } finally {
      scanning = false;
      progress = null;
    }
  }

  /** Triage writes by registry id, so map discovery paths back to rows. */
  async function loadRegistryIds() {
    try {
      const rows =
        await invokeClient.post<
          { id: number; discovery_path?: string | null; triage_decision?: string | null }[]
        >('get_all_projects');
      const ids: Record<string, number> = {};
      const dec: Record<string, TriageDecision | null> = {};
      for (const r of rows ?? []) {
        const key = r.discovery_path ?? null;
        if (!key) continue;
        ids[key] = r.id;
        dec[key] = (r.triage_decision as TriageDecision | null) ?? null;
      }
      idByPath = ids;
      decisions = dec;
    } catch (e) {
      log.warn('could not map registry ids', e);
    }
  }

  async function decide(p: DiscoveredProject, decision: TriageDecision) {
    const id = idByPath[p.path];
    if (!id) {
      toast.error('Not in the registry yet', `Run a scan before triaging ${p.name}.`);
      return;
    }
    const next = decisions[p.path] === decision ? null : decision;
    if (next === 'delete') {
      const ok = await confirmAction(
        'This only records the decision — nothing is removed from disk. Use the Disk tab to reclaim space.',
        `Mark ${p.name} for deletion?`
      );
      if (!ok) return;
    }
    decisions = { ...decisions, [p.path]: next };
    try {
      await setProjectTriage(id, next);
    } catch (e) {
      decisions = { ...decisions, [p.path]: decisions[p.path] ?? null };
      toast.error(`Could not save the decision for ${p.name}`, e);
    }
  }

  /** Applies one decision to every ticked project in a single round trip. */
  async function decideMarked(decision: TriageDecision) {
    const paths = [...marked].filter((p) => idByPath[p]);
    if (!paths.length) {
      toast.error(
        'Nothing to apply',
        'None of the selected projects are in the registry yet — run a scan first.'
      );
      return;
    }
    if (decision === 'delete') {
      const ok = await confirmAction(
        'This only records the decision — nothing is removed from disk.',
        `Mark ${paths.length} projects for deletion?`
      );
      if (!ok) return;
    }
    try {
      await setProjectTriageBulk(
        paths.map((p) => idByPath[p]),
        decision
      );
      const next = { ...decisions };
      for (const p of paths) next[p] = decision;
      decisions = next;
      marked = new Set();
      // The selection clearing is the only other sign the bulk write landed.
      toast.success(`${paths.length} marked ${decision}`);
    } catch (e) {
      toast.error(`Could not mark ${paths.length} projects ${decision}`, e);
    }
  }

  function toggleMark(path: string) {
    const next = new Set(marked);
    if (!next.delete(path)) next.add(path);
    marked = next;
  }

  /**
   * The triage loop: `j`/`k` to move, `1`–`4` to decide, `o` to open, `x` to
   * tick for a bulk decision. Only on the card tabs — the tree and the change
   * log have no cursor.
   */
  function onKeydown(e: KeyboardEvent) {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const el = e.target as HTMLElement | null;
    if (el && (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName))) return;
    if (!report || (tab !== 'triage' && tab !== 'risk')) return;
    const current = filtered[cursor];

    switch (e.key) {
      case 'j':
      case 'ArrowDown':
        move(1);
        break;
      case 'k':
      case 'ArrowUp':
        move(-1);
        break;
      case '1':
      case '2':
      case '3':
      case '4':
        if (current) void decide(current, DECISIONS[Number(e.key) - 1]);
        break;
      case 'o':
        if (current) void openFolder(current.abs);
        break;
      case 'x':
        if (current) toggleMark(current.path);
        break;
      case 'Escape':
        marked = new Set();
        selected = null;
        break;
      default:
        return;
    }
    e.preventDefault();
  }

  function move(delta: number) {
    if (!filtered.length) return;
    cursor = Math.min(filtered.length - 1, Math.max(0, cursor + delta));
    selected = filtered[cursor];
    // Moving real focus, not just the highlight: a screen reader follows the
    // cursor, and Tab from here resumes inside the row the reader is looking at.
    const row = document.getElementById(rowId(filtered[cursor].path));
    row?.focus({ preventScroll: true });
    row?.scrollIntoView({ block: 'nearest' });
  }

  /** `/` and `.` are legal in a logical id but not in a DOM id. */
  function rowId(path: string): string {
    return `audit-${path.replace(/[^a-zA-Z0-9]/g, '-')}`;
  }

  async function exportCsv() {
    try {
      const path = await saveDialog({
        defaultPath: 'project-audit.csv',
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });
      if (!path) return;
      exportedTo = await exportDiscoveryCsv(path, toCsv(filtered, decisions));
    } catch (e) {
      toast.error('Export failed', e);
    }
  }

  /**
   * Hands a project to the Disk utility, which owns measuring and deleting.
   * This view records what a project *is*; it never removes bytes itself.
   */
  function cleanInDisk(p: DiscoveredProject) {
    void goto(`/utilities/disk?tab=developer&path=${encodeURIComponent(p.abs)}`);
  }

  async function openFolder(path: string) {
    try {
      await invokeClient.post('open_project_in_explorer', { path });
    } catch (e) {
      toast.error('Could not open that folder', e);
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!--
	The configuration used to sit on top of the results: at 1280x800 the first
	triage row landed at y=676 of a 760px viewport and at 1024x600 none was on
	screen at all (UX-009). Scan config, the cursor row's facts and the bulk
	actions now live in a right rail, which ResponsivePanel folds into a Sheet
	below `md`.
-->
<div class="flex items-start gap-4">
  <div class="min-w-0 flex-1 space-y-3">
    {#if stats}
      <PageStats stats={statStrip} />
    {/if}

    <!-- Once the rail is a Sheet, this button is the only way into it. -->
    <div class="flex items-center gap-2 md:hidden">
      <Button variant="outline" size="sm" onclick={() => (railOpen = true)}>
        <RefreshCw class="mr-1 h-3 w-3" /> Scan and details
      </Button>
      {#if marked.size}
        <Badge variant="secondary">{marked.size} selected</Badge>
      {/if}
    </div>

    {#if !report && !loadingState}
      <p class="py-8 text-center text-sm text-muted-foreground">
        Nothing scanned yet — add a folder and press Scan and import.
      </p>
    {/if}

    {#if report}
      {#snippet cardList()}
        <div class="space-y-2">
          {#each filtered as p, i (p.path)}
            <AuditProjectCard
              project={p}
              id={rowId(p.path)}
              isCursor={i === cursor}
              isSelected={selected?.path === p.path}
              marked={marked.has(p.path)}
              decision={decisions[p.path] ?? null}
              onToggleMark={() => toggleMark(p.path)}
              onSelect={() => {
                cursor = i;
                selected = selected?.path === p.path ? null : p;
              }}
              onDecide={(d) => decide(p, d)}
              onOpen={openFolder}
              onClean={() => cleanInDisk(p)}
            />
          {:else}
            <p class="py-8 text-center text-sm text-muted-foreground">Nothing matches.</p>
          {/each}
        </div>
      {/snippet}

      <Tabs.Root value={tab} onValueChange={(v) => (tab = v as Tab)} class="gap-4">
        <div class="flex flex-wrap items-center gap-2">
          <Tabs.List>
            <Tabs.Trigger value="triage">Triage</Tabs.Trigger>
            <Tabs.Trigger value="risk">At risk</Tabs.Trigger>
            <Tabs.Trigger value="duplicates">Duplicates</Tabs.Trigger>
            <Tabs.Trigger value="tree">Tree</Tabs.Trigger>
            <Tabs.Trigger value="changes">
              Changes{changes.length ? ` (${changes.length})` : ''}
            </Tabs.Trigger>
          </Tabs.List>
          <Button
            size="sm"
            variant={sortBySize ? 'default' : 'outline'}
            onclick={() => (sortBySize = !sortBySize)}
            title="Order by reclaimable build output instead of last activity"
          >
            <HardDrive class="mr-1 h-3 w-3" /> By size
          </Button>
          <Button
            size="sm"
            variant="outline"
            onclick={exportCsv}
            title="Export the current list as CSV"
          >
            <Download class="mr-1 h-3 w-3" /> Export
          </Button>
          <Input
            bind:value={search}
            aria-label="Search the audit list"
            placeholder="Search…"
            class="ml-auto max-w-xs"
          />
        </div>

        {#if tab === 'triage' || tab === 'risk'}
          <p class="text-xs text-muted-foreground">
            <span class="font-mono">j</span>/<span class="font-mono">k</span> move ·
            <span class="font-mono">1</span>–<span class="font-mono">4</span>
            keep/archive/delete/review · <span class="font-mono">o</span> open folder ·
            <span class="font-mono">x</span>
            select ·
            <span class="font-mono">esc</span> clear
          </p>
        {/if}

        {#if exportedTo}
          <p class="font-mono text-xs text-muted-foreground">Written to {exportedTo}</p>
        {/if}

        <Tabs.Content value="changes"><AuditChangesList {changes} /></Tabs.Content>

        <Tabs.Content value="duplicates">
          <AuditDuplicateSets sets={duplicateSets} onInspect={(p) => (selected = p)} />
        </Tabs.Content>

        <Tabs.Content value="tree">
          <AuditTree
            projects={filtered}
            searching={!!search.trim()}
            onInspect={(p) => (selected = p)}
          />
        </Tabs.Content>

        <!-- Same list, different filter — `filtered` narrows it by the active tab. -->
        <Tabs.Content value="triage">{@render cardList()}</Tabs.Content>
        <Tabs.Content value="risk">{@render cardList()}</Tabs.Content>
      </Tabs.Root>
    {/if}
  </div>

  <!--
  	Read-only except for the scan controls, so the keyboard loop is untouched:
  	`onKeydown` already bails when the event target is an input, which covers
  	the path field in here exactly as it covers the search box.
  -->
  <ResponsivePanel
    bind:open={railOpen}
    side="right"
    title="Scan and triage"
    desktopClass="sticky top-0 max-h-[calc(100vh-7rem)] w-80 self-start rounded-lg border bg-card"
    borderClass=""
  >
    <div class="min-h-0 flex-1 space-y-4 overflow-y-auto p-3">
      <AuditScanPanel
        {roots}
        {scanning}
        {loadingState}
        {progress}
        {error}
        {importSummary}
        bind:scanOpen
        bind:newRootPath
        bind:measureSizes
        bind:markMissing
        onScan={scan}
        onCancel={() => cancelDiscovery()}
        onAddRoot={addRoot}
        onRemoveRoot={removeRoot}
        onPersist={persistConfig}
      />
      <AuditFocusedDetail project={focused} siblings={duplicateSiblings} />
      <AuditMarkedActions
        count={marked.size}
        onDecide={decideMarked}
        onClear={() => (marked = new Set())}
      />
    </div>
  </ResponsivePanel>
</div>
