<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { PageHeader, PageLoading } from "$lib/components/shell";
  import {
    createGitHubStatusQuery,
    GitHubConnectPrompt,
    GitHubWorkflowRunMonitor,
  } from "$lib/domains/github";
  import { ArrowLeft } from "@lucide/svelte";

  const owner = $derived($page.params.owner);
  const repo = $derived($page.params.repo);
  const runId = $derived(Number($page.params.runId));

  const statusQuery = createGitHubStatusQuery();
  const isConnected = $derived(statusQuery.data?.connected ?? false);
</script>

<svelte:head>
  <title>Workflow run #{runId} - {owner}/{repo} - GitHub - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader
    title="Workflow Run"
    description="{owner}/{repo} · run #{runId}"
  >
    {#snippet actions()}
      <Button
        variant="outline"
        onclick={() => goto(`/github/repos/${owner}/${repo}`)}
      >
        <ArrowLeft class="mr-2 h-4 w-4" />
        Back to repository
      </Button>
    {/snippet}
  </PageHeader>

  {#if statusQuery.isPending}
    <PageLoading message="Checking GitHub connection..." />
  {:else if !statusQuery.data?.connected}
    <GitHubConnectPrompt
      status={statusQuery.data}
      onConnected={() => statusQuery.refetch()}
    />
  {:else if !Number.isFinite(runId) || runId <= 0}
    <p class="text-sm text-destructive">Invalid workflow run ID.</p>
  {:else}
    <GitHubWorkflowRunMonitor {owner} {repo} {runId} enabled={isConnected} />
  {/if}
</div>
