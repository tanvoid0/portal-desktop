<script lang="ts">
  import { Progress } from '$lib/components/ui/progress';
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Badge } from '$lib/components/ui/badge';
  import {
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
  } from '$lib/components/ui/card';
  import { toast } from '$lib/utils/toast';
  import {
    aiModelOpsService,
    BUILD_STAGES,
    isJobRunning,
  } from '$lib/domains/ai/services/aiModelOpsService';
  import type {
    ModelProject,
    ModelRegistryEntry,
    ModelBuildJob,
    OllamaModel,
  } from '$lib/domains/ai/types/index.js';
  import Plus from '~icons/fa6-solid/plus';
  import Refresh from '~icons/fa6-solid/arrows-rotate';
  import Play from '~icons/fa6-solid/play';
  import DownloadIcon from '~icons/fa6-solid/download';
  import Close from '~icons/fa6-solid/xmark';
  import Spinner from '~icons/fa6-solid/spinner';
  import Chip from '~icons/fa6-solid/microchip';
  import Archive from '~icons/fa6-solid/box-archive';

  let projects = $state<ModelProject[]>([]);
  let ollama = $state<OllamaModel[]>([]);
  let registry = $state<ModelRegistryEntry[]>([]);
  let selected = $state<string | null>(null);
  let job = $state<ModelBuildJob | null>(null);
  let selectedStages = $state<string[]>([...BUILD_STAGES]);
  let registerAlias = $state('');
  let pullName = $state('');
  let loaded = $state(false);
  let busy = $state(false);

  let showNewProject = $state(false);
  let newName = $state('');
  let newDescription = $state('');
  let newBaseModel = $state('');

  let pollTimer: ReturnType<typeof setInterval> | undefined;

  const selectedProject = $derived(projects.find((p) => p.name === selected) ?? null);
  const orderedStages = $derived(BUILD_STAGES.filter((s) => selectedStages.includes(s)));

  onMount(() => {
    void refresh();
    return () => stopPolling();
  });

  async function refresh() {
    const [p, o, r] = await Promise.allSettled([
      aiModelOpsService.projects(),
      aiModelOpsService.ollamaModels(),
      aiModelOpsService.registry(),
    ]);
    if (p.status === 'fulfilled') {
      projects = p.value;
      if (!selected) selected = projects[0]?.name ?? null;
    } else {
      toast.error('Failed to load model projects', p.reason);
    }
    // Ollama being absent is normal on a fresh install — stay quiet, don't banner it.
    if (o.status === 'fulfilled') ollama = o.value;
    if (r.status === 'fulfilled') registry = r.value;
    else toast.error('Failed to load registry', r.reason);
    loaded = true;
  }

  function toggleStage(stage: string, on: boolean) {
    selectedStages = on
      ? [...selectedStages.filter((s) => s !== stage), stage]
      : selectedStages.filter((s) => s !== stage);
  }

  function startPolling() {
    stopPolling();
    pollTimer = setInterval(tickJob, 2000);
  }
  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = undefined;
    }
  }

  async function tickJob() {
    if (!job || !isJobRunning(job.status)) {
      stopPolling();
      return;
    }
    try {
      const updated = await aiModelOpsService.job(job.id);
      const wasRunning = isJobRunning(job.status);
      job = updated;
      if (wasRunning && !isJobRunning(updated.status)) {
        stopPolling();
        toast.success(`Build job #${updated.id}: ${updated.status}`);
        await refresh(); // a finished job changes the registry and Ollama lists
      }
    } catch (e) {
      stopPolling();
      toast.error('Lost track of the build job', e);
    }
  }

  function watchJob(started: ModelBuildJob) {
    job = started;
    if (isJobRunning(started.status)) startPolling();
    else void refresh();
  }

  async function startBuild() {
    if (!selected) return toast.error('Pick a model project first.');
    if (orderedStages.length === 0) return toast.error('Pick at least one stage.');
    busy = true;
    try {
      watchJob(
        await aiModelOpsService.startBuild(
          selected,
          [...orderedStages],
          registerAlias.trim() || null
        )
      );
    } catch (e) {
      toast.error('Failed to start build', e);
    } finally {
      busy = false;
    }
  }

  async function createProject() {
    if (!newName.trim()) return toast.error('Name is required.');
    busy = true;
    try {
      await aiModelOpsService.createProject(
        newName.trim(),
        newDescription.trim() || null,
        newBaseModel.trim() || null
      );
      toast.success(`Created model project “${newName.trim()}”.`);
      showNewProject = false;
      newName = newDescription = newBaseModel = '';
      await refresh();
    } catch (e) {
      toast.error('Failed to create project', e);
    } finally {
      busy = false;
    }
  }

  async function pullModel() {
    const name = pullName.trim();
    if (!name) return toast.error('Enter a model name to pull.');
    busy = true;
    try {
      watchJob(await aiModelOpsService.pullModel(name));
      pullName = '';
    } catch (e) {
      toast.error('Failed to pull model', e);
    } finally {
      busy = false;
    }
  }

  async function cancelJob() {
    if (!job) return;
    try {
      job = await aiModelOpsService.cancelJob(job.id);
      stopPolling();
    } catch (e) {
      toast.error('Failed to cancel job', e);
    }
  }

  async function activateEntry(entry: ModelRegistryEntry) {
    try {
      await aiModelOpsService.activateEntry(entry.id);
      toast.success(`Activated ${entry.ollama_tag}.`);
      await refresh();
    } catch (e) {
      toast.error('Failed to activate adapter', e);
    }
  }

  // ----- display helpers -----
  function num(p: Record<string, unknown>, key: string): number | undefined {
    const v = p[key];
    return typeof v === 'number' ? v : undefined;
  }
  function pct(p: Record<string, unknown>): number | undefined {
    const step = num(p, 'step');
    const total = num(p, 'total_steps');
    if (step === undefined || !total) return undefined;
    return Math.min(100, Math.round((step / total) * 100));
  }
  function progressLine(p: Record<string, unknown>): string {
    const parts: string[] = [];
    const step = num(p, 'step');
    const total = num(p, 'total_steps');
    if (step !== undefined && total) parts.push(`step ${step}/${total}`);
    const loss = num(p, 'loss');
    if (loss !== undefined) parts.push(`loss ${loss.toFixed(4)}`);
    const epoch = num(p, 'epoch');
    if (epoch !== undefined) parts.push(`epoch ${epoch.toFixed(2)}`);
    const eta = num(p, 'eta_s');
    if (eta !== undefined) parts.push(`eta ${shortDuration(eta)}`);
    return parts.join(' · ');
  }
  function shortDuration(seconds: number): string {
    const s = Math.max(0, Math.floor(seconds));
    if (s < 60) return `${s}s`;
    if (s < 3600) return `${Math.floor(s / 60)}m ${String(s % 60).padStart(2, '0')}s`;
    return `${Math.floor(s / 3600)}h ${String(Math.floor((s % 3600) / 60)).padStart(2, '0')}m`;
  }
  function formatSize(bytes: number | null | undefined): string {
    if (!bytes) return '';
    const gb = bytes / 1e9;
    if (gb >= 1) return `${gb.toFixed(1)} GB`;
    return `${Math.round(bytes / 1e6)} MB`;
  }
  function jobTone(status: string): 'default' | 'secondary' | 'destructive' {
    if (status === 'completed') return 'default';
    if (status === 'failed' || status === 'cancelled') return 'destructive';
    return 'secondary';
  }
</script>

<svelte:head>
  <title>Model ops - Portal Desktop</title>
</svelte:head>

<div class="space-y-4">
  <!-- Toolbar -->
  <div class="flex flex-wrap items-center justify-between gap-3">
    <p class="text-sm text-muted-foreground">
      Fine-tune adapters, watch build jobs, and manage local models.
    </p>
    <div class="flex gap-2">
      <Button variant="secondary" size="sm" onclick={() => (showNewProject = !showNewProject)}>
        <Plus class="mr-2 h-4 w-4" /> New project
      </Button>
      <Button variant="outline" size="sm" onclick={refresh} disabled={busy}>
        <Refresh class="mr-2 h-4 w-4" /> Refresh
      </Button>
    </div>
  </div>

  <!-- New project form -->
  {#if showNewProject}
    <Card>
      <CardHeader>
        <CardTitle>New model project</CardTitle>
      </CardHeader>
      <CardContent class="space-y-3">
        <div class="space-y-1.5">
          <Label for="np-name">Name</Label>
          <Input id="np-name" placeholder="my-adapter" bind:value={newName} />
        </div>
        <div class="space-y-1.5">
          <Label for="np-desc">Description</Label>
          <Input id="np-desc" placeholder="What is it for?" bind:value={newDescription} />
        </div>
        <div class="space-y-1.5">
          <Label for="np-base">Base model</Label>
          <Input id="np-base" placeholder="qwen2.5:7b" bind:value={newBaseModel} />
        </div>
        <div class="flex gap-2">
          <Button size="sm" onclick={createProject} disabled={busy || !newName.trim()}
            >Create</Button
          >
          <Button variant="ghost" size="sm" onclick={() => (showNewProject = false)}>Cancel</Button>
        </div>
      </CardContent>
    </Card>
  {/if}

  <!-- Projects + build launcher -->
  <Card>
    <CardHeader>
      <CardTitle>Model projects</CardTitle>
      <CardDescription>Pick a project, choose stages, and run the pipeline.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if projects.length === 0}
        <div class="flex flex-col items-center gap-2 py-8 text-muted-foreground">
          <Chip class="h-6 w-6" />
          <span class="text-sm">{loaded ? 'No model projects yet.' : 'Loading…'}</span>
        </div>
      {:else}
        <div class="space-y-1">
          {#each projects as project (project.id)}
            <Button
              variant="ghost"
              onclick={() => (selected = project.name)}
              class="flex h-auto w-full items-center justify-start gap-3 rounded-md border px-3 py-2 text-left font-normal {selected ===
              project.name
                ? 'border-primary bg-accent'
                : 'border-transparent hover:bg-accent/50'}"
            >
              <span class="font-medium">{project.name}</span>
              <span class="truncate text-sm text-muted-foreground">
                {project.description ?? '—'}
              </span>
              <Badge variant="secondary" class="ml-auto">
                {project.registry_entries.length}
                {project.registry_entries.length === 1 ? 'version' : 'versions'}
              </Badge>
            </Button>
          {/each}
        </div>

        <!-- Build launcher -->
        <div class="space-y-3 border-t pt-4">
          <div class="space-y-1.5">
            <span class="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
              Pipeline stages
            </span>
            <div class="flex flex-wrap gap-4">
              {#each BUILD_STAGES as stage (stage)}
                <div class="flex items-center gap-2">
                  <Checkbox
                    id={`stage-${stage}`}
                    checked={selectedStages.includes(stage)}
                    onCheckedChange={(v) => toggleStage(stage, v === true)}
                  />
                  <Label
                    for={`stage-${stage}`}
                    class="cursor-pointer text-sm font-normal capitalize"
                  >
                    {stage}
                  </Label>
                </div>
              {/each}
            </div>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Input
              class="w-full sm:w-72"
              placeholder="Register as (optional alias)"
              bind:value={registerAlias}
            />
            <Button
              onclick={startBuild}
              disabled={busy || !selectedProject || orderedStages.length === 0}
            >
              {#if busy}
                <Spinner class="mr-2 h-4 w-4 animate-spin" />
              {:else}
                <Play class="mr-2 h-4 w-4" />
              {/if}
              Start build
            </Button>
          </div>
        </div>
      {/if}
    </CardContent>
  </Card>

  <!-- Build job monitor -->
  {#if job}
    <Card>
      <CardHeader class="flex-row items-center justify-between space-y-0">
        <CardTitle>Build job #{job.id}</CardTitle>
        <div class="flex gap-2">
          {#if isJobRunning(job.status)}
            <Button variant="outline" size="sm" onclick={cancelJob}>Cancel</Button>
          {/if}
          <Button variant="ghost" size="icon" class="h-8 w-8" onclick={() => (job = null)}>
            <Close class="h-4 w-4" />
          </Button>
        </div>
      </CardHeader>
      <CardContent class="space-y-3">
        <div class="flex flex-wrap items-center gap-2 text-sm">
          <Badge variant={jobTone(job.status)} class="capitalize">{job.status}</Badge>
          <span class="text-muted-foreground">{job.stages.join(' → ')}</span>
          {#if job.current_stage}
            <span class="text-muted-foreground">· {job.current_stage}</span>
          {/if}
        </div>

        {#if pct(job.progress) !== undefined}
          <Progress value={pct(job.progress)} class="bg-muted" />
        {/if}
        {#if progressLine(job.progress)}
          <p class="text-xs text-muted-foreground">{progressLine(job.progress)}</p>
        {/if}

        {#if job.error_message}
          <p class="rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {job.error_message}
          </p>
        {/if}
        {#if job.log_tail}
          <pre
            class="max-h-48 overflow-auto rounded-md bg-muted p-3 font-mono text-xs whitespace-pre-wrap">{job.log_tail}</pre>
        {/if}
      </CardContent>
    </Card>
  {/if}

  <!-- Local models (Ollama) -->
  <Card>
    <CardHeader>
      <CardTitle>Local models</CardTitle>
      <CardDescription>Models Ollama has on this machine.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-3">
      <div class="flex flex-wrap items-center gap-2">
        <Input class="w-full sm:w-72" placeholder="qwen2.5:7b" bind:value={pullName} />
        <Button variant="secondary" onclick={pullModel} disabled={busy || !pullName.trim()}>
          <DownloadIcon class="mr-2 h-4 w-4" /> Pull
        </Button>
      </div>
      {#if ollama.length === 0}
        <p class="py-4 text-center text-sm text-muted-foreground">
          {loaded ? 'No local models found (is Ollama running?).' : 'Loading…'}
        </p>
      {:else}
        <div class="space-y-1">
          {#each ollama as model (model.name)}
            <div class="flex items-center justify-between rounded-md px-3 py-1.5 text-sm">
              <span class="font-mono">{model.name}</span>
              <span class="text-muted-foreground">{formatSize(model.size)}</span>
            </div>
          {/each}
        </div>
      {/if}
    </CardContent>
  </Card>

  <!-- Registry -->
  <Card>
    <CardHeader>
      <CardTitle>Registry</CardTitle>
      <CardDescription>Adapters registered from builds.</CardDescription>
    </CardHeader>
    <CardContent>
      {#if registry.length === 0}
        <div class="flex flex-col items-center gap-2 py-8 text-muted-foreground">
          <Archive class="h-6 w-6" />
          <span class="text-sm">{loaded ? 'Nothing registered yet.' : 'Loading…'}</span>
        </div>
      {:else}
        <div class="space-y-1">
          {#each registry as entry (entry.id)}
            <div class="flex flex-wrap items-center gap-3 rounded-md px-3 py-2 text-sm">
              <span class="font-medium">
                {entry.project_name ?? `#${entry.project_id}`}
              </span>
              <span class="text-muted-foreground">v{entry.version}</span>
              <span class="font-mono text-muted-foreground">{entry.ollama_tag}</span>
              {#if entry.eval_score !== null}
                <span class="text-muted-foreground">eval {entry.eval_score.toFixed(3)}</span>
              {/if}
              <div class="ml-auto flex items-center gap-2">
                {#if entry.is_active}
                  <Badge>Active</Badge>
                {:else}
                  <Button variant="outline" size="sm" onclick={() => activateEntry(entry)}>
                    Activate
                  </Button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </CardContent>
  </Card>
</div>
