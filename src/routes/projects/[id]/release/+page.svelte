<!--
  Release details: the config (ship/rehearse plan as a WorkflowTree, one node per
  app once the apps-build foreach resolves), the launcher (scope: full / server /
  apps subset × rehearse / ship), and the full release run history. A history row
  drills into the existing run page (per-step logs + Retry-failed).
  ReleaseCard stays the compact Actions-tab summary; this is the full view.
-->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import Rocket from '~icons/fa6-solid/rocket';
  import FlaskConical from '~icons/fa6-solid/flask';
  import CircleAlert from '~icons/fa6-solid/circle-exclamation';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import { createProjectQuery } from '$lib/domains/projects';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';
  import type { WorkflowPlanStep } from '$lib/domains/scripts/services/scriptExecutionService';
  import type { ReleaseDescription, ReleaseRun } from '$lib/domains/actions/types';
  import WorkflowTree from '$lib/domains/actions/components/WorkflowTree.svelte';
  import RunProgressBar from '$lib/domains/actions/components/RunProgressBar.svelte';
  import ShipDialog from '$lib/domains/actions/components/ShipDialog.svelte';

  const projectId = $derived(page.params.id ?? '');
  const projectQuery = createProjectQuery(() => projectId);
  const project = $derived(projectQuery.data ?? null);
  const projectName = $derived(project?.name ?? '');

  let described = $state<ReleaseDescription | null>(null);
  let loadError = $state<string | null>(null);
  let runs = $state<ReleaseRun[]>([]);
  let loaded = $state(false);

  let confirmOpen = $state(false);
  let launchMode = $state<'ship' | 'rehearse'>('ship');

  // The subset-shippable apps come from the resolved ship plan: each apps-build
  // fan-out node has id `<action>:<app>`. One source of truth (release-apps.txt).
  const apps = $derived(
    described
      ? [
          ...new Set(
            described.ship.steps
              .map((s) => s.id)
              .filter((id) => id.includes(':'))
              .map((id) => id.slice(id.indexOf(':') + 1))
          ),
        ]
      : []
  );

  // Rehearse steps are typed `{id,name}` but the backend returns dependsOn too.
  const toPlan = (steps: { id: string; name: string; dependsOn?: string[] }[]): WorkflowPlanStep[] =>
    steps.map((s) => ({ id: s.id, name: s.name, needs: s.dependsOn ?? [] }));

  async function load() {
    const id = Number(projectId);
    loadError = null;
    try {
      described = await invoke<ReleaseDescription | null>('release_describe', { projectId: id });
    } catch (e) {
      described = null;
      loadError = String(e);
    }
    runs = described
      ? await invoke<ReleaseRun[]>('release_list', { projectId: id, limit: 50 }).catch(() => [])
      : [];
    loaded = true;
  }

  // Runs can start elsewhere and there is no pipeline-run event to listen for, so
  // poll the history while this page is on screen. ponytail: polling.
  async function refreshRuns() {
    if (!described || document.hidden) return;
    runs = await invoke<ReleaseRun[]>('release_list', { projectId: Number(projectId), limit: 50 })
      .catch(() => runs);
  }

  function launch(mode: 'ship' | 'rehearse') {
    launchMode = mode;
    confirmOpen = true;
  }

  $effect(() => {
    if (project) {
      breadcrumbActions.setProjectSubpageBreadcrumbs(project.name, project.id, 'Release');
    }
    return () => breadcrumbActions.clearCustomBreadcrumbs();
  });

  onMount(() => {
    load();
    const timer = setInterval(refreshRuns, 3000);
    return () => clearInterval(timer);
  });
</script>

<svelte:head>
  <title>Release - {projectName || 'Project'} - Portal Desktop</title>
</svelte:head>

<div class="mx-auto flex w-full max-w-4xl flex-col gap-4 p-4">
  <Button variant="ghost" size="sm" class="w-fit gap-2" onclick={() => goto(`/projects/${projectId}?tab=actions`)}>
    <ArrowLeft class="h-4 w-4" /> Back to Actions
  </Button>

  {#if loaded && !described}
    <Alert variant="destructive">
      <CircleAlert />
      <AlertTitle>No release</AlertTitle>
      <AlertDescription>
        {loadError ?? 'This project declares no release: in .portal/pipeline.yml.'}
      </AlertDescription>
    </Alert>
  {:else if described}
    {@const r = described.release}
    <Card>
      <CardHeader class="pb-2">
        <CardTitle class="flex items-center gap-2 text-base">
          <Rocket class="h-4 w-4" /> Release
          {#if r.kind}<Badge variant="outline">{r.kind}</Badge>{/if}
        </CardTitle>
        <CardDescription>
          Ships <code>{r.ship}</code> ({described.ship.steps.length} steps){#if described.rehearse};
            rehearses with <code>{r.rehearse}</code>{/if}.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div class="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={!described.rehearse}
            onclick={() => launch('rehearse')}
          >
            <FlaskConical class="mr-2 h-4 w-4" /> Rehearse…
          </Button>
          <Button size="sm" onclick={() => launch('ship')}>
            <Rocket class="mr-2 h-4 w-4" /> Ship…
          </Button>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="pb-2">
        <CardTitle class="text-base">Ship plan</CardTitle>
        <CardDescription>One node per app once the apps-build fan-out resolves.</CardDescription>
      </CardHeader>
      <CardContent>
        <WorkflowTree plan={toPlan(described.ship.steps)} />
      </CardContent>
    </Card>

    {#if described.rehearse}
      <Card>
        <CardHeader class="pb-2">
          <CardTitle class="text-base">Rehearse plan</CardTitle>
        </CardHeader>
        <CardContent>
          <WorkflowTree plan={toPlan(described.rehearse.steps)} />
        </CardContent>
      </Card>
    {/if}

    <Card>
      <CardHeader class="pb-2">
        <CardTitle class="text-base">Run history</CardTitle>
        <CardDescription>{runs.length} run{runs.length === 1 ? '' : 's'}.</CardDescription>
      </CardHeader>
      <CardContent class="space-y-2">
        {#if !runs.length}
          <p class="text-sm text-muted-foreground">No runs yet.</p>
        {/if}
        {#each runs as run (run.id)}
          <Button
            variant="outline"
            size="sm"
            class="h-auto w-full flex-col items-stretch gap-1.5 py-2 font-normal"
            onclick={() => goto(`/projects/${projectId}/pipelines/run/${run.id}`)}
          >
            <span class="flex w-full justify-between">
              <span>{run.pipelineName}</span>
              <span class="text-muted-foreground">
                {run.status} · {new Date(run.startedAt).toLocaleString()}
              </span>
            </span>
            {#if run.progress && ['pending', 'running'].includes(run.status)}
              <RunProgressBar progress={run.progress} compact />
            {/if}
          </Button>
        {/each}
      </CardContent>
    </Card>

    {#if project}
      <ShipDialog {project} release={r} {apps} mode={launchMode} bind:open={confirmOpen} />
    {/if}
  {/if}
</div>
