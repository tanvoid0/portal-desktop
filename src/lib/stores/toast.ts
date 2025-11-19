/**
 * Production-ready Toast Store
 * Manages toast notifications across the application
 */

import { writable } from 'svelte/store';

export interface Toast {
	id: string;
	title?: string;
	description?: string;
	variant?: 'default' | 'success' | 'error' | 'warning' | 'info';
	duration?: number;
	action?: {
		label: string;
		onClick: () => void;
	};
}

interface ToastState {
	toasts: Toast[];
}

// Create the toast store
function createToastStore() {
	const { subscribe, set, update } = writable<ToastState>({
		toasts: []
	});

	return {
		subscribe,
		
		// Add a new toast
		add(toast: Omit<Toast, 'id'>) {
			const id = Math.random().toString(36).slice(2, 11);
			const newToast: Toast = {
				id,
				duration: 5000,
				...toast
			};

			update(state => ({
				...state,
				toasts: [...state.toasts, newToast]
			}));

			// Auto-remove toast after duration
			if (newToast.duration && newToast.duration > 0) {
				setTimeout(() => {
					this.remove(id);
				}, newToast.duration);
			}

			return id;
		},

		// Remove a toast by id
		remove(id: string) {
			update(state => ({
				...state,
				toasts: state.toasts.filter(toast => toast.id !== id)
			}));
		},

		// Clear all toasts
		clear() {
			update(state => ({
				...state,
				toasts: []
			}));
		},

		// Success toast
		success(title: string, description?: string, options?: Partial<Toast>) {
			return this.add({
				title,
				description,
				variant: 'success',
				...options
			});
		},

		// Error toast
		error(title: string, description?: string, options?: Partial<Toast>) {
			return this.add({
				title,
				description,
				variant: 'error',
				duration: 7000, // Longer duration for errors
				...options
			});
		},

		// Warning toast
		warning(title: string, description?: string, options?: Partial<Toast>) {
			return this.add({
				title,
				description,
				variant: 'warning',
				...options
			});
		},

		// Info toast
		info(title: string, description?: string, options?: Partial<Toast>) {
			return this.add({
				title,
				description,
				variant: 'info',
				...options
			});
		},

		// Promise toast - shows loading then success/error
		promise<T>(
			promise: Promise<T>,
			{
				loading,
				success,
				error
			}: {
				loading: string;
				success: string | ((data: T) => string);
				error: string | ((error: any) => string);
			}
		) {
			const loadingId = this.add({
				title: loading,
				variant: 'info',
				duration: 0 // Don't auto-dismiss loading toasts
			});

			promise
				.then(data => {
					this.remove(loadingId);
					this.success(
						typeof success === 'function' ? success(data) : success
					);
				})
				.catch(err => {
					this.remove(loadingId);
					this.error(
						typeof error === 'function' ? error(err) : error
					);
				});

			return promise;
		}
	};
}

// Create the store instance
export const toastStore = createToastStore();

// Convenience functions
export const toast = {
	add: (toast: Omit<Toast, 'id'>) => toastStore.add(toast),
	remove: (id: string) => toastStore.remove(id),
	clear: () => toastStore.clear(),
	success: (title: string, description?: string, options?: Partial<Toast>) => 
		toastStore.success(title, description, options),
	error: (title: string, description?: string, options?: Partial<Toast>) => 
		toastStore.error(title, description, options),
	warning: (title: string, description?: string, options?: Partial<Toast>) => 
		toastStore.warning(title, description, options),
	info: (title: string, description?: string, options?: Partial<Toast>) => 
		toastStore.info(title, description, options),
	promise: <T>(
		promise: Promise<T>,
		options: {
			loading: string;
			success: string | ((data: T) => string);
			error: string | ((error: any) => string);
		}
	) => toastStore.promise(promise, options)
};
