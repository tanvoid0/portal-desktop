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
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import * as Tabs from '$lib/components/ui/tabs';
  import { PageStats, ResponsivePanel, type PageStat } from '$lib/components/shell';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import ShieldAlert from '~icons/fa6-solid/shield-halved';
  import Copy from '~icons/fa6-solid/copy';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import ChevronDown from '~icons/fa6-solid/chevron-down';
  import HardDrive from '~icons/fa6-solid/hard-drive';
  import Download from '~icons/fa6-solid/download';
  import X from '~icons/fa6-solid/xmark';
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
  import type { DiscoveryProgress } from '$lib/domains/projects/generated/DiscoveryProgress';
  import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
  import type { DiscoveryReport } from '$lib/domains/projects/generated/DiscoveryReport';
  import type { DiscoveryChange } from '$lib/domains/projects/generated/DiscoveryChange';

  const log = logger.createScoped('ProjectAudit');

  type Tab = 'triage' | 'risk' | 'duplicates' | 'tree' | 'changes';

  const DECISIONS: TriageDecision[] = ['keep', 'archive', 'delete', 'review'];

  /**
   * `delete` sat in a row of four identical ghost buttons — the confirm dialog
   * only fires after the misclick, so the button has to read as dangerous first.
   */
  const DESTRUCTIVE_CLASS = 'text-destructive hover:text-destructive hover:bg-destructive/10';

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
  let expanded = $state<Record<string, boolean>>({});
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

  function ageLabel(days: number | null): string {
    if (days === null) return 'unknown';
    if (days < 1) return 'today';
    if (days < 30) return `${days}d`;
    if (days < 365) return `${Math.round(days / 30)}mo`;
    return `${(days / 365).toFixed(1)}y`;
  }

  function statusVariant(status: string) {
    return status === 'active' ? 'default' : status === 'recent' ? 'secondary' : 'outline';
  }

  /** Dormant and stale were both plain outline badges; the dash separates the
   * two without spending a second hue on it. */
  function statusClass(status: string) {
    return status === 'stale' ? 'border-dashed opacity-70' : '';
  }

  /** Groups projects by their bucket for the tree, preserving activity order. */
  const tree = $derived.by(() => {
    const byArea = new Map<string, Map<string, DiscoveredProject[]>>();
    for (const p of filtered) {
      if (!byArea.has(p.area)) byArea.set(p.area, new Map());
      const buckets = byArea.get(p.area)!;
      const bucket = p.bucket || p.area;
      if (!buckets.has(bucket)) buckets.set(bucket, []);
      buckets.get(bucket)!.push(p);
    }
    return byArea;
  });

  const percent = $derived(
    progress && progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 0
  );
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
            <!--
          `tabindex=-1` so `move()` can put real focus here; the ring is a
          thickness change rather than a hue change, so the cursor is still
          visible without colour vision. Card draws no border of its own, so
          a `border-*` class would have set a colour on a zero-width border.
        -->
            <Card
              id={rowId(p.path)}
              tabindex={-1}
              aria-current={i === cursor ? 'true' : undefined}
              class="audit-row outline-none focus-visible:ring-2 focus-visible:ring-ring {i ===
              cursor
                ? 'ring-2 ring-primary'
                : selected?.path === p.path
                  ? 'ring-1 ring-muted-foreground/30'
                  : ''}"
            >
              <CardContent class="flex flex-wrap items-center gap-2 p-3">
                <Checkbox
                  checked={marked.has(p.path)}
                  onCheckedChange={() => toggleMark(p.path)}
                  aria-label={`Select ${p.name}`}
                />
                <Button
                  variant="ghost"
                  class="h-auto min-w-0 flex-1 flex-col items-start justify-start p-1 text-left whitespace-normal"
                  onclick={() => {
                    cursor = i;
                    selected = selected?.path === p.path ? null : p;
                  }}
                >
                  <div class="flex flex-wrap items-center gap-2">
                    <span class="font-medium">{p.name}</span>
                    {#if p.moduleCount > 1}
                      <Badge variant="outline">{p.moduleCount} modules</Badge>
                    {/if}
                    {#each p.langs.slice(0, 3) as lang (lang)}
                      <Badge variant="secondary">{lang}</Badge>
                    {/each}
                    {#if p.duplicateOf.length}
                      <Badge variant="outline">dup: {p.duplicateOf[0]}</Badge>
                    {/if}
                  </div>
                  <div class="w-full truncate font-mono text-xs text-muted-foreground">
                    {p.path}
                  </div>
                </Button>

                {#if p.regenerableBytes > 0}
                  <Badge variant="outline" class="gap-1">
                    <HardDrive class="h-3 w-3" />{fmtBytes(p.regenerableBytes)}
                  </Badge>
                {/if}
                {#if p.riskLevel === 'high'}
                  <Badge variant="destructive" class="gap-1">
                    <ShieldAlert class="h-3 w-3" />{p.risk[0]}
                  </Badge>
                {:else if p.risk.length}
                  <Badge variant="outline">{p.risk[0]}</Badge>
                {/if}
                <Badge variant={statusVariant(p.status)} class={statusClass(p.status)}>
                  {p.status} · {ageLabel(p.ageDays)}
                </Badge>

                <div class="flex gap-1">
                  {#each DECISIONS as d (d)}
                    <Button
                      size="sm"
                      variant={decisions[p.path] === d
                        ? d === 'delete'
                          ? 'destructive'
                          : 'default'
                        : 'ghost'}
                      class={d === 'delete' && decisions[p.path] !== d ? DESTRUCTIVE_CLASS : ''}
                      onclick={() => decide(p, d)}
                    >
                      {d}
                    </Button>
                  {/each}
                  <Button size="sm" variant="ghost" onclick={() => openFolder(p.abs)}>
                    <FolderOpen class="h-3 w-3" />
                  </Button>
                </div>
              </CardContent>

              {#if selected?.path === p.path}
                <CardContent class="border-t pt-3 text-sm">
                  {#if p.description}
                    <p class="mb-2 text-muted-foreground">{p.description}</p>
                  {/if}
                  <div class="grid gap-1 md:grid-cols-2">
                    <div>
                      <span class="text-muted-foreground">Path:</span>
                      <span class="font-mono text-xs">{p.abs}</span>
                    </div>
                    <div>
                      <span class="text-muted-foreground">Git:</span>
                      {p.git.repos
                        ? `${p.git.repos} repo(s) · ${p.git.branch ?? '?'} · ${p.git.commits} commits`
                        : 'not a repo'}
                    </div>
                    {#if p.git.remote}
                      <div class="md:col-span-2">
                        <span class="text-muted-foreground">Remote:</span>
                        <span class="font-mono text-xs">{p.git.remote}</span>
                      </div>
                    {/if}
                    {#if p.risk.length}
                      <div class="md:col-span-2">
                        <span class="text-muted-foreground">Flags:</span>
                        {p.risk.join(' · ')}
                      </div>
                    {/if}
                    {#if p.regenerable.length}
                      <div class="flex flex-wrap items-center gap-2 md:col-span-2">
                        <span class="text-muted-foreground">Build output:</span>
                        <span>{p.regenerable.join(', ')}</span>
                        {#if p.regenerableBytes >= 0}
                          <Badge variant="outline">{fmtBytes(p.regenerableBytes)}</Badge>
                        {/if}
                        <Button size="sm" variant="outline" onclick={() => cleanInDisk(p)}>
                          <HardDrive class="mr-1 h-3 w-3" /> Clean in Disk utility
                        </Button>
                      </div>
                    {/if}
                  </div>

                  {#if p.modules.length > 1}
                    <div class="mt-3">
                      <div class="mb-1 text-xs text-muted-foreground uppercase">
                        Modules ({p.modules.length})
                      </div>
                      <div class="max-h-56 space-y-1 overflow-auto">
                        {#each p.modules as m (m.path)}
                          <div class="flex items-center gap-2 text-xs">
                            <span class="flex-1 truncate font-mono">{m.path}</span>
                            {#if m.ahead > 0}<Badge variant="destructive">+{m.ahead}</Badge>{/if}
                            {#if m.git}<Badge variant="outline">{m.branch ?? 'git'}</Badge>{/if}
                            <Button size="sm" variant="ghost" onclick={() => openFolder(m.abs)}>
                              <FolderOpen class="h-3 w-3" />
                            </Button>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}
                </CardContent>
              {/if}
            </Card>
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

        <Tabs.Content value="changes">
          <Card>
            <CardContent class="space-y-1 p-3">
              {#each changes as c (c.at + c.kind + c.path)}
                <div class="flex flex-wrap items-center gap-2 text-sm">
                  <Badge
                    variant={c.kind === 'removed'
                      ? 'destructive'
                      : c.kind === 'added'
                        ? 'default'
                        : 'secondary'}>{c.kind}</Badge
                  >
                  <span class="flex-1 truncate font-mono text-xs">{c.path}</span>
                  {#if c.from && c.from !== c.path}
                    <span class="truncate font-mono text-xs text-muted-foreground"
                      >was {c.from}</span
                    >
                  {:else if c.kind === 'moved' && c.detail}
                    <!-- Same id, new folder: the old absolute path is the whole story. -->
                    <span class="truncate font-mono text-xs text-muted-foreground"
                      >was {c.detail}</span
                    >
                  {/if}
                  <span class="text-xs text-muted-foreground"
                    >{new Date(c.at).toLocaleString()}</span
                  >
                </div>
              {:else}
                <p class="py-8 text-center text-sm text-muted-foreground">
                  Nothing yet — the first scan is the baseline. Rescan later and what appeared,
                  moved or vanished lands here.
                </p>
              {/each}
            </CardContent>
          </Card>
        </Tabs.Content>

        <Tabs.Content value="duplicates">
          <div class="space-y-3">
            {#each duplicateSets as { cluster, members } (cluster.kind + cluster.key)}
              <Card>
                <CardHeader class="pb-2">
                  <CardTitle class="flex items-center gap-2 text-sm">
                    <Copy class="h-4 w-4" />
                    {cluster.key}
                    <Badge variant="outline">{cluster.kind}</Badge>
                  </CardTitle>
                </CardHeader>
                <CardContent class="space-y-1">
                  {#each members as m, i (m.path)}
                    <div class="flex items-center gap-2 text-sm">
                      {#if i === 0}<Badge variant="default">newest</Badge>{/if}
                      <span class="flex-1 truncate font-mono text-xs">{m.path}</span>
                      <span class="text-xs text-muted-foreground">{ageLabel(m.ageDays)}</span>
                      <span class="text-xs text-muted-foreground">{m.git.commits} commits</span>
                      <Button size="sm" variant="ghost" onclick={() => (selected = m)}
                        >inspect</Button
                      >
                    </div>
                  {/each}
                </CardContent>
              </Card>
            {:else}
              <Card>
                <CardContent class="py-8 text-center text-sm text-muted-foreground">
                  No duplicate sets — nothing scanned shares a git remote or a folder name with
                  anything else.
                </CardContent>
              </Card>
            {/each}
          </div>
        </Tabs.Content>

        <Tabs.Content value="tree">
          <Card
            ><CardContent class="p-3 font-mono text-sm">
              {#each [...tree.entries()] as [area, buckets] (area)}
                <div class="py-1">
                  <Button
                    variant="ghost"
                    class="h-auto justify-start gap-1 rounded px-1 py-0 font-mono text-sm hover:bg-muted"
                    onclick={() => (expanded = { ...expanded, [area]: !expanded[area] })}
                  >
                    {#if expanded[area]}<ChevronDown class="h-3 w-3" />{:else}<ChevronRight
                        class="h-3 w-3"
                      />{/if}
                    <span class="font-semibold text-primary">{area}</span>
                    <span class="text-xs text-muted-foreground">
                      {[...buckets.values()].reduce((a, b) => a + b.length, 0)} projects
                    </span>
                  </Button>
                  {#if expanded[area]}
                    <div class="ml-4 border-l border-muted pl-3">
                      {#each [...buckets.entries()] as [bucket, items] (bucket)}
                        <div class="py-1">
                          <div class="text-xs text-muted-foreground">{bucket}</div>
                          {#each items as p (p.path)}
                            <Button
                              variant="ghost"
                              class="h-auto w-full justify-start gap-2 rounded px-1 py-0 text-left font-mono text-sm hover:bg-muted"
                              onclick={() => (selected = p)}
                            >
                              <span class="flex-1 truncate">{p.name}</span>
                              {#if p.moduleCount > 1}
                                <Badge variant="outline">{p.moduleCount} modules</Badge>
                              {/if}
                              <Badge variant={statusVariant(p.status)} class={statusClass(p.status)}
                                >{p.status}</Badge
                              >
                              <span class="text-xs text-muted-foreground"
                                >{ageLabel(p.ageDays)}</span
                              >
                            </Button>
                          {/each}
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {:else}
                <p class="py-8 text-center font-sans text-sm text-muted-foreground">
                  {search.trim()
                    ? 'Nothing matches that search.'
                    : 'Nothing to group yet — run a scan first.'}
                </p>
              {/each}
            </CardContent></Card
          >
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
      <section class="space-y-2">
        <div class="flex gap-2">
          <Button class="flex-1" onclick={scan} disabled={scanning || !roots.length}>
            <RefreshCw class={`mr-2 h-4 w-4 ${scanning ? 'animate-spin' : ''}`} />
            {scanning ? 'Scanning…' : 'Scan and import'}
          </Button>
          {#if scanning}
            <Button variant="ghost" onclick={() => cancelDiscovery()}>Cancel</Button>
          {/if}
        </div>

        <Button
          variant="ghost"
          class="h-auto w-full justify-start gap-1 px-1 py-1 text-xs text-muted-foreground"
          aria-expanded={scanOpen}
          onclick={() => (scanOpen = !scanOpen)}
        >
          {#if scanOpen}
            <ChevronDown class="h-3 w-3" />
          {:else}
            <ChevronRight class="h-3 w-3" />
          {/if}
          <FolderOpen class="h-3 w-3" />
          {loadingState
            ? 'Loading saved folders…'
            : `${roots.length} ${roots.length === 1 ? 'folder' : 'folders'} to scan`}
        </Button>

        {#if scanOpen}
          <div class="space-y-3">
            <div class="flex flex-wrap gap-2">
              {#each roots as root (root.path)}
                <Badge variant="secondary" class="max-w-full gap-1 font-mono text-xs">
                  <span class="truncate" title={root.path}>{root.path}</span>
                  <!-- 24px is the WCAG 2.2 floor for a pointer target; the glyph
                       stays 12px, only the hit area grows. -->
                  <Button
                    variant="ghost"
                    class="-mr-1 h-6 w-6 shrink-0 rounded-full p-0"
                    aria-label={`Remove ${root.path}`}
                    onclick={() => removeRoot(root.path)}
                    disabled={scanning}
                  >
                    <X class="h-3 w-3" />
                  </Button>
                </Badge>
              {:else}
                <p class="text-sm text-muted-foreground">
                  No folders yet — add the drive or directory your projects live under.
                </p>
              {/each}
            </div>

            <div class="flex gap-2">
              <Input
                bind:value={newRootPath}
                aria-label="Folder path"
                aria-describedby="audit-root-hint"
                placeholder={'D:\\production'}
              />
              <Button variant="outline" onclick={addRoot} disabled={scanning}>Add folder</Button>
            </div>
            <p id="audit-root-hint" class="text-xs text-muted-foreground">
              Leave the field empty and Add folder opens a folder picker.
            </p>

            <div class="space-y-2 text-sm">
              <label class="flex items-start gap-2">
                <Checkbox
                  checked={measureSizes}
                  onCheckedChange={(v) => {
                    measureSizes = v === true;
                    void persistConfig();
                  }}
                  disabled={scanning}
                />
                <span>
                  Measure reclaimable size
                  <span class="text-xs text-muted-foreground">walks build output — slower</span>
                </span>
              </label>
              <label class="flex items-start gap-2">
                <Checkbox
                  checked={markMissing}
                  onCheckedChange={(v) => {
                    markMissing = v === true;
                    void persistConfig();
                  }}
                  disabled={scanning}
                />
                <span>
                  Archive projects no longer on disk
                  <span class="text-xs text-muted-foreground">
                    roots that failed to open are skipped
                  </span>
                </span>
              </label>
            </div>
          </div>
        {/if}

        {#if scanning && progress}
          <div class="space-y-1">
            <Progress value={percent} aria-label="Scan progress" />
            <!-- The path changes many times a second; announcing it would drown
                 out everything else, so the pollable progressbar carries it. -->
            <p class="truncate font-mono text-xs text-muted-foreground" aria-hidden="true">
              {progress.phase === 'find' ? 'walking' : `${progress.done}/${progress.total}`}
              {progress.phase === 'size' ? ' sizing' : ''} — {progress.path}
            </p>
          </div>
        {/if}

        {#if error}
          <p class="text-sm text-destructive" role="alert">{error}</p>
        {/if}
        {#if importSummary}
          <p class="text-xs text-muted-foreground" role="status">{importSummary}</p>
        {/if}
      </section>

      <section class="space-y-2 border-t pt-3">
        <h3 class="text-xs font-semibold text-muted-foreground uppercase">Selected project</h3>
        {#if focused}
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-sm font-medium">{focused.name}</span>
            <Badge variant={statusVariant(focused.status)} class={statusClass(focused.status)}>
              {focused.status} · {ageLabel(focused.ageDays)}
            </Badge>
          </div>
          <p class="font-mono text-xs break-all text-muted-foreground">{focused.abs}</p>
          <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1 text-xs">
            <dt class="text-muted-foreground">Reclaimable</dt>
            <dd>
              {focused.regenerableBytes < 0 ? 'not measured' : fmtBytes(focused.regenerableBytes)}
            </dd>
            <dt class="text-muted-foreground">Last commit</dt>
            <dd>
              {focused.git.lastCommit
                ? new Date(focused.git.lastCommit).toLocaleDateString()
                : focused.git.repos
                  ? 'no commits'
                  : 'not a repo'}
            </dd>
            <dt class="text-muted-foreground">Unpushed</dt>
            <dd>{focused.git.ahead > 0 ? `${focused.git.ahead} commits` : 'none'}</dd>
            <dt class="text-muted-foreground">Remote</dt>
            <dd class="font-mono break-all">
              {focused.git.remote ?? (focused.git.repos ? 'none — local only' : '—')}
            </dd>
            <dt class="text-muted-foreground">Duplicates</dt>
            <dd>
              {#each duplicateSiblings as sibling (sibling)}
                <span class="block truncate font-mono" title={sibling}>{sibling}</span>
              {:else}
                none
              {/each}
            </dd>
          </dl>
          {#if focused.risk.length}
            <p class="text-xs">
              <span class="text-muted-foreground">Flags:</span>
              {focused.risk.join(' · ')}
            </p>
          {/if}
        {:else}
          <p class="text-xs text-muted-foreground">
            Nothing under the cursor — press <span class="font-mono">j</span> to start down the list.
          </p>
        {/if}
      </section>

      <section class="space-y-2 border-t pt-3">
        <div class="flex items-center justify-between gap-2">
          <h3 class="text-xs font-semibold text-muted-foreground uppercase">Marked</h3>
          <span class="text-xs tabular-nums">{marked.size} selected</span>
        </div>
        {#if marked.size}
          <div class="grid grid-cols-2 gap-1">
            {#each DECISIONS as d (d)}
              <Button
                size="sm"
                variant="outline"
                class={d === 'delete' ? DESTRUCTIVE_CLASS : ''}
                onclick={() => decideMarked(d)}>{d} all</Button
              >
            {/each}
          </div>
          <Button size="sm" variant="ghost" class="w-full" onclick={() => (marked = new Set())}>
            clear
          </Button>
        {:else}
          <p class="text-xs text-muted-foreground">
            Press <span class="font-mono">x</span> on a row to tick it for a bulk decision.
          </p>
        {/if}
      </section>
    </div>
  </ResponsivePanel>
</div>
