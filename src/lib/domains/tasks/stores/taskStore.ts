import { writable, derived, type Writable } from 'svelte/store';
import type { Task, TaskFilters, TaskStats, CreateTaskRequest, UpdateTaskRequest, TimeTrackingSession, ProductivityMetrics, TaskTemplate } from '../types';
import { tauriTaskService } from '../services/tauriTaskService';

// Core stores
export const tasks: Writable<Task[]> = writable([]);
export const selectedTask: Writable<Task | null> = writable(null);
export const selectedTaskIds: Writable<Set<string>> = writable(new Set());
export const isMultiSelectMode: Writable<boolean> = writable(false);
export const isLoading: Writable<boolean> = writable(false);
export const error: Writable<string | null> = writable(null);

// New advanced stores
export const timeTrackingSession: Writable<TimeTrackingSession | null> = writable(null);
export const savedViews: Writable<any[]> = writable([]); // Will be properly typed later
export const taskTemplates: Writable<any[]> = writable([]); // Will be properly typed later

// Filter stores
export const taskFilters: Writable<TaskFilters> = writable({});

// Derived stores
export const parentTasks = derived(tasks, ($tasks) => 
	$tasks.filter(task => !task.parentId)
);

export const filteredTasks = derived(
	[tasks, taskFilters],
	([$tasks, $filters]) => {
		let filtered = $tasks.filter(task => !task.parentId); // Only parent tasks

		// Apply search filter
		if ($filters.search?.trim()) {
			const query = $filters.search.toLowerCase();
			filtered = filtered.filter(task =>
				task.title.toLowerCase().includes(query) ||
				(task.description && task.description.toLowerCase().includes(query))
			);
		}

		// Apply status filter
		if ($filters.status && $filters.status.length > 0) {
			filtered = filtered.filter(task => $filters.status!.includes(task.status));
		}

		// Apply priority filter
		if ($filters.priority && $filters.priority.length > 0) {
			filtered = filtered.filter(task => $filters.priority!.includes(task.priority));
		}

		// Apply type filter
		if ($filters.type && $filters.type.length > 0) {
			filtered = filtered.filter(task => 
				task.type && $filters.type!.includes(task.type)
			);
		}

		// Apply resource filter
		if ($filters.resourceType && $filters.resourceId) {
			filtered = filtered.filter(task =>
				task.resourceType === $filters.resourceType &&
				task.resourceId === $filters.resourceId
			);
		}

		return filtered;
	}
);

export const taskStats = derived(parentTasks, ($parentTasks) => {
	const total = $parentTasks.length;
	const pending = $parentTasks.filter(t => t.status === 'pending').length;
	const inProgress = $parentTasks.filter(t => t.status === 'in-progress').length;
	const completed = $parentTasks.filter(t => t.status === 'completed').length;
	const cancelled = $parentTasks.filter(t => t.status === 'cancelled').length;
	const completionPercentage = total > 0 ? Math.round((completed / total) * 100) : 0;

	return {
		total,
		pending,
		inProgress,
		completed,
		cancelled,
		completionPercentage
	};
});

export const kanbanColumns = derived(filteredTasks, ($filteredTasks) => [
	{
		id: 'pending',
		title: 'To Do',
		color: 'bg-gray-100 dark:bg-gray-800',
		tasks: $filteredTasks.filter(t => t.status === 'pending')
	},
	{
		id: 'in-progress',
		title: 'In Progress',
		color: 'bg-blue-100 dark:bg-blue-900/20',
		tasks: $filteredTasks.filter(t => t.status === 'in-progress')
	},
	{
		id: 'completed',
		title: 'Done',
		color: 'bg-green-100 dark:bg-green-900/20',
		tasks: $filteredTasks.filter(t => t.status === 'completed')
	},
	{
		id: 'cancelled',
		title: 'Cancelled',
		color: 'bg-red-100 dark:bg-red-900/20',
		tasks: $filteredTasks.filter(t => t.status === 'cancelled')
	}
]);

// New advanced derived stores
export const overdueTasks = derived(tasks, ($tasks) => {
	const now = new Date();
	return $tasks.filter(task => 
		task.dueDate && 
		task.dueDate < now && 
		task.status !== 'completed' && 
		task.status !== 'cancelled'
	);
});

export const dueTodayTasks = derived(tasks, ($tasks) => {
	const today = new Date();
	today.setHours(0, 0, 0, 0);
	const tomorrow = new Date(today);
	tomorrow.setDate(tomorrow.getDate() + 1);
	
	return $tasks.filter(task => 
		task.dueDate && 
		task.dueDate >= today && 
		task.dueDate < tomorrow &&
		task.status !== 'completed' && 
		task.status !== 'cancelled'
	);
});

export const dueThisWeekTasks = derived(tasks, ($tasks) => {
	const today = new Date();
	const weekFromNow = new Date(today);
	weekFromNow.setDate(weekFromNow.getDate() + 7);
	
	return $tasks.filter(task => 
		task.dueDate && 
		task.dueDate >= today && 
		task.dueDate <= weekFromNow &&
		task.status !== 'completed' && 
		task.status !== 'cancelled'
	);
});

export const blockedTasks = derived(tasks, ($tasks) => {
	return $tasks.filter(task => 
		task.blockedBy && 
		task.blockedBy.length > 0 &&
		task.status !== 'completed' && 
		task.status !== 'cancelled'
	);
});

export const unestimatedTasks = derived(tasks, ($tasks) => {
	return $tasks.filter(task => 
		!task.estimatedTime &&
		task.status !== 'completed' && 
		task.status !== 'cancelled'
	);
});

export const currentlyTracking = derived(timeTrackingSession, ($session) => $session);

export const productivityMetrics = derived(tasks, ($tasks) => {
	const total = $tasks.length;
	const completed = $tasks.filter(t => t.status === 'completed').length;
	const completionRate = total > 0 ? (completed / total) * 100 : 0;
	
	const tasksWithTime = $tasks.filter(t => t.estimatedTime && t.actualTime);
	const timeAccuracy = tasksWithTime.length > 0 
		? tasksWithTime.reduce((acc, task) => {
			const accuracy = Math.abs((task.estimatedTime! - task.actualTime!) / task.estimatedTime!) * 100;
			return acc + (100 - accuracy);
		}, 0) / tasksWithTime.length
		: 0;
	
	// Calculate velocity (tasks completed per week)
	const oneWeekAgo = new Date();
	oneWeekAgo.setDate(oneWeekAgo.getDate() - 7);
	const recentCompleted = $tasks.filter(t => 
		t.status === 'completed' && 
		t.completedAt && 
		t.completedAt >= oneWeekAgo
	).length;
	
	const overdueCount = $tasks.filter(t => 
		t.dueDate && 
		t.dueDate < new Date() && 
		t.status !== 'completed' && 
		t.status !== 'cancelled'
	).length;
	
	const unestimatedCount = $tasks.filter(t => 
		!t.estimatedTime &&
		t.status !== 'completed' && 
		t.status !== 'cancelled'
	).length;
	
	const blockedCount = $tasks.filter(t => 
		t.blockedBy && 
		t.blockedBy.length > 0 &&
		t.status !== 'completed' && 
		t.status !== 'cancelled'
	).length;
	
	return {
		completionRate,
		timeAccuracy,
		velocity: recentCompleted,
		overdueCount,
		unestimatedCount,
		blockedCount
	} as ProductivityMetrics;
});

// Helper functions
export function getTaskSubtasks(taskId: string, allTasks: Task[]): Task[] {
	return allTasks.filter(task => task.parentId === taskId);
}

export function getSubtaskCount(taskId: string, allTasks: Task[]): number {
	return allTasks.filter(task => task.parentId === taskId).length;
}

// Task actions
export const taskActions = {
	// Load tasks from database
	async loadTasks(): Promise<void> {
		try {
			isLoading.set(true);
			error.set(null);
			
			const allTasks = await tauriTaskService.getTasks();
			tasks.set(allTasks);
			
			// Add success feedback
			if (allTasks.length > 0) {
				console.log(`✅ Loaded ${allTasks.length} tasks successfully`);
			}
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to load tasks';
			error.set(errorMessage);
			console.error('❌ Failed to load tasks:', err);
			
			// Show user-friendly error message
			if (typeof window !== 'undefined' && window.alert) {
				window.alert(`Failed to load tasks: ${errorMessage}`);
			}
		} finally {
			isLoading.set(false);
		}
	},

	// Task CRUD operations
	async createTask(request: CreateTaskRequest): Promise<Task> {
		// Validation
		if (!request.title?.trim()) {
			const errorMsg = 'Task title is required';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		if (request.title.length > 200) {
			const errorMsg = 'Task title must be less than 200 characters';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		if (request.description && request.description.length > 1000) {
			const errorMsg = 'Task description must be less than 1000 characters';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		try {
			isLoading.set(true);
			error.set(null);
			
			const newTask = await tauriTaskService.createTask(request);
			await this.loadTasks();
			return newTask;
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to create task';
			error.set(errorMessage);
			console.error('❌ Failed to create task:', err);
			throw err;
		} finally {
			isLoading.set(false);
		}
	},

	async updateTask(taskId: string, request: UpdateTaskRequest): Promise<Task> {
		// Validation
		if (request.title !== undefined && !request.title?.trim()) {
			const errorMsg = 'Task title cannot be empty';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		if (request.title && request.title.length > 200) {
			const errorMsg = 'Task title must be less than 200 characters';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		if (request.description && request.description.length > 1000) {
			const errorMsg = 'Task description must be less than 1000 characters';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		try {
			isLoading.set(true);
			error.set(null);
			
			const updatedTask = await tauriTaskService.updateTask(taskId, request);
			await this.loadTasks();
			return updatedTask;
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to update task';
			error.set(errorMessage);
			console.error('❌ Failed to update task:', err);
			throw err;
		} finally {
			isLoading.set(false);
		}
	},

	async deleteTask(taskId: string): Promise<void> {
		if (!taskId?.trim()) {
			const errorMsg = 'Task ID is required';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		try {
			isLoading.set(true);
			error.set(null);
			
			await tauriTaskService.deleteTask(taskId);
			await this.loadTasks();
			
			// Clear selection if the deleted task was selected
			selectedTask.update(current => 
				current?.id === taskId ? null : current
			);
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to delete task';
			error.set(errorMessage);
			console.error('❌ Failed to delete task:', err);
			throw err;
		} finally {
			isLoading.set(false);
		}
	},

	async deleteTasksBulk(taskIds: string[]): Promise<void> {
		if (!taskIds || taskIds.length === 0) {
			const errorMsg = 'No tasks selected for deletion';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		// Validate all task IDs
		const invalidIds = taskIds.filter(id => !id?.trim());
		if (invalidIds.length > 0) {
			const errorMsg = 'Invalid task IDs provided';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		try {
			isLoading.set(true);
			error.set(null);
			
			// Delete tasks one by one (Tauri doesn't have bulk delete)
			const deletePromises = taskIds.map(taskId => tauriTaskService.deleteTask(taskId));
			await Promise.all(deletePromises);
			
			await this.loadTasks();

			// Clear selection if any selected tasks were deleted
			selectedTaskIds.update(current => {
				const newSet = new Set(current);
				taskIds.forEach(id => newSet.delete(id));
				return newSet;
			});

			selectedTask.update(current => 
				current && taskIds.includes(current.id) ? null : current
			);
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to delete tasks';
			error.set(errorMessage);
			console.error('❌ Failed to delete tasks:', err);
			throw err;
		} finally {
			isLoading.set(false);
		}
	},

	async toggleTaskStatus(taskId: string): Promise<void> {
		if (!taskId?.trim()) {
			const errorMsg = 'Task ID is required';
			error.set(errorMsg);
			throw new Error(errorMsg);
		}

		try {
			// Get the current task from the database
			const task = await tauriTaskService.getTask(taskId);
			
			if (!task) {
				const errorMsg = 'Task not found';
				error.set(errorMsg);
				throw new Error(errorMsg);
			}

			const statusOrder = ['pending', 'in-progress', 'completed', 'cancelled'];
			const currentIndex = statusOrder.indexOf(task.status);
			const nextStatus = statusOrder[(currentIndex + 1) % statusOrder.length] as Task['status'];
			
			// Update the task status via the API
			await this.updateTask(taskId, { status: nextStatus });
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to toggle task status';
			error.set(errorMessage);
			console.error('❌ Failed to toggle task status:', err);
			throw err;
		}
	},


	// Selection and UI actions
	selectTask(task: Task | null): void {
		selectedTask.set(task);
	},

	toggleTaskSelection(taskId: string): void {
		selectedTaskIds.update(current => {
			const newSet = new Set(current);
			if (newSet.has(taskId)) {
				newSet.delete(taskId);
			} else {
				newSet.add(taskId);
			}
			return newSet;
		});
	},

	toggleMultiSelectMode(): void {
		isMultiSelectMode.update(current => !current);
		if (isMultiSelectMode) {
			selectedTaskIds.set(new Set());
		}
	},

	clearSelection(): void {
		selectedTaskIds.set(new Set());
	},

	selectAllTasks(allTaskIds: string[]): void {
		selectedTaskIds.set(new Set(allTaskIds));
	},

	// Filter actions
	setFilters(filters: Partial<TaskFilters>): void {
		taskFilters.update(current => ({ ...current, ...filters }));
	},

	clearFilters(): void {
		taskFilters.set({});
	},

	// New advanced actions
	async startTimeTracking(taskId: string): Promise<void> {
		const session: TimeTrackingSession = {
			taskId,
			startTime: new Date(),
			isActive: true
		};
		timeTrackingSession.set(session);
	},

	async stopTimeTracking(): Promise<void> {
		timeTrackingSession.update(session => {
			if (session) {
				const endTime = new Date();
				const duration = Math.round((endTime.getTime() - session.startTime.getTime()) / (1000 * 60)); // minutes
				
				// Update task with actual time
				tasks.update(currentTasks => {
					const task = currentTasks.find(t => t.id === session.taskId);
					if (task) {
						const newActualTime = (task.actualTime || 0) + duration;
						// Update the task in the array
						return currentTasks.map(t => 
							t.id === session.taskId 
								? { ...t, actualTime: newActualTime }
								: t
						);
					}
					return currentTasks;
				});
			}
			return null;
		});
	},

	async addComment(taskId: string, content: string, author: string): Promise<void> {
		// This would call a new Tauri command for adding comments
		// For now, we'll just reload tasks
		await this.loadTasks();
	},

	async deleteComment(commentId: string): Promise<void> {
		// This would call a new Tauri command for deleting comments
		// For now, we'll just reload tasks
		await this.loadTasks();
	},

	async addAttachment(taskId: string, name: string, url: string, type: string, size: number): Promise<void> {
		// This would call a new Tauri command for adding attachments
		// For now, we'll just reload tasks
		await this.loadTasks();
	},

	async deleteAttachment(attachmentId: string): Promise<void> {
		// This would call a new Tauri command for deleting attachments
		// For now, we'll just reload tasks
		await this.loadTasks();
	},

	async createTemplate(template: TaskTemplate): Promise<void> {
		// This would call a new Tauri command for creating templates
		// For now, we'll just update the local store
		taskTemplates.update(current => [...current, template]);
	},

	async updateTemplate(template: TaskTemplate): Promise<void> {
		// This would call a new Tauri command for updating templates
		// For now, we'll just update the local store
		taskTemplates.update(current => 
			current.map(t => t.id === template.id ? template : t)
		);
	},

	async deleteTemplate(templateId: string): Promise<void> {
		// This would call a new Tauri command for deleting templates
		// For now, we'll just update the local store
		taskTemplates.update(current => 
			current.filter(t => t.id !== templateId)
		);
	},

	async deleteSavedView(viewId: string): Promise<void> {
		// This would call a new Tauri command for deleting saved views
		// For now, we'll just update the local store
		savedViews.update(current => 
			current.filter(v => v.id !== viewId)
		);
	},

	async setDefaultView(viewId: string): Promise<void> {
		// This would call a new Tauri command for setting default view
		// For now, we'll just update the local store
		savedViews.update(current => 
			current.map(v => ({ ...v, isDefault: v.id === viewId }))
		);
	},

	async applyTemplate(templateId: string, taskData: Partial<CreateTaskRequest>): Promise<Task> {
		// This would get template data and apply it to task creation
		// For now, we'll just create the task with the provided data
		return await this.createTask(taskData as CreateTaskRequest);
	},

	async saveView(name: string, description: string, filters: TaskFilters, isDefault: boolean = false): Promise<void> {
		const newView = {
			id: Date.now().toString(),
			name,
			description,
			filters,
			isDefault,
			createdAt: new Date(),
			updatedAt: new Date()
		};
		savedViews.update(current => [...current, newView]);
	},

	async loadView(viewId: string): Promise<void> {
		savedViews.update(currentViews => {
			const view = currentViews.find(v => v.id === viewId);
			if (view) {
				this.setFilters(view.filters);
			}
			return currentViews;
		});
	},

	async checkDependencies(taskId: string): Promise<{ blocked: boolean; blockingTasks: string[] }> {
		let result = { blocked: false, blockingTasks: [] as string[] };
		
		tasks.update(currentTasks => {
			const task = currentTasks.find(t => t.id === taskId);
			if (!task || !task.blockedBy) {
				return currentTasks;
			}

			const blockingTasks = task.blockedBy.filter(blockingId => {
				const blockingTask = currentTasks.find(t => t.id === blockingId);
				return blockingTask && blockingTask.status !== 'completed';
			});

			result = {
				blocked: blockingTasks.length > 0,
				blockingTasks
			};
			
			return currentTasks;
		});
		
		return result;
	},

	async generateRecurringTasks(): Promise<void> {
		// This would check for recurring tasks that need to be generated
		// For now, we'll just reload tasks
		await this.loadTasks();
	}

};

// Initialize tasks from database on app start
taskActions.loadTasks();
