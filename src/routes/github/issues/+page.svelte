<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import { PageHeader, PageLoading, PageError, PageEmpty } from "$lib/components/shell";
  import {
    createGitHubIssuesQuery,
    createGitHubStatusQuery,
    githubService,
    GitHubConnectPrompt,
  } from "$lib/domains/github";
  import { toast } from "$lib/utils/toast";
  import { Bug, ExternalLink, Plus } from "@lucide/svelte";

  let filter: "assigned" | "created" = $state("assigned");
  let stateValue: "open" | "closed" | "all" = $state("open");
  let createDialogOpen = $state(false);
  let editDialogOpen = $state(false);
  let selectedIssue: any = $state(null);
  let createOwner = $state("");
  let createRepo = $state("");
  let createTitle = $state("");
  let createBody = $state("");
  let editTitle = $state("");
  let editBody = $state("");
  let submitting = $state(false);

  const statusQuery = createGitHubStatusQuery();
  const isConnected = $derived(statusQuery.data?.connected ?? false);
  const issuesQuery = createGitHubIssuesQuery(
    () => ({
      filter,
      state: stateValue,
      page: 1,
      perPage: 50,
    }),
    () => isConnected,
  );

  function openEditDialog(issue: any) {
    selectedIssue = issue;
    editTitle = issue.title;
    editBody = issue.body || "";
    editDialogOpen = true;
  }

  async function handleCreateIssue() {
    try {
      submitting = true;
      await githubService.createIssue({
        owner: createOwner.trim(),
        repo: createRepo.trim(),
        title: createTitle.trim(),
        body: createBody.trim() || undefined,
      });
      toast.success("Issue created");
      createDialogOpen = false;
      createTitle = "";
      createBody = "";
      await issuesQuery.refetch();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to create issue");
    } finally {
      submitting = false;
    }
  }

  async function handleUpdateIssue(stateValue?: "open" | "closed") {
    if (!selectedIssue?.repoFullName) return;
    const [owner, repo] = selectedIssue.repoFullName.split("/");
    try {
      submitting = true;
      await githubService.updateIssue({
        owner,
        repo,
        number: selectedIssue.number,
        title: editTitle.trim(),
        body: editBody.trim() || undefined,
        state: stateValue,
      });
      toast.success(stateValue ? `Issue ${stateValue}` : "Issue updated");
      editDialogOpen = false;
      await issuesQuery.refetch();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to update issue");
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head>
  <title>GitHub Issues - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader
    title="Issues"
    description="View and manage GitHub issues across your repositories"
  >
    {#snippet actions()}
      <Button onclick={() => (createDialogOpen = true)}>
        <Plus class="mr-2 h-4 w-4" />
        New Issue
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
  {:else}
    <div class="flex flex-wrap items-center gap-3">
      <Button
        variant={filter === "assigned" ? "default" : "outline"}
        onclick={() => (filter = "assigned")}
      >
        Assigned
      </Button>
      <Button
        variant={filter === "created" ? "default" : "outline"}
        onclick={() => (filter = "created")}
      >
        Created
      </Button>
      <Button
        variant={stateValue === "open" ? "default" : "outline"}
        onclick={() => (stateValue = "open")}
      >
        Open
      </Button>
      <Button
        variant={stateValue === "closed" ? "default" : "outline"}
        onclick={() => (stateValue = "closed")}
      >
        Closed
      </Button>
      <Button
        variant={stateValue === "all" ? "default" : "outline"}
        onclick={() => (stateValue = "all")}
      >
        All
      </Button>
    </div>

    {#if issuesQuery.isPending}
      <PageLoading message="Loading issues..." />
    {:else if issuesQuery.isError}
      <PageError
        title="Failed to load issues"
        message={issuesQuery.error instanceof Error
          ? issuesQuery.error.message
          : "Unable to load issues"}
        onRetry={() => issuesQuery.refetch()}
      />
    {:else if (issuesQuery.data ?? []).length === 0}
      <PageEmpty
        title="No issues found"
        description="Try a different filter or create a new issue."
        icon={Bug}
      />
    {:else}
      <div class="space-y-4">
        {#each issuesQuery.data ?? [] as issue}
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center justify-between gap-3">
                <div>
                  <div class="font-medium">#{issue.number} {issue.title}</div>
                  <div class="text-sm text-muted-foreground">
                    {issue.repoFullName || "Unknown repo"}
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <a
                    href={issue.htmlUrl}
                    target="_blank"
                    rel="noreferrer"
                    class="inline-flex items-center text-sm text-primary"
                  >
                    Open
                    <ExternalLink class="ml-1 h-4 w-4" />
                  </a>
                  <Button variant="outline" size="sm" onclick={() => openEditDialog(issue)}>
                    Edit
                  </Button>
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent class="space-y-2 text-sm">
              <div class="text-muted-foreground">
                {issue.body || "No description"}
              </div>
              <div class="flex flex-wrap gap-3 text-xs text-muted-foreground">
                <span>State: {issue.state}</span>
                {#if issue.authorLogin}
                  <span>Author: {issue.authorLogin}</span>
                {/if}
                {#if issue.labels.length > 0}
                  <span>Labels: {issue.labels.join(", ")}</span>
                {/if}
              </div>
            </CardContent>
          </Card>
        {/each}
      </div>
    {/if}
  {/if}

  <Dialog.Root open={createDialogOpen} onOpenChange={(open) => (createDialogOpen = open)}>
    <Dialog.Content class="max-w-2xl">
      <Dialog.Header>
        <Dialog.Title>Create Issue</Dialog.Title>
      </Dialog.Header>
      <div class="space-y-4">
        <div class="grid gap-3 md:grid-cols-2">
          <Input bind:value={createOwner} placeholder="owner" />
          <Input bind:value={createRepo} placeholder="repository" />
        </div>
        <Input bind:value={createTitle} placeholder="Issue title" />
        <Textarea bind:value={createBody} placeholder="Issue description" />
        <Button
          onclick={handleCreateIssue}
          disabled={
            submitting ||
            !createOwner.trim() ||
            !createRepo.trim() ||
            !createTitle.trim()
          }
        >
          Create Issue
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Root>

  <Dialog.Root open={editDialogOpen} onOpenChange={(open) => (editDialogOpen = open)}>
    <Dialog.Content class="max-w-2xl">
      <Dialog.Header>
        <Dialog.Title>Edit Issue</Dialog.Title>
      </Dialog.Header>
      <div class="space-y-4">
        <Input bind:value={editTitle} placeholder="Issue title" />
        <Textarea bind:value={editBody} placeholder="Issue description" />
        <div class="flex flex-wrap gap-2">
          <Button onclick={() => handleUpdateIssue(undefined)} disabled={submitting}>
            Save
          </Button>
          <Button
            variant="outline"
            onclick={() => handleUpdateIssue("closed")}
            disabled={submitting}
          >
            Close
          </Button>
          <Button
            variant="outline"
            onclick={() => handleUpdateIssue("open")}
            disabled={submitting}
          >
            Reopen
          </Button>
        </div>
      </div>
    </Dialog.Content>
  </Dialog.Root>
</div>
