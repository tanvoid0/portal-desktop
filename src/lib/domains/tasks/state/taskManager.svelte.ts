/**
 * State, URL selection, filters and handlers for the Tasks page. Construct it
 * during component init — it registers $effects.
 */
import { goto } from '$app/navigation';
import { page } from '$app/state';
import { listen } from '@tauri-apps/api/event';
import { taskActions, isLoading, error } from '../stores/taskStore';
import { taskUi, getSubtaskCount, hasProject } from './taskUi.svelte';
import { createTasksQuery } from '../queries/taskQueries';
import { invalidateTaskCaches } from '../queries/invalidateTasks';
import { queryClient } from '$lib/domains/shared/query';
import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
import { toastActions } from '$lib/utils/toast';
import { confirmAction } from '$lib/utils/confirm';
import type { Task, TaskStatus, TaskPriority } from '../types';
import { copyText } from '$lib/domains/shared/utils/clipboard';
import { openCoderSession } from '$lib/domains/coder/utils/launch';
import { openAgentsSession } from '$lib/domains/agents/utils/launch';
import { buildTaskReference, buildCoderPrompt } from '../utils/taskReference';

export type TaskView = 'board' | 'list' | 'archive';

const errMsg = (err: unknown) =>
  err instanceof Error ? err.message : 'An unexpected error occurred';

const toggled = <T>(list: T[], v: T) =>
  list.includes(v) ? list.filter((x) => x !== v) : [...list, v];

export class TaskManagerState {
  tasksQuery = createTasksQuery();
  projectsQuery = createProjectsQuery();

  searchQuery = $state('');
  showFilterModal = $state(false);
  showKeyboardShortcuts = $state(false);
  statusFilters = $state<TaskStatus[]>([]);
  priorityFilters = $state<TaskPriority[]>([]);
  typeFilters = $state<string[]>([]);

  // Selection lives in the URL: ?project=<id|all|none>&view=board|list|archive&task=<id>
  selectedProject = $derived(page.url.searchParams.get('project') ?? 'all');
  rawView = $derived(page.url.searchParams.get('view'));
  currentView = $derived<TaskView>(
    this.rawView === 'list' || this.rawView === 'archive' ? this.rawView : 'board'
  );
  /** The task shown in the right rail; null keeps the rail closed. */
  selectedTaskId = $derived(page.url.searchParams.get('task'));

  hasActiveFilters = $derived(
    this.searchQuery.trim() !== '' ||
      this.statusFilters.length > 0 ||
      this.priorityFilters.length > 0 ||
      this.typeFilters.length > 0
  );
  activeFilterCount = $derived(
    (this.statusFilters.length > 0 ? 1 : 0) +
      (this.priorityFilters.length > 0 ? 1 : 0) +
      (this.typeFilters.length > 0 ? 1 : 0) +
      (this.searchQuery.trim() !== '' ? 1 : 0)
  );

  // Sidebar data: only projects that actually have tasks.
  projectNames = $derived(
    new Map((this.projectsQuery.data ?? []).map((p) => [String(p.id), p.name]))
  );
  sidebarProjects = $derived(
    [...taskUi.projectCounts.known]
      .map((id) => ({ id, name: this.nameOf(id), open: taskUi.projectCounts.open[id] ?? 0 }))
      .sort((a, b) => b.open - a.open || a.name.localeCompare(b.name))
  );
  noProjectOpen = $derived(taskUi.projectCounts.open[''] ?? 0);
  // "No project" shows when any parent task lacks a project link (open or archived).
  hasNoProjectTasks = $derived(taskUi.parentTasks.some((t) => !hasProject(t)));

  projectLabel = $derived(
    this.selectedProject === 'all'
      ? (t: Task) => (hasProject(t) ? this.nameOf(t.resourceId!) : undefined)
      : undefined
  );

  hasAnyVisible = $derived(taskUi.boardTasks.length + taskUi.archivedTasks.length > 0);

  /**
   * Whether a dialog was open when the key went down, sampled in the capture
   * phase. By the time the bubble handler runs, the dialog has already closed
   * itself in response to the same Escape, so asking then always says "none
   * open" and the keypress falls through and drops the multi-selection.
   */
  private dialogOpenAtKeydown = false;

  constructor() {
    $effect(() => {
      if (this.tasksQuery.data) {
        taskUi.setTasks(this.tasksQuery.data);
      }
    });

    $effect(() => {
      const q = this.tasksQuery;
      isLoading.set(q.isPending);
      if (q.isError) {
        error.set(q.error instanceof Error ? q.error.message : 'Failed to load tasks');
      } else if (q.isSuccess) {
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

    // Kanban/List/Archive read taskUi.filteredTasks, so push the local filter state into it.
    $effect(() => {
      const project = this.selectedProject;
      const scoped = project !== 'all' && project !== 'none';
      taskUi.taskFilters = {
        search: this.searchQuery,
        status: this.statusFilters,
        priority: this.priorityFilters,
        type: this.typeFilters,
        noProject: project === 'none',
        resourceType: scoped ? 'project' : undefined,
        resourceId: scoped ? project : undefined,
      };
    });
    $effect(() => () => taskUi.clearFilters());
  }

  nameOf = (id: string) => this.projectNames.get(id) ?? `Project #${id}`;

  /**
   * One writer for the query string, so changing the project keeps the open
   * task and opening a task keeps the project/search context.
   */
  setParams(next: { project?: string; view?: TaskView; task?: string | null }, push = false) {
    const params = new URLSearchParams(page.url.search);
    if (next.project !== undefined) params.set('project', next.project);
    if (next.view !== undefined) params.set('view', next.view);
    if (next.task === null) params.delete('task');
    else if (next.task !== undefined) params.set('task', next.task);
    goto(`?${params.toString()}`, { replace: !push, reset: false });
  }

  navigate = (project: string, view: TaskView) => this.setParams({ project, view });

  /** Opening pushes, so Back closes the rail; closing replaces, so Back does not reopen it. */
  openTask = (taskId: string) => this.setParams({ task: taskId }, true);

  closeTask = () => this.setParams({ task: null });

  handleTaskSelect = (task: Task) => {
    taskActions.selectTask(task);
    if (navigator.vibrate) {
      navigator.vibrate(50);
    }
    this.openTask(task.id);
  };

  handleTaskCreate = () => {
    goto('/tasks/create');
  };

  toggleStatusFilter = (status: TaskStatus) => {
    this.statusFilters = toggled(this.statusFilters, status);
  };

  togglePriorityFilter = (priority: TaskPriority) => {
    this.priorityFilters = toggled(this.priorityFilters, priority);
  };

  toggleTypeFilter = (type: string) => {
    this.typeFilters = toggled(this.typeFilters, type);
  };

  clearAllFilters = () => {
    this.searchQuery = '';
    this.statusFilters = [];
    this.priorityFilters = [];
    this.typeFilters = [];
  };

  handleTaskStatusToggle = (taskId: string) => {
    taskActions.toggleTaskStatus(taskId);
  };

  handleMultiSelectToggle = () => {
    taskActions.toggleMultiSelectMode();
  };

  handleTaskSelection = (taskId: string) => {
    taskActions.toggleTaskSelection(taskId);
  };

  handleCreateSubtask = (parentTask: Task) => {
    goto(`/tasks/create?parentId=${parentTask.id}`);
  };

  /** Resolve a task's linked project (name + filesystem path) for the context menu. */
  taskProject(task: Task): { name?: string; path?: string; id?: string } {
    if (task.resourceType !== 'project' || !task.resourceId) return {};
    const p = (this.projectsQuery.data ?? []).find((pr) => String(pr.id) === task.resourceId);
    return { name: p?.name, path: p?.path, id: p ? String(p.id) : task.resourceId };
  }

  handleSetStatus = async (taskId: string, status: TaskStatus) => {
    try {
      await taskActions.updateTask(taskId, { status });
    } catch (err) {
      toastActions.error('Failed to update status', errMsg(err));
    }
  };

  handleCopyReference = (task: Task) => {
    void copyText(buildTaskReference(task, this.taskProject(task).name), 'Task reference copied');
  };

  handleLaunchCoder = (task: Task) => {
    const { name, path, id } = this.taskProject(task);
    openCoderSession({
      workspaceRoot: path,
      projectId: id ?? null,
      prompt: buildCoderPrompt(task, name),
    });
  };

  handleLaunchAgents = (task: Task) => {
    const { name, path } = this.taskProject(task);
    if (!path) return;
    openAgentsSession({ cwd: path, prompt: buildCoderPrompt(task, name), taskId: task.id });
  };

  handleProjectClick = (task: Task) => {
    const { id } = this.taskProject(task);
    if (id) goto(`/projects/${id}`);
  };

  handleEditTask = (task: Task) => {
    goto(`/tasks/${task.id}/edit`);
  };

  handleDeleteTask = async (task: Task) => {
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
      toastActions.error('Failed to delete task', errMsg(err));
    }
  };

  handleSelectAll = () => {
    taskActions.selectAllTasks(taskUi.boardTasks.map((task) => task.id));
  };

  handleClearSelection = () => {
    taskActions.clearSelection();
  };

  handleBulkDelete = async () => {
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
      toastActions.error('Failed to delete tasks', errMsg(err));
    }
  };

  // Keyboard shortcuts
  /** True while the event came from a text field, where native editing wins. */
  private isTypingTarget(event: KeyboardEvent): boolean {
    const el = event.target as HTMLElement | null;
    return !!el?.closest('input, textarea, [contenteditable="true"]');
  }

  /** True while any dialog is open — its own Escape handling must win. */
  private hasOpenDialog(): boolean {
    return !!document.querySelector('[role="dialog"], [role="alertdialog"]');
  }

  noteDialogState = (event: KeyboardEvent) => {
    if (event.key === 'Escape') this.dialogOpenAtKeydown = this.hasOpenDialog();
  };

  handleKeydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.metaKey) {
      if (event.key === 'n') {
        event.preventDefault();
        this.handleTaskCreate();
      } else if (event.key === 'f') {
        event.preventDefault();
        this.showFilterModal = true;
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
        this.showKeyboardShortcuts = true;
      } else if (event.key === 'm') {
        event.preventDefault();
        this.handleMultiSelectToggle();
      } else if (event.key === 'a' && taskUi.isMultiSelectMode) {
        // In a text field Ctrl+A means "select this text", not "select every task".
        if (this.isTypingTarget(event)) return;
        event.preventDefault();
        this.handleSelectAll();
      }
    } else if (event.key === 'Delete' && taskUi.selectedTaskIds.size > 0) {
      // Don't hijack Delete while typing in a field.
      if (this.isTypingTarget(event)) return;
      event.preventDefault();
      this.handleBulkDelete();
    } else if (event.key === 'Escape') {
      // A confirm dialog's own Escape would otherwise bubble to here and drop
      // the whole multi-selection behind it. Use the capture-phase sample:
      // the dialog has already closed by the time this runs.
      if (this.dialogOpenAtKeydown) return;
      if (this.showFilterModal) {
        this.showFilterModal = false;
      } else if (this.showKeyboardShortcuts) {
        this.showKeyboardShortcuts = false;
      } else if (taskUi.isMultiSelectMode) {
        this.handleMultiSelectToggle();
      }
    }
  };
}
