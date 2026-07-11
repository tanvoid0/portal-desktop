<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Separator } from "$lib/components/ui/separator";
  import { Input } from "$lib/components/ui/input";
  import Icon from "@iconify/svelte";
  import { goto } from "$app/navigation";
  import {
    taskActions,
    isLoading,
    error,
  } from "../stores/taskStore";
  import { taskUi, getSubtaskCount } from "../state/taskUi.svelte";
  import { createTasksQuery } from "../queries/taskQueries";
  import { toastActions } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";
  import LoadingSpinner from "$lib/components/ui/loading-spinner.svelte";
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
  } from "$lib/components/shell";
  import type { Task, TaskStatus, TaskPriority } from "../types";
  import TaskCard from "./TaskCard.svelte";
  import TaskStats from "./TaskStats.svelte";
  import TaskProgress from "./TaskProgress.svelte";
  import TaskFilterModal from "./TaskFilterModal.svelte";
  import KanbanBoard from "./KanbanBoard.svelte";
  import TaskList from "./TaskList.svelte";
  import QuickActions from "./QuickActions.svelte";
  import SmartFilters from "./SmartFilters.svelte";
  import SavedViews from "./SavedViews.svelte";
  import TimeTracker from "./TimeTracker.svelte";
  import TemplateManager from "./TemplateManager.svelte";

  const tasksQuery = createTasksQuery();

  $effect(() => {
    if (tasksQuery.data) {
      taskUi.setTasks(tasksQuery.data);
    }
  });

  $effect(() => {
    isLoading.set(tasksQuery.isPending);
    if (tasksQuery.isError) {
      error.set(
        tasksQuery.error instanceof Error
          ? tasksQuery.error.message
          : "Failed to load tasks",
      );
    } else if (tasksQuery.isSuccess) {
      error.set(null);
    }
  });

  // View state
  let currentView = $state<"kanban" | "list">("kanban");
  let searchQuery = $state("");
  let showSidebar = $state(true);
  let sidebarTab = $state<
    "actions" | "filters" | "views" | "templates" | "tracker"
  >("actions");

  // Filter modal state
  let showFilterModal = $state(false);

  // Keyboard shortcuts modal state
  let showKeyboardShortcuts = $state(false);

  // Delete modal state (bulk delete uses confirmAction)
  // Filter state
  let statusFilters = $state<TaskStatus[]>([]);
  let priorityFilters = $state<TaskPriority[]>([]);
  let typeFilters = $state<string[]>([]);


  // Computed values
  let parentTasks = $derived(taskUi.parentTasks);
  let filteredTasksWithSearch = $derived(
    parentTasks.filter((task) => {
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        return (
          task.title.toLowerCase().includes(query) ||
          (task.description && task.description.toLowerCase().includes(query))
        );
      }
      return true;
    }),
  );

  // Active filter states
  let activeStatusFilters = $derived(statusFilters);
  let activePriorityFilters = $derived(priorityFilters);
  let activeTypeFilters = $derived(typeFilters);
  let hasActiveFilters = $derived(
    searchQuery.trim() !== "" ||
      statusFilters.length > 0 ||
      priorityFilters.length > 0 ||
      typeFilters.length > 0,
  );
  let activeFilterCount = $derived(
    (statusFilters.length > 0 ? 1 : 0) +
      (priorityFilters.length > 0 ? 1 : 0) +
      (typeFilters.length > 0 ? 1 : 0) +
      (searchQuery.trim() !== "" ? 1 : 0),
  );

  // Helper functions
  function getTaskStatusColor(status: string) {
    switch (status) {
      case "completed":
        return "text-green-500";
      case "in-progress":
        return "text-blue-500";
      case "cancelled":
        return "text-red-500";
      default:
        return "text-gray-400";
    }
  }

  function getStatusBadgeColor(status: string) {
    switch (status) {
      case "completed":
        return "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300";
      case "in-progress":
        return "bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300";
      case "cancelled":
        return "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300";
      default:
        return "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300";
    }
  }

  function getPriorityColor(priority: string) {
    switch (priority) {
      case "high":
        return "text-red-500";
      case "medium":
        return "text-yellow-500";
      case "low":
        return "text-green-500";
      default:
        return "text-gray-400";
    }
  }

  function getTaskIcon(task: Task) {
    switch (task.status) {
      case "completed":
        return "mdi:check-circle";
      case "in-progress":
        return "mdi:progress-clock";
      case "cancelled":
        return "mdi:cancel";
      default:
        return "mdi:circle-outline";
    }
  }

  // Event handlers
  function handleTaskSelect(task: Task) {
    taskActions.selectTask(task);
    // Add haptic feedback for better UX
    if (navigator.vibrate) {
      navigator.vibrate(50);
    }
    goto(`/tasks/${task.id}`);
  }

  function handleTaskCreate() {
    // Add smooth transition
    goto("/tasks/create");
  }

  function handleTaskEdit(task: Task) {
    goto(`/tasks/${task.id}/edit`);
  }

  // Filter functions
  function toggleStatusFilter(status: TaskStatus) {
    if (statusFilters.includes(status)) {
      statusFilters = statusFilters.filter((s) => s !== status);
    } else {
      statusFilters = [...statusFilters, status];
    }
  }

  function togglePriorityFilter(priority: TaskPriority) {
    if (priorityFilters.includes(priority)) {
      priorityFilters = priorityFilters.filter((p) => p !== priority);
    } else {
      priorityFilters = [...priorityFilters, priority];
    }
  }

  function toggleTypeFilter(type: string) {
    if (typeFilters.includes(type)) {
      typeFilters = typeFilters.filter((t) => t !== type);
    } else {
      typeFilters = [...typeFilters, type];
    }
  }

  function clearAllFilters() {
    searchQuery = "";
    statusFilters = [];
    priorityFilters = [];
    typeFilters = [];
  }

  function handleTaskStatusToggle(taskId: string) {
    taskActions.toggleTaskStatus(taskId);
  }

  function handleMultiSelectToggle() {
    taskActions.toggleMultiSelectMode();
  }

  function handleTaskSelection(taskId: string) {
    taskActions.toggleTaskSelection(taskId);
  }

  function handleCreateSubtask(parentTask: Task) {
    // Navigate to create task page with parentId pre-filled
    goto(`/tasks/create?parentId=${parentTask.id}`);
  }

  function handleSelectAll() {
    taskActions.selectAllTasks(filteredTasksWithSearch.map((task) => task.id));
  }

  function handleClearSelection() {
    taskActions.clearSelection();
  }

  async function handleBulkDelete() {
    if (taskUi.selectedTaskIds.size > 0) {
      const confirmed = await confirmAction(
        `Are you sure you want to delete ${taskUi.selectedTaskIds.size} selected task${taskUi.selectedTaskIds.size === 1 ? "" : "s"}?`,
        "Delete tasks",
      );
      if (confirmed) {
        try {
          await taskActions.deleteTasksBulk(Array.from(taskUi.selectedTaskIds));
          toastActions.success(
            "Tasks deleted",
            `Successfully deleted ${taskUi.selectedTaskIds.size} task${taskUi.selectedTaskIds.size === 1 ? "" : "s"}`,
          );
        } catch (err) {
          toastActions.error(
            "Failed to delete tasks",
            err instanceof Error ? err.message : "An unexpected error occurred",
          );
        }
      }
    }
  }

  // Keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      if (event.key === "n") {
        event.preventDefault();
        handleTaskCreate();
      } else if (event.key === "f") {
        event.preventDefault();
        showFilterModal = true;
      } else if (event.key === "k") {
        event.preventDefault();
        // Focus search input
        const searchInput = document.querySelector(
          'input[placeholder="Search tasks..."]',
        ) as HTMLInputElement;
        if (searchInput) {
          searchInput.focus();
        }
      } else if (event.key === "h") {
        event.preventDefault();
        showKeyboardShortcuts = true;
      } else if (event.key === "m") {
        event.preventDefault();
        handleMultiSelectToggle();
      } else if (event.key === "a" && taskUi.isMultiSelectMode) {
        event.preventDefault();
        handleSelectAll();
      } else if (event.key === "Delete" && taskUi.selectedTaskIds.size > 0) {
        event.preventDefault();
        handleBulkDelete();
      }
    } else if (event.key === "Escape") {
      if (showFilterModal) {
        showFilterModal = false;
      } else if (showKeyboardShortcuts) {
        showKeyboardShortcuts = false;
      } else if (taskUi.isMultiSelectMode) {
        handleMultiSelectToggle();
      }
    }
  }

</script>

<svelte:window onkeydown={handleKeydown} />

<div class="container mx-auto space-y-6 bg-background p-6">
  <PageHeader title="Tasks" description="Organize and track your work efficiently">
    {#snippet actions()}
          <!-- View Toggle -->
          <div class="flex rounded-lg bg-muted p-1">
            <Button
              onclick={() => (currentView = "kanban")}
              variant={currentView === "kanban" ? "default" : "ghost"}
              size="sm"
              class="px-3 py-1.5 text-sm font-medium"
            >
              <Icon icon="mdi:view-column" class="mr-1.5 h-4 w-4" />
              Kanban
            </Button>
            <Button
              onclick={() => (currentView = "list")}
              variant={currentView === "list" ? "default" : "ghost"}
              size="sm"
              class="px-3 py-1.5 text-sm font-medium"
            >
              <Icon icon="mdi:format-list-bulleted" class="mr-1.5 h-4 w-4" />
              List
            </Button>
          </div>

          <!-- Multi-select Controls -->
          {#if taskUi.isMultiSelectMode}
            <div
              class="flex items-center gap-3 rounded-lg border border-warning-200 bg-warning-50 px-4 py-3 dark:border-warning-800 dark:bg-warning-900/20"
            >
              <Badge
                variant="secondary"
                class="bg-warning-100 text-warning-800 dark:bg-warning-800 dark:text-warning-100"
              >
                <Icon
                  icon="mdi:checkbox-multiple-marked"
                  class="mr-1 h-3 w-3"
                />
                {taskUi.selectedTaskIds.size} selected
              </Badge>
              <div class="flex items-center gap-2">
                <Button
                  onclick={handleSelectAll}
                  variant="outline"
                  size="sm"
                  class="h-7 px-2 text-xs"
                >
                  <Icon icon="mdi:select-all" class="mr-1 h-3 w-3" />
                  Select All
                </Button>
                <Button
                  onclick={handleClearSelection}
                  variant="outline"
                  size="sm"
                  class="h-7 px-2 text-xs"
                >
                  <Icon icon="mdi:close" class="mr-1 h-3 w-3" />
                  Clear
                </Button>
                <Button
                  onclick={handleBulkDelete}
                  disabled={taskUi.selectedTaskIds.size === 0}
                  variant="destructive"
                  size="sm"
                  class="h-7 px-2 text-xs"
                >
                  <Icon icon="mdi:delete" class="mr-1 h-3 w-3" />
                  Delete ({taskUi.selectedTaskIds.size})
                </Button>
                <Button
                  onclick={handleMultiSelectToggle}
                  variant="ghost"
                  size="sm"
                  class="h-7 px-2 text-xs"
                >
                  <Icon icon="mdi:close" class="mr-1 h-3 w-3" />
                  Cancel
                </Button>
              </div>
            </div>
          {:else}
            <Button
              onclick={handleMultiSelectToggle}
              variant="outline"
              class="flex items-center space-x-2"
            >
              <Icon icon="mdi:checkbox-multiple-marked" class="h-4 w-4" />
              <span>Multi-Select</span>
            </Button>
          {/if}

          <!-- Action Buttons -->
          <Button
            onclick={() => (showKeyboardShortcuts = true)}
            variant="ghost"
            class="flex items-center space-x-2"
            title="Keyboard Shortcuts (Ctrl+H)"
          >
            <Icon icon="mdi:keyboard" class="h-4 w-4" />
            <span>Shortcuts</span>
          </Button>
          <Button
            onclick={() => goto("/tasks/generate")}
            variant="outline"
            class="flex items-center space-x-2"
          >
            <Icon icon="lucide:sparkles" class="h-4 w-4" />
            <span>Generate Tasks with AI</span>
          </Button>
          <Button
            onclick={handleTaskCreate}
            class="flex items-center space-x-2"
          >
            <Icon icon="mdi:plus" class="h-4 w-4" />
            <span>New Task</span>
          </Button>
    {/snippet}
  </PageHeader>

  <!-- Main Content -->
  <div class="w-full">
    <!-- Stats and Progress Row -->
    <div class="mb-4 grid grid-cols-1 gap-4 lg:grid-cols-3">
      <!-- Left: Compact Stats -->
      <div class="lg:col-span-2">
        <TaskStats />
      </div>
      <!-- Right: Progress Bar -->
      <div class="lg:col-span-1">
        <TaskProgress />
      </div>
    </div>

    <!-- Search and Filters Row -->
    <Card class="mb-4">
      <CardContent class="p-3">
        <div class="flex items-center gap-3">
          <!-- Search Input -->
          <div class="relative flex-1">
            <Icon
              icon="mdi:magnify"
              class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
            />
            <Input
              type="text"
              placeholder="Search tasks..."
              bind:value={searchQuery}
              class="pl-10"
            />
          </div>

          <!-- Filter Button -->
          <Button
            variant="outline"
            onclick={() => (showFilterModal = !showFilterModal)}
            class="flex items-center gap-2"
          >
            <Icon icon="mdi:filter-variant" class="h-4 w-4" />
            Filters
            {#if hasActiveFilters}
              <Badge variant="secondary" class="ml-1 px-1.5 py-0.5 text-xs">
                {activeFilterCount}
              </Badge>
            {/if}
          </Button>
        </div>

        <!-- Active Filters Display -->
        {#if hasActiveFilters}
          <div class="divider-edge-t divider-edge-full mt-3 pt-3">
            <div class="flex flex-wrap items-center gap-2">
              <span class="text-xs font-medium text-muted-foreground"
                >Active filters:</span
              >

              {#if searchQuery.trim() !== ""}
                <Badge variant="outline" class="text-xs">
                  <Icon icon="mdi:magnify" class="mr-1 h-3 w-3" />
                  "{searchQuery}"
                  <Button
                    onclick={() => (searchQuery = "")}
                    variant="ghost"
                    size="sm"
                    class="ml-1 h-4 w-4 p-0 hover:bg-muted"
                  >
                    <Icon icon="mdi:close" class="h-3 w-3" />
                  </Button>
                </Badge>
              {/if}

              {#each activeStatusFilters as status}
                <Badge variant="outline" class="text-xs">
                  <Icon icon="mdi:circle" class="mr-1 h-3 w-3" />
                  {status === "pending"
                    ? "To Do"
                    : status === "in-progress"
                      ? "In Progress"
                      : status === "completed"
                        ? "Completed"
                        : "Cancelled"}
                  <Button
                    onclick={() => toggleStatusFilter(status)}
                    variant="ghost"
                    size="sm"
                    class="ml-1 h-4 w-4 p-0 hover:bg-muted"
                  >
                    <Icon icon="mdi:close" class="h-3 w-3" />
                  </Button>
                </Badge>
              {/each}

              {#each activePriorityFilters as priority}
                <Badge variant="outline" class="text-xs">
                  <Icon
                    icon="mdi:flag"
                    class="mr-1 h-3 w-3 {priority === 'high'
                      ? 'text-red-500'
                      : priority === 'medium'
                        ? 'text-yellow-500'
                        : 'text-green-500'}"
                  />
                  {priority.charAt(0).toUpperCase() + priority.slice(1)}
                  <Button
                    onclick={() => togglePriorityFilter(priority)}
                    variant="ghost"
                    size="sm"
                    class="ml-1 h-4 w-4 p-0 hover:bg-muted"
                  >
                    <Icon icon="mdi:close" class="h-3 w-3" />
                  </Button>
                </Badge>
              {/each}

              {#each activeTypeFilters as type}
                <Badge variant="outline" class="text-xs">
                  {type}
                  <Button
                    onclick={() => toggleTypeFilter(type)}
                    variant="ghost"
                    size="sm"
                    class="ml-1 h-4 w-4 p-0 hover:bg-muted"
                  >
                    <Icon icon="mdi:close" class="h-3 w-3" />
                  </Button>
                </Badge>
              {/each}

              <Button
                variant="outline"
                size="sm"
                onclick={clearAllFilters}
                class="ml-2 text-xs"
              >
                <Icon icon="mdi:filter-remove" class="mr-1 h-3 w-3" />
                Clear All
              </Button>
            </div>
          </div>
        {/if}
      </CardContent>
    </Card>

    <!-- Filter Modal -->
    {#if showFilterModal}
      <TaskFilterModal
        bind:open={showFilterModal}
        {searchQuery}
        {statusFilters}
        {priorityFilters}
        {typeFilters}
        onSearchChange={(value) => (searchQuery = value)}
        onStatusFilterChange={(filters) => (statusFilters = filters)}
        onPriorityFilterChange={(filters) => (priorityFilters = filters)}
        onTypeFilterChange={(filters) => (typeFilters = filters)}
        onClearAll={clearAllFilters}
      />
    {/if}

    <!-- Main Content -->
    {#if $isLoading}
      <PageLoading message="Loading tasks..." />
    {:else if $error}
      <PageError
        title="Failed to load tasks"
        message={$error}
        onRetry={() => tasksQuery.refetch()}
      />
    {:else if parentTasks.length === 0 && !hasActiveFilters}
      <PageEmpty
        title="No tasks yet"
        description="Create your first task to start organizing your work."
        actionLabel="Create Task"
        onAction={() => goto("/tasks/create")}
      />
    {:else if currentView === "kanban"}
      <KanbanBoard
        {handleTaskSelect}
        {handleTaskStatusToggle}
        {handleTaskSelection}
        {handleCreateSubtask}
        {getSubtaskCount}
        {getTaskStatusColor}
        {getStatusBadgeColor}
        {getPriorityColor}
        {getTaskIcon}
      />
    {:else if currentView === "list"}
      <TaskList
        {handleTaskSelect}
        {handleTaskStatusToggle}
        {handleTaskSelection}
        {handleCreateSubtask}
        {getSubtaskCount}
        {getTaskStatusColor}
        {getStatusBadgeColor}
        {getPriorityColor}
        {getTaskIcon}
      />
    {/if}
  </div>

  <!-- Keyboard Shortcuts Modal -->
  <Dialog.Root bind:open={showKeyboardShortcuts}>
    <Dialog.Content class="max-h-[80vh] max-w-2xl overflow-hidden">
      <Dialog.Header class="pb-3">
        <Dialog.Title class="text-lg">Keyboard Shortcuts</Dialog.Title>
      </Dialog.Header>

      <div class="space-y-6 overflow-y-auto px-6 pb-6">
          <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
            <div class="space-y-4">
              <h3 class="font-semibold text-foreground">
                Navigation & Actions
              </h3>
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-sm">Create new task</span>
                  <kbd class="rounded bg-muted px-2 py-1 text-xs">Ctrl+N</kbd>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-sm">Open filters</span>
                  <kbd class="rounded bg-muted px-2 py-1 text-xs">Ctrl+F</kbd>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-sm">Focus search</span>
                  <kbd class="rounded bg-muted px-2 py-1 text-xs">Ctrl+K</kbd>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-sm">Toggle multi-select</span>
                  <kbd class="rounded bg-muted px-2 py-1 text-xs">Ctrl+M</kbd>
                </div>
              </div>
            </div>

            <div class="space-y-4">
              <h3 class="font-semibold text-foreground">Multi-Select Mode</h3>
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-sm">Select all tasks</span>
                  <kbd class="rounded bg-muted px-2 py-1 text-xs">Ctrl+A</kbd>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-sm">Delete selected</span>
                  <kbd class="rounded bg-muted px-2 py-1 text-xs">Delete</kbd>
                </div>
              </div>

              <h3 class="font-semibold text-foreground">General</h3>
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-sm">Show shortcuts</span>
                  <kbd class="rounded bg-muted px-2 py-1 text-xs">Ctrl+H</kbd>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-sm">Close modal</span>
                  <kbd class="rounded bg-muted px-2 py-1 text-xs">Esc</kbd>
                </div>
              </div>
            </div>
          </div>

          <div class="divider-edge-t divider-edge-full pt-4">
            <div class="flex items-center justify-end">
              <Button onclick={() => (showKeyboardShortcuts = false)}>
                Got it
              </Button>
            </div>
          </div>
      </div>
    </Dialog.Content>
  </Dialog.Root>
</div>
