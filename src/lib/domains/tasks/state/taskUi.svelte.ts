/**
 * Task UI session state (not server data).
 * Server lists sync via setTasks() from TanStack Query in TaskManager.
 */

import type {
  Task,
  TaskFilters,
  TaskStats,
  TaskTemplate,
  SavedView,
  TimeTrackingSession,
  ProductivityMetrics,
  CreateTaskRequest,
} from "../types";

function applyTaskFilters(tasks: Task[], filters: TaskFilters): Task[] {
  let filtered = tasks.filter((task) => !task.parentId);

  if (filters.search?.trim()) {
    const query = filters.search.toLowerCase();
    filtered = filtered.filter(
      (task) =>
        task.title.toLowerCase().includes(query) ||
        (task.description && task.description.toLowerCase().includes(query)),
    );
  }

  if (filters.status && filters.status.length > 0) {
    filtered = filtered.filter((task) =>
      filters.status!.includes(task.status),
    );
  }

  if (filters.priority && filters.priority.length > 0) {
    filtered = filtered.filter((task) =>
      filters.priority!.includes(task.priority),
    );
  }

  if (filters.type && filters.type.length > 0) {
    filtered = filtered.filter(
      (task) => task.type && filters.type!.includes(task.type),
    );
  }

  if (filters.resourceType && filters.resourceId) {
    filtered = filtered.filter(
      (task) =>
        task.resourceType === filters.resourceType &&
        task.resourceId === filters.resourceId,
    );
  }

  return filtered;
}

function computeTaskStats(parentTasks: Task[]): TaskStats {
  const total = parentTasks.length;
  const pending = parentTasks.filter((t) => t.status === "pending").length;
  const inProgress = parentTasks.filter(
    (t) => t.status === "in-progress",
  ).length;
  const completed = parentTasks.filter((t) => t.status === "completed").length;
  const cancelled = parentTasks.filter((t) => t.status === "cancelled").length;
  const completionPercentage =
    total > 0 ? Math.round((completed / total) * 100) : 0;

  return {
    total,
    pending,
    inProgress,
    completed,
    cancelled,
    completionPercentage,
  };
}

function computeProductivityMetrics(tasks: Task[]): ProductivityMetrics {
  const total = tasks.length;
  const completed = tasks.filter((t) => t.status === "completed").length;
  const completionRate = total > 0 ? (completed / total) * 100 : 0;

  const tasksWithTime = tasks.filter((t) => t.estimatedTime && t.actualTime);
  const timeAccuracy =
    tasksWithTime.length > 0
      ? tasksWithTime.reduce((acc, task) => {
          const accuracy =
            Math.abs(
              (task.estimatedTime! - task.actualTime!) / task.estimatedTime!,
            ) * 100;
          return acc + (100 - accuracy);
        }, 0) / tasksWithTime.length
      : 0;

  const oneWeekAgo = new Date();
  oneWeekAgo.setDate(oneWeekAgo.getDate() - 7);
  const recentCompleted = tasks.filter(
    (t) =>
      t.status === "completed" && t.completedAt && t.completedAt >= oneWeekAgo,
  ).length;

  const overdueCount = tasks.filter(
    (t) =>
      t.dueDate &&
      t.dueDate < new Date() &&
      t.status !== "completed" &&
      t.status !== "cancelled",
  ).length;

  const unestimatedCount = tasks.filter(
    (t) =>
      !t.estimatedTime && t.status !== "completed" && t.status !== "cancelled",
  ).length;

  const blockedCount = tasks.filter(
    (t) =>
      t.blockedBy &&
      t.blockedBy.length > 0 &&
      t.status !== "completed" &&
      t.status !== "cancelled",
  ).length;

  return {
    completionRate,
    timeAccuracy,
    velocity: recentCompleted,
    overdueCount,
    unestimatedCount,
    blockedCount,
  };
}

export function getTaskSubtasks(taskId: string, allTasks: Task[]): Task[] {
  return allTasks.filter((task) => task.parentId === taskId);
}

export function getSubtaskCount(taskId: string, allTasks: Task[]): number {
  return allTasks.filter((task) => task.parentId === taskId).length;
}

export class TaskUiState {
  tasks = $state<Task[]>([]);
  selectedTask = $state<Task | null>(null);
  selectedTaskIds = $state<Set<string>>(new Set());
  isMultiSelectMode = $state(false);
  taskFilters = $state<TaskFilters>({});
  timeTrackingSession = $state<TimeTrackingSession | null>(null);
  savedViews = $state<SavedView[]>([]);
  taskTemplates = $state<TaskTemplate[]>([]);

  parentTasks = $derived(this.tasks.filter((task) => !task.parentId));

  filteredTasks = $derived(
    applyTaskFilters(this.parentTasks, this.taskFilters),
  );

  taskStats = $derived(computeTaskStats(this.parentTasks));

  kanbanColumns = $derived([
    {
      id: "pending",
      title: "To Do",
      color: "bg-gray-100 dark:bg-gray-800",
      tasks: this.filteredTasks.filter((t) => t.status === "pending"),
    },
    {
      id: "in-progress",
      title: "In Progress",
      color: "bg-blue-100 dark:bg-blue-900/20",
      tasks: this.filteredTasks.filter((t) => t.status === "in-progress"),
    },
    {
      id: "completed",
      title: "Done",
      color: "bg-green-100 dark:bg-green-900/20",
      tasks: this.filteredTasks.filter((t) => t.status === "completed"),
    },
    {
      id: "cancelled",
      title: "Cancelled",
      color: "bg-red-100 dark:bg-red-900/20",
      tasks: this.filteredTasks.filter((t) => t.status === "cancelled"),
    },
  ]);

  overdueTasks = $derived(
    this.tasks.filter(
      (task) =>
        task.dueDate &&
        task.dueDate < new Date() &&
        task.status !== "completed" &&
        task.status !== "cancelled",
    ),
  );

  dueTodayTasks = $derived.by(() => {
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);

    return this.tasks.filter(
      (task) =>
        task.dueDate &&
        task.dueDate >= today &&
        task.dueDate < tomorrow &&
        task.status !== "completed" &&
        task.status !== "cancelled",
    );
  });

  dueThisWeekTasks = $derived.by(() => {
    const today = new Date();
    const weekFromNow = new Date(today);
    weekFromNow.setDate(weekFromNow.getDate() + 7);

    return this.tasks.filter(
      (task) =>
        task.dueDate &&
        task.dueDate >= today &&
        task.dueDate <= weekFromNow &&
        task.status !== "completed" &&
        task.status !== "cancelled",
    );
  });

  blockedTasks = $derived(
    this.tasks.filter(
      (task) =>
        task.blockedBy &&
        task.blockedBy.length > 0 &&
        task.status !== "completed" &&
        task.status !== "cancelled",
    ),
  );

  unestimatedTasks = $derived(
    this.tasks.filter(
      (task) =>
        !task.estimatedTime &&
        task.status !== "completed" &&
        task.status !== "cancelled",
    ),
  );

  currentlyTracking = $derived(this.timeTrackingSession);

  productivityMetrics = $derived(computeProductivityMetrics(this.tasks));

  setTasks(tasks: Task[]): void {
    this.tasks = tasks;
  }

  selectTask(task: Task | null): void {
    this.selectedTask = task;
  }

  toggleTaskSelection(taskId: string): void {
    const next = new Set(this.selectedTaskIds);
    if (next.has(taskId)) {
      next.delete(taskId);
    } else {
      next.add(taskId);
    }
    this.selectedTaskIds = next;
  }

  toggleMultiSelectMode(): void {
    this.isMultiSelectMode = !this.isMultiSelectMode;
    this.selectedTaskIds = new Set();
  }

  clearSelection(): void {
    this.selectedTaskIds = new Set();
  }

  selectAllTasks(allTaskIds: string[]): void {
    this.selectedTaskIds = new Set(allTaskIds);
  }

  clearSelectedIfDeleted(taskId: string): void {
    if (this.selectedTask?.id === taskId) {
      this.selectedTask = null;
    }
  }

  clearSelectedIfDeletedBulk(taskIds: string[]): void {
    const next = new Set(this.selectedTaskIds);
    taskIds.forEach((id) => next.delete(id));
    this.selectedTaskIds = next;

    if (this.selectedTask && taskIds.includes(this.selectedTask.id)) {
      this.selectedTask = null;
    }
  }

  setFilters(filters: Partial<TaskFilters>): void {
    this.taskFilters = { ...this.taskFilters, ...filters };
  }

  clearFilters(): void {
    this.taskFilters = {};
  }

  startTimeTracking(taskId: string): void {
    this.timeTrackingSession = {
      taskId,
      startTime: new Date(),
      isActive: true,
    };
  }

  stopTimeTracking(): void {
    const session = this.timeTrackingSession;
    if (session) {
      const endTime = new Date();
      const duration = Math.round(
        (endTime.getTime() - session.startTime.getTime()) / (1000 * 60),
      );
      const task = this.tasks.find((t) => t.id === session.taskId);
      if (task) {
        const newActualTime = (task.actualTime || 0) + duration;
        this.tasks = this.tasks.map((t) =>
          t.id === session.taskId ? { ...t, actualTime: newActualTime } : t,
        );
      }
    }
    this.timeTrackingSession = null;
  }

  createTemplate(template: TaskTemplate): void {
    this.taskTemplates = [...this.taskTemplates, template];
  }

  updateTemplate(template: TaskTemplate): void {
    this.taskTemplates = this.taskTemplates.map((t) =>
      t.id === template.id ? template : t,
    );
  }

  deleteTemplate(templateId: string): void {
    this.taskTemplates = this.taskTemplates.filter((t) => t.id !== templateId);
  }

  saveView(
    name: string,
    description: string,
    filters: TaskFilters,
    isDefault: boolean = false,
  ): void {
    const newView: SavedView = {
      id: Date.now().toString(),
      name,
      description: description || undefined,
      filters,
      isDefault,
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    this.savedViews = [...this.savedViews, newView];
  }

  loadView(viewId: string): void {
    const view = this.savedViews.find((v) => v.id === viewId);
    if (view) {
      this.setFilters(view.filters);
    }
  }

  deleteSavedView(viewId: string): void {
    this.savedViews = this.savedViews.filter((v) => v.id !== viewId);
  }

  setDefaultView(viewId: string): void {
    this.savedViews = this.savedViews.map((v) => ({
      ...v,
      isDefault: v.id === viewId,
    }));
  }

  checkDependencies(
    taskId: string,
  ): { blocked: boolean; blockingTasks: string[] } {
    const task = this.tasks.find((t) => t.id === taskId);
    if (!task || !task.blockedBy) {
      return { blocked: false, blockingTasks: [] };
    }

    const blockingTasks = task.blockedBy.filter((blockingId) => {
      const blockingTask = this.tasks.find((t) => t.id === blockingId);
      return blockingTask && blockingTask.status !== "completed";
    });

    return {
      blocked: blockingTasks.length > 0,
      blockingTasks,
    };
  }

  async applyTemplate(
    templateId: string,
    taskData: Partial<CreateTaskRequest>,
    createTask: (request: CreateTaskRequest) => Promise<Task>,
  ): Promise<Task> {
    return createTask(taskData as CreateTaskRequest);
  }
}

export const taskUi = new TaskUiState();

export function createTaskUiState(): TaskUiState {
  return new TaskUiState();
}
