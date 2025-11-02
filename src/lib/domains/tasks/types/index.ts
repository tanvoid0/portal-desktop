export type TaskStatus = 'pending' | 'in-progress' | 'completed' | 'cancelled';

export type TaskPriority = 'low' | 'medium' | 'high';

export type TaskType = 'Story' | 'Bug' | 'Feature' | 'Task' | 'Epic' | 'Subtask';

// Actual TypeScript enums for direct use with Select component
export enum TaskStatusEnum {
	PENDING = 'pending',
	IN_PROGRESS = 'in-progress',
	COMPLETED = 'completed',
	CANCELLED = 'cancelled'
}

export enum TaskPriorityEnum {
	LOW = 'low',
	MEDIUM = 'medium',
	HIGH = 'high'
}

export enum TaskTypeEnum {
	STORY = 'Story',
	BUG = 'Bug',
	FEATURE = 'Feature',
	TASK = 'Task',
	EPIC = 'Epic',
	SUBTASK = 'Subtask'
}

// Recurring pattern enum
export enum RecurringPatternEnum {
	DAILY = 'daily',
	WEEKLY = 'weekly',
	MONTHLY = 'monthly',
	YEARLY = 'yearly'
}

// Enums for dropdown options
export const TASK_STATUS_OPTIONS = [
	{ value: 'pending' as const, label: 'To Do' },
	{ value: 'in-progress' as const, label: 'In Progress' },
	{ value: 'completed' as const, label: 'Completed' },
	{ value: 'cancelled' as const, label: 'Cancelled' }
] as const;

export const TASK_PRIORITY_OPTIONS = [
	{ value: 'low' as const, label: 'Low' },
	{ value: 'medium' as const, label: 'Medium' },
	{ value: 'high' as const, label: 'High' }
] as const;

export const TASK_TYPE_OPTIONS = [
	{ value: '' as const, label: 'Select type...' },
	{ value: 'Story' as const, label: 'Story' },
	{ value: 'Bug' as const, label: 'Bug' },
	{ value: 'Feature' as const, label: 'Feature' },
	{ value: 'Task' as const, label: 'Task' },
	{ value: 'Epic' as const, label: 'Epic' },
	{ value: 'Subtask' as const, label: 'Subtask' }
] as const;


export interface Task {
	id: string;
	title: string;
	description?: string;
	status: TaskStatus;
	priority: TaskPriority;
	type?: string; // Simple type field: "Story", "Bug", "Note", "Feature", etc.
	dueDate?: Date;
	createdAt: Date;
	updatedAt: Date;
	completedAt?: Date;
	parentId?: string;
	resourceId?: string;
	resourceType?: string;
	// New practical fields
	estimatedTime?: number; // in minutes
	actualTime?: number; // in minutes
	tags?: string[];
	assignee?: string;
	recurring?: RecurringTask;
	blockedBy?: string[]; // task IDs that block this task
	blocks?: string[]; // task IDs that this task blocks
	comments?: TaskComment[];
	attachments?: TaskAttachment[];
}


export interface CreateTaskRequest {
	title: string;
	description?: string;
	status: TaskStatus;
	priority: TaskPriority;
	type?: string;
	parentId?: string;
	resourceId?: string;
	resourceType?: string;
	dueDate?: Date;
	// New practical fields
	estimatedTime?: number;
	actualTime?: number;
	tags?: string[];
	assignee?: string;
	recurring?: RecurringTask;
	blockedBy?: string[];
	blocks?: string[];
}

export interface UpdateTaskRequest {
	title?: string;
	description?: string;
	status?: TaskStatus;
	priority?: TaskPriority;
	type?: string;
	parentId?: string;
	resourceId?: string;
	resourceType?: string;
	dueDate?: Date;
	// New practical fields
	estimatedTime?: number;
	actualTime?: number;
	tags?: string[];
	assignee?: string;
	recurring?: RecurringTask;
	blockedBy?: string[];
	blocks?: string[];
}


export interface TaskFilters {
	status?: TaskStatus[];
	priority?: TaskPriority[];
	type?: string[];
	parentId?: string;
	resourceId?: string;
	resourceType?: string;
	search?: string;
}

export interface TaskStats {
	total: number;
	pending: number;
	inProgress: number;
	completed: number;
	cancelled: number;
	completionPercentage: number;
}

export interface RecurringTask {
	pattern: 'daily' | 'weekly' | 'monthly' | 'yearly';
	interval: number; // every N days/weeks/months/years
	endDate?: Date;
	lastGenerated?: Date;
}

export interface TaskComment {
	id: string;
	taskId: string;
	content: string;
	author: string;
	createdAt: Date;
	updatedAt: Date;
}

export interface TaskAttachment {
	id: string;
	taskId: string;
	name: string;
	url: string;
	type: string;
	size: number;
	createdAt: Date;
}

export interface TaskTemplate {
	id: string;
	name: string;
	description?: string;
	defaultStatus: TaskStatus;
	defaultPriority: TaskPriority;
	defaultType?: string;
	defaultTags?: string[];
	defaultEstimatedTime?: number;
	createdAt: Date;
	updatedAt: Date;
}

export interface SavedView {
	id: string;
	name: string;
	description?: string;
	filters: TaskFilters;
	isDefault: boolean;
	createdAt: Date;
	updatedAt: Date;
}

export interface TimeTrackingSession {
	taskId: string;
	startTime: Date;
	endTime?: Date;
	duration?: number; // in minutes
	isActive: boolean;
}

export interface ProductivityMetrics {
	completionRate: number;
	timeAccuracy: number; // estimated vs actual time
	velocity: number; // tasks completed per week
	overdueCount: number;
	unestimatedCount: number;
	blockedCount: number;
}
