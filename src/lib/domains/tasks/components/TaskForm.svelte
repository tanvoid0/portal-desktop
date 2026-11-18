<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Textarea } from '$lib/components/ui/textarea';
  import Select from '$lib/components/ui/select.svelte';
  import { toastActions } from '$lib/utils/toast';
  import { taskActions } from '../stores/taskStore';
  import type {
    Task,
    CreateTaskRequest,
    UpdateTaskRequest,
    TaskStatus,
    TaskPriority,
  } from '../types';
  import { TASK_STATUS_OPTIONS, TASK_PRIORITY_OPTIONS, TASK_TYPE_OPTIONS } from '../types';
  import { ResourceType } from '$lib/domains/shared/types/resourceType';
  import { documentActions } from '$lib/domains/documents';
  import DocumentLinkSelector from '$lib/domains/documents/components/DocumentLinkSelector.svelte';
  import { goto } from '$app/navigation';
  import TaskFormField from './TaskFormField.svelte';
  import TaskTagInput from './TaskTagInput.svelte';
  import TaskFormAdvanced from './TaskFormAdvanced.svelte';

  interface Props {
    task?: Task;
    onSave?: (task: Task) => void;
    onCancel?: () => void;
    parentId?: string;
  }

  let { task, onSave, onCancel, parentId }: Props = $props();

  // Form state
  let title = $state(task?.title || '');
  let description = $state(task?.description || '');
  let status = $state<TaskStatus>(task?.status || 'pending');
  let priority = $state<TaskPriority>(task?.priority || 'medium');
  let type = $state(task?.type || '');
  let dueDate = $state(task?.dueDate ? task.dueDate.toISOString().split('T')[0] : '');

  // New advanced fields
  let estimatedTime = $state(task?.estimatedTime || 0);
  let actualTime = $state(task?.actualTime || 0);
  let tags = $state<string[]>(task?.tags || []);
  let assignee = $state(task?.assignee || '');
  let recurringPattern = $state(task?.recurring?.pattern || '');
  let recurringInterval = $state(task?.recurring?.interval || 1);
  let recurringEndDate = $state(
    task?.recurring?.endDate ? task.recurring.endDate.toISOString().split('T')[0] : ''
  );
  let blockedBy = $state<string[]>(task?.blockedBy || []);
  let blocks = $state<string[]>(task?.blocks || []);

  // Document linking
  let linkedDocumentId = $state<number | null>(
    task?.resourceType === ResourceType.DOCUMENT && task?.resourceId
      ? parseInt(task.resourceId)
      : null
  );

  // Load documents when component mounts
  $effect(() => {
    documentActions.loadDocuments();
  });

  // UI state
  let isSubmitting = $state(false);
  let errors = $state<Record<string, string>>({});

  // Validation
  function validateForm(): boolean {
    errors = {};

    if (!title.trim()) {
      errors.title = 'Title is required';
    }

    if (title.length > 100) {
      errors.title = 'Title must be less than 100 characters';
    }

    if (description && description.length > 500) {
      errors.description = 'Description must be less than 500 characters';
    }

    if (estimatedTime < 0) {
      errors.estimatedTime = 'Estimated time cannot be negative';
    }

    if (actualTime < 0) {
      errors.actualTime = 'Actual time cannot be negative';
    }

    return Object.keys(errors).length === 0;
  }

  // Form submission
  async function handleSubmit() {
    if (!validateForm()) {
      toastActions.error('Please fix the errors below');
      return;
    }

    try {
      isSubmitting = true;

      const taskData: CreateTaskRequest | UpdateTaskRequest = {
        title: title.trim(),
        description: description.trim() || undefined,
        status,
        priority,
        type: type.trim() || undefined,
        ...(dueDate ? { dueDate: new Date(dueDate) } : {}),
        ...(parentId ? { parentId } : {}),
        // Document linking
        resourceId: linkedDocumentId ? linkedDocumentId.toString() : undefined,
        resourceType: linkedDocumentId ? ResourceType.DOCUMENT : undefined,
        // New advanced fields
        estimatedTime: estimatedTime || undefined,
        actualTime: actualTime || undefined,
        tags: tags.length > 0 ? tags : undefined,
        assignee: assignee.trim() || undefined,
        recurring: recurringPattern
          ? {
              pattern: recurringPattern as any,
              interval: recurringInterval,
              endDate: recurringEndDate ? new Date(recurringEndDate) : undefined,
            }
          : undefined,
        blockedBy: blockedBy.length > 0 ? blockedBy : undefined,
        blocks: blocks.length > 0 ? blocks : undefined,
      };

      let savedTask: Task;
      if (task) {
        savedTask = await taskActions.updateTask(task.id, taskData as UpdateTaskRequest);
        toastActions.success('Task updated successfully');
      } else {
        savedTask = await taskActions.createTask(taskData as CreateTaskRequest);
        toastActions.success('Task created successfully');
      }

      onSave?.(savedTask);
    } catch (error) {
      toastActions.error(
        'Failed to save task',
        error instanceof Error ? error.message : 'An unexpected error occurred'
      );
    } finally {
      isSubmitting = false;
    }
  }

  // Keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      if (event.key === 'Enter') {
        event.preventDefault();
        handleSubmit();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        onCancel?.();
      }
    }
  }
</script>

<Card class="mx-auto max-w-2xl">
  <CardContent class="space-y-6" onkeydown={handleKeydown}>
    <TaskFormField label="Title *" for="title" error={errors.title}>
      <Input
        id="title"
        bind:value={title}
        placeholder="Enter task title..."
        class={errors.title ? 'border-destructive' : ''}
      />
    </TaskFormField>

    <TaskFormField label="Description" for="description" error={errors.description}>
      <Textarea
        id="description"
        bind:value={description}
        placeholder="Enter task description..."
        rows={3}
        class={errors.description ? 'border-destructive' : ''}
      />
    </TaskFormField>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <TaskFormField label="Status" for="status">
        <Select
          options={TASK_STATUS_OPTIONS}
          defaultValue={status}
          placeholder="Select status..."
          onSelect={(value) => (status = value as TaskStatus)}
        />
      </TaskFormField>
      <TaskFormField label="Priority" for="priority">
        <Select
          options={TASK_PRIORITY_OPTIONS}
          defaultValue={priority}
          placeholder="Select priority..."
          onSelect={(value) => (priority = value as TaskPriority)}
        />
      </TaskFormField>
    </div>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <TaskFormField label="Type" for="type">
        <Select
          options={TASK_TYPE_OPTIONS}
          defaultValue={type}
          placeholder="Select type..."
          onSelect={(value) => (type = value)}
        />
      </TaskFormField>
      <TaskFormField label="Due Date" for="dueDate">
        <Input id="dueDate" type="date" bind:value={dueDate} />
      </TaskFormField>
    </div>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <TaskFormField
        label="Estimated Time (minutes)"
        for="estimatedTime"
        error={errors.estimatedTime}
      >
        <Input
          id="estimatedTime"
          type="number"
          min="0"
          bind:value={estimatedTime}
          placeholder="e.g., 120 for 2 hours"
          class={errors.estimatedTime ? 'border-destructive' : ''}
        />
      </TaskFormField>
      <TaskFormField label="Actual Time (minutes)" for="actualTime" error={errors.actualTime}>
        <Input
          id="actualTime"
          type="number"
          min="0"
          bind:value={actualTime}
          placeholder="e.g., 90 for 1.5 hours"
          class={errors.actualTime ? 'border-destructive' : ''}
        />
      </TaskFormField>
    </div>

    <TaskFormField label="Tags">
      <TaskTagInput bind:tags />
    </TaskFormField>

    <TaskFormField label="Assignee" for="assignee">
      <Input id="assignee" bind:value={assignee} placeholder="Enter assignee name or email..." />
    </TaskFormField>

    <div class="space-y-2">
      <DocumentLinkSelector
        selectedDocumentId={linkedDocumentId}
        onSelect={(doc) => {
          linkedDocumentId = doc?.id || null;
        }}
        onCreateNew={() => {
          // Create new document and link it
          goto(
            `/documents/create?taskId=${task?.id || ''}&taskTitle=${encodeURIComponent(title || '')}`
          );
        }}
      />
    </div>

    <TaskFormAdvanced
      bind:recurringPattern
      bind:recurringInterval
      bind:recurringEndDate
      bind:blockedBy
      bind:blocks
    />

    <!-- Actions -->
    <div class="divider-edge-t divider-edge-full flex justify-end gap-3 pt-4">
      <Button variant="outline" onclick={onCancel} disabled={isSubmitting}>Cancel</Button>
      <Button onclick={handleSubmit} disabled={isSubmitting}>
        {task ? 'Update Task' : 'Create Task'}
      </Button>
    </div>

    <!-- Keyboard shortcuts hint -->
    <div class="text-xs text-muted-foreground">
      <kbd class="rounded bg-muted px-1.5 py-0.5 text-xs">Ctrl+Enter</kbd> to save,
      <kbd class="ml-1 rounded bg-muted px-1.5 py-0.5 text-xs">Ctrl+Escape</kbd> to cancel
    </div>
  </CardContent>
</Card>
