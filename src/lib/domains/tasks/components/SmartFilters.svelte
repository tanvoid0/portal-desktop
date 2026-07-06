<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { taskUi } from "../state/taskUi.svelte";
  import type { TaskFilters } from "../types";
  import Icon from "@iconify/svelte";

  interface Props {
    onFilterChange?: (filters: TaskFilters) => void;
  }

  let { onFilterChange }: Props = $props();

  const smartFilters = $derived([
    {
      id: "overdue",
      name: "Overdue Tasks",
      description: "Tasks past their due date",
      icon: "mdi:alert-circle",
      color: "text-red-600",
      count: taskUi.overdueTasks.length,
      filter: { status: "pending", dueDate: { before: new Date() } },
    },
    {
      id: "due-today",
      name: "Due Today",
      description: "Tasks due today",
      icon: "mdi:calendar-today",
      color: "text-yellow-600",
      count: taskUi.dueTodayTasks.length,
      filter: { status: "pending", dueDate: { today: true } },
    },
    {
      id: "blocked",
      name: "Blocked Tasks",
      description: "Tasks that are blocked by dependencies",
      icon: "mdi:lock",
      color: "text-orange-600",
      count: taskUi.blockedTasks.length,
      filter: { status: "pending", hasBlockers: true },
    },
    {
      id: "unestimated",
      name: "Unestimated",
      description: "Tasks without time estimates",
      icon: "mdi:clock-alert",
      color: "text-blue-600",
      count: taskUi.unestimatedTasks.length,
      filter: { status: "pending", estimatedTime: null },
    },
    {
      id: "high-priority",
      name: "High Priority",
      description: "High priority tasks",
      icon: "mdi:flag",
      color: "text-red-600",
      count: 0,
      filter: { status: "pending", priority: "high" },
    },
    {
      id: "in-progress",
      name: "In Progress",
      description: "Tasks currently being worked on",
      icon: "mdi:play-circle",
      color: "text-green-600",
      count: 0,
      filter: { status: "in_progress" },
    },
  ]);

  function applyFilter(filter: (typeof smartFilters)[number]) {
    onFilterChange?.(filter.filter as TaskFilters);
  }

  function clearFilters() {
    onFilterChange?.({});
  }
</script>

<Card class="w-full">
  <CardHeader>
    <div class="flex items-center justify-between">
      <CardTitle class="text-lg">Smart Filters</CardTitle>
      <Button variant="ghost" size="sm" onclick={clearFilters}>
        <Icon icon="mdi:close" class="mr-1 h-4 w-4" />
        Clear
      </Button>
    </div>
  </CardHeader>
  <CardContent class="p-4 pt-0">
    <div class="space-y-3">
      {#each smartFilters as filter}
        <Button
          variant="ghost"
          class="h-auto w-full justify-start p-3"
          onclick={() => applyFilter(filter)}
        >
          <div class="flex w-full items-center">
            <Icon icon={filter.icon} class="mr-3 h-5 w-5 {filter.color}" />
            <div class="flex-1 text-left">
              <div class="font-medium">{filter.name}</div>
              <div class="text-xs text-muted-foreground">
                {filter.description}
              </div>
            </div>
            {#if filter.count > 0}
              <Badge variant="secondary" class="ml-2">
                {filter.count}
              </Badge>
            {/if}
          </div>
        </Button>
      {/each}
    </div>
  </CardContent>
</Card>
