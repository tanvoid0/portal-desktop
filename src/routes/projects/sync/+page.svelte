<!--
	/projects/sync — pull, push and back up every project, then clear what is left.
	The run, the AI review and the filters live in module singletons, so leaving
	the page and coming back changes nothing.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { PageHeader, PageStats } from '$lib/components/shell';
  import type { PageStat } from '$lib/components/shell';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { Progress } from '$lib/components/ui/progress';
  import { ToggleGroup, ToggleGroupItem } from '$lib/components/ui/toggle-group';
  import * as Tabs from '$lib/components/ui/tabs';
  import EnvBackupSection from '$lib/domains/projects/components/EnvBackupSection.svelte';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import CircleCheck from '~icons/fa6-solid/circle-check';
  import Hand from '~icons/fa6-solid/hand';
  import CircleXmark from '~icons/fa6-solid/circle-xmark';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import { confirmAction } from '$lib/utils/confirm';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';
  import { vaultGate } from '$lib/domains/credentials';
  import { createSyncStatusQuery } from '$lib/domains/sync/queries/syncQueries';
  import PasswordDialog from '$lib/domains/sync/components/PasswordDialog.svelte';
  import SyncItemCard from '$lib/domains/projects/components/SyncItemCard.svelte';
  import { syncAll, PHASE_LABEL } from '$lib/domains/projects/state/syncAll.svelte';
  import { syncTriage } from '$lib/domains/projects/state/syncTriage.svelte';
  import { openSteps } from '$lib/domains/projects/utils/syncTriage';
  import { coderProblems, openAllInCoder } from '$lib/domains/projects/utils/syncCoder';

  const syncStatus = createSyncStatusQuery();
  const projectsQuery = createProjectsQuery();
  let pwOpen = $state(false);

  onMount(() => {
    breadcrumbActions.setCustomBreadcrumbs([
      { label: 'Projects', href: '/projects' },
      { label: 'Sync', href: '/projects/sync' },
    ]);
  });

  async function start() {
    if (syncAll.running) return;
    // A cancelled unlock still proceeds; the env phase then reports the vault as locked.
    await vaultGate.ensureUnlocked('Unlock to back up env files.');
    if (syncStatus.data?.configured) pwOpen = true;
    else void syncTriage.run(null);
  }

  async function applyReady() {
    const n = syncTriage.ready.length;
    const pushing = syncTriage.ready.filter((id) => syncTriage.items[id].plan.push).length;
    const ok = await confirmAction(
      `Apply the reviewed plan in ${n} ${n === 1 ? 'project' : 'projects'}? ${
        pushing ? `${pushing} of them also push.` : 'Nothing is pushed.'
      }`,
      'Apply all ready'
    );
    if (ok) await syncTriage.applyReady();
  }

  const projects = $derived(syncAll.report?.projects ?? []);
  const stuck = $derived(projects.filter((p) => coderProblems(p).length));

  const stats = $derived.by<PageStat[]>(() => {
    const open = projects.map((p) => openSteps(p.steps));
    return [
      { label: 'Synced', value: open.filter((s) => !s.length).length, icon: CircleCheck },
      { label: 'Need you', value: open.filter((s) => s.length).length, icon: Hand },
      { label: 'Ready to apply', value: syncTriage.ready.length, icon: Sparkles },
      {
        label: 'Failed',
        value: open.filter((s) => s.some((x) => x.status === 'failed')).length,
        icon: CircleXmark,
      },
    ];
  });

  // Every project is listed, synced or not, so any one can be synced on its own.
  const rows = $derived.by(() => {
    const synced = new Map(syncAll.rows.map((r) => [r.id, r]));
    return (projectsQuery.data ?? [])
      .filter((p) => !p.archived)
      .map(
        (p) =>
          synced.get(Number(p.id)) ?? { id: Number(p.id), name: p.name, path: p.path, steps: {} }
      );
  });
  // Before the first run there is nothing to need attention; show everything.
  const showAll = $derived(syncAll.view === 'all' || (!syncAll.report && !syncAll.running));

  const shown = $derived.by(() => {
    const query = syncAll.search.trim().toLowerCase();
    return rows
      .filter((row) => {
        if (query && !`${row.name} ${row.path ?? ''}`.toLowerCase().includes(query)) return false;
        if (showAll) return true;
        // A reviewed or opened project stays listed so its outcome can be read.
        return (
          !!syncTriage.items[row.id] ||
          syncAll.openEnv.includes(row.id) ||
          Object.values(row.steps).some((s) => s && s.status !== 'ok' && s.status !== 'skipped')
        );
      })
      .sort((a, b) => a.name.localeCompare(b.name));
  });

  const percent = $derived(
    syncAll.total > 0 ? Math.round((syncAll.done / syncAll.total) * 100) : 0
  );
</script>

<svelte:head>
  <title>Sync projects - Portal Desktop</title>
</svelte:head>

<!-- No PageContainer: the app shell already provides the gutter and width cap. -->
<div class="max-w-5xl space-y-4">
  <PageHeader title="Sync projects" description="Back up env files, then pull and push every repo">
    {#snippet actions()}
      {#if stuck.length > 1 && !syncAll.running}
        <Button variant="ghost" onclick={() => openAllInCoder(stuck)}>
          Open all in Coder ({stuck.length})
        </Button>
      {/if}
      {#if syncTriage.ready.length}
        <Button
          variant="outline"
          disabled={syncTriage.applyingAll || syncAll.running}
          onclick={applyReady}
        >
          Apply all ready ({syncTriage.ready.length})
        </Button>
      {/if}
      {#if syncAll.running}
        <Button variant="outline" onclick={() => syncAll.cancel()}>Cancel</Button>
      {:else}
        <Button
          disabled={syncStatus.isPending || syncTriage.applying || syncAll.busy.length > 0}
          onclick={start}
        >
          <RefreshCw class="mr-2 h-4 w-4" /> Sync all
        </Button>
      {/if}
    {/snippet}
  </PageHeader>

  {#if syncAll.running}
    <div class="space-y-1" role="status">
      <Progress value={percent} class="w-full" />
      <p class="text-xs text-muted-foreground tabular-nums">
        {PHASE_LABEL[syncAll.phase]} · {syncAll.done} / {syncAll.total}
      </p>
    </div>
  {:else if syncAll.report}
    <PageStats {stats} />
  {/if}

  <Tabs.Root value={syncAll.tab} onValueChange={(v) => (syncAll.tab = v as typeof syncAll.tab)}>
    <Tabs.List>
      <Tabs.Trigger value="repos">Projects</Tabs.Trigger>
      <Tabs.Trigger value="env">Environments</Tabs.Trigger>
    </Tabs.List>
  </Tabs.Root>

  {#if syncAll.tab === 'env'}
    <EnvBackupSection bind:filter={syncAll.envFilter} bind:lastRun={syncAll.envLastRun} />
  {:else}
    <div class="flex flex-wrap items-center gap-3">
      <ToggleGroup
        type="single"
        variant="outline"
        size="sm"
        bind:value={
          () => (showAll ? 'all' : 'attention'),
          // Clicking the active item would clear a single toggle group.
          (v) => v && (syncAll.view = v as typeof syncAll.view)
        }
        aria-label="Filter projects"
      >
        <ToggleGroupItem value="attention">Needs attention</ToggleGroupItem>
        <ToggleGroupItem value="all">All ({rows.length})</ToggleGroupItem>
      </ToggleGroup>
      <Input
        bind:value={syncAll.search}
        class="h-8 w-full max-w-56"
        placeholder="Search projects"
        aria-label="Search projects"
      />
      <div class="ml-auto flex items-center gap-2">
        <Switch
          id="auto-review"
          bind:checked={() => syncTriage.auto, (on) => syncTriage.setAuto(on)}
        />
        <Label
          for="auto-review"
          class="text-xs"
          title="Sends file names and diffs (likely secrets removed) to your configured AI provider"
        >
          Review changes with AI after a sync
        </Label>
      </div>
    </div>

    <ul class="space-y-2">
      {#each shown as row (row.id)}
        <SyncItemCard {row} />
      {:else}
        <li class="py-6 text-center text-sm text-muted-foreground">
          {#if projectsQuery.isPending}
            Loading projects…
          {:else if syncAll.search.trim()}
            No project matches.
          {:else if syncAll.running}
            Nothing needs attention so far.
          {:else if !rows.length}
            No projects yet.
          {:else}
            Nothing needs attention.
          {/if}
        </li>
      {/each}
    </ul>

    {#if syncAll.report}
      {@const report = syncAll.report}
      <div class="space-y-1 text-xs text-muted-foreground">
        <p>Last sync: {new Date(report.finishedAt).toLocaleString()}</p>
        {#if report.cancelled}<p>Cancelled before every project was synced.</p>{/if}
        {#if report.envNote}<p>Env: {report.envNote}</p>{/if}
        <p>
          Cloud snapshot: {report.snapshot.detail ?? report.snapshot.status}
          {#if report.snapshot.status !== 'ok'}
            · <a class="underline" href="/settings/sync">Sync settings</a>
          {/if}
        </p>
      </div>
    {/if}
  {/if}
</div>

<PasswordDialog
  bind:open={pwOpen}
  mode="backup"
  confirmTwice={!syncStatus.data?.cloud && !syncStatus.data?.cloudError}
  onSubmit={async (pw) => {
    void syncTriage.run(pw);
  }}
/>
