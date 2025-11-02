/**
 * Shared domain exports
 */

// Services
export { logger, createLogger, LogLevel } from './services/logger';
export { cache } from './services/cache';
export { eventBus, createEventBus } from './services/eventBus';

// Stores
export { themeStore, currentTheme, resolvedTheme } from './stores/themeStore';
export type { Theme } from './stores/themeStore';
export { 
	breadcrumbItems, 
	breadcrumbSettings, 
	breadcrumbActions,
	setBreadcrumbs,
	clearBreadcrumbs,
	setBreadcrumbConfig,
	resetBreadcrumbConfig
} from './stores/breadcrumbStore';
export type { BreadcrumbItem } from './stores/breadcrumbStore';
export { loadingState, loadingActions } from './stores/loadingState';
export type { LoadingState } from './stores/loadingState';

// Types
export type {
	BaseEntity,
	ApiResponse,
	PaginatedResponse,
	SortOptions,
	FilterOptions,
	QueryOptions,
	UserPreferences,
	SystemInfo,
	NotificationOptions,
	MenuItem,
	TabItem,
	DialogOptions,
	ToastOptions
} from './types';

// Utils
export {
	generateId,
	formatBytes,
	formatRelativeTime,
	debounce,
	throttle,
	deepClone,
	deepEqual,
	sleep,
	retry,
	isValidEmail,
	sanitizeFilename,
	getFileExtension,
	isEmpty,
	capitalize,
	kebabCase,
	camelCase,
	truncate,
	sortByCreatedAt,
	sortByUpdatedAt
} from './utils';