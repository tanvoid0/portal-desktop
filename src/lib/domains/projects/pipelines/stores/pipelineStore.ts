/**
 * Pipeline Store - Svelte store for pipeline state management
 */

import { writable, derived } from 'svelte/store';
import type { Pipeline, PipelineExecution } from '../types';
import { pipelineService } from '../services/pipelineService';

interface PipelineState {
	pipelines: Pipeline[];
	executions: Map<string, PipelineExecution[]>; // pipelineId -> executions
	currentPipeline: Pipeline | null;
	currentExecution: PipelineExecution | null;
	loading: boolean;
	error: string | null;
}

const initialState: PipelineState = {
	pipelines: [],
	executions: new Map(),
	currentPipeline: null,
	currentExecution: null,
	loading: false,
	error: null,
};

function createPipelineStore() {
	const { subscribe, set, update } = writable<PipelineState>(initialState);

	return {
		subscribe,

		/**
		 * Load pipelines for a project
		 */
		async loadPipelines(projectId: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				const pipelines = await pipelineService.getPipelines(projectId);
				update((state) => ({
					...state,
					pipelines,
					loading: false,
				}));
			} catch (error) {
				update((state) => ({
					...state,
					error: error instanceof Error ? error.message : 'Failed to load pipelines',
					loading: false,
				}));
			}
		},

		/**
		 * Set current pipeline
		 */
		setCurrentPipeline(pipeline: Pipeline | null) {
			update((state) => ({ ...state, currentPipeline: pipeline }));
		},

		/**
		 * Add or update a pipeline
		 */
		upsertPipeline(pipeline: Pipeline) {
			update((state) => {
				const index = state.pipelines.findIndex((p) => p.id === pipeline.id);
				if (index >= 0) {
					const pipelines = [...state.pipelines];
					pipelines[index] = pipeline;
					return { ...state, pipelines };
				} else {
					return { ...state, pipelines: [...state.pipelines, pipeline] };
				}
			});
		},

		/**
		 * Remove a pipeline
		 */
		removePipeline(pipelineId: string) {
			update((state) => ({
				...state,
				pipelines: state.pipelines.filter((p) => p.id !== pipelineId),
				currentPipeline:
					state.currentPipeline?.id === pipelineId ? null : state.currentPipeline,
			}));
		},

		/**
		 * Load executions for a pipeline
		 */
		async loadExecutions(pipelineId: string) {
			try {
				const executions = await pipelineService.getExecutions(pipelineId);
				update((state) => {
					const newExecutions = new Map(state.executions);
					newExecutions.set(pipelineId, executions);
					return { ...state, executions: newExecutions };
				});
			} catch (error) {
				update((state) => ({
					...state,
					error: error instanceof Error ? error.message : 'Failed to load executions',
				}));
			}
		},

		/**
		 * Set current execution
		 */
		setCurrentExecution(execution: PipelineExecution | null) {
			update((state) => ({ ...state, currentExecution: execution }));
		},

		/**
		 * Update execution in store
		 */
		updateExecution(execution: PipelineExecution) {
			update((state) => {
				const executions = state.executions.get(execution.pipelineId) || [];
				const index = executions.findIndex((e) => e.id === execution.id);
				if (index >= 0) {
					const updated = [...executions];
					updated[index] = execution;
					const newExecutions = new Map(state.executions);
					newExecutions.set(execution.pipelineId, updated);
					return {
						...state,
						executions: newExecutions,
						currentExecution:
							state.currentExecution?.id === execution.id
								? execution
								: state.currentExecution,
					};
				} else {
					const newExecutions = new Map(state.executions);
					newExecutions.set(execution.pipelineId, [...executions, execution]);
					return {
						...state,
						executions: newExecutions,
					};
				}
			});
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

export const pipelineStore = createPipelineStore();

// Derived stores
export const pipelines = derived(pipelineStore, ($store) => $store.pipelines);
export const currentPipeline = derived(pipelineStore, ($store) => $store.currentPipeline);
export const currentExecution = derived(pipelineStore, ($store) => $store.currentExecution);
export const isLoading = derived(pipelineStore, ($store) => $store.loading);
export const pipelineError = derived(pipelineStore, ($store) => $store.error);

