<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import Select from "$lib/components/ui/select.svelte";
  import Icon from "@iconify/svelte";
  import type { TaskStatus, TaskPriority } from "../types";

  interface Props {
    open: boolean;
    searchQuery: string;
    statusFilters: TaskStatus[];
    priorityFilters: TaskPriority[];
    typeFilters: string[];
    onSearchChange: (value: string) => void;
    onStatusFilterChange: (filters: TaskStatus[]) => void;
    onPriorityFilterChange: (filters: TaskPriority[]) => void;
    onTypeFilterChange: (filters: string[]) => void;
    onClearAll: () => void;
  }

  let {
    open = $bindable(false),
    searchQuery,
    statusFilters,
    priorityFilters,
    typeFilters,
    onSearchChange,
    onStatusFilterChange,
    onPriorityFilterChange,
    onTypeFilterChange,
    onClearAll,
  }: Props = $props();

  let localSearchQuery = $state(searchQuery);
  let localStatusFilters = $state([...statusFilters]);
  let localPriorityFilters = $state([...priorityFilters]);
  let localTypeFilters = $state([...typeFilters]);

  // Available filter options
  const statusOptions: { value: TaskStatus; label: string; icon: string }[] = [
    { value: "pending", label: "To Do", icon: "mdi:clock-outline" },
    { value: "in-progress", label: "In Progress", icon: "mdi:progress-clock" },
    { value: "completed", label: "Completed", icon: "mdi:check-circle" },
    { value: "cancelled", label: "Cancelled", icon: "mdi:cancel" },
  ];

  const priorityOptions: {
    value: TaskPriority;
    label: string;
    icon: string;
    color: string;
  }[] = [
    { value: "low", label: "Low", icon: "mdi:flag", color: "text-green-500" },
    {
      value: "medium",
      label: "Medium",
      icon: "mdi:flag",
      color: "text-yellow-500",
    },
    { value: "high", label: "High", icon: "mdi:flag", color: "text-red-500" },
  ];

  const typeOptions: { value: string; label: string }[] = [
    { value: "Story", label: "Story" },
    { value: "Bug", label: "Bug" },
    { value: "Feature", label: "Feature" },
    { value: "Note", label: "Note" },
    { value: "Task", label: "Task" },
    { value: "Epic", label: "Epic" },
  ];

  function toggleStatusFilter(status: TaskStatus) {
    if (localStatusFilters.includes(status)) {
      localStatusFilters = localStatusFilters.filter((s) => s !== status);
    } else {
      localStatusFilters = [...localStatusFilters, status];
    }
  }

  function togglePriorityFilter(priority: TaskPriority) {
    if (localPriorityFilters.includes(priority)) {
      localPriorityFilters = localPriorityFilters.filter((p) => p !== priority);
    } else {
      localPriorityFilters = [...localPriorityFilters, priority];
    }
  }

  function toggleTypeFilter(type: string) {
    if (localTypeFilters.includes(type)) {
      localTypeFilters = localTypeFilters.filter((t) => t !== type);
    } else {
      localTypeFilters = [...localTypeFilters, type];
    }
  }

  // Remove the old group filter function

  function applyFilters() {
    onSearchChange(localSearchQuery);
    onStatusFilterChange(localStatusFilters);
    onPriorityFilterChange(localPriorityFilters);
    onTypeFilterChange(localTypeFilters);
    open = false;
  }

  function clearAllFilters() {
    localSearchQuery = "";
    localStatusFilters = [];
    localPriorityFilters = [];
    localTypeFilters = [];
    onClearAll();
    open = false;
  }

  function handleClose() {
    // Reset local state to current props
    localSearchQuery = searchQuery;
    localStatusFilters = [...statusFilters];
    localPriorityFilters = [...priorityFilters];
    localTypeFilters = [...typeFilters];
    open = false;
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[80vh] max-w-2xl overflow-hidden">
    <Dialog.Header class="pb-3">
      <Dialog.Title class="text-lg">Advanced Filters</Dialog.Title>
    </Dialog.Header>

    <div class="space-y-6 overflow-y-auto px-6 pb-6">
        <!-- Search -->
        <div>
          <p class="mb-2 block text-sm font-medium text-foreground">Search</p>
          <div class="relative">
            <Icon
              icon="mdi:magnify"
              class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
            />
            <Input
              type="text"
              placeholder="Search tasks..."
              bind:value={localSearchQuery}
              class="pl-10"
            />
          </div>
        </div>

        <!-- Status Filters -->
        <div>
          <p class="mb-3 block text-sm font-medium text-foreground">Status</p>
          <div class="grid grid-cols-2 gap-2">
            {#each statusOptions as option}
              <label
                class="flex cursor-pointer items-center space-x-2 rounded-md p-2 hover:bg-muted/50"
              >
                <Checkbox
                  checked={localStatusFilters.includes(option.value)}
                  onCheckedChange={() => toggleStatusFilter(option.value)}
                />
                <Icon
                  icon={option.icon}
                  class="h-4 w-4 text-muted-foreground"
                />
                <span class="text-sm">{option.label}</span>
              </label>
            {/each}
          </div>
        </div>

        <!-- Priority Filters -->
        <div>
          <p class="mb-3 block text-sm font-medium text-foreground">Priority</p>
          <div class="grid grid-cols-3 gap-2">
            {#each priorityOptions as option}
              <label
                class="flex cursor-pointer items-center space-x-2 rounded-md p-2 hover:bg-muted/50"
              >
                <Checkbox
                  checked={localPriorityFilters.includes(option.value)}
                  onCheckedChange={() => togglePriorityFilter(option.value)}
                />
                <Icon icon={option.icon} class="h-4 w-4 {option.color}" />
                <span class="text-sm">{option.label}</span>
              </label>
            {/each}
          </div>
        </div>

        <!-- Type Filters -->
        <div>
          <p class="mb-3 block text-sm font-medium text-foreground">Types</p>
          <div class="grid grid-cols-2 gap-2">
            {#each typeOptions as typeOption}
              <label
                class="flex cursor-pointer items-center space-x-2 rounded-md p-2 hover:bg-muted/50"
              >
                <Checkbox
                  checked={localTypeFilters.includes(typeOption.value)}
                  onCheckedChange={() => toggleTypeFilter(typeOption.value)}
                />
                <span class="text-sm">{typeOption.label}</span>
              </label>
            {/each}
          </div>
        </div>

        <!-- Action Buttons -->
        <div
          class="flex items-center justify-between border-t border-border pt-4"
        >
          <Button
            variant="outline"
            onclick={clearAllFilters}
            class="text-red-600 hover:text-red-700"
          >
            <Icon icon="mdi:filter-remove" class="mr-2 h-4 w-4" />
            Clear All
          </Button>

          <div class="flex items-center gap-2">
            <Button variant="outline" onclick={handleClose}>Cancel</Button>
            <Button onclick={applyFilters}>Apply Filters</Button>
          </div>
        </div>
    </div>
  </Dialog.Content>
</Dialog.Root>
