<script lang="ts">
  import { goto } from "$app/navigation";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { PageEmpty, PageError, PageLoading } from "$lib/components/shell";
  import {
    createGitHubWorkflowRunsQuery,
    type GitHubWorkflowRun,
  } from "$lib/domains/github";
  import {
    formatWorkflowDuration,
    getWorkflowDisplayStatus,
    getWorkflowStatusBadgeVariant,
    getWorkflowStatusColor,
    getWorkflowStatusIcon,
    isWorkflowRunActive,
  } from "$lib/domains/github/utils/workflowDisplay";
  import { ExternalLink, RefreshCw, Workflow } from "@lucide/svelte";

  interface Props {
    owner: string;
    repo: string;
    enabled?: boolean;
  }

  let { owner, repo, enabled = true }: Props = $props();

  const runsQuery = createGitHubWorkflowRunsQuery(
    () => ({
      owner,
      repo,
      page: 1,
      perPage: 20,
    }),
    () => enabled,
  );

  const runs = $derived(runsQuery.data ?? []);
  const hasActiveRuns = $derived(runs.some(isWorkflowRunActive));

  function openRun(run: GitHubWorkflowRun) {
    goto(`/github/repos/${owner}/${repo}/actions/${run.id}`);
  }
</script>

{#if runsQuery.isPending}
  <PageLoading message="Loading workflow runs..." />
{:else if runsQuery.isError}
  <PageError
    title="Failed to load workflow runs"
    message={runsQuery.error instanceof Error
      ? runsQuery.error.message
      : "Unable to load GitHub Actions runs"}
    onRetry={() => runsQuery.refetch()}
  />
{:else if runs.length === 0}
  <PageEmpty
    title="No workflow runs yet"
    description="Push a commit or open a pull request to trigger GitHub Actions."
    icon={Workflow}
  />
{:else}
  <div class="space-y-3">
    <div class="flex items-center justify-between gap-2">
      <p class="text-sm text-muted-foreground">
        {#if hasActiveRuns}
          Live updates every 3 seconds while runs are active.
        {:else}
          Recent GitHub Actions workflow runs.
        {/if}
      </p>
      <Button
        variant="ghost"
        size="sm"
        onclick={() => runsQuery.refetch()}
        disabled={runsQuery.isFetching}
      >
        <RefreshCw
          class="h-4 w-4 {runsQuery.isFetching ? 'animate-spin' : ''}"
        />
      </Button>
    </div>

    <div class="space-y-2">
      {#each runs as run (run.id)}
        {@const displayStatus = getWorkflowDisplayStatus(run.status, run.conclusion)}
        {@const StatusIcon = getWorkflowStatusIcon(displayStatus)}
        <Button
          type="button"
          variant="outline"
          class="h-auto w-full justify-start rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
          onclick={() => openRun(run)}
        >
          <div class="flex flex-wrap items-center justify-between gap-2">
            <div class="flex min-w-0 items-center gap-2">
              <StatusIcon
                class="h-4 w-4 shrink-0 {getWorkflowStatusColor(displayStatus)}{displayStatus === 'running' ? ' animate-spin' : ''}"
              />
              <Badge variant={getWorkflowStatusBadgeVariant(displayStatus)}>
                {displayStatus}
              </Badge>
              <span class="truncate text-sm font-medium">
                {run.displayTitle || run.name}
              </span>
            </div>
            <span class="text-xs text-muted-foreground">
              #{run.runNumber}
            </span>
          </div>

          <div
            class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground"
          >
            <span>{run.event}</span>
            {#if run.headBranch}
              <span>{run.headBranch}</span>
            {/if}
            <span>
              {formatWorkflowDuration(run.runStartedAt, run.updatedAt)}
            </span>
            {#if run.createdAt}
              <span>{new Date(run.createdAt).toLocaleString()}</span>
            {/if}
          </div>
        </Button>
      {/each}
    </div>

    <div class="flex justify-end">
      <a
        href="https://github.com/{owner}/{repo}/actions"
        target="_blank"
        rel="noreferrer"
        class="inline-flex items-center text-xs text-primary"
      >
        View all on GitHub
        <ExternalLink class="ml-1 h-3 w-3" />
      </a>
    </div>
  </div>
{/if}
