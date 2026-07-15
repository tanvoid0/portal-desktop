<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { PageEmpty, PageLoading } from "$lib/components/shell";
  import { toast } from "$lib/utils/toast";
  import {
    Play,
    RefreshCw,
    Workflow,
    Terminal,
    GitBranch,
    Layers,
    History,
  } from "@lucide/svelte";
  import type { Project } from "$lib/domains/projects/types";
  import {
    actions,
    type UnifiedAction,
    type UnifiedWorkflow,
    type ActionRunResult,
    type ActionSource,
  } from "$lib/domains/actions";
  import { GitHubProjectActionsPanel } from "$lib/domains/github";
  import {
    scriptExecutionService,
    type ScriptExecutionInfo,
  } from "$lib/domains/scripts/services/scriptExecutionService";
  import {
    formatExecutionDuration,
    getExecutionStatusBadgeVariant,
    getExecutionStatusColor,
    getExecutionStatusIcon,
  } from "$lib/domains/projects/pipelines/utils/executionDisplay";
  import { untrack } from "svelte";

  interface Props {
    project: Project;
    enabled?: boolean;
  }

  let { project, enabled = true }: Props = $props();

  type SourceFilter = "all" | ActionSource;

  let loading = $state(true);
  let running = $state(false);
  let actionsList = $state<UnifiedAction[]>([]);
  let workflows = $state<UnifiedWorkflow[]>([]);
  let warnings = $state<string[]>([]);
  let filePath = $state<string | undefined>();
  let sourceFilter = $state<SourceFilter>("all");
  let selected = $state<Set<string>>(new Set());
  let lastResult = $state<ActionRunResult | null>(null);
  let runHistory = $state<ScriptExecutionInfo[]>([]);
  let historyLoading = $state(false);
  let selectedExecution = $state<ScriptExecutionInfo | null>(null);

  function normalizePath(path: string): string {
    return path.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
  }

  function isProjectExecution(exec: ScriptExecutionInfo): boolean {
    if (!exec.workingDirectory) return false;
    return normalizePath(exec.workingDirectory) === normalizePath(project.path);
  }

  const FILTERS: { id: SourceFilter; label: string }[] = [
    { id: "all", label: "All" },
    { id: "local", label: "Local" },
    { id: "file", label: "File" },
    { id: "github", label: "GitHub" },
    { id: "n8n", label: "n8n" },
  ];

  const filteredActions = $derived(
    sourceFilter === "all"
      ? actionsList
      : actionsList.filter((a) => a.source === sourceFilter),
  );

  const filteredWorkflows = $derived(
    sourceFilter === "all"
      ? workflows
      : workflows.filter((w) => w.source === sourceFilter),
  );

  function sourceIcon(source: ActionSource) {
    if (source === "github") return GitBranch;
    if (source === "n8n") return Layers;
    if (source === "file") return Workflow;
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
      toast.error(
        err instanceof Error ? err.message : "Failed to load actions catalog",
      );
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
      toast.info("Select one or more actions to run");
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
    try {
      const result = await actions.forProject(project).run(target);
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
          typeof target === "string"
            ? `Ran ${target}`
            : `Ran ${target.length} actions`,
          result.remoteRunId ? `Remote: ${result.remoteRunId}` : undefined,
        );
      } else {
        const failed =
          result.steps.find((s) => s.error) ??
          result.steps.find((s) => s.status === "failed");
        const err =
          failed?.error?.trim() ||
          (failed?.exitCode != null
            ? `Exit code ${failed.exitCode}`
            : null) ||
          "Action run finished with errors";
        toast.error(err);
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
      void loadCatalog();
      void loadHistory();
    });
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
        {running ? "Running…" : `Run selected (${selected.size})`}
      </Button>
    </div>
  </div>

  <div class="flex flex-col gap-6 lg:flex-row lg:items-start">
    <!-- Actions sidebar -->
    <aside
      class="w-full shrink-0 space-y-4 lg:sticky lg:top-4 lg:w-80 lg:max-h-[calc(100vh-8rem)] lg:overflow-y-auto xl:w-96"
    >
      <div class="flex flex-wrap gap-2">
        {#each FILTERS as filter (filter.id)}
          <Button
            size="sm"
            variant={sourceFilter === filter.id ? "default" : "outline"}
            onclick={() => (sourceFilter = filter.id)}
          >
            {filter.label}
          </Button>
        {/each}
      </div>

      {#if warnings.length > 0}
        <div
          class="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-sm"
        >
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
                        ? workflow.steps.map((s) => s.action).join(" → ")
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
                    <code
                      class="block truncate rounded bg-muted px-2 py-1 text-xs"
                    >
                      {action.command}
                    </code>
                  {:else}
                    <p
                      class="flex items-center gap-1 text-xs text-muted-foreground"
                    >
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
                class="gap-1"
              >
                <StatusIcon
                  class="h-3.5 w-3.5 {getExecutionStatusColor(
                    selectedExecution.status,
                  )}"
                />
                {selectedExecution.status}
              </Badge>
            </div>
          </CardHeader>
          <CardContent class="space-y-3">
            <div>
              <p class="mb-1 text-xs font-medium text-muted-foreground">
                Command
              </p>
              <code
                class="block whitespace-pre-wrap break-all rounded bg-muted px-2 py-1.5 text-xs"
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
                  selectedExecution.finishedAt,
                )}
              </span>
            </div>
            {#if selectedExecution.error}
              <div>
                <p class="mb-1 text-xs font-medium text-destructive">Error</p>
                <pre
                  class="max-h-40 overflow-auto whitespace-pre-wrap rounded border border-destructive/30 bg-destructive/5 px-2 py-1.5 text-xs text-destructive"
                >{selectedExecution.error}</pre>
              </div>
            {/if}
            {#if selectedExecution.output?.trim()}
              <div>
                <p class="mb-1 text-xs font-medium text-muted-foreground">
                  Output
                </p>
                <pre
                  class="max-h-72 overflow-auto whitespace-pre-wrap rounded bg-muted px-2 py-1.5 text-xs"
                >{selectedExecution.output}</pre>
              </div>
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
              {lastResult.success ? "Succeeded" : "Failed"} · {lastResult.cwd}
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-3">
            {#each lastResult.steps as step (step.actionId + step.name)}
              <div class="space-y-1 rounded border p-2">
                <div class="flex items-center justify-between gap-2 text-sm">
                  <span class="font-medium">{step.name}</span>
                  <Badge
                    variant={step.status === "success"
                      ? "default"
                      : step.status === "skipped"
                        ? "secondary"
                        : "destructive"}
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
                    class="max-h-32 overflow-auto whitespace-pre-wrap text-xs text-destructive"
                  >{step.error}</pre>
                {/if}
                {#if step.output?.trim()}
                  <pre
                    class="max-h-40 overflow-auto whitespace-pre-wrap rounded bg-muted px-2 py-1 text-xs"
                  >{step.output}</pre>
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
              <CardDescription>
                Script executions for this project (Actions runs)
              </CardDescription>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onclick={() => void loadHistory()}
              disabled={historyLoading}
            >
              <RefreshCw
                class="h-4 w-4 {historyLoading ? 'animate-spin' : ''}"
              />
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
                          execution.status,
                        )}{execution.status === 'running'
                          ? ' animate-spin'
                          : ''}"
                      />
                      <Badge
                        variant={getExecutionStatusBadgeVariant(
                          execution.status,
                        )}
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
                      Duration: {formatExecutionDuration(
                        execution.startedAt,
                        execution.finishedAt,
                      )}
                    </span>
                    {#if execution.exitCode != null}
                      <span>exit {execution.exitCode}</span>
                    {/if}
                    {#if execution.error}
                      <span class="truncate text-destructive"
                        >{execution.error}</span
                      >
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

  {#if sourceFilter === "all" || sourceFilter === "github"}
    <div class="space-y-3 border-t pt-6">
      <h3 class="text-sm font-medium text-muted-foreground">
        GitHub Actions runs
      </h3>
      <GitHubProjectActionsPanel {project} {enabled} />
    </div>
  {/if}
</div>
