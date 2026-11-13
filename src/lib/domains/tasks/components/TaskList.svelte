<script lang="ts">
  import FaPlus from '~icons/fa6-solid/plus';
  import FaTurnDown from '~icons/fa6-solid/turn-down';
  import FaFlag from '~icons/fa6-solid/flag';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Icon from '@iconify/svelte';
  import {
    taskUi as defaultTaskUi,
    getSubtaskCount,
    type TaskUiState,
  } from '../state/taskUi.svelte';
  import { defaultDisplay, type TaskDisplay } from '../utils/taskDisplay';
  import { taskStatusLabel, type Task, type TaskStatus } from '../types';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import { buildTaskMenu } from '../utils/taskMenu';

  interface Props {
    ui?: TaskUiState;
    handleTaskSelect: (task: Task) => void;
    handleTaskStatusToggle: (taskId: string) => void;
    handleTaskSelection: (taskId: string) => void;
    handleCreateSubtask: (task: Task) => void;
    /** Right-click menu actions — omit to hide the matching item. */
    onSetStatus?: (taskId: string, status: TaskStatus) => void;
    onCopyReference?: (task: Task) => void;
    onLaunchCoder?: (task: Task) => void;
    onEditTask?: (task: Task) => void;
    onDeleteTask?: (task: Task) => void;
    /** Project name shown next to the title (All-projects mode). */
    projectLabel?: (task: Task) => string | undefined;
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
    onSetStatus,
    onCopyReference,
    onLaunchCoder,
    onEditTask,
    onDeleteTask,
    projectLabel,
    display = defaultDisplay,
    showSubtaskActions = true,
  }: Props = $props();
  let taskUi = $derived(ui);

  function getTaskSubtasks(taskId: string): Task[] {
    return taskUi.tasks.filter((task) => task.parentId === taskId);
  }
</script>

<Card>
  <CardContent class="p-0">
    <div class="divide-y divide-border">
      {#each taskUi.boardTasks as task (task.id)}
        <div
          onclick={() =>
            taskUi.isMultiSelectMode ? handleTaskSelection(task.id) : handleTaskSelect(task)}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              taskUi.isMultiSelectMode ? handleTaskSelection(task.id) : handleTaskSelect(task);
            }
          }}
          role="group"
          tabindex="0"
          aria-label="Task: {task.title}"
          aria-current={taskUi.selectedTaskIds.has(task.id) ? 'true' : undefined}
          class="cursor-pointer p-3 transition-colors hover:bg-muted/50 {taskUi.selectedTaskIds.has(
            task.id
          )
            ? 'bg-warning-50 dark:bg-warning-900/20'
            : ''}"
        >
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
              <div {...menuProps} class="flex min-w-0 items-center justify-between">
                <!-- min-w-0 all the way down: the title's own truncate does nothing
                     unless every flex ancestor is allowed to shrink below content. -->
                <div class="flex min-w-0 items-center gap-3">
                  {#if taskUi.isMultiSelectMode}
                    <Checkbox
                      checked={taskUi.selectedTaskIds.has(task.id)}
                      onCheckedChange={() => handleTaskSelection(task.id)}
                      onclick={(e) => e.stopPropagation()}
                    />
                  {/if}
                  <div class="flex items-center gap-1">
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
                    >
                      <Icon
                        icon={display.icon(task)}
                        class="h-5 w-5 {display.statusColor(task.status)}"
                      />
                    </Button>
                  </div>

                  <div class="min-w-0">
                    <div class="flex min-w-0 items-center gap-2">
                      <!-- A title with no spaces has no break opportunity: without
                       min-w-0 + break-words it runs past the row and shoves the
                       badges out of view. -->
                      <h4 class="min-w-0 flex-1 truncate font-medium break-words text-foreground">
                        {task.title}
                      </h4>
                      {#if projectLabel?.(task)}
                        <Badge variant="outline" class="max-w-[10rem] truncate text-xs">
                          {projectLabel(task)}
                        </Badge>
                      {/if}
                      {#if getSubtaskCount(task.id, taskUi.tasks) > 0}
                        <div class="flex items-center gap-1 text-xs text-muted-foreground">
                          <FaTurnDown class="h-3 w-3" />
                          <span>{getSubtaskCount(task.id, taskUi.tasks)}</span>
                        </div>
                      {/if}
                    </div>
                    {#if task.description}
                      <p class="mt-1 text-sm text-muted-foreground">
                        {task.description}
                      </p>
                    {/if}
                    {#if task.tags?.length}
                      <p class="mt-1 text-xs text-muted-foreground">
                        {task.tags.map((t) => '#' + t).join(' ')}
                      </p>
                    {/if}
                  </div>
                </div>

                <div class="flex items-center gap-3">
                  {#if task.type}
                    <Badge variant="outline" class="text-xs">
                      {task.type}
                    </Badge>
                  {/if}

                  <div class="flex items-center gap-1">
                    <FaFlag class="h-4 w-4 {display.priorityColor(task.priority)}" />
                    <Badge variant="outline" class="text-xs uppercase">
                      {task.priority}
                    </Badge>
                  </div>

                  <Badge class="text-sm {display.statusBadge(task.status)}">
                    {taskStatusLabel(task.status)}
                  </Badge>
                </div>
              </div>
            {/snippet}
          </ContextMenuList>

          <!-- Subtasks in List View -->
          {#if getTaskSubtasks(task.id).length > 0}
            <div class="ml-6 space-y-1">
              {#each getTaskSubtasks(task.id) as subtask}
                <div
                  onclick={(e) => {
                    e.stopPropagation();
                    handleTaskSelect(subtask);
                  }}
                  onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      e.stopPropagation();
                      handleTaskSelect(subtask);
                    }
                  }}
                  role="group"
                  tabindex="0"
                  aria-label="Subtask: {subtask.title}"
                  class="cursor-pointer border-l-2 border-l-primary p-3 transition-colors hover:bg-muted/50"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <Button
                        onclick={(e) => {
                          e.stopPropagation();
                          handleTaskStatusToggle(subtask.id);
                        }}
                        variant="ghost"
                        size="sm"
                        class="h-8 w-8 p-0"
                      >
                        <Icon
                          icon={display.icon(subtask)}
                          class="h-5 w-5 {display.statusColor(subtask.status)}"
                        />
                      </Button>

                      <div>
                        <div class="flex items-center gap-2">
                          <FaTurnDown class="h-3 w-3 flex-shrink-0 text-primary" />
                          <h4 class="font-medium text-foreground">
                            {subtask.title}
                          </h4>
                        </div>
                        {#if subtask.description}
                          <p class="mt-1 text-sm text-muted-foreground">
                            {subtask.description}
                          </p>
                        {/if}
                      </div>
                    </div>

                    <div class="flex items-center gap-3">
                      {#if subtask.type}
                        <Badge variant="outline" class="text-xs">
                          {subtask.type}
                        </Badge>
                      {/if}

                      <div class="flex items-center gap-1">
                        <FaFlag class="h-4 w-4 {display.priorityColor(subtask.priority)}" />
                        <Badge variant="outline" class="text-xs uppercase">
                          {subtask.priority}
                        </Badge>
                      </div>

                      <Badge class="text-sm {display.statusBadge(subtask.status)}">
                        {taskStatusLabel(subtask.status)}
                      </Badge>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </CardContent>
</Card>
