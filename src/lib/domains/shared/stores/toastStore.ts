import { writable } from 'svelte/store';
import { createLogger } from '../services/logger';

export interface Toast {
	id: string;
	title?: string;
	description?: string;
	type?: 'success' | 'error' | 'warning' | 'info';
	duration?: number;
	action?: {
		label: string;
		onClick: () => void;
	};
}

interface ToastStore {
	toasts: Toast[];
}

const initialState: ToastStore = {
	toasts: []
};

export const toastStore = writable<ToastStore>(initialState);

// Create a scoped logger for toast operations
const toastLogger = createLogger('ToastStore');

/**
 * Extract error message from various error types
 */
function extractErrorMessage(error: unknown, fallback: string): string {
	if (error instanceof Error) {
		return error.message || fallback;
	}
	if (typeof error === 'string') {
		return error || fallback;
	}
	if (error && typeof error === 'object' && 'message' in error) {
		return String(error.message) || fallback;
	}
	return fallback;
}

export const toastActions = {
	show(toast: Omit<Toast, 'id'>) {
		const id = Math.random().toString(36).slice(2, 11);
		const newToast: Toast = {
			...toast,
			id
		};

		toastStore.update(state => ({
			...state,
			toasts: [...state.toasts, newToast]
		}));

		return id;
	},

	dismiss(id: string) {
		toastStore.update(state => ({
			...state,
			toasts: state.toasts.filter(toast => toast.id !== id)
		}));
	},

	success(title: string, description?: string, options?: Partial<Toast>) {
		toastLogger.info(`Toast: ${title}`, { type: 'success', description, ...options });
		return this.show({
			title,
			description,
			type: 'success',
			...options
		});
	},

	error(title: string, errorOrDescription?: unknown, options?: Partial<Toast>) {
		// Extract description from error or use provided description
		const description = errorOrDescription
			? extractErrorMessage(errorOrDescription, typeof errorOrDescription === 'string' ? errorOrDescription : '')
			: undefined;

		// Always log errors with full context
		toastLogger.error(`Toast Error: ${title}`, {
			description,
			error: errorOrDescription,
			...options
		});

		return this.show({
			title,
			description,
			type: 'error',
			duration: 0, // Don't auto-dismiss errors
			...options
		});
	},

	warning(title: string, description?: string, options?: Partial<Toast>) {
		toastLogger.warn(`Toast: ${title}`, { type: 'warning', description, ...options });
		return this.show({
			title,
			description,
			type: 'warning',
			...options
		});
	},

	info(title: string, description?: string, options?: Partial<Toast>) {
		toastLogger.info(`Toast: ${title}`, { type: 'info', description, ...options });
		return this.show({
			title,
			description,
			type: 'info',
			...options
		});
	},

	clear() {
		toastLogger.info('Toasts cleared');
		toastStore.set({ toasts: [] });
	}
};

// Export toast as an alias for toastActions
export const toast = toastActions;
