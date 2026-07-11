<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { PageError, PageLoading } from "$lib/components/shell";
  import {
    createGitHubWorkflowJobLogsQuery,
    createGitHubWorkflowRunQuery,
    type GitHubWorkflowJob,
  } from "$lib/domains/github";
  import {
    formatWorkflowDuration,
    getWorkflowDisplayStatus,
    getWorkflowStatusBadgeVariant,
    getWorkflowStatusColor,
    getWorkflowStatusIcon,
    isWorkflowJobActive,
    isWorkflowJobLogsFetchable,
    isWorkflowLogsUnavailableMessage,
    isWorkflowRunActive,
  } from "$lib/domains/github/utils/workflowDisplay";

  const WORKFLOW_POLL_INTERVAL_MS = 3_000;
  import { ChevronDown, ChevronRight, ExternalLink, RefreshCw } from "@lucide/svelte";

  interface Props {
    owner: string;
    repo: string;
    runId: number;
    enabled?: boolean;
  }

  let { owner, repo, runId, enabled = true }: Props = $props();

  let expandedJobId = $state<number | null>(null);

  const runQuery = createGitHubWorkflowRunQuery(
    () => owner,
    () => repo,
    () => runId,
    () => enabled,
  );

  const detail = $derived(runQuery.data);
  const run = $derived(detail?.run);
  const jobs = $derived(detail?.jobs ?? []);
  const isActive = $derived(
    (run && isWorkflowRunActive(run)) ||
      jobs.some((job) => isWorkflowJobActive(job.status)),
  );

  const expandedJob = $derived(
    expandedJobId != null ? jobs.find((job) => job.id === expandedJobId) : null,
  );

  const logsQuery = createGitHubWorkflowJobLogsQuery(
    () => owner,
    () => repo,
    () => expandedJobId ?? undefined,
    () =>
      enabled &&
      expandedJobId != null &&
      expandedJob != null &&
      isWorkflowJobLogsFetchable(expandedJob.status),
  );

  $effect(() => {
    if (!expandedJob || !isWorkflowJobActive(expandedJob.status)) return;
    if (!isWorkflowJobLogsFetchable(expandedJob.status)) return;
    const interval = setInterval(() => {
      void logsQuery.refetch();
    }, WORKFLOW_POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  });

  function toggleJob(job: GitHubWorkflowJob) {
    expandedJobId = expandedJobId === job.id ? null : job.id;
  }
</script>

{#if runQuery.isPending}
  <PageLoading message="Loading workflow run..." />
{:else if runQuery.isError}
  <PageError
    title="Failed to load workflow run"
    message={runQuery.error instanceof Error
      ? runQuery.error.message
      : "Unable to load workflow run details"}
    onRetry={() => runQuery.refetch()}
  />
{:else if run}
  {@const displayStatus = getWorkflowDisplayStatus(run.status, run.conclusion)}
  {@const StatusIcon = getWorkflowStatusIcon(displayStatus)}

  <Card>
    <CardHeader>
      <div class="flex items-start justify-between gap-3">
        <div class="space-y-2">
          <CardTitle class="flex flex-wrap items-center gap-2">
            <StatusIcon
              class="h-5 w-5 {getWorkflowStatusColor(displayStatus)}{displayStatus === 'running' ? ' animate-spin' : ''}"
            />
            <span>{run.displayTitle || run.name}</span>
            <Badge variant={getWorkflowStatusBadgeVariant(displayStatus)}>
              {displayStatus}
            </Badge>
          </CardTitle>
          <div class="flex flex-wrap gap-2 text-sm text-muted-foreground">
            <span>Run #{run.runNumber}</span>
            <span>{run.event}</span>
            {#if run.headBranch}
              <span>{run.headBranch}</span>
            {/if}
            <span>{run.headSha.slice(0, 7)}</span>
            <span>
              {formatWorkflowDuration(run.runStartedAt, run.updatedAt)}
            </span>
          </div>
        </div>
        <div class="flex items-center gap-2">
          {#if isActive}
            <Badge variant="secondary">Live</Badge>
          {/if}
          <Button
            variant="ghost"
            size="sm"
            onclick={() => runQuery.refetch()}
            disabled={runQuery.isFetching}
          >
            <RefreshCw
              class="h-4 w-4 {runQuery.isFetching ? 'animate-spin' : ''}"
            />
          </Button>
          <a
            href={run.htmlUrl}
            target="_blank"
            rel="noreferrer"
            class="inline-flex items-center text-sm text-primary"
          >
            Open on GitHub
            <ExternalLink class="ml-1 h-4 w-4" />
          </a>
        </div>
      </div>
    </CardHeader>
    <CardContent class="space-y-3">
      {#if jobs.length === 0}
        <p class="text-sm text-muted-foreground">No jobs found for this run.</p>
      {:else}
        {#each jobs as job (job.id)}
          {@const jobStatus = getWorkflowDisplayStatus(job.status, job.conclusion)}
          {@const JobStatusIcon = getWorkflowStatusIcon(jobStatus)}
          <div class="rounded-lg border">
            <Button
              type="button"
              variant="ghost"
              class="h-auto w-full justify-start rounded-lg p-3 text-left"
              onclick={() => toggleJob(job)}
            >
              <div class="flex w-full items-center justify-between gap-2">
                <div class="flex min-w-0 items-center gap-2">
                  {#if expandedJobId === job.id}
                    <ChevronDown class="h-4 w-4 shrink-0" />
                  {:else}
                    <ChevronRight class="h-4 w-4 shrink-0" />
                  {/if}
                  <JobStatusIcon
                    class="h-4 w-4 shrink-0 {getWorkflowStatusColor(jobStatus)}{jobStatus === 'running' ? ' animate-spin' : ''}"
                  />
                  <span class="truncate font-medium">{job.name}</span>
                  <Badge variant={getWorkflowStatusBadgeVariant(jobStatus)}>
                    {jobStatus}
                  </Badge>
                </div>
                <span class="text-xs text-muted-foreground">
                  {formatWorkflowDuration(job.startedAt, job.completedAt)}
                </span>
              </div>
            </Button>

            {#if expandedJobId === job.id}
              <div class="divider-edge-t divider-edge-full space-y-3 px-3 pb-3 pt-2">
                {#if job.steps.length > 0}
                  <div class="space-y-1">
                    {#each job.steps as step (step.number)}
                      {@const stepStatus = getWorkflowDisplayStatus(
                        step.status,
                        step.conclusion,
                      )}
                      {@const StepIcon = getWorkflowStatusIcon(stepStatus)}
                      <div
                        class="flex items-center justify-between gap-2 rounded px-2 py-1 text-sm"
                      >
                        <div class="flex min-w-0 items-center gap-2">
                          <StepIcon
                            class="h-3.5 w-3.5 shrink-0 {getWorkflowStatusColor(stepStatus)}{stepStatus === 'running' ? ' animate-spin' : ''}"
                          />
                          <span class="truncate">{step.name}</span>
                        </div>
                        <span class="text-xs text-muted-foreground">
                          {formatWorkflowDuration(step.startedAt, step.completedAt)}
                        </span>
                      </div>
                    {/each}
                  </div>
                {/if}

                <div class="space-y-2">
                  <div class="flex items-center justify-between gap-2">
                    <span class="text-sm font-medium">Job logs</span>
                    <a
                      href={job.htmlUrl}
                      target="_blank"
                      rel="noreferrer"
                      class="inline-flex items-center text-xs text-primary"
                    >
                      View on GitHub
                      <ExternalLink class="ml-1 h-3 w-3" />
                    </a>
                  </div>
                  {#if expandedJob && !isWorkflowJobLogsFetchable(expandedJob.status)}
                    <p class="text-sm text-muted-foreground">
                      Logs will appear once this job starts running.
                    </p>
                  {:else if logsQuery.isPending}
                    <PageLoading message="Loading job logs..." />
                  {:else if logsQuery.isError}
                    <p class="text-sm text-destructive">
                      {logsQuery.error instanceof Error
                        ? logsQuery.error.message
                        : "Failed to load job logs"}
                    </p>
                  {:else if logsQuery.data && isWorkflowLogsUnavailableMessage(logsQuery.data)}
                    <p class="text-sm text-muted-foreground">{logsQuery.data}</p>
                  {:else if logsQuery.data}
                    <pre
                      class="max-h-96 overflow-auto rounded-md bg-muted p-3 text-xs leading-relaxed"
                    >{logsQuery.data}</pre>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </CardContent>
  </Card>
{/if}
