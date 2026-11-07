<script lang="ts">
  import FaTurnDown from '~icons/fa6-solid/turn-down';
  import FaPlus from '~icons/fa6-solid/plus';
  import FaEye from '~icons/fa6-solid/eye';
  import FaPencil from '~icons/fa6-solid/pencil';
  import FaTrash from '~icons/fa6-solid/trash';
  import Icon from '@iconify/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import Select from '$lib/components/ui/select.svelte';
  import LoadingSpinner from '$lib/components/ui/loading-spinner.svelte';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { taskActions } from '../stores/taskStore';
  import { getTaskStatusColor, getStatusBadgeColor, getTaskIcon } from '../utils/taskDisplay';
  import {
    TASK_STATUS_OPTIONS,
    TASK_PRIORITY_OPTIONS,
    TASK_TYPE_OPTIONS,
    taskStatusLabel,
    type Task,
    type TaskStatus,
    type TaskPriority,
  } from '../types';

  interface Props {
    parentId: string;
    subtasks: Task[];
    /** Bound so an outside "Add Subtask" button can open the inline form. */
    showAddForm?: boolean;
    /** Shows a subtask; the view affordances are hidden when it is not supplied. */
    onOpenSubtask?: (taskId: string) => void;
  }

  let { parentId, subtasks, showAddForm = $bindable(false), onOpenSubtask }: Props = $props();

  let newSubtaskTitle = $state('');
  let newSubtaskDescription = $state('');
  let newSubtaskPriority = $state<TaskPriority>('medium');
  let newSubtaskType = $state('');
  let isAddingSubtask = $state(false);

  let editingSubtaskId = $state<string | null>(null);
  let editSubtaskTitle = $state('');
  let editSubtaskDescription = $state('');
  let editSubtaskPriority = $state<TaskPriority>('medium');
  let editSubtaskType = $state('');
  let editSubtaskStatus = $state<TaskStatus>('pending');

  function resetAddForm() {
    newSubtaskTitle = '';
    newSubtaskDescription = '';
    newSubtaskPriority = 'medium';
    newSubtaskType = '';
  }

  async function handleAddSubtask() {
    if (!newSubtaskTitle.trim()) return;

    isAddingSubtask = true;
    try {
      await taskActions.createTask({
        title: newSubtaskTitle,
        description: newSubtaskDescription,
        status: 'pending',
        priority: newSubtaskPriority,
        type: newSubtaskType,
        parentId,
      });

      resetAddForm();
      showAddForm = false;

      toastActions.success('Subtask created', 'Subtask has been created successfully');
    } catch (err) {
      toastActions.error(
        'Failed to create subtask',
        err instanceof Error ? err.message : 'Unknown error'
      );
    } finally {
      isAddingSubtask = false;
    }
  }

  function startEditSubtask(subtask: Task) {
    editingSubtaskId = subtask.id;
    editSubtaskTitle = subtask.title;
    editSubtaskDescription = subtask.description || '';
    editSubtaskPriority = subtask.priority;
    editSubtaskType = subtask.type || '';
    editSubtaskStatus = subtask.status;
  }

  async function handleSaveSubtask() {
    if (!editingSubtaskId) return;

    try {
      await taskActions.updateTask(editingSubtaskId, {
        title: editSubtaskTitle,
        description: editSubtaskDescription,
        priority: editSubtaskPriority,
        type: editSubtaskType,
        status: editSubtaskStatus,
      });

      editingSubtaskId = null;
      toastActions.success('Subtask updated', 'Subtask has been updated successfully');
    } catch (err) {
      toastActions.error(
        'Failed to update subtask',
        err instanceof Error ? err.message : 'Unknown error'
      );
    }
  }

  function cancelEditSubtask() {
    editingSubtaskId = null;
    editSubtaskTitle = '';
    editSubtaskDescription = '';
    editSubtaskPriority = 'medium';
    editSubtaskType = '';
    editSubtaskStatus = 'pending';
  }

  async function handleDeleteSubtask(subtaskId: string) {
    const confirmed = await confirmAction(
      'Are you sure you want to delete this subtask?',
      'Delete subtask',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!confirmed) return;

    try {
      await taskActions.deleteTask(subtaskId);
      toastActions.success('Subtask deleted', 'Subtask has been deleted successfully');
    } catch (err) {
      toastActions.error(
        'Failed to delete subtask',
        err instanceof Error ? err.message : 'Unknown error'
      );
    }
  }
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <FaTurnDown class="h-5 w-5" />
        Subtasks
      </div>
      <Button variant="outline" size="sm" onclick={() => (showAddForm = !showAddForm)}>
        <FaPlus class="mr-1 h-4 w-4" />
        {showAddForm ? 'Cancel' : 'Add Subtask'}
      </Button>
    </CardTitle>
  </CardHeader>
  <CardContent>
    <!-- Add Subtask Form -->
    {#if showAddForm}
      <div class="mb-4 rounded-lg border border-border bg-muted/20 p-4">
        <h4 class="mb-3 font-medium">Add New Subtask</h4>
        <div class="space-y-3">
          <div>
            <Label for="new-subtask-title">Title</Label>
            <Input
              id="new-subtask-title"
              bind:value={newSubtaskTitle}
              placeholder="Enter subtask title..."
            />
          </div>
          <div>
            <Label for="new-subtask-description">Description</Label>
            <Textarea
              id="new-subtask-description"
              bind:value={newSubtaskDescription}
              placeholder="Enter subtask description..."
              rows={2}
            />
          </div>
          <div class="grid grid-cols-1 gap-3 @md:grid-cols-2">
            <div>
              <Label for="new-subtask-priority">Priority</Label>
              <Select
                options={TASK_PRIORITY_OPTIONS}
                defaultValue={newSubtaskPriority}
                placeholder="Select priority..."
                onSelect={(value) => (newSubtaskPriority = value as TaskPriority)}
              />
            </div>
            <div>
              <Label for="new-subtask-type">Type</Label>
              <Select
                options={TASK_TYPE_OPTIONS}
                defaultValue={newSubtaskType}
                placeholder="Select type..."
                onSelect={(value) => (newSubtaskType = value)}
              />
            </div>
          </div>
          <div class="flex gap-2">
            <Button
              onclick={handleAddSubtask}
              disabled={!newSubtaskTitle.trim() || isAddingSubtask}
              class="flex-1"
            >
              {#if isAddingSubtask}
                <LoadingSpinner size="sm" class="mr-2" />
              {/if}
              Add Subtask
            </Button>
            <Button
              variant="outline"
              onclick={() => {
                showAddForm = false;
                resetAddForm();
              }}
            >
              Cancel
            </Button>
          </div>
        </div>
      </div>
    {/if}

    <!-- Subtasks List -->
    {#if subtasks.length > 0}
      <div class="space-y-2">
        {#each subtasks as subtask (subtask.id)}
          {#if editingSubtaskId === subtask.id}
            <!-- Edit Subtask Form -->
            <div class="rounded-lg border border-primary bg-primary/5 p-4">
              <h4 class="mb-3 font-medium">Edit Subtask</h4>
              <div class="space-y-3">
                <div>
                  <Label for="edit-subtask-title">Title</Label>
                  <Input
                    id="edit-subtask-title"
                    bind:value={editSubtaskTitle}
                    placeholder="Enter subtask title..."
                  />
                </div>
                <div>
                  <Label for="edit-subtask-description">Description</Label>
                  <Textarea
                    id="edit-subtask-description"
                    bind:value={editSubtaskDescription}
                    placeholder="Enter subtask description..."
                    rows={2}
                  />
                </div>
                <div class="grid grid-cols-1 gap-3 @md:grid-cols-3">
                  <div>
                    <Label for="edit-subtask-status">Status</Label>
                    <Select
                      options={TASK_STATUS_OPTIONS}
                      defaultValue={editSubtaskStatus}
                      placeholder="Select status..."
                      onSelect={(value) => (editSubtaskStatus = value as TaskStatus)}
                    />
                  </div>
                  <div>
                    <Label for="edit-subtask-priority">Priority</Label>
                    <Select
                      options={TASK_PRIORITY_OPTIONS}
                      defaultValue={editSubtaskPriority}
                      placeholder="Select priority..."
                      onSelect={(value) => (editSubtaskPriority = value as TaskPriority)}
                    />
                  </div>
                  <div>
                    <Label for="edit-subtask-type">Type</Label>
                    <Select
                      options={TASK_TYPE_OPTIONS}
                      defaultValue={editSubtaskType}
                      placeholder="Select type..."
                      onSelect={(value) => (editSubtaskType = value)}
                    />
                  </div>
                </div>
                <div class="flex gap-2">
                  <Button onclick={handleSaveSubtask} class="flex-1">Save Changes</Button>
                  <Button variant="outline" onclick={cancelEditSubtask}>Cancel</Button>
                </div>
              </div>
            </div>
          {:else}
            <!-- Subtask Display -->
            <div class="rounded-lg border border-border p-3 transition-colors hover:bg-muted/50">
              <div class="flex items-center justify-between gap-2">
                <div class="flex min-w-0 flex-1 items-center gap-2">
                  <Icon
                    icon={getTaskIcon(subtask)}
                    class="h-4 w-4 shrink-0 {getTaskStatusColor(subtask.status)}"
                  />
                  {#if onOpenSubtask}
                    <Button
                      variant="link"
                      class="h-auto min-w-0 p-0 text-left font-medium"
                      onclick={() => onOpenSubtask?.(subtask.id)}
                      title="View subtask details"
                    >
                      <span class="truncate">{subtask.title}</span>
                    </Button>
                  {:else}
                    <span class="truncate font-medium">{subtask.title}</span>
                  {/if}
                  {#if subtask.type}
                    <Badge variant="outline" class="shrink-0 text-xs">{subtask.type}</Badge>
                  {/if}
                </div>
                <div class="flex shrink-0 items-center gap-2">
                  <Badge class={getStatusBadgeColor(subtask.status)}>
                    {taskStatusLabel(subtask.status)}
                  </Badge>
                  <div class="flex gap-1">
                    {#if onOpenSubtask}
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => onOpenSubtask?.(subtask.id)}
                        title="View subtask details"
                      >
                        <FaEye class="h-4 w-4" />
                      </Button>
                    {/if}
                    <Button
                      variant="ghost"
                      size="sm"
                      onclick={() => startEditSubtask(subtask)}
                      title="Edit subtask"
                    >
                      <FaPencil class="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onclick={() => handleDeleteSubtask(subtask.id)}
                      title="Delete subtask"
                      class="text-destructive hover:text-destructive"
                    >
                      <FaTrash class="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </div>
              {#if subtask.description}
                <p class="mt-2 ml-6 text-sm text-muted-foreground">
                  {subtask.description}
                </p>
              {/if}
            </div>
          {/if}
        {/each}
      </div>
    {:else}
      <div class="py-8 text-center text-muted-foreground">
        <FaTurnDown class="mx-auto mb-2 h-8 w-8 opacity-50" />
        <p>No subtasks yet</p>
        <Button variant="outline" size="sm" onclick={() => (showAddForm = true)} class="mt-2">
          <FaPlus class="mr-1 h-4 w-4" />
          Add First Subtask
        </Button>
      </div>
    {/if}
  </CardContent>
</Card>
