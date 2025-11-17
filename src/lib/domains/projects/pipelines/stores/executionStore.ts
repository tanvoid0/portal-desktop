/**
 * Execution Store - Svelte store for pipeline execution state
 */

import { writable, derived } from 'svelte/store';
import type { PipelineExecution, StepExecution } from '../types';
import { executionService } from '../services/executionService';

interface ExecutionState {
	executions: Map<string, PipelineExecution>; // executionId -> execution
	currentExecution: PipelineExecution | null;
	loading: boolean;
	error: string | null;
}

const initialState: ExecutionState = {
	executions: new Map(),
	currentExecution: null,
	loading: false,
	error: null,
};

function createExecutionStore() {
	const { subscribe, set, update } = writable<ExecutionState>(initialState);

	return {
		subscribe,

		/**
		 * Add or update execution
		 */
		updateExecution(execution: PipelineExecution) {
			update((state) => {
				const newExecutions = new Map(state.executions);
				newExecutions.set(execution.id, execution);
				return {
					...state,
					executions: newExecutions,
					currentExecution:
						state.currentExecution?.id === execution.id
							? execution
							: state.currentExecution,
				};
			});
		},

		/**
		 * Set current execution
		 */
		setCurrentExecution(execution: PipelineExecution | null) {
			update((state) => ({ ...state, currentExecution: execution }));
		},

		/**
		 * Get execution by ID
		 */
		getExecution(executionId: string): PipelineExecution | null {
			let result: PipelineExecution | null = null;
			update((state) => {
				result = state.executions.get(executionId) || null;
				return state;
			});
			return result;
		},

		/**
		 * Clear error
		 */
		clearError() {
			update((state) => ({ ...state, error: null }));
		},

		/**
		 * Reset store
		 */
		reset() {
			set(initialState);
		},
	};
}

export const executionStore = createExecutionStore();

// Derived stores
export const currentExecution = derived(executionStore, ($store) => $store.currentExecution);
export const executionLoading = derived(executionStore, ($store) => $store.loading);
export const executionError = derived(executionStore, ($store) => $store.error);

