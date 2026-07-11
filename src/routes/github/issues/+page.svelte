<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import Select from "$lib/components/ui/select.svelte";
  import { Textarea } from "$lib/components/ui/textarea";
  import * as Dialog from "$lib/components/ui/dialog";
  import { PageHeader, PageLoading, PageError, PageEmpty } from "$lib/components/shell";
  import {
    createGitHubIssuesQuery,
    createGitHubRepositoriesQuery,
    createGitHubStatusQuery,
    githubService,
    GitHubConnectPrompt,
    type GitHubIssue,
  } from "$lib/domains/github";
  import { createTaskUiState } from "$lib/domains/tasks/state/taskUi.svelte";
  import KanbanBoard from "$lib/domains/tasks/components/KanbanBoard.svelte";
  import TaskList from "$lib/domains/tasks/components/TaskList.svelte";
  import type { Task } from "$lib/domains/tasks/types";
  import { toast } from "$lib/utils/toast";
  import { Bug, ExternalLink, Plus } from "@lucide/svelte";
  import Icon from "@iconify/svelte";

  let filter: "assigned" | "created" = $state("assigned");
  let stateValue: "open" | "closed" | "all" = $state("open");
  let createDialogOpen = $state(false);
  let editDialogOpen = $state(false);
  let selectedIssue: GitHubIssue | null = $state(null);
  let createRepoFullName = $state("");
  let createTitle = $state("");
  let createBody = $state("");
  let editTitle = $state("");
  let editBody = $state("");
  let submitting = $state(false);
  let currentView = $state<"kanban" | "list">("kanban");

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
  const reposQuery = createGitHubRepositoriesQuery(
    () => "",
    () => isConnected && createDialogOpen,
  );
  const repoOptions = $derived(
    (reposQuery.data ?? []).map((repo) => ({
      value: repo.fullName,
      label: repo.fullName,
    })),
  );

  // Reuse the tasks board UI: map GitHub issues onto the Task shape and give
  // the board its own state instance (not the local-tasks singleton).
  const issueUi = createTaskUiState();
  const issueByTaskId = new Map<string, GitHubIssue>();

  function issueTaskId(issue: GitHubIssue): string {
    return `github:${issue.repoFullName ?? "unknown"}#${issue.number}`;
  }

  function issueToTask(issue: GitHubIssue): Task {
    const id = issueTaskId(issue);
    issueByTaskId.set(id, issue);
    return {
      id,
      title: `#${issue.number} ${issue.title}`,
      description: issue.body,
      status: issue.state === "closed" ? "completed" : "pending",
      priority: "medium",
      type: issue.labels[0] ?? "Bug",
      createdAt: issue.createdAt ? new Date(issue.createdAt) : new Date(),
      updatedAt: issue.updatedAt ? new Date(issue.updatedAt) : new Date(),
      resourceId: String(issue.id),
      resourceType: "github_issue",
    };
  }

  $effect(() => {
    issueByTaskId.clear();
    issueUi.setTasks((issuesQuery.data ?? []).map(issueToTask));
  });

  function openEditDialog(issue: GitHubIssue) {
    selectedIssue = issue;
    editTitle = issue.title;
    editBody = issue.body || "";
    editDialogOpen = true;
  }

  function handleTaskSelect(task: Task) {
    const issue = issueByTaskId.get(task.id);
    if (issue) openEditDialog(issue);
  }

  async function setIssueState(issue: GitHubIssue, newState: "open" | "closed") {
    if (!issue.repoFullName) return;
    const [owner, repo] = issue.repoFullName.split("/");
    try {
      await githubService.updateIssue({
        owner,
        repo,
        number: issue.number,
        state: newState,
      });
      toast.success(`Issue ${newState}`);
      await issuesQuery.refetch();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to update issue");
    }
  }

  // Kanban drag / status-icon toggle both boil down to open vs closed for GitHub.
  function handleTaskMove(taskId: string, newStatus: string) {
    const issue = issueByTaskId.get(taskId);
    if (!issue) return;
    setIssueState(issue, newStatus === "pending" || newStatus === "in-progress" ? "open" : "closed");
  }

  function handleTaskStatusToggle(taskId: string) {
    const issue = issueByTaskId.get(taskId);
    if (!issue) return;
    setIssueState(issue, issue.state === "closed" ? "open" : "closed");
  }

  function noop() {}
  function zero() {
    return 0;
  }

  async function handleCreateIssue() {
    const [owner, repo] = createRepoFullName.split("/");
    if (!owner || !repo) return;
    try {
      submitting = true;
      await githubService.createIssue({
        owner,
        repo,
        title: createTitle.trim(),
        body: createBody.trim() || undefined,
      });
      toast.success("Issue created");
      createDialogOpen = false;
      createRepoFullName = "";
      createTitle = "";
      createBody = "";
      await issuesQuery.refetch();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to create issue");
    } finally {
      submitting = false;
    }
  }

  async function handleUpdateIssue(nextState?: "open" | "closed") {
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
        state: nextState,
      });
      toast.success(nextState ? `Issue ${nextState}` : "Issue updated");
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
      <div class="flex rounded-lg bg-muted p-1">
        <Button
          onclick={() => (currentView = "kanban")}
          variant={currentView === "kanban" ? "default" : "ghost"}
          size="sm"
          class="px-3 py-1.5 text-sm font-medium"
        >
          <Icon icon="mdi:view-column" class="mr-1.5 h-4 w-4" />
          Kanban
        </Button>
        <Button
          onclick={() => (currentView = "list")}
          variant={currentView === "list" ? "default" : "ghost"}
          size="sm"
          class="px-3 py-1.5 text-sm font-medium"
        >
          <Icon icon="mdi:format-list-bulleted" class="mr-1.5 h-4 w-4" />
          List
        </Button>
      </div>
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
    {:else if currentView === "kanban"}
      <KanbanBoard
        ui={issueUi}
        {handleTaskSelect}
        {handleTaskStatusToggle}
        handleTaskSelection={noop}
        handleCreateSubtask={noop}
        getSubtaskCount={zero}
        getTaskStatusColor={(status) => (status === "completed" ? "text-red-500" : "text-green-500")}
        getStatusBadgeColor={(status) =>
          status === "completed"
            ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300"
            : "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300"}
        getPriorityColor={() => "text-muted-foreground"}
        getTaskIcon={(task) => (task.status === "completed" ? "mdi:source-merge" : "mdi:source-branch")}
        onTaskMove={handleTaskMove}
        showSubtaskActions={false}
      />
    {:else}
      <TaskList
        ui={issueUi}
        {handleTaskSelect}
        {handleTaskStatusToggle}
        handleTaskSelection={noop}
        handleCreateSubtask={noop}
        getSubtaskCount={zero}
        getTaskStatusColor={(status) => (status === "completed" ? "text-red-500" : "text-green-500")}
        getStatusBadgeColor={(status) =>
          status === "completed"
            ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300"
            : "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300"}
        getPriorityColor={() => "text-muted-foreground"}
        getTaskIcon={(task) => (task.status === "completed" ? "mdi:source-merge" : "mdi:source-branch")}
        showSubtaskActions={false}
      />
    {/if}
  {/if}

  <Dialog.Root open={createDialogOpen} onOpenChange={(open) => (createDialogOpen = open)}>
    <Dialog.Content class="max-w-2xl">
      <Dialog.Header>
        <Dialog.Title>Create Issue</Dialog.Title>
      </Dialog.Header>
      <div class="space-y-4">
        <Select
          bind:value={createRepoFullName}
          options={repoOptions}
          placeholder={reposQuery.isPending ? "Loading repositories..." : "Select a repository"}
        />
        <Input bind:value={createTitle} placeholder="Issue title" />
        <Textarea bind:value={createBody} placeholder="Issue description" />
        <Button
          onclick={handleCreateIssue}
          disabled={submitting || !createRepoFullName || !createTitle.trim()}
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
        {#if selectedIssue?.htmlUrl}
          <a
            href={selectedIssue.htmlUrl}
            target="_blank"
            rel="noreferrer"
            class="inline-flex items-center text-sm text-primary"
          >
            Open on GitHub
            <ExternalLink class="ml-1 h-4 w-4" />
          </a>
        {/if}
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
