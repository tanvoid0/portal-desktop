import { writable } from "svelte/store";
import type {
  Task,
  TaskFilters,
  CreateTaskRequest,
  UpdateTaskRequest,
  TaskTemplate,
} from "../types";
import { tauriTaskService } from "../services/tauriTaskService";
import { fetchAllTasks } from "../api/taskApi";
import {
  queryClient,
  invalidateDashboardOverview,
  queryKeys,
} from "$lib/domains/shared/query";
import { invalidateTaskCaches } from "../queries/invalidateTasks";
import { taskUi } from "../state/taskUi.svelte";

// Mutation loading/error (list loading comes from TanStack Query)
export const isLoading = writable(false);
export const error = writable<string | null>(null);

// Task mutations
export const taskActions = {
  async loadTasks(): Promise<void> {
    try {
      isLoading.set(true);
      error.set(null);

      const allTasks = await fetchAllTasks();
      taskUi.setTasks(allTasks);
      queryClient.setQueryData(queryKeys.tasks.all, allTasks);

      if (allTasks.length > 0) {
        console.log(`✅ Loaded ${allTasks.length} tasks successfully`);
      }
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load tasks";
      error.set(errorMessage);
      console.error("❌ Failed to load tasks:", err);

      if (typeof window !== "undefined" && window.alert) {
        window.alert(`Failed to load tasks: ${errorMessage}`);
      }
    } finally {
      isLoading.set(false);
    }
  },

  async createTask(request: CreateTaskRequest): Promise<Task> {
    if (!request.title?.trim()) {
      const errorMsg = "Task title is required";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    if (request.title.length > 200) {
      const errorMsg = "Task title must be less than 200 characters";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    if (request.description && request.description.length > 1000) {
      const errorMsg = "Task description must be less than 1000 characters";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    try {
      isLoading.set(true);
      error.set(null);

      const newTask = await tauriTaskService.createTask(request);
      await this.loadTasks();
      invalidateTaskCaches(queryClient, newTask.id);
      invalidateDashboardOverview(queryClient);

      return newTask;
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to create task";
      error.set(errorMessage);
      console.error("❌ Failed to create task:", err);
      throw err;
    } finally {
      isLoading.set(false);
    }
  },

  async updateTask(taskId: string, request: UpdateTaskRequest): Promise<Task> {
    if (request.title !== undefined && !request.title?.trim()) {
      const errorMsg = "Task title cannot be empty";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    if (request.title && request.title.length > 200) {
      const errorMsg = "Task title must be less than 200 characters";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    if (request.description && request.description.length > 1000) {
      const errorMsg = "Task description must be less than 1000 characters";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    try {
      isLoading.set(true);
      error.set(null);

      const updatedTask = await tauriTaskService.updateTask(taskId, request);
      await this.loadTasks();
      invalidateTaskCaches(queryClient, taskId);
      invalidateDashboardOverview(queryClient);

      return updatedTask;
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to update task";
      error.set(errorMessage);
      console.error("❌ Failed to update task:", err);
      throw err;
    } finally {
      isLoading.set(false);
    }
  },

  async deleteTask(taskId: string): Promise<void> {
    if (!taskId?.trim()) {
      const errorMsg = "Task ID is required";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    try {
      isLoading.set(true);
      error.set(null);

      await tauriTaskService.deleteTask(taskId);
      await this.loadTasks();
      invalidateTaskCaches(queryClient, taskId);
      invalidateDashboardOverview(queryClient);

      taskUi.clearSelectedIfDeleted(taskId);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to delete task";
      error.set(errorMessage);
      console.error("❌ Failed to delete task:", err);
      throw err;
    } finally {
      isLoading.set(false);
    }
  },

  async deleteTasksBulk(taskIds: string[]): Promise<void> {
    if (!taskIds || taskIds.length === 0) {
      const errorMsg = "No tasks selected for deletion";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    const invalidIds = taskIds.filter((id) => !id?.trim());
    if (invalidIds.length > 0) {
      const errorMsg = "Invalid task IDs provided";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    try {
      isLoading.set(true);
      error.set(null);

      const deletePromises = taskIds.map((taskId) =>
        tauriTaskService.deleteTask(taskId),
      );
      await Promise.all(deletePromises);

      await this.loadTasks();
      taskIds.forEach((id) => invalidateTaskCaches(queryClient, id));
      invalidateDashboardOverview(queryClient);

      taskUi.clearSelectedIfDeletedBulk(taskIds);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to delete tasks";
      error.set(errorMessage);
      console.error("❌ Failed to delete tasks:", err);
      throw err;
    } finally {
      isLoading.set(false);
    }
  },

  async toggleTaskStatus(taskId: string): Promise<void> {
    if (!taskId?.trim()) {
      const errorMsg = "Task ID is required";
      error.set(errorMsg);
      throw new Error(errorMsg);
    }

    try {
      const task = await tauriTaskService.getTask(taskId);

      if (!task) {
        const errorMsg = "Task not found";
        error.set(errorMsg);
        throw new Error(errorMsg);
      }

      const statusOrder = ["pending", "in-progress", "completed", "cancelled"];
      const currentIndex = statusOrder.indexOf(task.status);
      const nextStatus = statusOrder[
        (currentIndex + 1) % statusOrder.length
      ] as Task["status"];

      await this.updateTask(taskId, { status: nextStatus });
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to toggle task status";
      error.set(errorMessage);
      console.error("❌ Failed to toggle task status:", err);
      throw err;
    }
  },

  selectTask(task: Task | null): void {
    taskUi.selectTask(task);
  },

  toggleTaskSelection(taskId: string): void {
    taskUi.toggleTaskSelection(taskId);
  },

  toggleMultiSelectMode(): void {
    taskUi.toggleMultiSelectMode();
  },

  clearSelection(): void {
    taskUi.clearSelection();
  },

  selectAllTasks(allTaskIds: string[]): void {
    taskUi.selectAllTasks(allTaskIds);
  },

  setFilters(filters: Partial<TaskFilters>): void {
    taskUi.setFilters(filters);
  },

  clearFilters(): void {
    taskUi.clearFilters();
  },

  async startTimeTracking(taskId: string): Promise<void> {
    taskUi.startTimeTracking(taskId);
  },

  async stopTimeTracking(): Promise<void> {
    taskUi.stopTimeTracking();
  },

  async addComment(
    taskId: string,
    content: string,
    author: string,
  ): Promise<void> {
    await this.loadTasks();
  },

  async deleteComment(commentId: string): Promise<void> {
    await this.loadTasks();
  },

  async addAttachment(
    taskId: string,
    name: string,
    url: string,
    type: string,
    size: number,
  ): Promise<void> {
    await this.loadTasks();
  },

  async deleteAttachment(attachmentId: string): Promise<void> {
    await this.loadTasks();
  },

  async createTemplate(template: TaskTemplate): Promise<void> {
    taskUi.createTemplate(template);
  },

  async updateTemplate(template: TaskTemplate): Promise<void> {
    taskUi.updateTemplate(template);
  },

  async deleteTemplate(templateId: string): Promise<void> {
    taskUi.deleteTemplate(templateId);
  },

  async deleteSavedView(viewId: string): Promise<void> {
    taskUi.deleteSavedView(viewId);
  },

  async setDefaultView(viewId: string): Promise<void> {
    taskUi.setDefaultView(viewId);
  },

  async applyTemplate(
    templateId: string,
    taskData: Partial<CreateTaskRequest>,
  ): Promise<Task> {
    return taskUi.applyTemplate(templateId, taskData, (request) =>
      this.createTask(request),
    );
  },

  async saveView(
    name: string,
    description: string,
    filters: TaskFilters,
    isDefault: boolean = false,
  ): Promise<void> {
    taskUi.saveView(name, description, filters, isDefault);
  },

  async loadView(viewId: string): Promise<void> {
    taskUi.loadView(viewId);
  },

  async checkDependencies(
    taskId: string,
  ): Promise<{ blocked: boolean; blockingTasks: string[] }> {
    return taskUi.checkDependencies(taskId);
  },

  async generateRecurringTasks(): Promise<void> {
    await this.loadTasks();
  },
};
