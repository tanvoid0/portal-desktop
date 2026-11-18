<script lang="ts">
  import FaClipboard from '~icons/fa6-solid/clipboard';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { PageLoading, PageError } from '$lib/components/shell';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { taskActions } from '../stores/taskStore';
  import { createTasksQuery } from '../queries/taskQueries';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { ResourceType } from '$lib/domains/shared/types/resourceType';
  import { openClaudeSession } from '../utils/openClaudeSession';
  import { createQuery } from '@tanstack/svelte-query';
  import { agentSessionsQuery } from '$lib/domains/agents/queries/agentsQueries';
  import { openAgentsSessionById } from '$lib/domains/agents/utils/launch';
  import SubtaskList from './SubtaskList.svelte';
  import TaskInfoCard from './TaskInfoCard.svelte';
  import TaskLinkedDocument from './TaskLinkedDocument.svelte';
  import TaskQuickActions from './TaskQuickActions.svelte';
  import TaskDetailHeader from './TaskDetailHeader.svelte';
  import TaskDetailEditForm from './TaskDetailEditForm.svelte';
  import TaskDetailReadView from './TaskDetailReadView.svelte';
  import type { Task, TaskStatus, TaskPriority, UpdateTaskRequest } from '../types';

  interface Props {
    taskId: string;
    /** Leading header button; hidden when not supplied (the board rail has its own close). */
    onBack?: () => void;
    /** Fired after the task is deleted — the host decides where to go. */
    onDeleted?: () => void;
    /** Shows another task (parent or subtask) in place of this one. */
    onOpenTask?: (taskId: string) => void;
  }

  let { taskId, onBack, onDeleted, onOpenTask }: Props = $props();

  const tasksQuery = createTasksQuery();

  const allTasks = $derived(tasksQuery.data ?? []);
  const task = $derived(allTasks.find((t: Task) => t.id === taskId) ?? null);
  const subtasks = $derived(task ? allTasks.filter((t: Task) => t.parentId === task.id) : []);

  const projectsQuery = createProjectsQuery();
  const taskProjectPath = $derived(
    task?.resourceType === ResourceType.PROJECT && task.resourceId
      ? (projectsQuery.data ?? []).find((p) => String(p.id) === task.resourceId)?.path
      : undefined
  );

  const agentSessionsResult = createQuery(agentSessionsQuery);

  // A session claimed via the task board's `session:<id>` tag may now also be
  // an Agents-page run (it persists `backend_session_id`) — prefer jumping
  // there over a bare terminal when one matches, falling back otherwise.
  function handleResumeSession(sessionId: string) {
    const agentsMatch = (agentSessionsResult.data ?? []).find(
      (s) => s.backend_session_id === sessionId
    );
    if (agentsMatch) {
      openAgentsSessionById(agentsMatch.id);
      return;
    }
    openClaudeSession(sessionId, {
      title: `Claude: ${task?.title ?? sessionId}`,
      cwd: taskProjectPath,
    });
  }
  const isLoading = $derived(tasksQuery.isPending);
  const error = $derived(
    tasksQuery.isError
      ? tasksQuery.error instanceof Error
        ? tasksQuery.error.message
        : 'Failed to load task'
      : tasksQuery.isSuccess && !task
        ? 'Task not found'
        : null
  );

  let isEditing = $state(false);
  let isSaving = $state(false);

  // Edit form state
  let editTitle = $state('');
  let editDescription = $state('');
  let editStatus = $state<TaskStatus>('pending');
  let editPriority = $state<TaskPriority>('medium');
  let editType = $state('');
  let editDueDate = $state('');

  let showAddSubtask = $state(false);

  async function reloadTaskData() {
    const result = await tasksQuery.refetch();
    if (result.isError) {
      toastActions.error(
        'Failed to load task',
        result.error instanceof Error ? result.error.message : 'Unknown error'
      );
      return;
    }
    const loaded = result.data?.find((t: Task) => t.id === taskId);
    if (!loaded) {
      toastActions.error('Task not found', 'The requested task could not be found');
    }
  }

  function resetEditForm(t: Task) {
    editTitle = t.title;
    editDescription = t.description || '';
    editStatus = t.status;
    editPriority = t.priority;
    editType = t.type || '';
    editDueDate = t.dueDate ? t.dueDate.toISOString().split('T')[0] : '';
  }

  // Keep the edit form in step with the task being shown.
  $effect(() => {
    if (task) resetEditForm(task);
  });

  async function handleSave() {
    if (!task) return;

    try {
      isSaving = true;
      const updateData: UpdateTaskRequest = {
        title: editTitle.trim(),
        description: editDescription.trim() || undefined,
        status: editStatus,
        priority: editPriority,
        type: editType.trim() || undefined,
        dueDate: editDueDate ? new Date(editDueDate) : undefined,
      };

      await taskActions.updateTask(task.id, updateData);
      toastActions.success('Task updated successfully');
      isEditing = false;
    } catch (err) {
      toastActions.error(
        'Failed to update task',
        err instanceof Error ? err.message : 'An unexpected error occurred'
      );
    } finally {
      isSaving = false;
    }
  }

  function handleEdit() {
    isEditing = true;
  }

  function handleCancel() {
    if (task) resetEditForm(task);
    isEditing = false;
  }

  async function handleDelete() {
    if (!task) return;

    const confirmed = await confirmAction(
      `Are you sure you want to delete "${task.title}"?${subtasks.length ? ` Also deletes ${subtasks.length} subtask${subtasks.length === 1 ? '' : 's'}.` : ''} This action cannot be undone.`,
      'Delete task',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!confirmed) return;

    try {
      await taskActions.deleteTask(task.id);
      toastActions.success('Task deleted successfully');
      onDeleted?.();
    } catch (err) {
      toastActions.error(
        'Failed to delete task',
        err instanceof Error ? err.message : 'An unexpected error occurred'
      );
    }
  }

  // Keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      if (event.key === 'e') {
        event.preventDefault();
        if (!isEditing) handleEdit();
      } else if (event.key === 's' && isEditing) {
        event.preventDefault();
        handleSave();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        if (isEditing) handleCancel();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="@container">
  {#if isLoading}
    <PageLoading message="Loading task..." />
  {:else if error}
    <PageError title="Failed to load task" message={error} onRetry={reloadTaskData} />
  {:else if task}
    <div>
      <TaskDetailHeader
        {task}
        {onBack}
        {isEditing}
        {isSaving}
        onEdit={handleEdit}
        onCancel={handleCancel}
        onSave={handleSave}
        onDelete={handleDelete}
        onResumeSession={handleResumeSession}
      />

      <div class="grid grid-cols-1 gap-6 @3xl:grid-cols-3">
        <!-- Main Content -->
        <div class="space-y-6 @3xl:col-span-2">
          <!-- Task Details -->
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <FaClipboard class="h-5 w-5" />
                Task Details
              </CardTitle>
            </CardHeader>
            <CardContent class="space-y-4">
              {#if isEditing}
                <TaskDetailEditForm
                  bind:title={editTitle}
                  bind:description={editDescription}
                  bind:status={editStatus}
                  bind:priority={editPriority}
                  bind:type={editType}
                  bind:dueDate={editDueDate}
                />
              {:else}
                <TaskDetailReadView {task} />
              {/if}
            </CardContent>
          </Card>

          <!-- Subtasks -->
          {#if !task.parentId}
            <SubtaskList
              parentId={task.id}
              {subtasks}
              bind:showAddForm={showAddSubtask}
              onOpenSubtask={onOpenTask}
            />
          {/if}
        </div>

        <!-- Sidebar -->
        <div class="space-y-6">
          <TaskInfoCard {task} {onOpenTask} />
          <TaskLinkedDocument {task} />
          <TaskQuickActions {task} bind:showAddSubtask />
        </div>
      </div>

      <!-- Keyboard shortcuts hint -->
      <div class="mt-6 text-center text-xs text-muted-foreground">
        <kbd class="rounded bg-muted px-1.5 py-0.5 text-xs">Ctrl+E</kbd> to edit,
        <kbd class="ml-1 rounded bg-muted px-1.5 py-0.5 text-xs">Ctrl+S</kbd> to save,
        <kbd class="ml-1 rounded bg-muted px-1.5 py-0.5 text-xs">Esc</kbd> to cancel
      </div>
    </div>
  {/if}
</div>
