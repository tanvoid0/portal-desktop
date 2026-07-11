<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import { PageEmpty, PageLoading } from "$lib/components/shell";
  import type { Project } from "$lib/domains/projects/types";
  import { getProjectGitBranch } from "$lib/domains/projects/utils/display";
  import {
    createGitHubProjectLinkQuery,
    createGitHubStatusQuery,
    GitHubConnectPrompt,
    GitHubWorkflowRunsPanel,
  } from "$lib/domains/github";
  import { parseGitHubRemote } from "$lib/domains/github/utils/parseGitHubRemote";
  import { ExternalLink, Workflow } from "@lucide/svelte";

  interface Props {
    project: Project;
    enabled?: boolean;
  }

  let { project, enabled = true }: Props = $props();

  let filterByBranch = $state(false);

  const statusQuery = createGitHubStatusQuery();
  const linkQuery = createGitHubProjectLinkQuery(
    () => project.id,
    () => enabled,
  );

  const isConnected = $derived(statusQuery.data?.connected ?? false);
  const link = $derived(linkQuery.data);
  const parsedRemote = $derived(parseGitHubRemote(project.git_repository));
  const repoOwner = $derived(link?.repoOwner ?? parsedRemote?.owner ?? null);
  const repoName = $derived(link?.repoName ?? parsedRemote?.repo ?? null);
  const branch = $derived(
    link?.defaultBranch ?? getProjectGitBranch(project) ?? undefined,
  );
  const hasRepo = $derived(Boolean(repoOwner && repoName));
  const repoFullName = $derived(
    repoOwner && repoName ? `${repoOwner}/${repoName}` : null,
  );
  const activeBranchFilter = $derived(filterByBranch && branch ? branch : undefined);
</script>

{#if statusQuery.isPending || linkQuery.isPending}
  <PageLoading message="Loading GitHub CI/CD..." />
{:else if !statusQuery.data?.connected}
  <GitHubConnectPrompt
    status={statusQuery.data}
    onConnected={() => statusQuery.refetch()}
  />
{:else if !hasRepo}
  <PageEmpty
    title="No GitHub repository linked"
    description="Link this project to a GitHub repository to track Actions workflows here."
    icon={Workflow}
    actionLabel={project.git_repository && parsedRemote
      ? `Open ${parsedRemote.owner}/${parsedRemote.repo}`
      : "Browse GitHub repositories"}
    onAction={() =>
      project.git_repository && parsedRemote
        ? goto(`/github/repos/${parsedRemote.owner}/${parsedRemote.repo}`)
        : goto("/github/repos")}
  />
  {#if project.git_repository && parsedRemote}
    <div class="flex justify-center">
      <Button variant="outline" onclick={() => goto("/github/repos")}>
        Browse GitHub repositories
      </Button>
    </div>
  {/if}
{:else}
  <div class="space-y-4">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div>
        <h2 class="text-xl font-semibold">GitHub Actions</h2>
        <p class="text-sm text-muted-foreground">
          CI/CD runs for {repoFullName}
          {#if activeBranchFilter}
            on branch {activeBranchFilter}
          {/if}
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-3">
        {#if branch}
          <div class="flex items-center gap-2">
            <Switch
              id="filter-by-branch"
              bind:checked={filterByBranch}
            />
            <Label for="filter-by-branch" class="text-sm font-normal">
              Current branch only ({branch})
            </Label>
          </div>
        {/if}
        <Button
          variant="outline"
          size="sm"
          onclick={() => goto(`/github/repos/${repoOwner}/${repoName}`)}
        >
          <ExternalLink class="mr-2 h-4 w-4" />
          Repository details
        </Button>
      </div>
    </div>

    <GitHubWorkflowRunsPanel
      owner={repoOwner!}
      repo={repoName!}
      branch={activeBranchFilter}
      enabled={enabled && isConnected}
    />
  </div>
{/if}
