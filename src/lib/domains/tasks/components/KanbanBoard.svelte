<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Icon from "@iconify/svelte";
  import { taskActions } from "../stores/taskStore";
  import { taskUi } from "../state/taskUi.svelte";
  import {
    createDragDropState,
    handleDragStart,
    handleDragEnd,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    getDropZoneClasses,
    getTaskCardClasses,
  } from "../utils/dragDrop";
  import type { Task } from "../types";
  import TaskCard from "./TaskCard.svelte";

  interface Props {
    handleTaskSelect: (task: Task) => void;
    handleTaskStatusToggle: (taskId: string) => void;
    handleTaskSelection: (taskId: string) => void;
    handleCreateSubtask: (task: Task) => void;
    getSubtaskCount: (taskId: string, allTasks: Task[]) => number;
    getTaskStatusColor: (status: string) => string;
    getStatusBadgeColor: (status: string) => string;
    getPriorityColor: (priority: string) => string;
    getTaskIcon: (task: Task) => string;
  }

  let {
    handleTaskSelect,
    handleTaskStatusToggle,
    handleTaskSelection,
    handleCreateSubtask,
    getSubtaskCount,
    getTaskStatusColor,
    getStatusBadgeColor,
    getPriorityColor,
    getTaskIcon,
  }: Props = $props();

  // Drag and drop state
  let dragDropState = $state(createDragDropState());

  function getTaskSubtasks(taskId: string): Task[] {
    return taskUi.tasks.filter((task) => task.parentId === taskId);
  }

  async function handleTaskMove(taskId: string, newStatus: string) {
    try {
      await taskActions.updateTask(taskId, { status: newStatus as any });
    } catch (error) {
      console.error("Failed to move task:", error);
    }
  }
</script>

<div class="grid grid-cols-1 gap-4 md:grid-cols-4">
  {#each taskUi.kanbanColumns as column}
    <div
      class="space-y-3 {getDropZoneClasses(
        column.id,
        dragDropState,
        'min-h-[200px] rounded-lg border-2 border-dashed border-transparent p-2',
      )}"
      role="region"
      aria-label="Drop zone for {column.title} tasks"
      ondragover={(e) => handleDragOver(e, column.id, dragDropState)}
      ondragleave={(e) => handleDragLeave(e, dragDropState)}
      ondrop={(e) => handleDrop(e, column.id, dragDropState, handleTaskMove)}
    >
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-semibold text-foreground">{column.title}</h3>
        <span
          class="rounded-full bg-muted px-2 py-1 text-xs text-muted-foreground"
        >
          {column.tasks.length}
        </span>
      </div>

      <div class="space-y-2">
        {#each column.tasks as task}
          <div
            draggable="true"
            ondragstart={(e) => handleDragStart(e, task, dragDropState)}
            ondragend={(e) => handleDragEnd(e, dragDropState)}
            onclick={() =>
              taskUi.isMultiSelectMode
                ? handleTaskSelection(task.id)
                : handleTaskSelect(task)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                taskUi.isMultiSelectMode
                  ? handleTaskSelection(task.id)
                  : handleTaskSelect(task);
              }
            }}
            role="button"
            tabindex="0"
            aria-label="Select task: {task.title}"
            class="cursor-pointer transition-shadow hover:shadow-md {taskUi.selectedTaskIds.has(
              task.id,
            )
              ? 'bg-warning-50 ring-2 ring-warning-500 dark:bg-warning-900/20'
              : ''} {getTaskCardClasses(task, dragDropState)}"
          >
            <Card>
              <CardContent class="p-3">
                <div class="mb-1 flex items-start justify-between">
                  {#if taskUi.isMultiSelectMode}
                    <Checkbox
                      checked={taskUi.selectedTaskIds.has(task.id)}
                      onCheckedChange={() => handleTaskSelection(task.id)}
                      onclick={(e) => e.stopPropagation()}
                      class="mt-1"
                    />
                  {/if}
                  <h4 class="text-sm font-medium text-foreground">
                    {task.title}
                  </h4>
                  <div class="flex items-center gap-1">
                    {#if getSubtaskCount(task.id, taskUi.tasks) > 0}
                      <div
                        class="flex items-center gap-1 text-xs text-muted-foreground"
                      >
                        <Icon
                          icon="mdi:subdirectory-arrow-right"
                          class="h-3 w-3"
                        />
                        <span>{getSubtaskCount(task.id, taskUi.tasks)}</span>
                      </div>
                    {/if}
                    <Icon
                      icon="mdi:flag"
                      class="h-3 w-3 {getPriorityColor(task.priority)}"
                    />
                    <span class="text-xs uppercase text-muted-foreground"
                      >{task.priority}</span
                    >
                  </div>
                </div>

                {#if task.description}
                  <p class="mb-3 line-clamp-2 text-xs text-muted-foreground">
                    {task.description}
                  </p>
                {/if}

                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    {#if task.type}
                      <Badge variant="outline" class="text-xs">
                        {task.type}
                      </Badge>
                    {/if}
                  </div>

                  <div class="flex items-center gap-1">
                    {#if !task.parentId}
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
                        <Icon
                          icon="mdi:plus"
                          class="h-4 w-4 text-muted-foreground"
                        />
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
                        icon={getTaskIcon(task)}
                        class="h-4 w-4 {getTaskStatusColor(task.status)}"
                      />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>

            <!-- Subtasks in Kanban View -->
            {#if getTaskSubtasks(task.id).length > 0}
              <div class="ml-4 mt-2 space-y-2">
                {#each getTaskSubtasks(task.id) as subtask}
                  <div
                    onclick={(e) => {
                      e.stopPropagation();
                      handleTaskSelect(subtask);
                    }}
                    onkeydown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        e.stopPropagation();
                        handleTaskSelect(subtask);
                      }
                    }}
                    role="button"
                    tabindex="0"
                    aria-label="Select subtask: {subtask.title}"
                    class="cursor-pointer transition-shadow hover:shadow-md"
                  >
                    <Card
                      class="border-l-4 border-l-primary bg-primary/5 dark:bg-primary/10"
                    >
                      <CardContent class="p-3">
                        <div class="mb-2 flex items-start justify-between">
                          <div class="flex items-center gap-2">
                            <Icon
                              icon="mdi:subdirectory-arrow-right"
                              class="h-3 w-3 flex-shrink-0 text-primary"
                            />
                            <h4 class="text-sm font-medium text-foreground">
                              {subtask.title}
                            </h4>
                          </div>
                          <div class="flex items-center gap-1">
                            <Icon
                              icon="mdi:flag"
                              class="h-3 w-3 {getPriorityColor(
                                subtask.priority,
                              )}"
                            />
                            <span
                              class="text-xs uppercase text-muted-foreground"
                              >{subtask.priority}</span
                            >
                          </div>
                        </div>

                        {#if subtask.description}
                          <p
                            class="mb-2 line-clamp-2 text-xs text-muted-foreground"
                          >
                            {subtask.description}
                          </p>
                        {/if}

                        <div class="flex items-center justify-between">
                          <div class="flex items-center gap-2">
                            <Badge
                              class="text-xs {getStatusBadgeColor(
                                subtask.status,
                              )}"
                            >
                              {subtask.status}
                            </Badge>
                          </div>

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
                              icon={getTaskIcon(subtask)}
                              class="h-4 w-4 {getTaskStatusColor(
                                subtask.status,
                              )}"
                            />
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/each}
</div>
