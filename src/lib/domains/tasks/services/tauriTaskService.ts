import { invoke } from '@tauri-apps/api/core';
import type { Task, CreateTaskRequest, UpdateTaskRequest, TaskFilters } from '../types';

export interface TauriTaskResponse {
  id: number;
  title: string;
  description: string | null;
  status: string;
  priority: string;
  type_: string | null;
  parent_id: number | null;
  resource_id: string | null;
  resource_type: string | null;
  due_date: string | null;
  completed_at: string | null;
  created_at: string | null;
  updated_at: string | null;
  // New advanced fields
  estimated_time: number | null;
  actual_time: number | null;
  tags: string | null; // JSON array of strings
  assignee: string | null;
  recurring_pattern: string | null;
  recurring_interval: number | null;
  recurring_end_date: string | null;
  recurring_last_generated: string | null;
  blocked_by: string | null; // JSON array of task IDs
  blocks: string | null; // JSON array of task IDs
}

export interface TauriCreateTaskCommand {
  title: string;
  description: string | null;
  status: string;
  priority: string;
  type_: string | null;
  parent_id: number | null;
  resource_id: string | null;
  resource_type: string | null;
  due_date: string | null;
  // New advanced fields
  estimated_time: number | null;
  actual_time: number | null;
  tags: string | null; // JSON array of strings
  assignee: string | null;
  recurring_pattern: string | null;
  recurring_interval: number | null;
  recurring_end_date: string | null;
  recurring_last_generated: string | null;
  blocked_by: string | null; // JSON array of task IDs
  blocks: string | null; // JSON array of task IDs
}

export interface TauriUpdateTaskCommand {
  title: string | null;
  description: string | null;
  status: string | null;
  priority: string | null;
  type_: string | null;
  parent_id: number | null;
  resource_id: string | null;
  resource_type: string | null;
  due_date: string | null;
  // New advanced fields
  estimated_time: number | null;
  actual_time: number | null;
  tags: string | null; // JSON array of strings
  assignee: string | null;
  recurring_pattern: string | null;
  recurring_interval: number | null;
  recurring_end_date: string | null;
  recurring_last_generated: string | null;
  blocked_by: string | null; // JSON array of task IDs
  blocks: string | null; // JSON array of task IDs
}

export interface TauriTaskFiltersCommand {
  status: string[] | null;
  priority: string[] | null;
  type_: string[] | null;
  parent_id: number | null;
  resource_id: string | null;
  resource_type: string | null;
}

// Convert Tauri response to frontend Task type
function convertTauriTaskToTask(tauriTask: TauriTaskResponse): Task {
  return {
    id: tauriTask.id.toString(),
    title: tauriTask.title,
    description: tauriTask.description || undefined,
    status: tauriTask.status as any,
    priority: tauriTask.priority as any,
    type: tauriTask.type_ || undefined,
    parentId: tauriTask.parent_id?.toString(),
    resourceId: tauriTask.resource_id || undefined,
    resourceType: tauriTask.resource_type || undefined,
    dueDate: tauriTask.due_date ? new Date(tauriTask.due_date) : undefined,
    completedAt: tauriTask.completed_at ? new Date(tauriTask.completed_at) : undefined,
    createdAt: tauriTask.created_at ? new Date(tauriTask.created_at) : new Date(),
    updatedAt: tauriTask.updated_at ? new Date(tauriTask.updated_at) : new Date(),
    // New advanced fields
    estimatedTime: tauriTask.estimated_time || undefined,
    actualTime: tauriTask.actual_time || undefined,
    tags: tauriTask.tags ? JSON.parse(tauriTask.tags) : undefined,
    assignee: tauriTask.assignee || undefined,
    recurring: tauriTask.recurring_pattern ? {
      pattern: tauriTask.recurring_pattern as any,
      interval: tauriTask.recurring_interval || 1,
      endDate: tauriTask.recurring_end_date ? new Date(tauriTask.recurring_end_date) : undefined,
      lastGenerated: tauriTask.recurring_last_generated ? new Date(tauriTask.recurring_last_generated) : undefined,
    } : undefined,
    blockedBy: tauriTask.blocked_by ? JSON.parse(tauriTask.blocked_by) : undefined,
    blocks: tauriTask.blocks ? JSON.parse(tauriTask.blocks) : undefined,
    comments: [], // Will be loaded separately
    attachments: [], // Will be loaded separately
  };
}

// Convert frontend Task to Tauri command
function convertTaskToTauriCreateCommand(task: CreateTaskRequest): TauriCreateTaskCommand {
  return {
    title: task.title,
    description: task.description || null,
    status: task.status,
    priority: task.priority,
    type_: task.type || null,
    parent_id: task.parentId ? parseInt(task.parentId) : null,
    resource_id: task.resourceId || null,
    resource_type: task.resourceType || null,
    due_date: task.dueDate ? task.dueDate.toISOString() : null,
    // New advanced fields
    estimated_time: task.estimatedTime || null,
    actual_time: task.actualTime || null,
    tags: task.tags ? JSON.stringify(task.tags) : null,
    assignee: task.assignee || null,
    recurring_pattern: task.recurring?.pattern || null,
    recurring_interval: task.recurring?.interval || null,
    recurring_end_date: task.recurring?.endDate ? task.recurring.endDate.toISOString() : null,
    recurring_last_generated: task.recurring?.lastGenerated ? task.recurring.lastGenerated.toISOString() : null,
    blocked_by: task.blockedBy ? JSON.stringify(task.blockedBy) : null,
    blocks: task.blocks ? JSON.stringify(task.blocks) : null,
  };
}

// Convert frontend Task to Tauri update command
function convertTaskToTauriUpdateCommand(task: UpdateTaskRequest): TauriUpdateTaskCommand {
  return {
    title: task.title || null,
    description: task.description || null,
    status: task.status || null,
    priority: task.priority || null,
    type_: task.type || null,
    parent_id: task.parentId ? parseInt(task.parentId) : null,
    resource_id: task.resourceId || null,
    resource_type: task.resourceType || null,
    due_date: task.dueDate ? task.dueDate.toISOString() : null,
    // New advanced fields
    estimated_time: task.estimatedTime || null,
    actual_time: task.actualTime || null,
    tags: task.tags ? JSON.stringify(task.tags) : null,
    assignee: task.assignee || null,
    recurring_pattern: task.recurring?.pattern || null,
    recurring_interval: task.recurring?.interval || null,
    recurring_end_date: task.recurring?.endDate ? task.recurring.endDate.toISOString() : null,
    recurring_last_generated: task.recurring?.lastGenerated ? task.recurring.lastGenerated.toISOString() : null,
    blocked_by: task.blockedBy ? JSON.stringify(task.blockedBy) : null,
    blocks: task.blocks ? JSON.stringify(task.blocks) : null,
  };
}

// Convert frontend filters to Tauri filters
function convertFiltersToTauriFilters(filters: TaskFilters): TauriTaskFiltersCommand {
  return {
    status: filters.status && filters.status.length > 0 ? filters.status : null,
    priority: filters.priority && filters.priority.length > 0 ? filters.priority : null,
    type_: filters.type && filters.type.length > 0 ? filters.type : null,
    parent_id: filters.parentId ? parseInt(filters.parentId) : null,
    resource_id: filters.resourceId || null,
    resource_type: filters.resourceType || null,
  };
}

export class TauriTaskService {
  async createTask(task: CreateTaskRequest): Promise<Task> {
    const command = convertTaskToTauriCreateCommand(task);
    const response = await invoke<TauriTaskResponse>('create_task', { command });
    return convertTauriTaskToTask(response);
  }

  async updateTask(id: string, task: UpdateTaskRequest): Promise<Task> {
    const command = convertTaskToTauriUpdateCommand(task);
    const response = await invoke<TauriTaskResponse>('update_task', { 
      id: parseInt(id), 
      command 
    });
    return convertTauriTaskToTask(response);
  }

  async deleteTask(id: string): Promise<void> {
    await invoke('delete_task', { id: parseInt(id) });
  }

  async getTask(id: string): Promise<Task | null> {
    const response = await invoke<TauriTaskResponse | null>('get_task', { id: parseInt(id) });
    return response ? convertTauriTaskToTask(response) : null;
  }

  async getTasks(filters?: TaskFilters): Promise<Task[]> {
    const tauriFilters = filters ? convertFiltersToTauriFilters(filters) : null;
    const response = await invoke<TauriTaskResponse[]>('get_tasks', { filters: tauriFilters });
    return response.map(convertTauriTaskToTask);
  }

  async getSubtasks(parentId: string): Promise<Task[]> {
    const response = await invoke<TauriTaskResponse[]>('get_subtasks', { parent_id: parseInt(parentId) });
    return response.map(convertTauriTaskToTask);
  }

  async getMainTasks(): Promise<Task[]> {
    const response = await invoke<TauriTaskResponse[]>('get_main_tasks');
    return response.map(convertTauriTaskToTask);
  }

  async getTaskCount(): Promise<number> {
    return await invoke<number>('get_task_count');
  }

  // New advanced methods
  async getOverdueTasks(): Promise<Task[]> {
    const response = await invoke<TauriTaskResponse[]>('get_overdue_tasks');
    return response.map(convertTauriTaskToTask);
  }

  async getDueTodayTasks(): Promise<Task[]> {
    const response = await invoke<TauriTaskResponse[]>('get_due_today_tasks');
    return response.map(convertTauriTaskToTask);
  }

  async getUnestimatedTasks(): Promise<Task[]> {
    const response = await invoke<TauriTaskResponse[]>('get_unestimated_tasks');
    return response.map(convertTauriTaskToTask);
  }
}

export const tauriTaskService = new TauriTaskService();
