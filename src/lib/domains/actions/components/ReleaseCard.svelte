<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
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
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible';
  import { aiProviderService } from '$lib/domains/ai/services/aiProviderService';
  import { toast } from '$lib/utils/toast';
  import Rocket from '~icons/fa6-solid/rocket';
  import FlaskConical from '~icons/fa6-solid/flask';
  import LoaderCircle from '~icons/fa6-solid/spinner';
  import CircleAlert from '~icons/fa6-solid/circle-exclamation';
  import TriangleAlert from '~icons/fa6-solid/triangle-exclamation';
  import Pencil from '~icons/fa6-solid/pencil';
  import Settings2 from '~icons/fa6-solid/sliders';
  import type { Project } from '$lib/domains/projects/types';
  import type { ReleaseDescription, ReleaseRun } from '../types';
  import ReleaseEditor from './ReleaseEditor.svelte';
  import RunProgressBar from './RunProgressBar.svelte';
  import ShipDialog from './ShipDialog.svelte';

  type Proposal = { yaml: string; source: 'scripts' | 'generated'; summary: string };
  type Issue = { level: 'error' | 'warn'; message: string };

  let { project }: { project: Project } = $props();

  const projectId = $derived(parseInt(project.id, 10));

  let loaded = $state(false);
  let described = $state<ReleaseDescription | null>(null);
  let loadError = $state<string | null>(null);
  let runs = $state<ReleaseRun[]>([]);
  let confirmOpen = $state(false);
  let launchMode = $state<'ship' | 'rehearse'>('ship');

  // The apps the launcher can ship as a subset, from the resolved ship plan: each
  // `apps-build` fan-out node has id `<action>:<app>`. One source of truth
  // (scripts/release-apps.txt) — never hardcoded here.
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

  function launch(mode: 'ship' | 'rehearse') {
    launchMode = mode;
    confirmOpen = true;
  }

  let proposal = $state<Proposal | null>(null);
  let proposeError = $state<string | null>(null);
  let yaml = $state('');
  let yamlError = $state<string | null>(null);
  let saving = $state(false);
  let customizeOpen = $state(false);

  let issues = $state<Issue[]>([]);
  let issuesOpen = $state(false);

  let editOpen = $state(false);
  let editYaml = $state('');
  let editError = $state<string | null>(null);

  let aiProbe: Promise<boolean> | null = null;
  const aiAvailable = () =>
    (aiProbe ??= aiProviderService.testProvider('AgentPlatform').then(
      () => true,
      () => false
    ));

  async function refresh() {
    loadError = null;
    editOpen = false;
    editYaml = '';
    editError = null;
    try {
      described = await invoke<ReleaseDescription | null>('release_describe', { projectId });
    } catch (e) {
      described = null;
      loadError = String(e);
    }
    runs = described
      ? await invoke<ReleaseRun[]>('release_list', { projectId, limit: 5 }).catch(() => [])
      : [];
    issues = described
      ? await invoke<Issue[]>('release_health', { projectId }).catch(() => [])
      : [];
    loaded = true;
    if (!described) await propose();
  }

  async function propose() {
    proposeError = null;
    try {
      proposal = await invoke<Proposal>('release_propose', { projectId });
      yaml = proposal.yaml;
    } catch (e) {
      proposal = null;
      proposeError = String(e);
    }
  }

  async function validate(text: string, set: (msg: string | null) => void) {
    try {
      await invoke('release_validate', { yaml: text });
      set(null);
      return true;
    } catch (e) {
      set(String(e));
      return false;
    }
  }

  async function save(text: string, set: (msg: string | null) => void) {
    saving = true;
    try {
      if (!(await validate(text, set))) return;
      const path = await invoke<string>('release_write', { projectId, yaml: text });
      toast.success(`Release file written: ${path}`);
      await refresh();
    } catch (e) {
      toast.error(String(e));
    } finally {
      saving = false;
    }
  }

  async function openEdit(open: boolean) {
    if (!open || editYaml) return;
    for (const name of ['pipeline.yml', 'pipeline.yaml']) {
      try {
        editYaml = await invoke<string>('coder_read_file', {
          workspaceRoot: project.path,
          path: `.portal/${name}`,
        });
        return;
      } catch {
        // try the next extension
      }
    }
    editError = 'Could not read .portal/pipeline.yml';
  }

  // Rehearsing and shipping both go through ShipDialog (scope picker), shared with
  // the project header's Release button.

  // Runs can start outside this card (daemon, another window), and there is no
  // pipeline event to listen for — so re-read the run list while the card is on screen.
  // ponytail: polling; swap for a pipeline-run event if the backend grows one.
  async function refreshRuns() {
    if (!described || document.hidden) return;
    runs = await invoke<ReleaseRun[]>('release_list', { projectId, limit: 5 }).catch(() => runs);
  }

  onMount(() => {
    refresh();
    const timer = setInterval(refreshRuns, 3000);
    return () => clearInterval(timer);
  });
</script>

{#if loaded && !described}
  {#if loadError}
    <Alert variant="destructive">
      <CircleAlert />
      <AlertTitle>Release file has a problem</AlertTitle>
      <AlertDescription>
        <p>{loadError}</p>
        <p class="text-muted-foreground">Fix .portal/pipeline.yml, or recreate it below</p>
      </AlertDescription>
    </Alert>
  {/if}
  <Card>
    <CardHeader class="pb-2">
      <CardTitle class="flex items-center gap-2 text-base">
        <Rocket class="h-4 w-4" /> Set up release
        {#if proposal}<Badge variant="outline">{proposal.source}</Badge>{/if}
      </CardTitle>
      <CardDescription>
        {#if proposal}
          {proposal.summary}
        {:else if proposeError}
          {proposeError}
        {:else}
          Looking for something to release…
        {/if}
      </CardDescription>
    </CardHeader>
    {#if proposal}
      <CardContent class="space-y-3">
        <Button size="sm" disabled={saving} onclick={() => save(yaml, (m) => (yamlError = m))}>
          {#if saving}<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />{/if}
          Create release file
        </Button>
        <Collapsible bind:open={customizeOpen}>
          <CollapsibleTrigger>
            {#snippet child({ props })}
              <Button {...props} variant="ghost" size="sm" class="text-muted-foreground">
                <Settings2 class="mr-2 h-4 w-4" /> Customize
              </Button>
            {/snippet}
          </CollapsibleTrigger>
          <CollapsibleContent class="pt-2">
            {#if customizeOpen}
              <ReleaseEditor
                {projectId}
                bind:yaml
                bind:error={yamlError}
                {aiAvailable}
                onvalidate={() => validate(yaml, (m) => (yamlError = m))}
              />
            {/if}
          </CollapsibleContent>
        </Collapsible>
      </CardContent>
    {/if}
  </Card>
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
    <CardContent class="space-y-3">
      <div class="flex flex-wrap gap-2">
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
        <Button
          variant="ghost"
          size="sm"
          class="text-muted-foreground"
          onclick={() => goto(`/projects/${project.id}/release`)}
        >
          Details
        </Button>
      </div>
      {#each runs as run (run.id)}
        <Button
          variant="outline"
          size="sm"
          class="h-auto w-full flex-col items-stretch gap-1.5 py-2 font-normal"
          onclick={() => goto(`/projects/${project.id}/pipelines/run/${run.id}`)}
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
      {#if issues.length}
        <Collapsible bind:open={issuesOpen}>
          <CollapsibleTrigger>
            {#snippet child({ props })}
              <Button {...props} variant="ghost" size="sm" class="h-auto p-0 text-muted-foreground">
                {issues.length}
                {issues.length === 1 ? 'thing' : 'things'} to check
              </Button>
            {/snippet}
          </CollapsibleTrigger>
          <CollapsibleContent>
            <ul class="space-y-1 pt-2 text-sm">
              {#each issues as issue (issue.message)}
                <li class="flex items-start gap-2">
                  {#if issue.level === 'error'}
                    <CircleAlert class="mt-0.5 h-4 w-4 shrink-0 text-destructive" />
                  {:else}
                    <TriangleAlert class="mt-0.5 h-4 w-4 shrink-0 text-amber-500" />
                  {/if}
                  <span>{issue.message}</span>
                </li>
              {/each}
            </ul>
          </CollapsibleContent>
        </Collapsible>
      {/if}
      <Collapsible bind:open={editOpen} onOpenChange={openEdit}>
        <CollapsibleTrigger>
          {#snippet child({ props })}
            <Button {...props} variant="ghost" size="sm" class="text-muted-foreground">
              <Pencil class="mr-2 h-4 w-4" /> Edit with AI
            </Button>
          {/snippet}
        </CollapsibleTrigger>
        <CollapsibleContent class="space-y-3 pt-2">
          {#if editOpen && editYaml}
            <ReleaseEditor
              {projectId}
              bind:yaml={editYaml}
              bind:error={editError}
              {aiAvailable}
              onvalidate={() => validate(editYaml, (m) => (editError = m))}
            />
            <Button
              size="sm"
              disabled={saving}
              onclick={() => save(editYaml, (m) => (editError = m))}
            >
              {#if saving}<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />{/if}
              Apply
            </Button>
          {:else if editError}
            <p class="text-xs text-destructive">{editError}</p>
          {/if}
        </CollapsibleContent>
      </Collapsible>
    </CardContent>
  </Card>

  <ShipDialog {project} release={r} {apps} mode={launchMode} bind:open={confirmOpen} />
{/if}
