/**
 * Deployment Store - State management for deployments
 */

import { writable, derived, get } from 'svelte/store';
import { deploymentService } from '../services/deploymentService';
import { logger, loadingState, loadingActions } from '$lib/domains/shared';
import type { 
	Deployment, 
	DockerContainer,
	DeploymentCreateRequest,
	DeploymentUpdateRequest
} from '../types/index';
import { DeploymentStatus } from '../types/index';

// Core stores
export const deployments = writable<Deployment[]>([]);
export const containers = writable<DockerContainer[]>([]);
export const deploymentLoadingState = loadingState;

// Derived stores
export const isLoadingDeployments = derived(deploymentLoadingState, ($state) => $state.isLoading);
export const deploymentError = derived(deploymentLoadingState, ($state) => $state.error);

// Deployment statistics
export const deploymentStats = derived(deployments, ($deployments) => {
	const stats = {
		total: $deployments.length,
		running: $deployments.filter(d => d.status === 'running').length,
		stopped: $deployments.filter(d => d.status === 'stopped').length,
		building: $deployments.filter(d => d.status === 'creating').length,
		error: $deployments.filter(d => d.status === 'failed').length,
		byProject: {} as Record<string, number>
	};

	// Count by project
	$deployments.forEach(deployment => {
		stats.byProject[deployment.projectPath] = (stats.byProject[deployment.projectPath] || 0) + 1;
	});

	return stats;
});

// Filtered deployments
export const filteredDeployments = derived(
	[deployments],
	([$deployments]) => $deployments
);

// Deployment actions
export const deploymentActions = {
	/**
	 * Load all deployments
	 */
	async loadDeployments() {
		loadingActions.setLoading(true);
		try {
			logger.info('deploymentStore', 'Loading deployments');
			
			const deploymentList = await deploymentService.getDeployments();
			deployments.set(deploymentList);
			
			logger.info('Deployments loaded successfully', { 
				context: 'deploymentStore',
				count: deploymentList.length 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to load deployments';
			logger.error(errorMessage, { context: 'deploymentStore', error });
			loadingActions.setError(errorMessage);
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Create a new deployment
	 */
	async createDeployment(request: DeploymentCreateRequest) {
		loadingActions.setLoading(true);
		try {
			logger.info('Creating deployment', { 
				context: 'deploymentStore',
				name: request.name, 
				projectPath: request.projectPath 
			});
			
			const deployment = await deploymentService.createDeployment(request);
			
			// Add to local store
			deployments.update(current => [...current, deployment]);
			
			logger.info('Deployment created successfully', { 
				context: 'deploymentStore',
				deploymentId: deployment.id 
			});
			
			return deployment;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to create deployment';
			logger.error(errorMessage, { 
				context: 'deploymentStore.createDeployment',
				error,
				name: request.name, 
				projectPath: request.projectPath
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Start a deployment
	 */
	async startDeployment(deploymentId: string) {
		loadingActions.setLoading(true);
		try {
			logger.info('Starting deployment', { context: 'deploymentStore', deploymentId });
			
			const deployment = await deploymentService.startDeployment(deploymentId);
			
			// Update in local store
			deployments.update(current => 
				current.map(d => d.id === deploymentId ? deployment : d)
			);
			
			logger.info('Deployment started successfully', { context: 'deploymentStore', deploymentId });
			
			return deployment;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to start deployment';
			logger.error(errorMessage, { 
				context: 'deploymentStore.startDeployment', 
				error,
				deploymentId
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Stop a deployment
	 */
	async stopDeployment(deploymentId: string) {
		loadingActions.setLoading(true);
		try {
			logger.info('Stopping deployment', { 
				context: 'deploymentStore', 
				data: { deploymentId } 
			});
			
			const deployment = await deploymentService.stopDeployment(deploymentId);
			
			// Update in local store
			deployments.update(current => 
				current.map(d => d.id === deploymentId ? deployment : d)
			);
			
			logger.info('Deployment stopped successfully', { 
				context: 'deploymentStore', 
				data: { deploymentId } 
			});
			
			return deployment;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to stop deployment';
			logger.error(errorMessage, { 
				context: 'deploymentStore.stopDeployment', 
				error,
				data: { deploymentId }
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Delete a deployment
	 */
	async deleteDeployment(deploymentId: string) {
		loadingActions.setLoading(true);
		try {
			logger.info('Deleting deployment', { 
				context: 'deploymentStore', 
				data: { deploymentId } 
			});
			
			await deploymentService.deleteDeployment(deploymentId);
			
			// Remove from local store
			deployments.update(current => 
				current.filter(d => d.id !== deploymentId)
			);
			
			logger.info('Deployment deleted successfully', { 
				context: 'deploymentStore', 
				data: { deploymentId } 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to delete deployment';
			logger.error(errorMessage, { 
				context: 'deploymentStore.deleteDeployment', 
				error,
				data: { deploymentId }
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Update a deployment
	 */
	async updateDeployment(deploymentId: string, request: DeploymentUpdateRequest) {
		loadingActions.setLoading(true);
		try {
			logger.info('Updating deployment', { 
				context: 'deploymentStore', 
				data: { deploymentId } 
			});
			
			const deployment = await deploymentService.updateDeployment(deploymentId, request);
			
			// Update in local store
			deployments.update(current => 
				current.map(d => d.id === deploymentId ? deployment : d)
			);
			
			logger.info('Deployment updated successfully', { 
				context: 'deploymentStore', 
				data: { deploymentId } 
			});
			
			return deployment;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to update deployment';
			logger.error(errorMessage, { 
				context: 'deploymentStore.updateDeployment', 
				error,
				data: { deploymentId }
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Get deployment logs
	 */
	async getDeploymentLogs(deploymentId: string, tail?: number) {
		try {
			logger.info('Getting deployment logs', { 
				context: 'deploymentStore', 
				data: { deploymentId, tail } 
			});
			
			const logs = await deploymentService.getDeploymentLogs(deploymentId, tail);
			
			logger.info('Deployment logs retrieved', { 
				context: 'deploymentStore', 
				data: { deploymentId, logCount: logs.length } 
			});
			
			return logs;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to get deployment logs';
			logger.error(errorMessage, { 
				context: 'deploymentStore.getDeploymentLogs', 
				error,
				data: { deploymentId }
			});
			throw error;
		}
	},

	/**
	 * Refresh deployment statuses
	 */
	async refreshDeploymentStatuses() {
		loadingActions.setLoading(true);
		try {
			logger.info('Refreshing deployment statuses', { context: 'deploymentStore' });
			
			const updatedDeployments = await deploymentService.refreshDeploymentStatuses();
			deployments.set(updatedDeployments);
			
			logger.info('Deployment statuses refreshed', { 
				context: 'deploymentStore', 
				data: { count: updatedDeployments.length } 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to refresh deployment statuses';
			logger.error(errorMessage, { 
				context: 'deploymentStore.refreshDeploymentStatuses', 
				error 
			});
			loadingActions.setError(errorMessage);
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Load containers
	 */
	async loadContainers() {
		loadingActions.setLoading(true);
		try {
			logger.info('Loading containers', { context: 'deploymentStore' });
			
			const containerList = await deploymentService.listContainers();
			containers.set(containerList);
			
			logger.info('Containers loaded', { 
				context: 'deploymentStore', 
				data: { count: containerList.length } 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to load containers';
			logger.error(errorMessage, { 
				context: 'deploymentStore.loadContainers', 
				error 
			});
			loadingActions.setError(errorMessage);
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Get deployment by ID
	 */
	getDeploymentById(deploymentId: string): Deployment | undefined {
		let foundDeployment: Deployment | undefined;
		deployments.update(current => {
			foundDeployment = current.find(d => d.id === deploymentId);
			return current;
		});
		return foundDeployment;
	},

	/**
	 * Get deployments by project ID
	 */
	getDeploymentsByProject(projectId: string): Deployment[] {
		let projectDeployments: Deployment[] = [];
		deployments.update(current => {
			projectDeployments = current.filter(d => d.metadata.projectId === projectId);
			return current;
		});
		return projectDeployments;
	},

	/**
	 * Clear error state
	 */
	clearError() {
		loadingActions.setError(null);
	},

	/**
	 * Poll CLI deployment statuses
	 */
	async pollCliDeploymentStatuses(intervalMs: number = 5000) {
		const currentDeployments = get(deployments);
		const cliDeployments = currentDeployments.filter((d: Deployment) => d.type === 'cli' && d.status === 'running');
		
		for (const deployment of cliDeployments) {
			try {
				const isRunning = await deploymentService.getProcessStatus(deployment.id);
				if (!isRunning && deployment.status === 'running') {
					// Process stopped, update status
					deployments.update((current: Deployment[]) =>
						current.map((d: Deployment) => 
							d.id === deployment.id 
								? { ...d, status: DeploymentStatus.STOPPED }
								: d
						)
					);
				}
			} catch (error) {
				logger.error('Failed to poll CLI deployment status', {
					context: 'deploymentStore',
					error,
					deploymentId: deployment.id
				});
			}
		}
	}
};
