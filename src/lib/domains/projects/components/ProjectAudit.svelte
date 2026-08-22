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
  import {
    FolderOpen,
    RefreshCw,
    ShieldAlert,
    Copy,
    ChevronRight,
    ChevronDown,
    HardDrive,
    Download,
    X,
  } from '@lucide/svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { onDestroy, onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { logger } from '$lib/domains/shared/services/logger';
  import { confirmAction } from '$lib/utils/confirm';
  import { invokeClient } from '$lib/utils/invokeClient';
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
  /** Keyboard position in `filtered`. */
  let cursor = $state(0);
  /** Paths ticked for a bulk decision. */
  let marked = $state<Set<string>>(new Set());

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
      measureSizes = state.config.measureSizes;
      markMissing = state.config.markMissing;
      report = state.report;
      changes = state.changes;
      if (state.report) await loadRegistryIds();
    } catch (e) {
      log.warn('could not load saved audit state', e);
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
      error = e instanceof Error ? e.message : String(e);
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
      error = 'This project is not in the registry yet — run a scan first.';
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
      error = e instanceof Error ? e.message : String(e);
    }
  }

  /** Applies one decision to every ticked project in a single round trip. */
  async function decideMarked(decision: TriageDecision) {
    const paths = [...marked].filter((p) => idByPath[p]);
    if (!paths.length) {
      error = 'None of the selected projects are in the registry yet — run a scan first.';
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
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
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
    document.getElementById(rowId(filtered[cursor].path))?.scrollIntoView({ block: 'nearest' });
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
      error = e instanceof Error ? e.message : String(e);
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
      error = e instanceof Error ? e.message : String(e);
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
    return status === 'active'
      ? 'default'
      : status === 'recent'
        ? 'secondary'
        : status === 'dormant'
          ? 'outline'
          : 'outline';
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

<div class="space-y-4">
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="flex items-center gap-2 text-base">
        <FolderOpen class="h-4 w-4" /> Folders to scan
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-3">
      <div class="flex flex-wrap gap-2">
        {#each roots as root (root.path)}
          <Badge variant="secondary" class="gap-1 font-mono text-xs">
            {root.path}
            <Button
              variant="ghost"
              class="h-auto p-0"
              aria-label={`Remove ${root.path}`}
              onclick={() => removeRoot(root.path)}
            >
              <X class="h-3 w-3" />
            </Button>
          </Badge>
        {:else}
          <p class="text-muted-foreground text-sm">
            No folders yet — add the drive or directory your projects live under.
          </p>
        {/each}
      </div>

      <div class="flex gap-2">
        <Input bind:value={newRootPath} placeholder="D:\\production  (or leave empty to browse)" />
        <Button variant="outline" onclick={addRoot} disabled={scanning}>Add folder</Button>
        <Button onclick={scan} disabled={scanning || !roots.length}>
          <RefreshCw class={`mr-2 h-4 w-4 ${scanning ? 'animate-spin' : ''}`} />
          {scanning ? 'Scanning…' : 'Scan and import'}
        </Button>
        {#if scanning}
          <Button variant="ghost" onclick={() => cancelDiscovery()}>Cancel</Button>
        {/if}
      </div>

      <div class="flex flex-wrap items-center gap-6 text-sm">
        <label class="flex items-center gap-2">
          <Checkbox
            checked={measureSizes}
            onCheckedChange={(v) => {
              measureSizes = v === true;
              void persistConfig();
            }}
            disabled={scanning}
          />
          <span>Measure reclaimable size</span>
          <span class="text-muted-foreground text-xs">walks build output — slower</span>
        </label>
        <label class="flex items-center gap-2">
          <Checkbox
            checked={markMissing}
            onCheckedChange={(v) => {
              markMissing = v === true;
              void persistConfig();
            }}
            disabled={scanning}
          />
          <span>Archive projects no longer on disk</span>
          <span class="text-muted-foreground text-xs">roots that failed to open are skipped</span>
        </label>
      </div>

      {#if scanning && progress}
        <div class="space-y-1">
          <Progress value={percent} />
          <p class="text-muted-foreground truncate font-mono text-xs">
            {progress.phase === 'find' ? 'walking' : `${progress.done}/${progress.total}`}
            {progress.phase === 'size' ? ' sizing' : ''} — {progress.path}
          </p>
        </div>
      {/if}

      {#if error}
        <p class="text-destructive text-sm">{error}</p>
      {/if}
      {#if importSummary}
        <p class="text-muted-foreground text-sm">{importSummary}</p>
      {/if}
    </CardContent>
  </Card>

  {#if stats}
    <div class="grid grid-cols-2 gap-3 md:grid-cols-6">
      <Card
        ><CardContent class="p-3">
          <div class="text-2xl font-semibold">{stats.projects}</div>
          <div class="text-muted-foreground text-xs uppercase">Projects</div>
          <div class="text-muted-foreground text-xs">{stats.roots} roots</div>
        </CardContent></Card
      >
      <Card
        ><CardContent class="p-3">
          <div class="text-2xl font-semibold">{stats.active}</div>
          <div class="text-muted-foreground text-xs uppercase">Active</div>
          <div class="text-muted-foreground text-xs">
            {stats.recent} recent · {stats.dormant} dormant · {stats.stale} stale
          </div>
        </CardContent></Card
      >
      <Card
        ><CardContent class="p-3">
          <div class="text-destructive text-2xl font-semibold">{highRisk.length}</div>
          <div class="text-muted-foreground text-xs uppercase">Work at risk</div>
          <div class="text-muted-foreground text-xs">{stats.unpushed} unpushed commits</div>
        </CardContent></Card
      >
      <Card
        ><CardContent class="p-3">
          <div class="text-2xl font-semibold">{report?.clusters.length ?? 0}</div>
          <div class="text-muted-foreground text-xs uppercase">Duplicate sets</div>
          <div class="text-muted-foreground text-xs">same remote or name</div>
        </CardContent></Card
      >
      <Card
        ><CardContent class="p-3">
          <div class="text-2xl font-semibold">{undecided}</div>
          <div class="text-muted-foreground text-xs uppercase">Undecided</div>
          <div class="text-muted-foreground text-xs">of {stats.projects}</div>
        </CardContent></Card
      >
      <Card
        ><CardContent class="p-3">
          <div class="text-2xl font-semibold">
            {stats.regenerableBytes < 0 ? '—' : fmtBytes(stats.regenerableBytes)}
          </div>
          <div class="text-muted-foreground text-xs uppercase">Reclaimable</div>
          <div class="text-muted-foreground text-xs">
            {stats.regenerableBytes < 0 ? 'not measured' : 'build output'}
          </div>
        </CardContent></Card
      >
    </div>
  {/if}

  {#if report}
    <div class="flex flex-wrap items-center gap-2">
      {#each [['triage', 'Triage'], ['risk', 'At risk'], ['duplicates', 'Duplicates'], ['tree', 'Tree'], ['changes', `Changes${changes.length ? ` (${changes.length})` : ''}`]] as [key, label] (key)}
        <Button
          size="sm"
          variant={tab === key ? 'default' : 'outline'}
          onclick={() => (tab = key as Tab)}
        >
          {label}
        </Button>
      {/each}
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
      <Input bind:value={search} placeholder="Search…" class="ml-auto max-w-xs" />
    </div>

    {#if tab === 'triage' || tab === 'risk'}
      <p class="text-muted-foreground text-xs">
        <span class="font-mono">j</span>/<span class="font-mono">k</span> move ·
        <span class="font-mono">1</span>–<span class="font-mono">4</span> keep/archive/delete/review
        · <span class="font-mono">o</span> open folder · <span class="font-mono">x</span> select ·
        <span class="font-mono">esc</span> clear
      </p>
    {/if}

    {#if exportedTo}
      <p class="text-muted-foreground font-mono text-xs">Written to {exportedTo}</p>
    {/if}

    {#if marked.size}
      <Card>
        <CardContent class="flex flex-wrap items-center gap-2 p-3">
          <span class="text-sm font-medium">{marked.size} selected</span>
          {#each DECISIONS as d (d)}
            <Button size="sm" variant="outline" onclick={() => decideMarked(d)}>{d} all</Button>
          {/each}
          <Button size="sm" variant="ghost" onclick={() => (marked = new Set())}>clear</Button>
        </CardContent>
      </Card>
    {/if}

    {#if tab === 'changes'}
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
                <span class="text-muted-foreground truncate font-mono text-xs">was {c.from}</span>
              {:else if c.kind === 'moved' && c.detail}
                <!-- Same id, new folder: the old absolute path is the whole story. -->
                <span class="text-muted-foreground truncate font-mono text-xs">was {c.detail}</span>
              {/if}
              <span class="text-muted-foreground text-xs">{new Date(c.at).toLocaleString()}</span>
            </div>
          {:else}
            <p class="text-muted-foreground py-8 text-center text-sm">
              Nothing yet — the first scan is the baseline. Rescan later and what appeared, moved or
              vanished lands here.
            </p>
          {/each}
        </CardContent>
      </Card>
    {:else if tab === 'duplicates'}
      <div class="space-y-3">
        {#each report.clusters as cluster (cluster.kind + cluster.key)}
          {@const members = cluster.members
            .map((m) => projects.find((p) => p.path === m))
            .filter((p): p is DiscoveredProject => !!p)
            .sort((a, b) => b.activity - a.activity)}
          {#if members.length > 1}
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
                    <span class="text-muted-foreground text-xs">{ageLabel(m.ageDays)}</span>
                    <span class="text-muted-foreground text-xs">{m.git.commits} commits</span>
                    <Button size="sm" variant="ghost" onclick={() => (selected = m)}>inspect</Button
                    >
                  </div>
                {/each}
              </CardContent>
            </Card>
          {/if}
        {/each}
      </div>
    {:else if tab === 'tree'}
      <Card
        ><CardContent class="p-3 font-mono text-sm">
          {#each [...tree.entries()] as [area, buckets] (area)}
            <div class="py-1">
              <Button
                variant="ghost"
                class="hover:bg-muted h-auto justify-start gap-1 rounded px-1 py-0 font-mono text-sm"
                onclick={() => (expanded = { ...expanded, [area]: !expanded[area] })}
              >
                {#if expanded[area]}<ChevronDown class="h-3 w-3" />{:else}<ChevronRight
                    class="h-3 w-3"
                  />{/if}
                <span class="text-primary font-semibold">{area}</span>
                <span class="text-muted-foreground text-xs">
                  {[...buckets.values()].reduce((a, b) => a + b.length, 0)} projects
                </span>
              </Button>
              {#if expanded[area]}
                <div class="border-muted ml-4 border-l pl-3">
                  {#each [...buckets.entries()] as [bucket, items] (bucket)}
                    <div class="py-1">
                      <div class="text-muted-foreground text-xs">{bucket}</div>
                      {#each items as p (p.path)}
                        <Button
                          variant="ghost"
                          class="hover:bg-muted h-auto w-full justify-start gap-2 rounded px-1 py-0 text-left font-mono text-sm"
                          onclick={() => (selected = p)}
                        >
                          <span class="flex-1 truncate">{p.name}</span>
                          {#if p.moduleCount > 1}
                            <Badge variant="outline">{p.moduleCount} modules</Badge>
                          {/if}
                          <Badge variant={statusVariant(p.status)}>{p.status}</Badge>
                          <span class="text-muted-foreground text-xs">{ageLabel(p.ageDays)}</span>
                        </Button>
                      {/each}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </CardContent></Card
      >
    {:else}
      <div class="space-y-2">
        {#each filtered as p, i (p.path)}
          <Card
            id={rowId(p.path)}
            class={i === cursor
              ? 'border-primary'
              : selected?.path === p.path
                ? 'border-muted'
                : ''}
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
                <div class="text-muted-foreground w-full truncate font-mono text-xs">{p.path}</div>
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
              <Badge variant={statusVariant(p.status)}>{p.status} · {ageLabel(p.ageDays)}</Badge>

              <div class="flex gap-1">
                {#each DECISIONS as d (d)}
                  <Button
                    size="sm"
                    variant={decisions[p.path] === d ? 'default' : 'ghost'}
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
                  <p class="text-muted-foreground mb-2">{p.description}</p>
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
                    <div class="text-muted-foreground mb-1 text-xs uppercase">
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
          <p class="text-muted-foreground py-8 text-center text-sm">Nothing matches.</p>
        {/each}
      </div>
    {/if}
  {/if}
</div>
