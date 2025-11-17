/**
 * Execution Service - Pipeline execution management
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { logger } from '@/lib/domains/shared';
import type {
	PipelineExecution,
	ExecutePipelineRequest,
	StepExecution,
	ExecutionStatus,
} from '../types';
import { resolveDependencies, validateDependencies } from '../utils/dependencyResolver';

const log = logger.createScoped('ExecutionService');

export class ExecutionService {
	private static instance: ExecutionService;
	private executionListeners = new Map<string, (execution: PipelineExecution) => void>();

	static getInstance(): ExecutionService {
		if (!ExecutionService.instance) {
			ExecutionService.instance = new ExecutionService();
			ExecutionService.instance.setupEventListeners();
		}
		return ExecutionService.instance;
	}

	/**
	 * Setup event listeners for execution updates
	 */
	private async setupEventListeners(): Promise<void> {
		try {
			await listen<PipelineExecution>('pipeline-execution-update', (event) => {
				const execution = event.payload;
				const listener = this.executionListeners.get(execution.id);
				if (listener) {
					listener(execution);
				}
			});
		} catch (error) {
			log.error('Failed to setup execution event listeners', error);
		}
	}

	/**
	 * Execute a pipeline
	 */
	async executePipeline(request: ExecutePipelineRequest): Promise<PipelineExecution> {
		try {
			log.info('Executing pipeline', { pipelineId: request.pipelineId });

			// Validate pipeline before execution
			const pipeline = await invoke<any>('get_pipeline', {
				pipelineId: request.pipelineId,
			});
			if (!pipeline) {
				throw new Error('Pipeline not found');
			}

			const validation = validateDependencies(pipeline.steps);
			if (!validation.valid) {
				throw new Error(`Pipeline validation failed: ${validation.errors.join(', ')}`);
			}

			const execution = await invoke<PipelineExecution>('execute_pipeline', { request });
			log.info('Pipeline execution started', { executionId: execution.id });
			return execution;
		} catch (error) {
			log.error('Failed to execute pipeline', error);
			throw error;
		}
	}

	/**
	 * Get execution status
	 */
	async getExecution(executionId: string): Promise<PipelineExecution | null> {
		try {
			const execution = await invoke<PipelineExecution | null>('get_pipeline_execution', {
				executionId,
			});
			return execution;
		} catch (error) {
			log.error('Failed to get execution', error);
			throw error;
		}
	}

	/**
	 * Cancel a running execution
	 */
	async cancelExecution(executionId: string): Promise<void> {
		try {
			log.info('Cancelling execution', { executionId });
			await invoke('cancel_pipeline_execution', { executionId });
			log.info('Execution cancelled', { executionId });
		} catch (error) {
			log.error('Failed to cancel execution', error);
			throw error;
		}
	}

	/**
	 * Subscribe to execution updates
	 */
	subscribeToExecution(
		executionId: string,
		callback: (execution: PipelineExecution) => void
	): () => void {
		this.executionListeners.set(executionId, callback);
		return () => {
			this.executionListeners.delete(executionId);
		};
	}

	/**
	 * Get execution logs for a step
	 */
	async getStepLogs(executionId: string, stepId: string): Promise<string[]> {
		try {
			const logs = await invoke<string[]>('get_step_execution_logs', {
				executionId,
				stepId,
			});
			return logs;
		} catch (error) {
			log.error('Failed to get step logs', error);
			return [];
		}
	}

	/**
	 * Retry a failed step
	 */
	async retryStep(executionId: string, stepId: string): Promise<void> {
		try {
			log.info('Retrying step', { executionId, stepId });
			await invoke('retry_step_execution', { executionId, stepId });
			log.info('Step retry initiated', { executionId, stepId });
		} catch (error) {
			log.error('Failed to retry step', error);
			throw error;
		}
	}
}

export const executionService = ExecutionService.getInstance();

