<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '$lib/components/ui/alert-dialog';
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
  import {
    Rocket,
    FlaskConical,
    LoaderCircle,
    CircleAlert,
    TriangleAlert,
    Pencil,
    Settings2,
  } from '@lucide/svelte';
  import type { Project } from '$lib/domains/projects/types';
  import type { ReleaseDescription, ReleaseRun } from '../types';
  import ReleaseEditor from './ReleaseEditor.svelte';

  type Proposal = { yaml: string; source: 'scripts' | 'generated'; summary: string };
  type Issue = { level: 'error' | 'warn'; message: string };

  let { project }: { project: Project } = $props();

  const projectId = $derived(parseInt(project.id, 10));

  let loaded = $state(false);
  let described = $state<ReleaseDescription | null>(null);
  let loadError = $state<string | null>(null);
  let runs = $state<ReleaseRun[]>([]);
  let busy = $state(false);
  let confirmOpen = $state(false);

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

  async function start(command: 'release_rehearse' | 'release_ship') {
    busy = true;
    try {
      const run = await invoke<{ id: string }>(command, { projectId });
      goto(`/projects/${project.id}/pipelines/run/${run.id}`);
    } catch (e) {
      toast.error(String(e));
    } finally {
      busy = false;
    }
  }

  onMount(refresh);
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
      <div class="flex gap-2">
        <Button
          variant="outline"
          size="sm"
          disabled={busy || !described.rehearse}
          onclick={() => start('release_rehearse')}
        >
          <FlaskConical class="mr-2 h-4 w-4" /> Rehearse
        </Button>
        <Button size="sm" disabled={busy} onclick={() => (confirmOpen = true)}>
          <Rocket class="mr-2 h-4 w-4" /> Ship…
        </Button>
      </div>
      {#each runs as run (run.id)}
        <Button
          variant="outline"
          size="sm"
          class="w-full justify-between font-normal"
          onclick={() => goto(`/projects/${project.id}/pipelines/run/${run.id}`)}
        >
          <span>{run.pipelineName}</span>
          <span class="text-muted-foreground">
            {run.status} · {new Date(run.startedAt).toLocaleString()}
          </span>
        </Button>
      {/each}
      {#if issues.length}
        <Collapsible bind:open={issuesOpen}>
          <CollapsibleTrigger>
            {#snippet child({ props })}
              <Button {...props} variant="ghost" size="sm" class="text-muted-foreground h-auto p-0">
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
                    <CircleAlert class="text-destructive mt-0.5 h-4 w-4 shrink-0" />
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
            <p class="text-destructive text-xs">{editError}</p>
          {/if}
        </CollapsibleContent>
      </Collapsible>
    </CardContent>
  </Card>

  <AlertDialog bind:open={confirmOpen}>
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>Ship {project.name}?</AlertDialogTitle>
        <AlertDialogDescription>
          This runs <code>{r.ship}</code> for real. It can deploy servers and publish builds, and killing
          it midway does not undo what already went out.
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction onclick={() => start('release_ship')}>Ship</AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
{/if}
