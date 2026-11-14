<script lang="ts">
  import FaTableColumns from '~icons/fa6-solid/table-columns';
  import FaList from '~icons/fa6-solid/list';
  import FaSquareCheck from '~icons/fa6-solid/square-check';
  import FaCheckDouble from '~icons/fa6-solid/check-double';
  import FaXmark from '~icons/fa6-solid/xmark';
  import FaTrash from '~icons/fa6-solid/trash';
  import FaKeyboard from '~icons/fa6-solid/keyboard';
  import FaPlus from '~icons/fa6-solid/plus';
  import FaMagnifyingGlass from '~icons/fa6-solid/magnifying-glass';
  import FaFilter from '~icons/fa6-solid/filter';
  import FaCircle from '~icons/fa6-solid/circle';
  import FaFlag from '~icons/fa6-solid/flag';
  import FaFilterCircleXmark from '~icons/fa6-solid/filter-circle-xmark';
  import FaBoxArchive from '~icons/fa6-solid/box-archive';
  import FaLayerGroup from '~icons/fa6-solid/layer-group';
  import FaFolder from '~icons/fa6-solid/folder';
  import FaInbox from '~icons/fa6-solid/inbox';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { listen } from '@tauri-apps/api/event';
  import { taskActions, isLoading, error } from '../stores/taskStore';
  import { taskUi, getSubtaskCount, hasProject } from '../state/taskUi.svelte';
  import { createTasksQuery } from '../queries/taskQueries';
  import { invalidateTaskCaches } from '../queries/invalidateTasks';
  import { queryClient } from '$lib/domains/shared/query';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
    ShellSidebarLayout,
    ResponsivePanel,
  } from '$lib/components/shell';
  import { taskStatusLabel, type Task, type TaskStatus, type TaskPriority } from '../types';
  import { copyText } from '$lib/domains/shared/utils/clipboard';
  import { openCoderSession } from '$lib/domains/coder/utils/launch';
  import { openAgentsSession } from '$lib/domains/agents/utils/launch';
  import { buildTaskReference, buildCoderPrompt } from '../utils/taskReference';
  import TaskDetail from './TaskDetail.svelte';
  import TaskFilterModal from './TaskFilterModal.svelte';
  import KanbanBoard from './KanbanBoard.svelte';
  import TaskList from './TaskList.svelte';
  import ArchiveView from './ArchiveView.svelte';

  const tasksQuery = createTasksQuery();
  const projectsQuery = createProjectsQuery();

  $effect(() => {
    if (tasksQuery.data) {
      taskUi.setTasks(tasksQuery.data);
    }
  });

  $effect(() => {
    isLoading.set(tasksQuery.isPending);
    if (tasksQuery.isError) {
      error.set(
        tasksQuery.error instanceof Error ? tasksQuery.error.message : 'Failed to load tasks'
      );
    } else if (tasksQuery.isSuccess) {
      error.set(null);
    }
  });

  // Tasks can change from outside the app (daemon RPC) too, not just our own
  // mutations — refetch on the backend's fan-out event rather than polling.
  $effect(() => {
    let unlisten: (() => void) | undefined;
    listen('tasks://changed', () => invalidateTaskCaches(queryClient)).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  });

  // Selection lives in the URL: ?project=<id|all|none>&view=board|list|archive&task=<id>
  type View = 'board' | 'list' | 'archive';
  const selectedProject = $derived(page.url.searchParams.get('project') ?? 'all');
  const rawView = $derived(page.url.searchParams.get('view'));
  const currentView = $derived<View>(
    rawView === 'list' || rawView === 'archive' ? rawView : 'board'
  );
  /** The task shown in the right rail; null keeps the rail closed. */
  const selectedTaskId = $derived(page.url.searchParams.get('task'));

  /**
   * One writer for the query string, so changing the project keeps the open
   * task and opening a task keeps the project/search context.
   */
  function setParams(next: { project?: string; view?: View; task?: string | null }, push = false) {
    const params = new URLSearchParams(page.url.search);
    if (next.project !== undefined) params.set('project', next.project);
    if (next.view !== undefined) params.set('view', next.view);
    if (next.task === null) params.delete('task');
    else if (next.task !== undefined) params.set('task', next.task);
    goto(`?${params.toString()}`, { replace: !push, reset: false });
  }

  function navigate(project: string, view: View) {
    setParams({ project, view });
  }

  /** Opening pushes, so Back closes the rail; closing replaces, so Back does not reopen it. */
  function openTask(taskId: string) {
    setParams({ task: taskId }, true);
  }

  function closeTask() {
    setParams({ task: null });
  }

  let searchQuery = $state('');
  let showFilterModal = $state(false);
  let showKeyboardShortcuts = $state(false);
  let statusFilters = $state<TaskStatus[]>([]);
  let priorityFilters = $state<TaskPriority[]>([]);
  let typeFilters = $state<string[]>([]);

  // Kanban/List/Archive read taskUi.filteredTasks, so push the local filter state into it.
  $effect(() => {
    taskUi.taskFilters = {
      search: searchQuery,
      status: statusFilters,
      priority: priorityFilters,
      type: typeFilters,
      noProject: selectedProject === 'none',
      resourceType: selectedProject !== 'all' && selectedProject !== 'none' ? 'project' : undefined,
      resourceId:
        selectedProject !== 'all' && selectedProject !== 'none' ? selectedProject : undefined,
    };
  });
  $effect(() => () => taskUi.clearFilters());

  const hasActiveFilters = $derived(
    searchQuery.trim() !== '' ||
      statusFilters.length > 0 ||
      priorityFilters.length > 0 ||
      typeFilters.length > 0
  );
  const activeFilterCount = $derived(
    (statusFilters.length > 0 ? 1 : 0) +
      (priorityFilters.length > 0 ? 1 : 0) +
      (typeFilters.length > 0 ? 1 : 0) +
      (searchQuery.trim() !== '' ? 1 : 0)
  );

  // Sidebar data: only projects that actually have tasks.
  const projectNames = $derived(
    new Map((projectsQuery.data ?? []).map((p) => [String(p.id), p.name]))
  );
  const nameOf = (id: string) => projectNames.get(id) ?? `Project #${id}`;
  const sidebarProjects = $derived(
    [...taskUi.projectCounts.known]
      .map((id) => ({ id, name: nameOf(id), open: taskUi.projectCounts.open[id] ?? 0 }))
      .sort((a, b) => b.open - a.open || a.name.localeCompare(b.name))
  );
  const noProjectOpen = $derived(taskUi.projectCounts.open[''] ?? 0);
  // "No project" shows when any parent task lacks a project link (open or archived).
  const hasNoProjectTasks = $derived(taskUi.parentTasks.some((t) => !hasProject(t)));

  const projectLabel = $derived(
    selectedProject === 'all'
      ? (t: Task) => (hasProject(t) ? nameOf(t.resourceId!) : undefined)
      : undefined
  );

  const hasAnyVisible = $derived(taskUi.boardTasks.length + taskUi.archivedTasks.length > 0);

  // Event handlers
  function handleTaskSelect(task: Task) {
    taskActions.selectTask(task);
    if (navigator.vibrate) {
      navigator.vibrate(50);
    }
    openTask(task.id);
  }

  function handleTaskCreate() {
    goto('/tasks/create');
  }

  // Filter functions
  function toggleStatusFilter(status: TaskStatus) {
    statusFilters = statusFilters.includes(status)
      ? statusFilters.filter((s) => s !== status)
      : [...statusFilters, status];
  }

  function togglePriorityFilter(priority: TaskPriority) {
    priorityFilters = priorityFilters.includes(priority)
      ? priorityFilters.filter((p) => p !== priority)
      : [...priorityFilters, priority];
  }

  function toggleTypeFilter(type: string) {
    typeFilters = typeFilters.includes(type)
      ? typeFilters.filter((t) => t !== type)
      : [...typeFilters, type];
  }

  function clearAllFilters() {
    searchQuery = '';
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
    goto(`/tasks/create?parentId=${parentTask.id}`);
  }

  /** Resolve a task's linked project (name + filesystem path) for the context menu. */
  function taskProject(task: Task): { name?: string; path?: string; id?: string } {
    if (task.resourceType !== 'project' || !task.resourceId) return {};
    const p = (projectsQuery.data ?? []).find((pr) => String(pr.id) === task.resourceId);
    return { name: p?.name, path: p?.path, id: p ? String(p.id) : task.resourceId };
  }

  async function handleSetStatus(taskId: string, status: TaskStatus) {
    try {
      await taskActions.updateTask(taskId, { status });
    } catch (err) {
      toastActions.error(
        'Failed to update status',
        err instanceof Error ? err.message : 'An unexpected error occurred'
      );
    }
  }

  function handleCopyReference(task: Task) {
    void copyText(buildTaskReference(task, taskProject(task).name), 'Task reference copied');
  }

  function handleLaunchCoder(task: Task) {
    const { name, path, id } = taskProject(task);
    openCoderSession({
      workspaceRoot: path,
      projectId: id ?? null,
      prompt: buildCoderPrompt(task, name),
    });
  }

  function handleLaunchAgents(task: Task) {
    const { name, path } = taskProject(task);
    if (!path) return;
    openAgentsSession({ cwd: path, prompt: buildCoderPrompt(task, name), taskId: task.id });
  }

  function handleProjectClick(task: Task) {
    const { id } = taskProject(task);
    if (id) goto(`/projects/${id}`);
  }

  function handleEditTask(task: Task) {
    goto(`/tasks/${task.id}/edit`);
  }

  async function handleDeleteTask(task: Task) {
    const subtaskCount = getSubtaskCount(task.id, taskUi.tasks);
    const confirmed = await confirmAction(
      `Delete "${task.title}"?${subtaskCount ? ` Also deletes ${subtaskCount} subtask${subtaskCount === 1 ? '' : 's'}.` : ''}`,
      'Delete task',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!confirmed) return;
    try {
      await taskActions.deleteTask(task.id);
      toastActions.success('Task deleted');
    } catch (err) {
      toastActions.error(
        'Failed to delete task',
        err instanceof Error ? err.message : 'An unexpected error occurred'
      );
    }
  }

  function handleSelectAll() {
    taskActions.selectAllTasks(taskUi.boardTasks.map((task) => task.id));
  }

  function handleClearSelection() {
    taskActions.clearSelection();
  }

  async function handleBulkDelete() {
    // Capture first: deleteTasksBulk empties the selection.
    const ids = Array.from(taskUi.selectedTaskIds);
    if (ids.length === 0) return;
    const n = ids.length;
    const plural = n === 1 ? '' : 's';
    const subtaskCount = ids.reduce((sum, id) => sum + getSubtaskCount(id, taskUi.tasks), 0);
    const confirmed = await confirmAction(
      `Are you sure you want to delete ${n} selected task${plural}?${subtaskCount ? ` Also deletes ${subtaskCount} subtask${subtaskCount === 1 ? '' : 's'}.` : ''}`,
      'Delete tasks',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!confirmed) return;
    try {
      await taskActions.deleteTasksBulk(ids);
      toastActions.success('Tasks deleted', `Successfully deleted ${n} task${plural}`);
    } catch (err) {
      toastActions.error(
        'Failed to delete tasks',
        err instanceof Error ? err.message : 'An unexpected error occurred'
      );
    }
  }

  // Keyboard shortcuts
  /** True while the event came from a text field, where native editing wins. */
  function isTypingTarget(event: KeyboardEvent): boolean {
    const el = event.target as HTMLElement | null;
    return !!el?.closest('input, textarea, [contenteditable="true"]');
  }

  /** True while any dialog is open — its own Escape handling must win. */
  function hasOpenDialog(): boolean {
    return !!document.querySelector('[role="dialog"], [role="alertdialog"]');
  }

  /**
   * Whether a dialog was open when the key went down, sampled in the capture
   * phase. By the time the bubble handler runs, the dialog has already closed
   * itself in response to the same Escape, so asking then always says "none
   * open" and the keypress falls through and drops the multi-selection.
   */
  let dialogOpenAtKeydown = false;

  function noteDialogState(event: KeyboardEvent) {
    if (event.key === 'Escape') dialogOpenAtKeydown = hasOpenDialog();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      if (event.key === 'n') {
        event.preventDefault();
        handleTaskCreate();
      } else if (event.key === 'f') {
        event.preventDefault();
        showFilterModal = true;
      } else if (event.key === 'k') {
        event.preventDefault();
        const searchInput = document.querySelector(
          'input[placeholder="Search tasks..."]'
        ) as HTMLInputElement;
        if (searchInput) {
          searchInput.focus();
        }
      } else if (event.key === 'h') {
        event.preventDefault();
        showKeyboardShortcuts = true;
      } else if (event.key === 'm') {
        event.preventDefault();
        handleMultiSelectToggle();
      } else if (event.key === 'a' && taskUi.isMultiSelectMode) {
        // In a text field Ctrl+A means "select this text", not "select every task".
        if (isTypingTarget(event)) return;
        event.preventDefault();
        handleSelectAll();
      }
    } else if (event.key === 'Delete' && taskUi.selectedTaskIds.size > 0) {
      // Don't hijack Delete while typing in a field.
      if (isTypingTarget(event)) return;
      event.preventDefault();
      handleBulkDelete();
    } else if (event.key === 'Escape') {
      // A confirm dialog's own Escape would otherwise bubble to here and drop
      // the whole multi-selection behind it. Use the capture-phase sample:
      // the dialog has already closed by the time this runs.
      if (dialogOpenAtKeydown) return;
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

<svelte:window onkeydowncapture={noteDialogState} onkeydown={handleKeydown} />

{#snippet navItem(label: string, count: number, active: boolean, onclick: () => void, Ico: any)}
  <Button
    {onclick}
    variant={active ? 'secondary' : 'ghost'}
    size="sm"
    class="h-7 w-full justify-between gap-2 px-2"
    aria-current={active ? 'true' : undefined}
  >
    <span class="flex min-w-0 items-center gap-2">
      <Ico class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
      <span class="truncate" title={label}>{label}</span>
    </span>
    <Badge variant="outline" class="shrink-0 px-1.5">{count}</Badge>
  </Button>
{/snippet}

<div class="flex h-full min-h-0 w-full flex-col overflow-hidden bg-background">
  <div class="shrink-0 px-4 py-4 md:px-6">
    <PageHeader title="Tasks" description="Organize and track your work efficiently">
      {#snippet actions()}
        {#if !taskUi.isMultiSelectMode}
          <Button onclick={handleMultiSelectToggle} variant="outline" size="sm">
            <FaSquareCheck class="mr-1.5 h-4 w-4" />
            Multi-Select
          </Button>
        {/if}
        <Button
          onclick={() => (showKeyboardShortcuts = true)}
          variant="ghost"
          size="sm"
          title="Keyboard Shortcuts (Ctrl+H)"
        >
          <FaKeyboard class="mr-1.5 h-4 w-4" />
          Shortcuts
        </Button>
        <Button onclick={handleTaskCreate} size="sm">
          <FaPlus class="mr-1.5 h-4 w-4" />
          New Task
        </Button>
      {/snippet}
    </PageHeader>
  </div>

  <ShellSidebarLayout mobileTriggerLabel="Projects menu">
    {#snippet sidebar()}
      <nav class="flex flex-col gap-1 p-3" aria-label="Task projects">
        {@render navItem(
          'All projects',
          taskUi.projectCounts.openTotal,
          selectedProject === 'all' && currentView !== 'archive',
          () => navigate('all', currentView === 'archive' ? 'board' : currentView),
          FaLayerGroup
        )}
        <div class="divider-edge-t divider-edge-full my-1"></div>
        {#each sidebarProjects as p (p.id)}
          {@render navItem(
            p.name,
            p.open,
            selectedProject === p.id && currentView !== 'archive',
            () => navigate(p.id, currentView === 'archive' ? 'board' : currentView),
            FaFolder
          )}
        {/each}
        {#if hasNoProjectTasks}
          {@render navItem(
            'No project',
            noProjectOpen,
            selectedProject === 'none' && currentView !== 'archive',
            () => navigate('none', currentView === 'archive' ? 'board' : currentView),
            FaInbox
          )}
        {/if}
        <div class="divider-edge-t divider-edge-full my-1"></div>
        {@render navItem(
          'Archive',
          taskUi.projectCounts.archived,
          currentView === 'archive',
          () => navigate(selectedProject, 'archive'),
          FaBoxArchive
        )}
      </nav>
    {/snippet}

    <!--
      Bounded flex column, not a scroller: the toolbar stays put and the content
      region below owns the scrolling. Board view scrolls inside KanbanBoard
      (columns scroll vertically, the row scrolls horizontally) so its horizontal
      scrollbar sits at the viewport edge instead of at the foot of a tall page.
    -->
    <div class="flex min-h-0 min-w-0 flex-1 overflow-hidden">
      <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-hidden p-4 md:p-6">
        <!-- One toolbar row. The per-column counts live in the column headers. -->
        <div class="flex flex-wrap items-center gap-2">
          <div class="relative min-w-[200px] flex-1 sm:max-w-sm">
            <FaMagnifyingGlass
              class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
            />
            <Input
              type="text"
              placeholder="Search tasks..."
              bind:value={searchQuery}
              class="h-9 pl-9"
            />
          </div>

          <Button variant="outline" size="sm" onclick={() => (showFilterModal = !showFilterModal)}>
            <FaFilter class="mr-1.5 h-4 w-4" />
            Filters
            {#if hasActiveFilters}
              <Badge variant="secondary" class="ml-1.5 px-1.5 py-0.5 text-xs">
                {activeFilterCount}
              </Badge>
            {/if}
          </Button>

          <div class="flex rounded-lg bg-muted p-0.5">
            <Button
              onclick={() => navigate(selectedProject, 'board')}
              variant={currentView === 'board' ? 'default' : 'ghost'}
              size="sm"
              class="h-8 px-3"
            >
              <FaTableColumns class="mr-1.5 h-4 w-4" />
              Kanban
            </Button>
            <Button
              onclick={() => navigate(selectedProject, 'list')}
              variant={currentView === 'list' ? 'default' : 'ghost'}
              size="sm"
              class="h-8 px-3"
            >
              <FaList class="mr-1.5 h-4 w-4" />
              List
            </Button>
          </div>

          {#if hasActiveFilters}
            {#if searchQuery.trim() !== ''}
              <Badge variant="outline" class="text-xs">
                <FaMagnifyingGlass class="mr-1 h-3 w-3" />
                "{searchQuery}"
                <Button
                  onclick={() => (searchQuery = '')}
                  variant="ghost"
                  size="sm"
                  class="ml-1 h-4 w-4 p-0 hover:bg-muted"
                  aria-label="Clear search"
                >
                  <FaXmark class="h-3 w-3" />
                </Button>
              </Badge>
            {/if}

            {#each statusFilters as status (status)}
              <Badge variant="outline" class="text-xs">
                <FaCircle class="mr-1 h-3 w-3" />
                {taskStatusLabel(status)}
                <Button
                  onclick={() => toggleStatusFilter(status)}
                  variant="ghost"
                  size="sm"
                  class="ml-1 h-4 w-4 p-0 hover:bg-muted"
                  aria-label="Remove status filter"
                >
                  <FaXmark class="h-3 w-3" />
                </Button>
              </Badge>
            {/each}

            {#each priorityFilters as priority (priority)}
              <Badge variant="outline" class="text-xs">
                <FaFlag
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
                  aria-label="Remove priority filter"
                >
                  <FaXmark class="h-3 w-3" />
                </Button>
              </Badge>
            {/each}

            {#each typeFilters as type (type)}
              <Badge variant="outline" class="text-xs">
                {type}
                <Button
                  onclick={() => toggleTypeFilter(type)}
                  variant="ghost"
                  size="sm"
                  class="ml-1 h-4 w-4 p-0 hover:bg-muted"
                  aria-label="Remove type filter"
                >
                  <FaXmark class="h-3 w-3" />
                </Button>
              </Badge>
            {/each}

            <Button variant="ghost" size="sm" onclick={clearAllFilters} class="h-7 text-xs">
              <FaFilterCircleXmark class="mr-1 h-3 w-3" />
              Clear All
            </Button>
          {/if}
        </div>

        <!-- Multi-select controls (only while active) -->
        {#if taskUi.isMultiSelectMode}
          <div
            class="flex flex-wrap items-center gap-3 rounded-lg border border-warning-200 bg-warning-50 px-4 py-2 dark:border-warning-800 dark:bg-warning-900/20"
          >
            <Badge
              variant="secondary"
              class="bg-warning-100 text-warning-800 dark:bg-warning-800 dark:text-warning-100"
            >
              <FaSquareCheck class="mr-1 h-3 w-3" />
              {taskUi.selectedTaskIds.size} selected
            </Badge>
            <div class="flex items-center gap-2">
              <Button
                onclick={handleSelectAll}
                variant="outline"
                size="sm"
                class="h-7 px-2 text-xs"
              >
                <FaCheckDouble class="mr-1 h-3 w-3" />
                Select All
              </Button>
              <Button
                onclick={handleClearSelection}
                variant="outline"
                size="sm"
                class="h-7 px-2 text-xs"
              >
                <FaXmark class="mr-1 h-3 w-3" />
                Clear
              </Button>
              <Button
                onclick={handleBulkDelete}
                disabled={taskUi.selectedTaskIds.size === 0}
                variant="destructive"
                size="sm"
                class="h-7 px-2 text-xs"
              >
                <FaTrash class="mr-1 h-3 w-3" />
                Delete ({taskUi.selectedTaskIds.size})
              </Button>
              <Button
                onclick={handleMultiSelectToggle}
                variant="ghost"
                size="sm"
                class="h-7 px-2 text-xs"
              >
                <FaXmark class="mr-1 h-3 w-3" />
                Cancel
              </Button>
            </div>
          </div>
        {/if}

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
        <div
          class="min-h-0 flex-1 {currentView === 'board' ? 'overflow-hidden' : 'overflow-y-auto'}"
        >
          {#if $isLoading}
            <PageLoading message="Loading tasks..." />
          {:else if $error}
            <PageError
              title="Failed to load tasks"
              message={$error}
              onRetry={() => tasksQuery.refetch()}
            />
          {:else if taskUi.parentTasks.length === 0 && !hasActiveFilters}
            <PageEmpty
              title="No tasks yet"
              description="Create your first task to start organizing your work."
              actionLabel="Create Task"
              onAction={() => goto('/tasks/create')}
            />
          {:else if currentView === 'archive'}
            {#if taskUi.archivedTasks.length === 0 && hasActiveFilters}
              <!-- archivedTasks is already search/filter-filtered, so an empty list
                 under active filters means "no match", not "nothing archived". -->
              <PageEmpty
                title="No matching archived tasks"
                description="No archived tasks match the current project, search or filters."
                actionLabel="Clear filters"
                onAction={clearAllFilters}
              />
            {:else if taskUi.archivedTasks.length === 0}
              <PageEmpty
                title="Archive is empty"
                description="Tasks completed more than 7 days ago and cancelled tasks show up here."
              />
            {:else}
              <ArchiveView
                tasks={taskUi.archivedTasks}
                onSelect={handleTaskSelect}
                {projectLabel}
                onProjectClick={handleProjectClick}
              />
            {/if}
          {:else if !hasAnyVisible}
            <PageEmpty
              title="No matching tasks"
              description="No tasks match the current project, search or filters."
              actionLabel="Clear filters"
              onAction={clearAllFilters}
            />
          {:else if currentView === 'board'}
            <KanbanBoard
              {handleTaskSelect}
              {handleTaskStatusToggle}
              {handleTaskSelection}
              {handleCreateSubtask}
              onSetStatus={handleSetStatus}
              onCopyReference={handleCopyReference}
              onLaunchCoder={handleLaunchCoder}
              onLaunchAgents={handleLaunchAgents}
              onEditTask={handleEditTask}
              onDeleteTask={handleDeleteTask}
              {projectLabel}
              onProjectClick={handleProjectClick}
              onShowArchive={() => navigate(selectedProject, 'archive')}
            />
          {:else}
            <TaskList
              {handleTaskSelect}
              {handleTaskStatusToggle}
              {handleTaskSelection}
              {handleCreateSubtask}
              onSetStatus={handleSetStatus}
              onCopyReference={handleCopyReference}
              onLaunchCoder={handleLaunchCoder}
              onLaunchAgents={handleLaunchAgents}
              onEditTask={handleEditTask}
              onDeleteTask={handleDeleteTask}
              {projectLabel}
              onProjectClick={handleProjectClick}
            />
          {/if}
        </div>
      </div>

      <!--
        Task detail opens beside the board instead of navigating away, so scroll
        position, the project filter and the search text all survive. The URL is
        the record of what is open, which is what makes reload and Back work.
      -->
      {#if selectedTaskId}
        <ResponsivePanel
          bind:open={() => true, (open) => !open && closeTask()}
          side="right"
          desktopClass="w-[20rem] xl:w-[26rem]"
        >
          {#snippet header()}
            <div class="flex items-center justify-between gap-2 px-4 py-3">
              <h2 class="text-sm font-semibold">Task</h2>
              <Button variant="ghost" size="sm" onclick={closeTask} aria-label="Close task details">
                <FaXmark class="h-4 w-4" />
              </Button>
            </div>
          {/snippet}
          <div class="min-h-0 flex-1 overflow-y-auto p-4">
            <TaskDetail taskId={selectedTaskId} onDeleted={closeTask} onOpenTask={openTask} />
          </div>
        </ResponsivePanel>
      {/if}
    </div>
  </ShellSidebarLayout>

  <!-- Keyboard Shortcuts Modal -->
  <Dialog.Root bind:open={showKeyboardShortcuts}>
    <Dialog.Content class="max-h-[80vh] max-w-2xl overflow-hidden">
      <Dialog.Header class="pb-3">
        <Dialog.Title class="text-lg">Keyboard Shortcuts</Dialog.Title>
      </Dialog.Header>

      <div class="space-y-6 overflow-y-auto px-6 pb-6">
        <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
          <div class="space-y-4">
            <h3 class="font-semibold text-foreground">Navigation & Actions</h3>
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
            <Button onclick={() => (showKeyboardShortcuts = false)}>Got it</Button>
          </div>
        </div>
      </div>
    </Dialog.Content>
  </Dialog.Root>
</div>
