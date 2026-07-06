<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Icon from "@iconify/svelte";
  import { taskUi } from "../state/taskUi.svelte";
  import type { Task } from "../types";

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

  function getTaskSubtasks(taskId: string): Task[] {
    return taskUi.tasks.filter((task) => task.parentId === taskId);
  }
</script>

<Card>
  <CardContent class="p-0">
    <div class="divide-y divide-border">
      {#each taskUi.filteredTasks as task}
        <div
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
          class="cursor-pointer p-3 transition-colors hover:bg-muted/50 {taskUi.selectedTaskIds.has(
            task.id,
          )
            ? 'bg-warning-50 dark:bg-warning-900/20'
            : ''}"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              {#if taskUi.isMultiSelectMode}
                <Checkbox
                  checked={taskUi.selectedTaskIds.has(task.id)}
                  onCheckedChange={() => handleTaskSelection(task.id)}
                  onclick={(e) => e.stopPropagation()}
                />
              {/if}
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
                    class="h-5 w-5 {getTaskStatusColor(task.status)}"
                  />
                </Button>
              </div>

              <div>
                <div class="flex items-center gap-2">
                  <h4 class="font-medium text-foreground">{task.title}</h4>
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
                </div>
                {#if task.description}
                  <p class="mt-1 text-sm text-muted-foreground">
                    {task.description}
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
                <Icon
                  icon="mdi:flag"
                  class="h-4 w-4 {getPriorityColor(task.priority)}"
                />
                <Badge variant="outline" class="text-xs uppercase">
                  {task.priority}
                </Badge>
              </div>

              <Badge class="text-sm {getStatusBadgeColor(task.status)}">
                {task.status}
              </Badge>
            </div>
          </div>

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
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      e.stopPropagation();
                      handleTaskSelect(subtask);
                    }
                  }}
                  role="button"
                  tabindex="0"
                  aria-label="Select subtask: {subtask.title}"
                  class="cursor-pointer border-l-4 border-l-primary bg-primary/5 p-3 transition-colors hover:bg-muted/50 dark:bg-primary/10"
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
                          icon={getTaskIcon(subtask)}
                          class="h-5 w-5 {getTaskStatusColor(subtask.status)}"
                        />
                      </Button>

                      <div>
                        <div class="flex items-center gap-2">
                          <Icon
                            icon="mdi:subdirectory-arrow-right"
                            class="h-3 w-3 flex-shrink-0 text-primary"
                          />
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
                        <Icon
                          icon="mdi:flag"
                          class="h-4 w-4 {getPriorityColor(subtask.priority)}"
                        />
                        <Badge variant="outline" class="text-xs uppercase">
                          {subtask.priority}
                        </Badge>
                      </div>

                      <Badge
                        class="text-sm {getStatusBadgeColor(subtask.status)}"
                      >
                        {subtask.status}
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
