<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Textarea } from '$lib/components/ui/textarea';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { PageEmpty, PageLoading } from '$lib/components/shell';
  import { toast } from '$lib/utils/toast';
  import Play from '~icons/fa6-solid/play';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Workflow from '~icons/fa6-solid/diagram-project';
  import Terminal from '~icons/fa6-solid/terminal';
  import GitBranch from '~icons/fa6-solid/code-branch';
  import Layers from '~icons/fa6-solid/layer-group';
  import History from '~icons/fa6-solid/clock-rotate-left';
  import Stethoscope from '~icons/fa6-solid/stethoscope';
  import Copy from '~icons/fa6-solid/copy';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import type { Project } from '$lib/domains/projects/types';
  import {
    actions,
    type UnifiedAction,
    type UnifiedWorkflow,
    type ActionRunResult,
    type ActionSource,
  } from '$lib/domains/actions';
  import { GitHubProjectActionsPanel } from '$lib/domains/github';
  import ReleaseCard from './ReleaseCard.svelte';
  import {
    scriptExecutionService,
    type ScriptExecutionInfo,
  } from '$lib/domains/scripts/services/scriptExecutionService';
  import {
    formatExecutionDuration,
    getExecutionStatusBadgeVariant,
    getExecutionStatusBadgeClass,
    getExecutionStatusColor,
    getExecutionStatusIcon,
  } from '$lib/domains/projects/pipelines/utils/executionDisplay';
  import { untrack } from 'svelte';
  import { goto } from '$app/navigation';
  import { coderSession } from '$lib/domains/coder/state/coderSession.svelte.js';
  import {
    diagnose,
    generatePipeline,
    getDoctorRun,
    getGenerateRun,
    resetDoctorRun,
    dismissReviewPointer,
    verifyProposal,
  } from '../state/doctor.svelte';
  import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';
  import type { PipelineFixReport } from '../state/doctor.svelte';

  interface Props {
    project: Project;
    enabled?: boolean;
  }

  let { project, enabled = true }: Props = $props();

  type SourceFilter = 'all' | ActionSource;

  let loading = $state(true);
  let running = $state(false);
  let actionsList = $state<UnifiedAction[]>([]);
  let workflows = $state<UnifiedWorkflow[]>([]);
  let warnings = $state<string[]>([]);
  let filePath = $state<string | undefined>();
  let sourceFilter = $state<SourceFilter>('all');
  let selected = $state<Set<string>>(new Set());
  let lastResult = $state<ActionRunResult | null>(null);
  let seenProjectId: string | number | undefined;
  let runHistory = $state<ScriptExecutionInfo[]>([]);
  let historyLoading = $state(false);
  let selectedExecution = $state<ScriptExecutionInfo | null>(null);
  // Live, not-yet-persisted stdout/stderr for the running execution (QA-130).
  let liveOutput = $state('');

  const runningExecId = $derived(
    selectedExecution?.status === 'running' ? selectedExecution.id : null
  );

  const doctorRun = $derived(getDoctorRun(project.id));
  const doctorBusy = $derived(
    doctorRun.status === 'diagnosing' || doctorRun.status === 'verifying'
  );
  // Doctor state outlives this component (QA-156): back from Coder mid-run, the
  // card still shows the failure being worked on.
  const shownFailure = $derived(lastResult?.failure ?? doctorRun.failure);

  // Stage 4 — requirement-driven pipeline generation.
  let requirement = $state('');
  const genRun = $derived(getGenerateRun(project.id));
  const genRunning = $derived(
    genRun.status === 'generating' ||
      genRun.status === 'validating' ||
      genRun.status === 'previewing'
  );

  async function generate() {
    if (genRunning || !requirement.trim()) return;
    await generatePipeline(project, requirement);
    // Pick up the new/changed actions the agent wrote.
    await loadCatalog({ silent: true });
    await loadHistory();
    const run = getGenerateRun(project.id);
    if (run.status === 'done') toast.success('Pipeline generated — review the changes in Coder');
    else if (run.status === 'failed' && run.message) toast.error(run.message);
  }

  async function openThread(id: string) {
    await coderSession.selectThread(id);
    void goto('/ai/coder');
  }

  async function verifyFix() {
    const verify = await verifyProposal(project);
    if (verify) {
      lastResult = verify;
      const ls = verify.steps.find((s) => s.executionId);
      if (ls?.executionId) void openExecution(ls.executionId);
      if (verify.success) toast.success('Verified — review the changes in Coder');
    }
    await loadHistory();
  }

  function reportMarkdown(r: PipelineFixReport): string {
    const profile = r.frameworks.length
      ? `${r.packageManager} · ${r.frameworks.join(', ')}`
      : r.packageManager;
    return [
      `Portal default needs a fix — action \`${r.actionId}\``,
      '',
      `- Profile: ${profile}`,
      `- Portal default command (failed): \`${r.defaultCommand}\``,
      `- Command that passed: \`${r.greenCommand}\``,
      '',
      'Portal generated the wrong default for this profile. Update `buildDefaultActions`',
      'in `src/lib/domains/actions/defaults.ts` so this profile maps to the passing command.',
    ].join('\n');
  }

  async function copyReport(r: PipelineFixReport) {
    try {
      await navigator.clipboard.writeText(reportMarkdown(r));
      toast.success('Report copied to clipboard');
    } catch {
      toast.error('Could not copy report');
    }
  }

  function normalizePath(path: string): string {
    return path.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
  }

  function isProjectExecution(exec: ScriptExecutionInfo): boolean {
    if (!exec.workingDirectory) return false;
    return normalizePath(exec.workingDirectory) === normalizePath(project.path);
  }

  const FILTERS: { id: SourceFilter; label: string }[] = [
    { id: 'all', label: 'All' },
    { id: 'local', label: 'Local' },
    { id: 'file', label: 'File' },
    { id: 'github', label: 'GitHub' },
    { id: 'n8n', label: 'n8n' },
  ];

  const filteredActions = $derived(
    sourceFilter === 'all' ? actionsList : actionsList.filter((a) => a.source === sourceFilter)
  );

  const filteredWorkflows = $derived(
    sourceFilter === 'all' ? workflows : workflows.filter((w) => w.source === sourceFilter)
  );

  function sourceIcon(source: ActionSource) {
    if (source === 'github') return GitBranch;
    if (source === 'n8n') return Layers;
    if (source === 'file') return Workflow;
    return Terminal;
  }

  async function loadCatalog(opts?: { silent?: boolean }) {
    if (!enabled) return;
    const silent = opts?.silent ?? actionsList.length > 0;
    if (!silent) loading = true;
    try {
      const catalog = await actions.catalogForProject(project);
      actionsList = catalog.actions;
      workflows = catalog.workflows;
      warnings = catalog.warnings;
      filePath = catalog.filePath;
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to load actions catalog');
    } finally {
      loading = false;
    }
  }

  async function loadHistory() {
    historyLoading = true;
    try {
      const recent = await scriptExecutionService.getRecentExecutions(50);
      runHistory = recent.filter(isProjectExecution);
      // Keep selected detail in sync with fresh data
      if (selectedExecution) {
        const updated = runHistory.find((e) => e.id === selectedExecution!.id);
        selectedExecution = updated ?? selectedExecution;
      }
    } catch {
      // history is best-effort
    } finally {
      historyLoading = false;
    }
  }

  async function openExecution(executionId: string) {
    try {
      const exec = await scriptExecutionService.getExecution(executionId);
      if (exec) {
        selectedExecution = exec;
        return;
      }
    } catch {
      // fall through
    }
    const fromList = runHistory.find((e) => e.id === executionId);
    if (fromList) selectedExecution = fromList;
  }

  function toggleSelect(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selected = next;
  }

  async function runSelected() {
    const ids = [...selected];
    if (ids.length === 0) {
      toast.info('Select one or more actions to run');
      return;
    }
    await runTarget(ids.length === 1 ? ids[0]! : ids);
  }

  async function runTarget(target: string | string[], event?: Event) {
    event?.preventDefault();
    event?.stopPropagation();
    if (running) return;

    running = true;
    lastResult = null;
    liveOutput = '';
    resetDoctorRun(project.id);
    try {
      const result = await actions.forProject(project).run(target, {
        // Surface the run the moment its process spawns — not after it finishes.
        onStepStart: ({ executionId }) => {
          void openExecution(executionId);
          void loadHistory();
        },
      });
      lastResult = result;
      // Prefer showing the script execution record (has persisted output)
      const localStep = result.steps.find((s) => s.executionId);
      if (localStep?.executionId) {
        await openExecution(localStep.executionId);
      } else {
        selectedExecution = null;
      }
      if (result.success) {
        toast.success(
          typeof target === 'string' ? `Ran ${target}` : `Ran ${target.length} actions`,
          result.remoteRunId ? `Remote: ${result.remoteRunId}` : undefined
        );
      } else {
        const failed =
          result.steps.find((s) => s.error) ?? result.steps.find((s) => s.status === 'failed');
        // Short + auto-dismissing: the failure card / run details carry the output.
        const err =
          (failed?.exitCode != null ? `Exit code ${failed.exitCode}` : null) ||
          failed?.error?.trim().split('\n')[0]?.slice(0, 120) ||
          'Action run finished with errors';
        toast.error(typeof target === 'string' ? `${target} failed` : 'Run failed', err, {
          duration: 5000,
        });
      }
      await loadHistory();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err));
    } finally {
      running = false;
    }
  }

  $effect(() => {
    const projectId = project?.id;
    const isEnabled = enabled;
    if (!isEnabled || !projectId) return;
    // Only re-fetch when the project id / tab enablement changes —
    // not when the project object identity updates from query refetches.
    untrack(() => {
      // The panel is reused across /projects/[id] navigations: drop the previous
      // project's failure/selection so Diagnose never runs A's failure against B.
      if (seenProjectId !== undefined && seenProjectId !== projectId) {
        lastResult = null;
        selectedExecution = null;
        selected = new Set();
        requirement = '';
      }
      seenProjectId = projectId;
      void loadCatalog();
      void loadHistory();
    });
  });

  // While the selected run is in flight, poll its live output + status every 1s
  // so the user sees progress and the terminal result without a manual refresh.
  $effect(() => {
    const id = runningExecId;
    if (!id) {
      liveOutput = '';
      return;
    }
    let cancelled = false;
    const tick = async () => {
      try {
        const [info, live] = await Promise.all([
          scriptExecutionService.getExecution(id),
          scriptExecutionService.getLiveOutput(id),
        ]);
        if (cancelled) return;
        liveOutput = live.join('');
        if (info && selectedExecution?.id === id) selectedExecution = info;
      } catch {
        // best-effort; the run still resolves via runTarget's await
      }
    };
    void tick();
    const interval = setInterval(() => void tick(), 1000);
    return () => {
      cancelled = true;
      clearInterval(interval);
      void loadHistory();
    };
  });
</script>

<div class="space-y-6">
  <div class="flex flex-wrap items-start justify-between gap-3">
    <div>
      <h2 class="text-xl font-semibold">Actions</h2>
      <p class="text-sm text-muted-foreground">
        Smart defaults from project metadata
        {#if filePath}
          · overrides from <code class="text-xs">{filePath}</code>
        {/if}
        · GitHub & n8n when available
      </p>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button
        variant="outline"
        size="sm"
        class="gap-2"
        onclick={() => {
          void loadCatalog();
          void loadHistory();
        }}
        disabled={loading || running}
      >
        <RefreshCw class="h-4 w-4" />
        Refresh
      </Button>
      <Button
        type="button"
        size="sm"
        class="gap-2"
        onclick={() => void runSelected()}
        disabled={running || selected.size === 0}
      >
        <Play class="h-4 w-4" />
        {running ? 'Running…' : `Run selected (${selected.size})`}
      </Button>
    </div>
  </div>

  {#snippet platformDownActions()}
    {#if aiPlatform.status === 'unreachable'}
      <Button
        type="button"
        size="sm"
        variant="outline"
        disabled={aiPlatform.starting}
        onclick={() => void aiPlatform.start()}
      >
        {aiPlatform.starting ? 'Launching…' : 'Launch Agent Platform'}
      </Button>
    {/if}
    <Button
      type="button"
      size="sm"
      variant="outline"
      onclick={() => void coderSession.handleBackendProviderChange('claude-cli')}
    >
      Use Claude Code (CLI)
    </Button>
  {/snippet}

  <div class="flex flex-col gap-6 xl:flex-row xl:items-start">
    <!-- Actions sidebar -->
    <aside
      class="w-full shrink-0 space-y-4 xl:sticky xl:top-4 xl:max-h-[calc(100vh-8rem)] xl:w-80 xl:overflow-y-auto"
    >
      <div class="flex flex-wrap gap-2">
        {#each FILTERS as filter (filter.id)}
          <Button
            size="sm"
            variant={sourceFilter === filter.id ? 'default' : 'outline'}
            onclick={() => (sourceFilter = filter.id)}
          >
            {filter.label}
          </Button>
        {/each}
      </div>

      {#if warnings.length > 0}
        <div class="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-sm">
          {#each warnings as warning (warning)}
            <p>{warning}</p>
          {/each}
        </div>
      {/if}

      {#if loading}
        <PageLoading message="Loading actions catalog…" />
      {:else if filteredActions.length === 0 && filteredWorkflows.length === 0}
        <PageEmpty
          title="No actions available"
          description="Add framework/package manager metadata, or create a .portal/pipeline.yml in the project root."
          icon={Workflow}
        />
      {:else}
        {#if filteredWorkflows.length > 0}
          <div class="space-y-2">
            <h3 class="text-sm font-medium text-muted-foreground">Workflows</h3>
            <div class="space-y-2">
              {#each filteredWorkflows as workflow (workflow.id)}
                {@const Icon = sourceIcon(workflow.source)}
                <Card>
                  <CardHeader class="pb-2">
                    <div class="flex items-start justify-between gap-2">
                      <CardTitle class="text-sm">{workflow.name}</CardTitle>
                      <Badge variant="secondary">{workflow.source}</Badge>
                    </div>
                    {#if workflow.description}
                      <CardDescription class="text-xs">
                        {workflow.description}
                      </CardDescription>
                    {/if}
                  </CardHeader>
                  <CardContent class="flex items-center justify-between gap-2">
                    <p
                      class="flex min-w-0 items-center gap-1 truncate text-xs text-muted-foreground"
                    >
                      <Icon class="h-3.5 w-3.5 shrink-0" />
                      {workflow.steps.length > 0
                        ? workflow.steps.map((s) => s.action).join(' → ')
                        : workflow.runner}
                    </p>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      class="shrink-0 gap-1"
                      disabled={running}
                      onclick={(e) => void runTarget(workflow.id, e)}
                    >
                      <Play class="h-3.5 w-3.5" />
                      Run
                    </Button>
                  </CardContent>
                </Card>
              {/each}
            </div>
          </div>
        {/if}

        <div class="space-y-2">
          <h3 class="text-sm font-medium text-muted-foreground">Actions</h3>
          <div class="space-y-2">
            {#each filteredActions as action (action.id)}
              {@const Icon = sourceIcon(action.source)}
              <Card>
                <CardHeader class="pb-2">
                  <div class="flex items-start gap-2">
                    <Checkbox
                      checked={selected.has(action.id)}
                      onCheckedChange={() => toggleSelect(action.id)}
                      class="mt-1"
                    />
                    <div class="min-w-0 flex-1">
                      <div class="flex items-start justify-between gap-2">
                        <CardTitle class="text-sm">{action.name}</CardTitle>
                        <Badge variant="outline">{action.source}</Badge>
                      </div>
                      {#if action.description}
                        <CardDescription class="text-xs">
                          {action.description}
                        </CardDescription>
                      {/if}
                    </div>
                  </div>
                </CardHeader>
                <CardContent class="space-y-2">
                  {#if action.command}
                    <code class="block truncate rounded bg-muted px-2 py-1 text-xs">
                      {action.command}
                    </code>
                  {:else}
                    <p class="flex items-center gap-1 text-xs text-muted-foreground">
                      <Icon class="h-3.5 w-3.5" />
                      {action.runner} runner
                    </p>
                  {/if}
                  <Button
                    type="button"
                    size="sm"
                    class="w-full gap-1"
                    disabled={running}
                    onclick={(e) => void runTarget(action.id, e)}
                  >
                    <Play class="h-3.5 w-3.5" />
                    Run
                  </Button>
                </CardContent>
              </Card>
            {/each}
          </div>
        </div>
      {/if}
    </aside>

    <!-- Run details + history -->
    <div class="min-w-0 flex-1 space-y-4">
      <Card>
        <CardHeader class="pb-2">
          <CardTitle class="flex items-center gap-2 text-base">
            <Sparkles class="h-4 w-4" />
            Generate a pipeline
          </CardTitle>
          <CardDescription>
            Describe what you want and an agent writes or extends
            <code class="text-xs">.portal/pipeline.yml</code> for this project's stack. Nothing is committed
            — review the changes in Coder.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-2">
          <Textarea
            bind:value={requirement}
            rows={3}
            placeholder="e.g. Add a lint + typecheck gate and a release workflow that builds then tests."
            disabled={genRunning}
          />
          <div class="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              size="sm"
              class="gap-1"
              disabled={genRunning || running || doctorBusy || !requirement.trim()}
              onclick={() => void generate()}
            >
              <Sparkles class="h-3.5 w-3.5" />
              {genRunning ? 'Generating…' : 'Generate'}
            </Button>
            {#if genRun.status !== 'idle'}
              <span class="text-xs text-muted-foreground">
                {genRun.status === 'generating'
                  ? 'Generating…'
                  : genRun.status === 'validating'
                    ? 'Validating…'
                    : genRun.status === 'previewing'
                      ? 'Dry-checking…'
                      : genRun.status === 'done'
                        ? 'Done'
                        : 'Failed'}
                {#if genRun.message}· {genRun.message}{/if}
              </span>
            {/if}
            {#if genRun.waiting}
              <span class="text-xs text-amber-600 dark:text-amber-400">
                Waiting for your approval in Coder
              </span>
            {/if}
            {#if genRun.unavailable}
              {@render platformDownActions()}
            {/if}
            {#if genRun.threadId}
              <Button
                type="button"
                size="sm"
                variant="link"
                onclick={() => void openThread(genRun.threadId!)}
              >
                {genRunning ? 'Open in Coder' : 'Review changes in Coder'}
              </Button>
            {/if}
          </div>
        </CardContent>
      </Card>

      {#if shownFailure}
        {@const f = shownFailure}
        {@const run = doctorRun}
        <Card class="border-destructive/40">
          <CardHeader class="pb-2">
            <div class="flex items-center justify-between gap-2">
              <CardTitle class="text-base text-destructive">Run failed</CardTitle>
              <Badge variant="outline" class="capitalize">{f.kind}</Badge>
            </div>
            {#if f.hint}
              <CardDescription>{f.hint}</CardDescription>
            {/if}
          </CardHeader>
          <CardContent class="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              size="sm"
              variant="outline"
              class="gap-1"
              disabled={doctorBusy || genRunning || run.status === 'awaiting-verify'}
              onclick={async () => {
                const verify = await diagnose(project, f);
                if (verify) {
                  // Reflect the verify run as the latest result; on success the
                  // "Run failed" card clears because verify.failure is undefined.
                  lastResult = verify;
                  const ls = verify.steps.find((s) => s.executionId);
                  if (ls?.executionId) void openExecution(ls.executionId);
                  if (verify.success)
                    toast.success('Doctor fixed it — review the changes in Coder');
                }
                const after = getDoctorRun(project.id);
                if (after.unavailable && after.message) toast.error(after.message);
                await loadHistory();
              }}
            >
              <Stethoscope class="h-3.5 w-3.5" />
              Diagnose & fix
            </Button>
            {#if run.status !== 'idle'}
              <span class="text-xs text-muted-foreground">
                {run.status === 'diagnosing'
                  ? 'Diagnosing…'
                  : run.status === 'verifying'
                    ? 'Verifying…'
                    : run.status === 'fixed'
                      ? 'Fixed'
                      : run.status === 'awaiting-verify'
                        ? 'Fix proposed'
                        : run.unavailable
                          ? 'Could not start'
                          : 'Gave up'}
                {#if run.message}· {run.message}{/if}
              </span>
            {/if}
            {#if run.waiting}
              <span class="text-xs text-amber-600 dark:text-amber-400">
                Waiting for your approval in Coder
              </span>
            {/if}
            {#if run.unavailable}
              {@render platformDownActions()}
            {/if}
            {#if run.threadId}
              <Button
                type="button"
                size="sm"
                variant="link"
                onclick={() => void openThread(run.threadId!)}
              >
                {run.status === 'diagnosing' ? 'Open in Coder' : 'Review changes in Coder'}
              </Button>
            {/if}
          </CardContent>
        </Card>
      {/if}

      {#if doctorRun.proposal}
        {@const pr = doctorRun.proposal}
        <Card class="border-amber-500/40">
          <CardHeader class="pb-2">
            <CardTitle class="text-base">Review the agent's change</CardTitle>
            <CardDescription>
              Nothing has run. {pr.note ?? 'The agent changed what this action runs.'}
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-2">
            <div>
              <p class="mb-1 text-xs font-medium text-muted-foreground">Was</p>
              <code class="block rounded bg-muted px-2 py-1 text-xs break-all">
                {pr.oldCommand || '—'}
              </code>
            </div>
            <div>
              <p class="mb-1 text-xs font-medium text-muted-foreground">
                Agent wrote (not run yet)
              </p>
              <code class="block rounded bg-muted px-2 py-1 text-xs break-all">
                {pr.newCommand || '—'}
              </code>
            </div>
            {#if pr.scriptChanges.length}
              <div>
                <p class="mb-1 text-xs font-medium text-muted-foreground">
                  package.json scripts changed
                </p>
                {#each pr.scriptChanges as line (line)}
                  <code class="mb-1 block rounded bg-muted px-2 py-1 text-xs break-all">{line}</code
                  >
                {/each}
              </div>
            {/if}
            <div class="flex flex-wrap items-center gap-2">
              <Button type="button" size="sm" class="gap-1" onclick={() => void verifyFix()}>
                <Play class="h-3.5 w-3.5" />
                Verify
              </Button>
              {#if doctorRun.reviewThreadId}
                <Button
                  type="button"
                  size="sm"
                  variant="link"
                  onclick={() => void openThread(doctorRun.reviewThreadId!)}
                >
                  Review changes in Coder
                </Button>
              {/if}
            </div>
          </CardContent>
        </Card>
      {/if}

      {#if doctorRun.reviewThreadId && !shownFailure && !doctorRun.proposal}
        <Card class="border-primary/30">
          <CardContent class="flex flex-wrap items-center gap-2 py-3">
            <span class="text-sm">
              {doctorRun.status === 'fixed' ? 'Fixed' : 'Agent changes'} · pending your review in Coder
            </span>
            <Button
              type="button"
              size="sm"
              variant="link"
              onclick={() => void openThread(doctorRun.reviewThreadId!)}
            >
              Review changes in Coder
            </Button>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              class="ml-auto"
              onclick={() => dismissReviewPointer(project.id)}
            >
              Dismiss
            </Button>
          </CardContent>
        </Card>
      {/if}

      {#if doctorRun.report}
        {@const r = doctorRun.report}
        <Card class="border-amber-500/40">
          <CardHeader class="pb-2">
            <div class="flex flex-wrap items-center justify-between gap-2">
              <CardTitle class="text-base">Portal default needs a fix</CardTitle>
              <Badge variant="outline" class="shrink-0">suggestion</Badge>
            </div>
            <CardDescription>
              A <code class="text-xs">.portal/pipeline.yml</code> override fixed the
              <code class="text-xs">{r.actionId}</code> action. Portal's built-in default was wrong for
              this profile — review it, nothing was changed in Portal.
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-2">
            <p class="text-xs text-muted-foreground">
              Profile: {r.packageManager}{r.frameworks.length
                ? ` · ${r.frameworks.join(', ')}`
                : ''}
            </p>
            <div>
              <p class="mb-1 text-xs font-medium text-muted-foreground">Portal default (failed)</p>
              <code class="block rounded bg-muted px-2 py-1 text-xs break-all">
                {r.defaultCommand || '—'}
              </code>
            </div>
            <div>
              <p class="mb-1 text-xs font-medium text-muted-foreground">Command that passed</p>
              <code class="block rounded bg-muted px-2 py-1 text-xs break-all">
                {r.greenCommand || '—'}
              </code>
            </div>
            <Button
              type="button"
              size="sm"
              variant="outline"
              class="gap-1"
              onclick={() => void copyReport(r)}
            >
              <Copy class="h-3.5 w-3.5" />
              Copy report
            </Button>
          </CardContent>
        </Card>
      {/if}

      {#if selectedExecution}
        {@const StatusIcon = getExecutionStatusIcon(selectedExecution.status)}
        <Card>
          <CardHeader>
            <div class="flex items-start justify-between gap-2">
              <div>
                <CardTitle class="text-base">Run details</CardTitle>
                <CardDescription>
                  {selectedExecution.status}
                  {#if selectedExecution.exitCode != null}
                    · exit {selectedExecution.exitCode}
                  {/if}
                  · {selectedExecution.workingDirectory ?? project.path}
                </CardDescription>
              </div>
              <Badge
                variant={getExecutionStatusBadgeVariant(selectedExecution.status)}
                class="gap-1 {getExecutionStatusBadgeClass(selectedExecution.status)}"
              >
                <StatusIcon
                  class="h-3.5 w-3.5 {getExecutionStatusColor(selectedExecution.status)}"
                />
                {selectedExecution.status}
              </Badge>
            </div>
          </CardHeader>
          <CardContent class="space-y-3">
            <div>
              <p class="mb-1 text-xs font-medium text-muted-foreground">Command</p>
              <code
                class="block rounded bg-muted px-2 py-1.5 text-xs break-all whitespace-pre-wrap"
              >
                {selectedExecution.command}
              </code>
            </div>
            <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
              <span>
                Started {new Date(selectedExecution.startedAt).toLocaleString()}
              </span>
              <span>
                Duration: {formatExecutionDuration(
                  selectedExecution.startedAt,
                  selectedExecution.finishedAt
                )}
              </span>
            </div>
            {#if selectedExecution.error}
              <div>
                <p class="mb-1 text-xs font-medium text-destructive">Error</p>
                <pre
                  class="max-h-40 overflow-auto rounded border border-destructive/30 bg-destructive/5 px-2 py-1.5 text-xs whitespace-pre-wrap text-destructive">{selectedExecution.error}</pre>
              </div>
            {/if}
            {#if selectedExecution.status === 'running' && liveOutput.trim()}
              <div>
                <p class="mb-1 text-xs font-medium text-muted-foreground">Output (live)</p>
                <pre
                  class="max-h-72 overflow-auto rounded bg-muted px-2 py-1.5 text-xs whitespace-pre-wrap">{liveOutput}</pre>
              </div>
            {:else if selectedExecution.output?.trim()}
              <div>
                <p class="mb-1 text-xs font-medium text-muted-foreground">Output</p>
                <pre
                  class="max-h-72 overflow-auto rounded bg-muted px-2 py-1.5 text-xs whitespace-pre-wrap">{selectedExecution.output}</pre>
              </div>
            {:else if selectedExecution.status === 'running'}
              <p class="text-xs text-muted-foreground">Running…</p>
            {:else if !selectedExecution.error}
              <p class="text-xs text-muted-foreground">No output captured.</p>
            {/if}
          </CardContent>
        </Card>
      {:else if lastResult}
        <Card>
          <CardHeader>
            <CardTitle class="text-base">Last run</CardTitle>
            <CardDescription>
              {lastResult.success ? 'Succeeded' : 'Failed'} · {lastResult.cwd}
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-3">
            {#each lastResult.steps as step (step.actionId + step.name)}
              <div class="space-y-1 rounded border p-2">
                <div class="flex items-center justify-between gap-2 text-sm">
                  <span class="font-medium">{step.name}</span>
                  <Badge
                    variant={step.status === 'success'
                      ? 'default'
                      : step.status === 'skipped'
                        ? 'secondary'
                        : 'destructive'}
                  >
                    {step.status}
                  </Badge>
                </div>
                {#if step.command}
                  <code class="block truncate text-xs text-muted-foreground">
                    {step.command}
                  </code>
                {/if}
                {#if step.exitCode != null}
                  <p class="text-xs text-muted-foreground">
                    Exit code {step.exitCode}
                  </p>
                {/if}
                {#if step.error}
                  <pre
                    class="max-h-32 overflow-auto text-xs whitespace-pre-wrap text-destructive">{step.error}</pre>
                {/if}
                {#if step.output?.trim()}
                  <pre
                    class="max-h-40 overflow-auto rounded bg-muted px-2 py-1 text-xs whitespace-pre-wrap">{step.output}</pre>
                {/if}
                {#if step.executionId}
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    class="mt-1"
                    onclick={() => void openExecution(step.executionId!)}
                  >
                    View full details
                  </Button>
                {/if}
              </div>
            {/each}
          </CardContent>
        </Card>
      {/if}

      <Card>
        <CardHeader>
          <div class="flex items-center justify-between gap-2">
            <div>
              <CardTitle class="flex items-center gap-2 text-base">
                <History class="h-5 w-5" />
                Local run history
              </CardTitle>
              <CardDescription>Script executions for this project (Actions runs)</CardDescription>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onclick={() => void loadHistory()}
              disabled={historyLoading}
            >
              <RefreshCw class="h-4 w-4 {historyLoading ? 'animate-spin' : ''}" />
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {#if historyLoading && runHistory.length === 0}
            <PageLoading message="Loading history…" />
          {:else if runHistory.length === 0}
            <PageEmpty
              title="No action runs yet"
              description="Run an action to see command output and exit details here."
              icon={History}
            />
          {:else}
            <div class="space-y-2">
              {#each runHistory as execution (execution.id)}
                {@const StatusIcon = getExecutionStatusIcon(execution.status)}
                <Button
                  type="button"
                  variant="outline"
                  class="h-auto w-full justify-start rounded-lg border p-3 text-left transition-colors hover:bg-muted/50 {selectedExecution?.id ===
                  execution.id
                    ? 'border-primary bg-primary/5'
                    : ''}"
                  onclick={() => void openExecution(execution.id)}
                >
                  <div class="flex w-full flex-wrap items-center justify-between gap-2">
                    <div class="flex min-w-0 items-center gap-2">
                      <StatusIcon
                        class="h-4 w-4 shrink-0 {getExecutionStatusColor(
                          execution.status
                        )}{execution.status === 'running' ? ' animate-spin' : ''}"
                      />
                      <Badge
                        variant={getExecutionStatusBadgeVariant(execution.status)}
                        class={getExecutionStatusBadgeClass(execution.status)}
                      >
                        {execution.status}
                      </Badge>
                      <span class="truncate text-sm font-medium">
                        {execution.command}
                      </span>
                    </div>
                    <span class="text-xs text-muted-foreground">
                      {new Date(execution.startedAt).toLocaleString()}
                    </span>
                  </div>
                  <div
                    class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground"
                  >
                    <span>
                      Duration: {formatExecutionDuration(execution.startedAt, execution.finishedAt)}
                    </span>
                    {#if execution.exitCode != null}
                      <span>exit {execution.exitCode}</span>
                    {/if}
                    {#if execution.error}
                      <span class="truncate text-destructive">{execution.error}</span>
                    {/if}
                  </div>
                </Button>
              {/each}
            </div>
          {/if}
        </CardContent>
      </Card>
    </div>
  </div>

  <ReleaseCard {project} />

  {#if sourceFilter === 'all' || sourceFilter === 'github'}
    <div class="space-y-3 border-t pt-6">
      <h3 class="text-sm font-medium text-muted-foreground">GitHub Actions runs</h3>
      <GitHubProjectActionsPanel {project} {enabled} />
    </div>
  {/if}
</div>
