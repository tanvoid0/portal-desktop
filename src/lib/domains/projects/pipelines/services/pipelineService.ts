/**
 * Pipeline Service - Frontend business logic for pipeline management
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '@/lib/domains/shared';
import type {
	Pipeline,
	CreatePipelineRequest,
	UpdatePipelineRequest,
	PipelineExecution,
	ExecutePipelineRequest,
} from '../types';

const log = logger.createScoped('PipelineService');

export class PipelineService {
	private static instance: PipelineService;

	static getInstance(): PipelineService {
		if (!PipelineService.instance) {
			PipelineService.instance = new PipelineService();
		}
		return PipelineService.instance;
	}

	/**
	 * Get all pipelines for a project
	 */
	async getPipelines(projectId: string): Promise<Pipeline[]> {
		try {
			log.info('Loading pipelines', { projectId });
			const pipelines = await invoke<Pipeline[]>('get_pipelines', { projectId });
			log.info('Pipelines loaded', { projectId, count: pipelines.length });
			return pipelines;
		} catch (error) {
			log.error('Failed to load pipelines', error);
			throw error;
		}
	}

	/**
	 * Get a specific pipeline by ID
	 */
	async getPipeline(pipelineId: string): Promise<Pipeline | null> {
		try {
			log.info('Loading pipeline', { pipelineId });
			const pipeline = await invoke<Pipeline | null>('get_pipeline', { pipelineId });
			log.info('Pipeline loaded', { pipelineId, found: !!pipeline });
			return pipeline;
		} catch (error) {
			log.error('Failed to load pipeline', error);
			throw error;
		}
	}

	/**
	 * Create a new pipeline
	 */
	async createPipeline(request: CreatePipelineRequest): Promise<Pipeline> {
		try {
			log.info('Creating pipeline', { name: request.name, projectId: request.projectId });
			const pipeline = await invoke<Pipeline>('create_pipeline', { request });
			log.info('Pipeline created', { id: pipeline.id });
			return pipeline;
		} catch (error) {
			log.error('Failed to create pipeline', error);
			throw error;
		}
	}

	/**
	 * Update an existing pipeline
	 */
	async updatePipeline(pipelineId: string, request: UpdatePipelineRequest): Promise<Pipeline> {
		try {
			log.info('Updating pipeline', { pipelineId });
			const pipeline = await invoke<Pipeline>('update_pipeline', { pipelineId, request });
			log.info('Pipeline updated', { pipelineId });
			return pipeline;
		} catch (error) {
			log.error('Failed to update pipeline', error);
			throw error;
		}
	}

	/**
	 * Delete a pipeline
	 */
	async deletePipeline(pipelineId: string): Promise<void> {
		try {
			log.info('Deleting pipeline', { pipelineId });
			await invoke('delete_pipeline', { pipelineId });
			log.info('Pipeline deleted', { pipelineId });
		} catch (error) {
			log.error('Failed to delete pipeline', error);
			throw error;
		}
	}

	/**
	 * Execute a pipeline
	 */
	async executePipeline(request: ExecutePipelineRequest): Promise<PipelineExecution> {
		try {
			log.info('Executing pipeline', { pipelineId: request.pipelineId });
			const execution = await invoke<PipelineExecution>('execute_pipeline', { request });
			log.info('Pipeline execution started', { executionId: execution.id });
			return execution;
		} catch (error) {
			log.error('Failed to execute pipeline', error);
			throw error;
		}
	}

	/**
	 * Get pipeline execution by ID
	 */
	async getExecution(executionId: string): Promise<PipelineExecution | null> {
		try {
			log.info('Loading execution', { executionId });
			const execution = await invoke<PipelineExecution | null>('get_pipeline_execution', {
				executionId,
			});
			return execution;
		} catch (error) {
			log.error('Failed to load execution', error);
			throw error;
		}
	}

	/**
	 * Get all executions for a pipeline
	 */
	async getExecutions(pipelineId: string): Promise<PipelineExecution[]> {
		try {
			log.info('Loading executions', { pipelineId });
			const executions = await invoke<PipelineExecution[]>('get_pipeline_executions', {
				pipelineId,
			});
			return executions;
		} catch (error) {
			log.error('Failed to load executions', error);
			throw error;
		}
	}

	/**
	 * Cancel a running pipeline execution
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
	 * Duplicate a pipeline
	 */
	async duplicatePipeline(pipelineId: string, newName: string): Promise<Pipeline> {
		try {
			log.info('Duplicating pipeline', { pipelineId, newName });
			const pipeline = await invoke<Pipeline>('duplicate_pipeline', {
				pipelineId,
				newName,
			});
			log.info('Pipeline duplicated', { originalId: pipelineId, newId: pipeline.id });
			return pipeline;
		} catch (error) {
			log.error('Failed to duplicate pipeline', error);
			throw error;
		}
	}

	/**
	 * Enable or disable a pipeline
	 */
	async setPipelineEnabled(pipelineId: string, enabled: boolean): Promise<void> {
		try {
			log.info('Setting pipeline enabled state', { pipelineId, enabled });
			await invoke('set_pipeline_enabled', { pipelineId, enabled });
			log.info('Pipeline enabled state updated', { pipelineId, enabled });
		} catch (error) {
			log.error('Failed to set pipeline enabled state', error);
			throw error;
		}
	}
}

export const pipelineService = PipelineService.getInstance();

