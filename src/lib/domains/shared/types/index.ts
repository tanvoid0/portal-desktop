/**
 * Shared types for Portal Desktop
 */

export interface BaseEntity {
	id: string;
	createdAt: Date;
	updatedAt: Date;
}

export interface ApiResponse<T = unknown> {
	success: boolean;
	data?: T;
	error?: string;
	message?: string;
}

export interface PaginatedResponse<T = unknown> extends ApiResponse<T[]> {
	pagination: {
		page: number;
		limit: number;
		total: number;
		totalPages: number;
	};
}

export interface SortOptions {
	field: string;
	direction: 'asc' | 'desc';
}

export interface FilterOptions {
	[key: string]: unknown;
}

export interface QueryOptions {
	page?: number;
	limit?: number;
	sort?: SortOptions;
	filter?: FilterOptions;
	search?: string;
}

export interface UserPreferences {
	theme: 'light' | 'dark' | 'system';
	language: string;
	notifications: {
		enabled: boolean;
		sound: boolean;
		desktop: boolean;
	};
	terminal: {
		fontSize: number;
		fontFamily: string;
		theme: string;
	};
	projects: {
		defaultPath: string;
		autoSave: boolean;
	};
}

export interface SystemInfo {
	platform: string;
	arch: string;
	version: string;
	hostname: string;
	username: string;
	homeDir: string;
}

export interface NotificationOptions {
	title: string;
	message: string;
	type?: 'info' | 'success' | 'warning' | 'error';
	duration?: number;
	actions?: Array<{
		label: string;
		action: () => void;
	}>;
}

export interface MenuItem {
	id: string;
	label: string;
	icon?: string;
	action?: () => void;
	children?: MenuItem[];
	disabled?: boolean;
	separator?: boolean;
}

export interface TabItem {
	id: string;
	title: string;
	icon?: string;
	closable?: boolean;
	data?: unknown;
}

export interface DialogOptions {
	title: string;
	message: string;
	type?: 'info' | 'warning' | 'error' | 'confirm';
	buttons?: Array<{
		label: string;
		action: () => void;
		variant?: 'primary' | 'secondary' | 'destructive';
	}>;
}

export interface ToastOptions {
	title: string;
	description?: string;
	type?: 'info' | 'success' | 'warning' | 'error';
	duration?: number;
	action?: {
		label: string;
		onClick: () => void;
	};
}