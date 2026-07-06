<script lang="ts">
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Select from "$lib/components/ui/select.svelte";
  import Icon from "@iconify/svelte";
  import { taskActions } from "../stores/taskStore";
  import { taskUi } from "../state/taskUi.svelte";
  import type { TaskStatus, TaskPriority } from "../types";

  let statusFilters = $state<TaskStatus[]>([]);
  let priorityFilters = $state<TaskPriority[]>([]);
  let typeFilters = $state<string[]>([]);

  // Update filters when store changes
  $effect(() => {
    statusFilters = taskUi.taskFilters.status || [];
    priorityFilters = taskUi.taskFilters.priority || [];
    typeFilters = taskUi.taskFilters.type || [];
  });

  function toggleStatusFilter(status: TaskStatus) {
    const newFilters = statusFilters.includes(status)
      ? statusFilters.filter((s) => s !== status)
      : [...statusFilters, status];

    statusFilters = newFilters;
    taskActions.setFilters({
      status: newFilters.length > 0 ? newFilters : undefined,
    });
  }

  function togglePriorityFilter(priority: TaskPriority) {
    const newFilters = priorityFilters.includes(priority)
      ? priorityFilters.filter((p) => p !== priority)
      : [...priorityFilters, priority];

    priorityFilters = newFilters;
    taskActions.setFilters({
      priority: newFilters.length > 0 ? newFilters : undefined,
    });
  }

  function toggleTypeFilter(type: string) {
    const newFilters = typeFilters.includes(type)
      ? typeFilters.filter((t) => t !== type)
      : [...typeFilters, type];

    typeFilters = newFilters;
    taskActions.setFilters({
      type: newFilters.length > 0 ? newFilters : undefined,
    });
  }

  function clearAllFilters() {
    statusFilters = [];
    priorityFilters = [];
    typeFilters = [];
    taskActions.clearFilters();
  }

  const hasActiveFilters = $derived(
    statusFilters.length > 0 ||
      priorityFilters.length > 0 ||
      typeFilters.length > 0,
  );
</script>

<Card class="mb-4">
  <CardContent class="p-3">
    <div class="mb-3 flex items-center justify-between">
      <h3 class="text-xs font-medium text-foreground">Filters</h3>
      {#if hasActiveFilters}
        <Button
          onclick={clearAllFilters}
          variant="ghost"
          size="sm"
          class="text-xs"
        >
          <Icon icon="mdi:close" class="mr-1 h-3 w-3" />
          Clear All
        </Button>
      {/if}
    </div>

    <div class="space-y-3">
      <!-- Status Filters -->
      <div>
        <fieldset>
          <legend class="mb-1 block text-xs font-medium text-muted-foreground"
            >Status</legend
          >
          <div class="flex flex-wrap gap-2">
            {#each ["pending", "in-progress", "completed", "cancelled"] as status}
              <Button
                onclick={() => toggleStatusFilter(status as TaskStatus)}
                variant={statusFilters.includes(status as TaskStatus)
                  ? "default"
                  : "outline"}
                size="sm"
                class="text-xs"
              >
                {status === "pending"
                  ? "To Do"
                  : status === "in-progress"
                    ? "In Progress"
                    : status === "completed"
                      ? "Completed"
                      : "Cancelled"}
              </Button>
            {/each}
          </div>
        </fieldset>
      </div>

      <!-- Priority Filters -->
      <div>
        <fieldset>
          <legend class="mb-1 block text-xs font-medium text-muted-foreground"
            >Priority</legend
          >
          <div class="flex flex-wrap gap-2">
            {#each ["low", "medium", "high"] as priority}
              <Button
                onclick={() => togglePriorityFilter(priority as TaskPriority)}
                variant={priorityFilters.includes(priority as TaskPriority)
                  ? "default"
                  : "outline"}
                size="sm"
                class="text-xs"
              >
                <Icon
                  icon="mdi:flag"
                  class="mr-1 h-3 w-3 {priority === 'high'
                    ? 'text-red-500'
                    : priority === 'medium'
                      ? 'text-yellow-500'
                      : 'text-green-500'}"
                />
                {priority.charAt(0).toUpperCase() + priority.slice(1)}
              </Button>
            {/each}
          </div>
        </fieldset>
      </div>

      <!-- Type Filters -->
      <div>
        <div>
          <fieldset>
            <legend class="mb-1 block text-xs font-medium text-muted-foreground"
              >Types</legend
            >
            <div class="flex flex-wrap gap-2">
              {#each ["Story", "Bug", "Feature", "Note", "Task", "Epic"] as type}
                <Button
                  onclick={() => toggleTypeFilter(type)}
                  variant={typeFilters.includes(type) ? "default" : "outline"}
                  size="sm"
                  class="text-xs"
                >
                  {type}
                </Button>
              {/each}
            </div>
          </fieldset>
        </div>
      </div>

      <!-- Active Filters Summary -->
      {#if hasActiveFilters}
        <div class="mt-4 border-t border-border pt-4">
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-xs text-muted-foreground">Active filters:</span>
            {#each statusFilters as status}
              <Badge variant="secondary" class="text-xs">
                Status: {status}
              </Badge>
            {/each}
            {#each priorityFilters as priority}
              <Badge variant="secondary" class="text-xs">
                Priority: {priority}
              </Badge>
            {/each}
            {#each typeFilters as type}
              <Badge variant="secondary" class="text-xs">
                Type: {type}
              </Badge>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </CardContent>
</Card>
