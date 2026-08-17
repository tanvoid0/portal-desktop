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
  import {
    FolderOpen,
    RefreshCw,
    ShieldAlert,
    Copy,
    ChevronRight,
    ChevronDown,
    X,
  } from '@lucide/svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { onDestroy } from 'svelte';
  import { logger } from '$lib/domains/shared/services/logger';
  import { confirmAction } from '$lib/utils/confirm';
  import { invokeClient } from '$lib/utils/invokeClient';
  import {
    cancelDiscovery,
    defaultDiscoveryConfig,
    discoverAndImport,
    DISCOVERY_PROGRESS_EVENT,
    setProjectTriage,
    type TriageDecision,
  } from '$lib/domains/projects/api/discoveryApi';
  import type { DiscoveryProgress } from '$lib/domains/projects/generated/DiscoveryProgress';
  import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
  import type { DiscoveryReport } from '$lib/domains/projects/generated/DiscoveryReport';

  const log = logger.createScoped('ProjectAudit');

  type Tab = 'triage' | 'risk' | 'duplicates' | 'tree';

  let roots = $state<{ name: string; path: string }[]>([]);
  let newRootPath = $state('');
  let report = $state<DiscoveryReport | null>(null);
  let scanning = $state(false);
  let progress = $state<DiscoveryProgress | null>(null);
  let error = $state<string | null>(null);
  let tab = $state<Tab>('triage');
  let search = $state('');
  let selected = $state<DiscoveredProject | null>(null);
  let decisions = $state<Record<string, TriageDecision | null>>({});
  let idByPath = $state<Record<string, number>>({});
  let expanded = $state<Record<string, boolean>>({});
  let importSummary = $state<string | null>(null);

  let unlisten: UnlistenFn | null = null;
  listen<DiscoveryProgress>(DISCOVERY_PROGRESS_EVENT, (e) => {
    progress = e.payload;
  })
    .then((fn) => (unlisten = fn))
    .catch((e) => log.warn('progress listener failed', e));

  onDestroy(() => unlisten?.());

  const projects = $derived(report?.projects ?? []);

  const filtered = $derived.by(() => {
    const q = search.trim().toLowerCase();
    let list = projects;
    if (tab === 'risk') list = list.filter((p) => p.riskLevel === 'high' || p.riskLevel === 'med');
    if (tab === 'duplicates') list = list.filter((p) => p.duplicateOf.length > 0);
    if (!q) return list;
    return list.filter((p) =>
      `${p.path} ${p.name} ${p.langs.join(' ')} ${p.description ?? ''}`.toLowerCase().includes(q)
    );
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
      const name =
        picked
          .replace(/[\\/]+$/, '')
          .split(/[\\/]/)
          .pop() ?? 'root';
      if (roots.some((r) => r.path === picked)) return;
      roots = [...roots, { name, path: picked }];
      newRootPath = '';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function removeRoot(path: string) {
    roots = roots.filter((r) => r.path !== path);
  }

  async function scan() {
    if (!roots.length) {
      error = 'Add at least one folder to scan.';
      return;
    }
    scanning = true;
    error = null;
    importSummary = null;
    progress = null;
    try {
      const result = await discoverAndImport(defaultDiscoveryConfig(roots), false);
      report = result.report;
      importSummary =
        `${result.inserted.length} added · ${result.updated.length} refreshed · ` +
        `${result.adopted.length} linked to existing rows` +
        (result.missing.length ? ` · ${result.missing.length} not found on disk` : '');
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
            <button
              type="button"
              aria-label={`Remove ${root.path}`}
              onclick={() => removeRoot(root.path)}
            >
              <X class="h-3 w-3" />
            </button>
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

      {#if scanning && progress}
        <div class="space-y-1">
          <Progress value={percent} />
          <p class="text-muted-foreground truncate font-mono text-xs">
            {progress.phase === 'find' ? 'walking' : `${progress.done}/${progress.total}`}
            — {progress.path}
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
    <div class="grid grid-cols-2 gap-3 md:grid-cols-5">
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
    </div>
  {/if}

  {#if report}
    <div class="flex flex-wrap items-center gap-2">
      {#each [['triage', 'Triage'], ['risk', 'At risk'], ['duplicates', 'Duplicates'], ['tree', 'Tree']] as [key, label] (key)}
        <Button
          size="sm"
          variant={tab === key ? 'default' : 'outline'}
          onclick={() => (tab = key as Tab)}
        >
          {label}
        </Button>
      {/each}
      <Input bind:value={search} placeholder="Search…" class="ml-auto max-w-xs" />
    </div>

    {#if tab === 'duplicates'}
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
              <button
                type="button"
                class="hover:bg-muted flex items-center gap-1 rounded px-1"
                onclick={() => (expanded = { ...expanded, [area]: !expanded[area] })}
              >
                {#if expanded[area]}<ChevronDown class="h-3 w-3" />{:else}<ChevronRight
                    class="h-3 w-3"
                  />{/if}
                <span class="text-primary font-semibold">{area}</span>
                <span class="text-muted-foreground text-xs">
                  {[...buckets.values()].reduce((a, b) => a + b.length, 0)} projects
                </span>
              </button>
              {#if expanded[area]}
                <div class="border-muted ml-4 border-l pl-3">
                  {#each [...buckets.entries()] as [bucket, items] (bucket)}
                    <div class="py-1">
                      <div class="text-muted-foreground text-xs">{bucket}</div>
                      {#each items as p (p.path)}
                        <button
                          type="button"
                          class="hover:bg-muted flex w-full items-center gap-2 rounded px-1 text-left"
                          onclick={() => (selected = p)}
                        >
                          <span class="flex-1 truncate">{p.name}</span>
                          {#if p.moduleCount > 1}
                            <Badge variant="outline">{p.moduleCount} modules</Badge>
                          {/if}
                          <Badge variant={statusVariant(p.status)}>{p.status}</Badge>
                          <span class="text-muted-foreground text-xs">{ageLabel(p.ageDays)}</span>
                        </button>
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
        {#each filtered as p (p.path)}
          <Card class={selected?.path === p.path ? 'border-primary' : ''}>
            <CardContent class="flex flex-wrap items-center gap-2 p-3">
              <button
                type="button"
                class="min-w-0 flex-1 text-left"
                onclick={() => (selected = selected?.path === p.path ? null : p)}
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
                <div class="text-muted-foreground truncate font-mono text-xs">{p.path}</div>
              </button>

              {#if p.riskLevel === 'high'}
                <Badge variant="destructive" class="gap-1">
                  <ShieldAlert class="h-3 w-3" />{p.risk[0]}
                </Badge>
              {:else if p.risk.length}
                <Badge variant="outline">{p.risk[0]}</Badge>
              {/if}
              <Badge variant={statusVariant(p.status)}>{p.status} · {ageLabel(p.ageDays)}</Badge>

              <div class="flex gap-1">
                {#each ['keep', 'archive', 'delete', 'review'] as d (d)}
                  <Button
                    size="sm"
                    variant={decisions[p.path] === d ? 'default' : 'ghost'}
                    onclick={() => decide(p, d as TriageDecision)}
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
                    <div class="md:col-span-2">
                      <span class="text-muted-foreground">Build output:</span>
                      {p.regenerable.join(', ')} — clean from the Disk tab
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
