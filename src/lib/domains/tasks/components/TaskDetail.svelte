<script lang="ts">
  import FaArrowLeft from '~icons/fa6-solid/arrow-left';
  import FaWandMagicSparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import FaPencil from '~icons/fa6-solid/pencil';
  import FaTrash from '~icons/fa6-solid/trash';
  import FaClipboard from '~icons/fa6-solid/clipboard';
  import FaFlag from '~icons/fa6-solid/flag';
  import FaCalendar from '~icons/fa6-solid/calendar';
  import FaHourglassHalf from '~icons/fa6-solid/hourglass-half';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import Icon from '@iconify/svelte';
  import MarkdownView from '$lib/components/ui/markdown-view.svelte';
  import { PageLoading, PageError } from '$lib/components/shell';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { taskActions } from '../stores/taskStore';
  import { createTasksQuery } from '../queries/taskQueries';
  import {
    getTaskStatusColor,
    getStatusBadgeColor,
    getPriorityColor,
    getTaskIcon,
    formatMinutes,
  } from '../utils/taskDisplay';
  import SubtaskList from './SubtaskList.svelte';
  import TaskInfoCard from './TaskInfoCard.svelte';
  import TaskLinkedDocument from './TaskLinkedDocument.svelte';
  import TaskQuickActions from './TaskQuickActions.svelte';
  import {
    TASK_STATUS_OPTIONS,
    TASK_PRIORITY_OPTIONS,
    taskStatusLabel,
    type Task,
    type TaskStatus,
    type TaskPriority,
    type UpdateTaskRequest,
  } from '../types';

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
      <!-- Header -->
      <div class="mb-6 flex flex-wrap items-center justify-between gap-2">
        <div class="flex items-center gap-4">
          {#if onBack}
            <Button variant="ghost" onclick={onBack} class="flex items-center gap-2">
              <FaArrowLeft class="h-4 w-4" />
              Back to Tasks
            </Button>
            <Separator orientation="vertical" class="h-6" />
          {/if}
          <div class="flex items-center gap-2">
            <Icon icon={getTaskIcon(task)} class="h-5 w-5 {getTaskStatusColor(task.status)}" />
            <Badge class={getStatusBadgeColor(task.status)}>
              {taskStatusLabel(task.status)}
            </Badge>
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          {#if isEditing}
            <Button variant="outline" onclick={handleCancel} disabled={isSaving}>Cancel</Button>
            <Button onclick={handleSave} disabled={isSaving}>Save Changes</Button>
          {:else}
            <Button
              variant="outline"
              onclick={() => goto(`/tasks/${task.id}/generate`)}
              class="flex items-center gap-2"
            >
              <FaWandMagicSparkles class="h-4 w-4" />
              Generate Tasks with AI
            </Button>
            <Button variant="outline" onclick={handleEdit} class="flex items-center gap-2">
              <FaPencil class="h-4 w-4" />
              Edit
            </Button>
            <Button variant="destructive" onclick={handleDelete} class="flex items-center gap-2">
              <FaTrash class="h-4 w-4" />
              Delete
            </Button>
          {/if}
        </div>
      </div>

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
                <div class="space-y-4">
                  <div>
                    <Label for="edit-title">Title</Label>
                    <Input
                      id="edit-title"
                      bind:value={editTitle}
                      placeholder="Enter task title..."
                    />
                  </div>
                  <div>
                    <Label for="edit-description">Description</Label>
                    <Textarea
                      id="edit-description"
                      bind:value={editDescription}
                      placeholder="Enter task description..."
                      rows={4}
                    />
                  </div>
                  <div class="grid grid-cols-1 gap-4 @md:grid-cols-2">
                    <div>
                      <Label for="edit-status">Status</Label>
                      <Select
                        options={TASK_STATUS_OPTIONS}
                        defaultValue={editStatus}
                        placeholder="Select status..."
                        onSelect={(value) => (editStatus = value as TaskStatus)}
                      />
                    </div>
                    <div>
                      <Label for="edit-priority">Priority</Label>
                      <Select
                        options={TASK_PRIORITY_OPTIONS}
                        defaultValue={editPriority}
                        placeholder="Select priority..."
                        onSelect={(value) => (editPriority = value as TaskPriority)}
                      />
                    </div>
                  </div>
                  <div class="grid grid-cols-1 gap-4 @md:grid-cols-2">
                    <div>
                      <Label for="edit-type">Type</Label>
                      <Input
                        id="edit-type"
                        bind:value={editType}
                        placeholder="e.g., Story, Bug, Feature..."
                      />
                    </div>
                    <div>
                      <Label for="edit-due-date">Due Date</Label>
                      <Input id="edit-due-date" type="date" bind:value={editDueDate} />
                    </div>
                  </div>
                </div>
              {:else}
                <div class="space-y-4">
                  <div>
                    <h2 class="text-2xl font-bold text-foreground">
                      {task.title}
                    </h2>
                    {#if task.description}
                      <div class="mt-3">
                        <MarkdownView content={task.description} truncateAt={500} />
                      </div>
                    {:else}
                      <p class="mt-2 text-muted-foreground italic">No description provided</p>
                    {/if}
                  </div>

                  <div class="grid grid-cols-1 gap-4 @md:grid-cols-2">
                    <div>
                      <Label class="text-sm font-medium text-muted-foreground">Status</Label>
                      <div class="mt-1 flex items-center gap-2">
                        <Icon
                          icon={getTaskIcon(task)}
                          class="h-4 w-4 {getTaskStatusColor(task.status)}"
                        />
                        <Badge class={getStatusBadgeColor(task.status)}>
                          {taskStatusLabel(task.status)}
                        </Badge>
                      </div>
                    </div>

                    <div>
                      <Label class="text-sm font-medium text-muted-foreground">Priority</Label>
                      <div class="mt-1 flex items-center gap-2">
                        <FaFlag class="h-4 w-4 {getPriorityColor(task.priority)}" />
                        <Badge variant="outline" class="uppercase">
                          {task.priority}
                        </Badge>
                      </div>
                    </div>
                  </div>

                  {#if task.type}
                    <div>
                      <Label class="text-sm font-medium text-muted-foreground">Type</Label>
                      <div class="mt-1">
                        <Badge variant="outline">{task.type}</Badge>
                      </div>
                    </div>
                  {/if}

                  {#if task.dueDate}
                    <div>
                      <Label class="text-sm font-medium text-muted-foreground">Due Date</Label>
                      <div class="mt-1 flex items-center gap-2">
                        <FaCalendar class="h-4 w-4 text-muted-foreground" />
                        <span class="text-sm">{new Date(task.dueDate).toLocaleDateString()}</span>
                      </div>
                    </div>
                  {/if}

                  {#if task.assignee}
                    <div>
                      <Label class="text-sm font-medium text-muted-foreground">Assignee</Label>
                      <div class="mt-1 text-sm">{task.assignee}</div>
                    </div>
                  {/if}

                  {#if task.estimatedTime || task.actualTime}
                    <div>
                      <Label class="text-sm font-medium text-muted-foreground">Time</Label>
                      <div class="mt-1 flex items-center gap-2 text-sm">
                        <FaHourglassHalf class="h-4 w-4 text-muted-foreground" />
                        <span>
                          {task.estimatedTime ? formatMinutes(task.estimatedTime) : '-'} estimated{task.actualTime
                            ? ` / ${formatMinutes(task.actualTime)} spent`
                            : ''}
                        </span>
                      </div>
                    </div>
                  {/if}

                  {#if task.tags && task.tags.length > 0}
                    <div>
                      <Label class="text-sm font-medium text-muted-foreground">Tags</Label>
                      <div class="mt-1 flex flex-wrap gap-1">
                        {#each task.tags as tag, i (i)}
                          <Badge variant="outline">{tag}</Badge>
                        {/each}
                      </div>
                    </div>
                  {/if}
                </div>
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
