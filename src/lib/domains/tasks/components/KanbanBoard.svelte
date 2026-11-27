<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Tooltip from '$lib/components/ui/tooltip';
  import { taskActions } from '../stores/taskStore';
  import { toastActions } from '$lib/utils/toast';
  import {
    taskUi as defaultTaskUi,
    getSubtaskCount,
    type TaskUiState,
  } from '../state/taskUi.svelte';
  import {
    createDragDropState,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    getDropZoneClasses,
  } from '../utils/dragDrop';
  import { defaultDisplay, type TaskDisplay } from '../utils/taskDisplay';
  import type { Task, TaskStatus } from '../types';
  import { buildTaskMenu } from '../utils/taskMenu';
  import KanbanTaskCard from './KanbanTaskCard.svelte';
  import KanbanSubtaskCard from './KanbanSubtaskCard.svelte';

  interface Props {
    ui?: TaskUiState;
    handleTaskSelect: (task: Task) => void;
    handleTaskStatusToggle: (taskId: string) => void;
    handleTaskSelection: (taskId: string) => void;
    handleCreateSubtask: (task: Task) => void;
    /** Project name to show on the card (All-projects mode); undefined hides it. */
    projectLabel?: (task: Task) => string | undefined;
    /** Navigates to the task's linked project when its badge is clicked. */
    onProjectClick?: (task: Task) => void;
    onTaskMove?: (taskId: string, newStatus: string) => void;
    onShowArchive?: () => void;
    /** Right-click menu actions — omit to hide the matching item. */
    onSetStatus?: (taskId: string, status: TaskStatus) => void;
    onCopyReference?: (task: Task) => void;
    onLaunchCoder?: (task: Task) => void;
    onLaunchAgents?: (task: Task) => void;
    onEditTask?: (task: Task) => void;
    onDeleteTask?: (task: Task) => void;
    /** Override status/priority colours and icons (GitHub issues use their own). */
    display?: TaskDisplay;
    showSubtaskActions?: boolean;
    /** No drag-and-drop (boards that mirror state owned elsewhere). */
    readonly?: boolean;
  }

  let {
    ui = defaultTaskUi,
    handleTaskSelect,
    handleTaskStatusToggle,
    handleTaskSelection,
    handleCreateSubtask,
    projectLabel,
    onProjectClick,
    onTaskMove,
    onShowArchive,
    onSetStatus,
    onCopyReference,
    onLaunchCoder,
    onLaunchAgents,
    onEditTask,
    onDeleteTask,
    display = defaultDisplay,
    showSubtaskActions = true,
    readonly = false,
  }: Props = $props();
  let taskUi = $derived(ui);

  // Drag and drop state
  let dragDropState = $state(createDragDropState());

  function getTaskSubtasks(taskId: string): Task[] {
    return taskUi.tasks.filter((task) => task.parentId === taskId);
  }

  async function handleTaskMove(taskId: string, newStatus: string) {
    if (onTaskMove) {
      onTaskMove(taskId, newStatus);
      return;
    }
    try {
      await taskActions.updateTask(taskId, { status: newStatus as any });
    } catch (error) {
      console.error('Failed to move task:', error);
      toastActions.error(
        'Failed to move task',
        error instanceof Error ? error.message : 'An unexpected error occurred'
      );
    }
  }

  function activate(task: Task) {
    if (taskUi.isMultiSelectMode) handleTaskSelection(task.id);
    else handleTaskSelect(task);
  }
</script>

<!--
  Height-bound so the horizontal scrollbar lands at the viewport edge. Let the
  board grow to its tallest column instead and that scrollbar ends up at the
  foot of a page thousands of pixels tall, which reads as "the right columns
  are cropped and I cannot scroll to them".
-->
<div class="flex h-full min-h-0 gap-4 overflow-x-auto pb-2">
  {#each taskUi.kanbanColumns as column (column.id)}
    <div
      class="flex min-h-0 min-w-[300px] flex-1 flex-col gap-3 {getDropZoneClasses(
        column.id,
        dragDropState,
        'rounded-lg border-2 border-dashed border-transparent p-2'
      )}"
      role="region"
      aria-label={readonly ? `${column.title} column` : `Drop zone for ${column.title} tasks`}
      ondragover={readonly ? undefined : (e) => handleDragOver(e, column.id, dragDropState)}
      ondragleave={readonly ? undefined : (e) => handleDragLeave(e, dragDropState)}
      ondrop={readonly ? undefined : (e) => handleDrop(e, column.id, dragDropState, handleTaskMove)}
    >
      <div class="flex shrink-0 items-center justify-between">
        {#if column.hint}
          <Tooltip.Root>
            <Tooltip.Trigger>
              {#snippet child({ props })}
                <h3 {...props} class="text-sm font-semibold text-foreground">{column.title}</h3>
              {/snippet}
            </Tooltip.Trigger>
            <Tooltip.Content>{column.hint}</Tooltip.Content>
          </Tooltip.Root>
        {:else}
          <h3 class="text-sm font-semibold text-foreground">{column.title}</h3>
        {/if}
        <Badge variant="secondary" class="border-0 bg-muted py-1 font-normal text-muted-foreground">
          {column.tasks.length}
        </Badge>
      </div>

      <!-- Each column scrolls on its own, so the board never outgrows the viewport. -->
      <div class="min-h-0 flex-1 space-y-3 overflow-y-auto pr-1">
        {#each column.tasks as task (task.id)}
          <div>
            <KanbanTaskCard
              {task}
              menuItems={buildTaskMenu(task, {
                onOpen: handleTaskSelect,
                onSetStatus,
                onCreateSubtask: showSubtaskActions ? handleCreateSubtask : undefined,
                onCopyReference,
                onLaunchCoder,
                onLaunchAgents,
                onEdit: onEditTask,
                onDelete: onDeleteTask,
              })}
              {display}
              subCount={getSubtaskCount(task.id, taskUi.tasks)}
              project={projectLabel?.(task)}
              selected={taskUi.selectedTaskIds.has(task.id)}
              multiSelectMode={taskUi.isMultiSelectMode}
              {readonly}
              {showSubtaskActions}
              {dragDropState}
              onActivate={() => activate(task)}
              onToggleSelection={() => handleTaskSelection(task.id)}
              onProjectClick={onProjectClick ? () => onProjectClick(task) : undefined}
              onCreateSubtask={() => handleCreateSubtask(task)}
              onToggleStatus={() => handleTaskStatusToggle(task.id)}
            />

            {#each getTaskSubtasks(task.id) as subtask (subtask.id)}
              <KanbanSubtaskCard
                {subtask}
                menuItems={buildTaskMenu(subtask, {
                  onOpen: handleTaskSelect,
                  onSetStatus,
                  onCopyReference,
                  onLaunchCoder,
                  onLaunchAgents,
                  onEdit: onEditTask,
                  onDelete: onDeleteTask,
                })}
                {display}
                {readonly}
                onOpen={() => handleTaskSelect(subtask)}
                onToggleStatus={() => handleTaskStatusToggle(subtask.id)}
              />
            {/each}
          </div>
        {/each}
      </div>

      {#if column.olderCount > 0}
        <Button
          variant="link"
          size="sm"
          class="h-auto shrink-0 p-0 text-xs text-muted-foreground"
          onclick={() => onShowArchive?.()}
        >
          {column.olderCount} older in Archive
        </Button>
      {/if}
    </div>
  {/each}
</div>
