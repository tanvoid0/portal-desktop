/**
 * Loading state management utilities
 */

import { writable } from 'svelte/store';

export interface LoadingState {
	isLoading: boolean;
	message?: string;
	progress?: number;
	error?: string | null;
}

const initialState: LoadingState = {
	isLoading: false
};

export const loadingState = writable<LoadingState>(initialState);

export const loadingActions = {
	setLoading: (isLoading: boolean, message?: string, progress?: number) => {
		loadingState.update(state => ({
			...state,
			isLoading,
			message,
			progress
		}));
	},

	setError: (error: string | null) => {
		loadingState.update(state => ({
			...state,
			error
		}));
	},

	startLoading: (message?: string) => {
		loadingActions.setLoading(true, message);
	},

	stopLoading: () => {
		loadingActions.setLoading(false);
	},

	updateProgress: (progress: number, message?: string) => {
		loadingState.update(state => ({
			...state,
			progress,
			message: message || state.message
		}));
	}
};
