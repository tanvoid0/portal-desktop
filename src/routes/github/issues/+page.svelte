<script lang="ts">
  import FaTableColumns from '~icons/fa6-solid/table-columns';
  import FaList from '~icons/fa6-solid/list';

  import { Button } from '$lib/components/ui/button';
  import { PageHeader, PageLoading, PageEmpty } from '$lib/components/shell';
  import {
    createGitHubIssuesQuery,
    createGitHubRepositoriesQuery,
    createGitHubStatusQuery,
    githubService,
    GitHubConnectPrompt,
    GitHubError,
    type GitHubIssue,
  } from '$lib/domains/github';
  import GitHubIssueCreateDialog from '$lib/domains/github/components/GitHubIssueCreateDialog.svelte';
  import GitHubIssueEditDialog from '$lib/domains/github/components/GitHubIssueEditDialog.svelte';
  import { isTokenUnreadableError } from '$lib/domains/github/utils/tokenError';
  import { createTaskUiState } from '$lib/domains/tasks/state/taskUi.svelte';
  import KanbanBoard from '$lib/domains/tasks/components/KanbanBoard.svelte';
  import TaskList from '$lib/domains/tasks/components/TaskList.svelte';
  import type { Task } from '$lib/domains/tasks/types';
  import type { TaskDisplay } from '$lib/domains/tasks/utils/taskDisplay';
  import { toast } from '$lib/utils/toast';
  import Bug from '~icons/fa6-solid/bug';
  import Plus from '~icons/fa6-solid/plus';

  let filter: 'assigned' | 'created' = $state('assigned');
  let stateValue: 'open' | 'closed' | 'all' = $state('open');
  let createDialogOpen = $state(false);
  let editDialogOpen = $state(false);
  let selectedIssue: GitHubIssue | null = $state(null);
  let createRepoFullName = $state('');
  let createTitle = $state('');
  let createBody = $state('');
  let editTitle = $state('');
  let editBody = $state('');
  let submitting = $state(false);
  let currentView = $state<'kanban' | 'list'>('kanban');

  const statusQuery = createGitHubStatusQuery();
  const isConnected = $derived(statusQuery.data?.connected ?? false);
  const issuesQuery = createGitHubIssuesQuery(
    () => ({
      filter,
      state: stateValue,
      page: 1,
      perPage: 50,
    }),
    () => isConnected
  );
  const tokenUnreadable = $derived(isTokenUnreadableError(issuesQuery.error));
  const reposQuery = createGitHubRepositoriesQuery(
    () => '',
    () => isConnected && createDialogOpen
  );
  const repoOptions = $derived(
    (reposQuery.data ?? []).map((repo) => ({
      value: repo.fullName,
      label: repo.fullName,
    }))
  );

  // Reuse the tasks board UI: map GitHub issues onto the Task shape and give
  // the board its own state instance (not the local-tasks singleton).
  const issueUi = createTaskUiState();
  issueUi.archiveEnabled = false;
  const issueByTaskId = new Map<string, GitHubIssue>();

  function issueTaskId(issue: GitHubIssue): string {
    return `github:${issue.repoFullName ?? 'unknown'}#${issue.number}`;
  }

  function issueToTask(issue: GitHubIssue): Task {
    const id = issueTaskId(issue);
    issueByTaskId.set(id, issue);
    return {
      id,
      title: `#${issue.number} ${issue.title}`,
      description: issue.body ?? undefined,
      status: issue.state === 'closed' ? 'completed' : 'pending',
      priority: 'medium',
      type: issue.labels[0] ?? 'Bug',
      createdAt: issue.createdAt ? new Date(issue.createdAt) : new Date(),
      updatedAt: issue.updatedAt ? new Date(issue.updatedAt) : new Date(),
      resourceId: String(issue.id),
      resourceType: 'github_issue',
    };
  }

  $effect(() => {
    issueByTaskId.clear();
    issueUi.setTasks((issuesQuery.data ?? []).map(issueToTask));
  });

  function openEditDialog(issue: GitHubIssue) {
    selectedIssue = issue;
    editTitle = issue.title;
    editBody = issue.body || '';
    editDialogOpen = true;
  }

  function handleTaskSelect(task: Task) {
    const issue = issueByTaskId.get(task.id);
    if (issue) openEditDialog(issue);
  }

  async function setIssueState(issue: GitHubIssue, newState: 'open' | 'closed') {
    if (!issue.repoFullName) return;
    const [owner, repo] = issue.repoFullName.split('/');
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
      toast.error(error instanceof Error ? error.message : 'Failed to update issue');
    }
  }

  // Kanban drag / status-icon toggle both boil down to open vs closed for GitHub.
  function handleTaskMove(taskId: string, newStatus: string) {
    const issue = issueByTaskId.get(taskId);
    if (!issue) return;
    setIssueState(
      issue,
      newStatus === 'pending' || newStatus === 'in-progress' ? 'open' : 'closed'
    );
  }

  function handleTaskStatusToggle(taskId: string) {
    const issue = issueByTaskId.get(taskId);
    if (!issue) return;
    setIssueState(issue, issue.state === 'closed' ? 'open' : 'closed');
  }

  const issueDisplay: TaskDisplay = {
    statusColor: (status) => (status === 'completed' ? 'text-red-500' : 'text-green-500'),
    statusBadge: (status) =>
      status === 'completed'
        ? 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300'
        : 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300',
    priorityColor: () => 'text-muted-foreground',
    icon: (task) => (task.status === 'completed' ? 'mdi:source-merge' : 'mdi:source-branch'),
  };

  function noop() {}

  async function handleCreateIssue() {
    const [owner, repo] = createRepoFullName.split('/');
    if (!owner || !repo) return;
    try {
      submitting = true;
      await githubService.createIssue({
        owner,
        repo,
        title: createTitle.trim(),
        body: createBody.trim() || undefined,
      });
      toast.success('Issue created');
      createDialogOpen = false;
      createRepoFullName = '';
      createTitle = '';
      createBody = '';
      await issuesQuery.refetch();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to create issue');
    } finally {
      submitting = false;
    }
  }

  async function handleUpdateIssue(nextState?: 'open' | 'closed') {
    if (!selectedIssue?.repoFullName) return;
    const [owner, repo] = selectedIssue.repoFullName.split('/');
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
      toast.success(nextState ? `Issue ${nextState}` : 'Issue updated');
      editDialogOpen = false;
      await issuesQuery.refetch();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to update issue');
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head>
  <title>GitHub Issues - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader title="Issues" description="View and manage GitHub issues across your repositories">
    {#snippet actions()}
      <div class="flex rounded-lg bg-muted p-1">
        <Button
          onclick={() => (currentView = 'kanban')}
          variant={currentView === 'kanban' ? 'default' : 'ghost'}
          size="sm"
          class="px-3 py-1.5 text-sm font-medium"
        >
          <FaTableColumns class="mr-1.5 h-4 w-4" />
          Kanban
        </Button>
        <Button
          onclick={() => (currentView = 'list')}
          variant={currentView === 'list' ? 'default' : 'ghost'}
          size="sm"
          class="px-3 py-1.5 text-sm font-medium"
        >
          <FaList class="mr-1.5 h-4 w-4" />
          List
        </Button>
      </div>
      <Button onclick={() => (createDialogOpen = true)} disabled={tokenUnreadable}>
        <Plus class="mr-2 h-4 w-4" />
        New Issue
      </Button>
    {/snippet}
  </PageHeader>

  {#if statusQuery.isPending}
    <PageLoading message="Checking GitHub connection..." />
  {:else if !statusQuery.data?.connected}
    <GitHubConnectPrompt status={statusQuery.data} onConnected={() => statusQuery.refetch()} />
  {:else}
    <div class="flex flex-wrap items-center gap-3">
      {#each [{ label: 'Assigned', active: filter === 'assigned', set: () => (filter = 'assigned') }, { label: 'Created', active: filter === 'created', set: () => (filter = 'created') }, { label: 'Open', active: stateValue === 'open', set: () => (stateValue = 'open') }, { label: 'Closed', active: stateValue === 'closed', set: () => (stateValue = 'closed') }, { label: 'All', active: stateValue === 'all', set: () => (stateValue = 'all') }] as f (f.label)}
        <Button variant={f.active ? 'default' : 'outline'} onclick={f.set}>{f.label}</Button>
      {/each}
    </div>

    {#if issuesQuery.isPending}
      <PageLoading message="Loading issues..." />
    {:else if issuesQuery.isError}
      <GitHubError
        title="Failed to load issues"
        message={issuesQuery.error instanceof Error
          ? issuesQuery.error.message
          : 'Unable to load issues'}
        onRetry={() => issuesQuery.refetch()}
      />
    {:else if (issuesQuery.data ?? []).length === 0}
      <PageEmpty
        title="No issues found"
        description="Try a different filter or create a new issue."
        icon={Bug}
      />
    {:else if currentView === 'kanban'}
      <KanbanBoard
        ui={issueUi}
        {handleTaskSelect}
        {handleTaskStatusToggle}
        handleTaskSelection={noop}
        handleCreateSubtask={noop}
        display={issueDisplay}
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
        display={issueDisplay}
        showSubtaskActions={false}
      />
    {/if}
  {/if}

  <GitHubIssueCreateDialog
    bind:open={createDialogOpen}
    {repoOptions}
    reposPending={reposQuery.isPending}
    bind:repoFullName={createRepoFullName}
    bind:title={createTitle}
    bind:body={createBody}
    {submitting}
    onCreate={handleCreateIssue}
  />

  <GitHubIssueEditDialog
    bind:open={editDialogOpen}
    bind:title={editTitle}
    bind:body={editBody}
    htmlUrl={selectedIssue?.htmlUrl}
    {submitting}
    onUpdate={handleUpdateIssue}
  />
</div>
