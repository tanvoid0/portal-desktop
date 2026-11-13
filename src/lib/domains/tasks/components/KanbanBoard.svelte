<script lang="ts">
  import FaTurnDown from '~icons/fa6-solid/turn-down';
  import FaFlag from '~icons/fa6-solid/flag';
  import FaPlus from '~icons/fa6-solid/plus';
  import FaClock from '~icons/fa6-solid/clock';
  import FaLock from '~icons/fa6-solid/lock';
  import FaUser from '~icons/fa6-solid/user';
  import FaFolder from '~icons/fa6-solid/folder';
  import FaCircleExclamation from '~icons/fa6-solid/circle-exclamation';
  import FaCalendarWeek from '~icons/fa6-solid/calendar-week';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Icon from '@iconify/svelte';
  import { taskActions } from '../stores/taskStore';
  import { toastActions } from '$lib/utils/toast';
  import {
    taskUi as defaultTaskUi,
    getSubtaskCount,
    type TaskUiState,
  } from '../state/taskUi.svelte';
  import {
    createDragDropState,
    handleDragStart,
    handleDragEnd,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    getDropZoneClasses,
    getTaskCardClasses,
  } from '../utils/dragDrop';
  import {
    defaultDisplay,
    isOverdue,
    isDueToday,
    isDueThisWeek,
    formatMinutes,
    type TaskDisplay,
  } from '../utils/taskDisplay';
  import { taskStatusLabel, type Task, type TaskStatus } from '../types';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import { buildTaskMenu } from '../utils/taskMenu';

  interface Props {
    ui?: TaskUiState;
    handleTaskSelect: (task: Task) => void;
    handleTaskStatusToggle: (taskId: string) => void;
    handleTaskSelection: (taskId: string) => void;
    handleCreateSubtask: (task: Task) => void;
    /** Project name to show on the card (All-projects mode); undefined hides it. */
    projectLabel?: (task: Task) => string | undefined;
    onTaskMove?: (taskId: string, newStatus: string) => void;
    onShowArchive?: () => void;
    /** Right-click menu actions — omit to hide the matching item. */
    onSetStatus?: (taskId: string, status: TaskStatus) => void;
    onCopyReference?: (task: Task) => void;
    onLaunchCoder?: (task: Task) => void;
    onEditTask?: (task: Task) => void;
    onDeleteTask?: (task: Task) => void;
    /** Override status/priority colours and icons (GitHub issues use their own). */
    display?: TaskDisplay;
    showSubtaskActions?: boolean;
  }

  let {
    ui = defaultTaskUi,
    handleTaskSelect,
    handleTaskStatusToggle,
    handleTaskSelection,
    handleCreateSubtask,
    projectLabel,
    onTaskMove,
    onShowArchive,
    onSetStatus,
    onCopyReference,
    onLaunchCoder,
    onEditTask,
    onDeleteTask,
    display = defaultDisplay,
    showSubtaskActions = true,
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
      aria-label="Drop zone for {column.title} tasks"
      ondragover={(e) => handleDragOver(e, column.id, dragDropState)}
      ondragleave={(e) => handleDragLeave(e, dragDropState)}
      ondrop={(e) => handleDrop(e, column.id, dragDropState, handleTaskMove)}
    >
      <div class="flex shrink-0 items-center justify-between">
        <h3 class="text-sm font-semibold text-foreground">{column.title}</h3>
        <span class="rounded-full bg-muted px-2 py-1 text-xs text-muted-foreground">
          {column.tasks.length}
        </span>
      </div>

      <!-- Each column scrolls on its own, so the board never outgrows the viewport. -->
      <div class="min-h-0 flex-1 space-y-3 overflow-y-auto pr-1">
        {#each column.tasks as task (task.id)}
          {@const subCount = getSubtaskCount(task.id, taskUi.tasks)}
          {@const project = projectLabel?.(task)}
          {@const tags = task.tags ?? []}
          {@const blocked = !!task.blockedBy && task.blockedBy.length > 0}
          <div>
            <ContextMenuList
              items={buildTaskMenu(task, {
                onOpen: handleTaskSelect,
                onSetStatus,
                onCreateSubtask: showSubtaskActions ? handleCreateSubtask : undefined,
                onCopyReference,
                onLaunchCoder,
                onEdit: onEditTask,
                onDelete: onDeleteTask,
              })}
            >
              {#snippet trigger(menuProps)}
                <Card
                  {...menuProps}
                  draggable="true"
                  ondragstart={(e: DragEvent) => handleDragStart(e, task, dragDropState)}
                  ondragend={(e: DragEvent) => handleDragEnd(e, dragDropState)}
                  onclick={() => activate(task)}
                  onkeydown={(e: KeyboardEvent) => {
                    if (e.target !== e.currentTarget) return;
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      activate(task);
                    }
                  }}
                  role="group"
                  tabindex={0}
                  aria-label="Task: {task.title}"
                  aria-current={taskUi.selectedTaskIds.has(task.id) ? 'true' : undefined}
                  class="cursor-pointer gap-0 py-0 hover:shadow-md {taskUi.selectedTaskIds.has(
                    task.id
                  )
                    ? 'bg-warning-50 ring-2 ring-warning-500 dark:bg-warning-900/20'
                    : ''} {getTaskCardClasses(task, dragDropState)}"
                >
                  <CardContent class="space-y-2 p-3">
                    <div class="flex items-start gap-2">
                      {#if taskUi.isMultiSelectMode}
                        <Checkbox
                          checked={taskUi.selectedTaskIds.has(task.id)}
                          onCheckedChange={() => handleTaskSelection(task.id)}
                          onclick={(e) => e.stopPropagation()}
                          class="mt-0.5 shrink-0"
                        />
                      {/if}
                      <h4
                        class="line-clamp-2 min-w-0 flex-1 text-sm font-medium break-words text-foreground"
                      >
                        {task.title}
                      </h4>
                      <div class="flex shrink-0 items-center pt-0.5">
                        <span
                          title="{task.priority} priority"
                          aria-label="{task.priority} priority"
                        >
                          <FaFlag class="h-3 w-3 {display.priorityColor(task.priority)}" />
                        </span>
                      </div>
                    </div>

                    {#if task.description}
                      <p class="line-clamp-2 text-xs text-muted-foreground">{task.description}</p>
                    {/if}

                    {#if task.type || isOverdue(task) || isDueToday(task) || isDueThisWeek(task) || blocked || tags.length > 0}
                      <div class="flex flex-wrap items-center gap-1">
                        {#if task.type}
                          <Badge variant="outline">{task.type}</Badge>
                        {/if}
                        {#if isOverdue(task)}
                          <Badge variant="destructive"><FaCircleExclamation /> Overdue</Badge>
                        {:else if isDueToday(task)}
                          <Badge
                            variant="secondary"
                            class="bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300"
                          >
                            <FaClock /> Today
                          </Badge>
                        {:else if isDueThisWeek(task)}
                          <Badge variant="outline"><FaCalendarWeek /> This week</Badge>
                        {/if}
                        {#if blocked}
                          <Badge
                            variant="outline"
                            class="border-red-200 text-red-600 dark:border-red-800 dark:text-red-400"
                          >
                            <FaLock /> Blocked
                          </Badge>
                        {/if}
                        {#each tags.slice(0, 2) as tag (tag)}
                          <Badge variant="secondary" class="max-w-[8rem] truncate">{tag}</Badge>
                        {/each}
                        {#if tags.length > 2}
                          <Badge variant="secondary" title={tags.slice(2).join(', ')}>
                            +{tags.length - 2}
                          </Badge>
                        {/if}
                      </div>
                    {/if}

                    <div class="flex items-center justify-between gap-2">
                      <div
                        class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground"
                      >
                        {#if project}
                          <span class="flex min-w-0 items-center gap-1" title={project}>
                            <FaFolder class="h-3 w-3 shrink-0" />
                            <span class="max-w-[10rem] truncate">{project}</span>
                          </span>
                        {/if}
                        {#if subCount > 0}
                          <span class="flex items-center gap-1" title="{subCount} subtasks">
                            <FaTurnDown class="h-3 w-3" />{subCount}
                          </span>
                        {/if}
                        {#if task.assignee}
                          <span class="flex items-center gap-1">
                            <FaUser class="h-3 w-3" />
                            <span class="max-w-[8rem] truncate">{task.assignee}</span>
                          </span>
                        {/if}
                        {#if task.estimatedTime}
                          <span class="flex items-center gap-1">
                            <FaClock class="h-3 w-3" />{formatMinutes(task.estimatedTime)}
                          </span>
                        {/if}
                      </div>

                      <div class="flex shrink-0 items-center gap-1">
                        {#if showSubtaskActions && !task.parentId}
                          <Button
                            onclick={(e) => {
                              e.stopPropagation();
                              handleCreateSubtask(task);
                            }}
                            variant="ghost"
                            size="sm"
                            class="h-8 w-8 p-0"
                            title="Create subtask"
                            aria-label="Create subtask"
                          >
                            <FaPlus class="h-4 w-4 text-muted-foreground" />
                          </Button>
                        {/if}
                        <Button
                          onclick={(e) => {
                            e.stopPropagation();
                            handleTaskStatusToggle(task.id);
                          }}
                          variant="ghost"
                          size="sm"
                          class="h-8 w-8 p-0"
                          aria-label="Toggle task status"
                          title="Toggle task status"
                        >
                          <Icon
                            icon={display.icon(task)}
                            class="h-4 w-4 {display.statusColor(task.status)}"
                          />
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              {/snippet}
            </ContextMenuList>

            <!-- Subtasks in Kanban View -->
            {#each getTaskSubtasks(task.id) as subtask (subtask.id)}
              <ContextMenuList
                items={buildTaskMenu(subtask, {
                  onOpen: handleTaskSelect,
                  onSetStatus,
                  onCopyReference,
                  onLaunchCoder,
                  onEdit: onEditTask,
                  onDelete: onDeleteTask,
                })}
              >
                {#snippet trigger(menuProps)}
                  <Card
                    {...menuProps}
                    onclick={() => handleTaskSelect(subtask)}
                    onkeydown={(e: KeyboardEvent) => {
                      if (e.target !== e.currentTarget) return;
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        handleTaskSelect(subtask);
                      }
                    }}
                    role="group"
                    tabindex={0}
                    aria-label="Subtask: {subtask.title}"
                    borderAccent="left"
                    class="mt-2 ml-4 cursor-pointer gap-0 py-0 hover:shadow-md"
                  >
                    <CardContent class="space-y-2 p-3">
                      <div class="flex items-start gap-2">
                        <FaTurnDown class="mt-1 h-3 w-3 shrink-0 text-primary" />
                        <h4
                          class="line-clamp-2 min-w-0 flex-1 text-sm font-medium break-words text-foreground"
                        >
                          {subtask.title}
                        </h4>
                        <span
                          class="shrink-0 pt-0.5"
                          title="{subtask.priority} priority"
                          aria-label="{subtask.priority} priority"
                        >
                          <FaFlag class="h-3 w-3 {display.priorityColor(subtask.priority)}" />
                        </span>
                      </div>

                      {#if subtask.description}
                        <p class="line-clamp-2 text-xs text-muted-foreground">
                          {subtask.description}
                        </p>
                      {/if}

                      <div class="flex items-center justify-between gap-2">
                        <Badge class={display.statusBadge(subtask.status)}>
                          {taskStatusLabel(subtask.status)}
                        </Badge>
                        <Button
                          onclick={(e) => {
                            e.stopPropagation();
                            handleTaskStatusToggle(subtask.id);
                          }}
                          variant="ghost"
                          size="sm"
                          class="h-8 w-8 shrink-0 p-0"
                          aria-label="Toggle subtask status"
                          title="Toggle subtask status"
                        >
                          <Icon
                            icon={display.icon(subtask)}
                            class="h-4 w-4 {display.statusColor(subtask.status)}"
                          />
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                {/snippet}
              </ContextMenuList>
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
