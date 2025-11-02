/**
 * Deployment Service - Frontend business logic for deployment management
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '$lib/domains/shared';
import type { 
	Deployment, 
	DockerContainer,
	DeploymentCreateRequest,
	DeploymentUpdateRequest
} from '../types';
import { DeploymentStatus } from '../types';

export class DeploymentService {
	private static instance: DeploymentService;

	static getInstance(): DeploymentService {
		if (!DeploymentService.instance) {
			DeploymentService.instance = new DeploymentService();
		}
		return DeploymentService.instance;
	}

	/**
	 * Create a new deployment
	 */
	async createDeployment(request: DeploymentCreateRequest): Promise<Deployment> {
		try {
			logger.info('Creating deployment', { context: 'DeploymentService',  
				name: request.name, 
				projectPath: request.projectPath 
			});
			
			const deployment = await invoke<Deployment>('create_deployment_command', {
				request
			});
			
			logger.info('Deployment created successfully', { context: 'DeploymentService',  
				deploymentId: deployment.id 
			});
			
			return deployment;
		} catch (error) {
			logger.error('Failed to create deployment', {
				context: 'DeploymentService',
				error,
				data: { name: request.name, projectPath: request.projectPath }
			});
			throw error;
		}
	}

	/**
	 * Get all deployments
	 */
	async getDeployments(): Promise<Deployment[]> {
		try {
			logger.info('Getting deployments', { context: 'DeploymentService' });
			
			const deployments = await invoke<Deployment[]>('get_deployments_command');
			
			logger.info('Deployments retrieved', { 
				context: 'DeploymentService', 
				data: { count: deployments.length } 
			});
			
			return deployments;
		} catch (error) {
			logger.error('Failed to get deployments', {
				context: 'DeploymentService',
				error
			});
			throw error;
		}
	}

	/**
	 * Get deployment by ID
	 */
	async getDeployment(deploymentId: string): Promise<Deployment> {
		try {
			logger.info('Getting deployment', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			const deployment = await invoke<Deployment>('get_deployment_command', { 
				deploymentId 
			});
			
			logger.info('Deployment retrieved', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			return deployment;
		} catch (error) {
			logger.error('Failed to get deployment', {
				context: 'DeploymentService',
				error,
				data: { deploymentId }
			});
			throw error;
		}
	}

	/**
	 * Start a deployment
	 */
	async startDeployment(deploymentId: string): Promise<Deployment> {
		try {
			logger.info('Starting deployment', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			const deployment = await invoke<Deployment>('start_deployment_command', { 
				deploymentId 
			});
			
			logger.info('Deployment started successfully', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			return deployment;
		} catch (error) {
			logger.error('Failed to start deployment', {
				context: 'DeploymentService',
				error,
				data: { deploymentId }
			});
			throw error;
		}
	}

	/**
	 * Stop a deployment
	 */
	async stopDeployment(deploymentId: string): Promise<Deployment> {
		try {
			logger.info('Stopping deployment', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			const deployment = await invoke<Deployment>('stop_deployment_command', { 
				deploymentId 
			});
			
			logger.info('Deployment stopped successfully', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			return deployment;
		} catch (error) {
			logger.error('Failed to stop deployment', {
				context: 'DeploymentService',
				error,
				data: { deploymentId }
			});
			throw error;
		}
	}

	/**
	 * Delete a deployment
	 */
	async deleteDeployment(deploymentId: string): Promise<void> {
		try {
			logger.info('Deleting deployment', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			await invoke('delete_deployment_command', { deploymentId });
			
			logger.info('Deployment deleted successfully', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
		} catch (error) {
			logger.error('Failed to delete deployment', {
				context: 'DeploymentService',
				error,
				data: { deploymentId }
			});
			throw error;
		}
	}

	/**
	 * Update a deployment
	 */
	async updateDeployment(deploymentId: string, request: DeploymentUpdateRequest): Promise<Deployment> {
		try {
			logger.info('Updating deployment', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			const deployment = await invoke<Deployment>('update_deployment_command', {
				deploymentId,
				request
			});
			
			logger.info('Deployment updated successfully', { 
				context: 'DeploymentService', 
				data: { deploymentId } 
			});
			
			return deployment;
		} catch (error) {
			logger.error('Failed to update deployment', {
				context: 'DeploymentService',
				error,
				data: { deploymentId }
			});
			throw error;
		}
	}

	/**
	 * Get deployment logs
	 */
	async getDeploymentLogs(deploymentId: string, tail?: number): Promise<string[]> {
		try {
			logger.info('Getting deployment logs', { 
				context: 'DeploymentService', 
				data: { deploymentId, tail } 
			});
			
			const logs = await invoke<string[]>('get_deployment_logs_command', { 
				deploymentId,
				tail
			});
			
			logger.info('Deployment logs retrieved', { 
				context: 'DeploymentService', 
				data: { deploymentId, logCount: logs.length } 
			});
			
			return logs;
		} catch (error) {
			logger.error('Failed to get deployment logs', {
				context: 'DeploymentService',
				error,
				data: { deploymentId }
			});
			throw error;
		}
	}

	/**
	 * Refresh deployment statuses
	 */
	async refreshDeploymentStatuses(): Promise<Deployment[]> {
		try {
			logger.info('Refreshing deployment statuses', { context: 'DeploymentService' });
			
			const deployments = await invoke<Deployment[]>('refresh_deployment_statuses_command');
			
			logger.info('Deployment statuses refreshed', { 
				context: 'DeploymentService', 
				data: { count: deployments.length } 
			});
			
			return deployments;
		} catch (error) {
			logger.error('Failed to refresh deployment statuses', {
				context: 'DeploymentService',
				error
			});
			throw error;
		}
	}

	/**
	 * List Docker containers
	 */
	async listContainers(): Promise<DockerContainer[]> {
		try {
			logger.info('Listing containers', { context: 'DeploymentService' });
			
			const containers = await invoke<DockerContainer[]>('list_containers_command');
			
			logger.info('Containers retrieved', { 
				context: 'DeploymentService', 
				data: { count: containers.length } 
			});
			
			return containers;
		} catch (error) {
			logger.error('Failed to list containers', {
				context: 'DeploymentService',
				error
			});
			throw error;
		}
	}

	/**
	 * Get deployment status color
	 */
	getStatusColor(status: DeploymentStatus): string {
		const colors: Record<DeploymentStatus, string> = {
			[DeploymentStatus.CREATING]: 'text-yellow-600',
			[DeploymentStatus.RUNNING]: 'text-green-600',
			[DeploymentStatus.STOPPED]: 'text-gray-600',
			[DeploymentStatus.FAILED]: 'text-red-600',
			[DeploymentStatus.RESTARTING]: 'text-blue-600',
			[DeploymentStatus.REMOVING]: 'text-gray-400'
		};
		return colors[status] || 'text-gray-400';
	}

	/**
	 * Get deployment status icon
	 */
	getStatusIcon(status: DeploymentStatus): string {
		const icons: Record<DeploymentStatus, string> = {
			[DeploymentStatus.CREATING]: '🔨',
			[DeploymentStatus.RUNNING]: '🟢',
			[DeploymentStatus.STOPPED]: '⏹️',
			[DeploymentStatus.FAILED]: '❌',
			[DeploymentStatus.RESTARTING]: '🔄',
			[DeploymentStatus.REMOVING]: '🗑️'
		};
		return icons[status] || '❓';
	}

	/**
	 * Format deployment duration
	 */
	formatDuration(createdAt: string, updatedAt: string): string {
		const created = new Date(createdAt);
		const updated = new Date(updatedAt);
		const duration = updated.getTime() - created.getTime();
		
		const seconds = Math.floor(duration / 1000);
		const minutes = Math.floor(seconds / 60);
		const hours = Math.floor(minutes / 60);
		const days = Math.floor(hours / 24);

		if (days > 0) return `${days}d ${hours % 24}h`;
		if (hours > 0) return `${hours}h ${minutes % 60}m`;
		if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
		return `${seconds}s`;
	}

	/**
	 * Validate deployment request
	 */
	validateDeploymentRequest(request: Partial<DeploymentCreateRequest>): string[] {
		const errors: string[] = [];

		if (!request.name?.trim()) {
			errors.push('Name is required');
		}

		if (!request.projectPath?.trim()) {
			errors.push('Project path is required');
		}

		if (!request.projectType) {
			errors.push('Project type is required');
		}

		return errors;
	}

	/**
	 * Generate deployment name from project
	 */
	generateDeploymentName(projectName: string, projectType: string): string {
		const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
		return `${projectName}-${projectType}-${timestamp}`;
	}

	/**
	 * Get project type from path
	 */
	detectProjectType(projectPath: string): string {
		// This would be more sophisticated in a real implementation
		// For now, we'll do basic detection based on common files
		if (projectPath.includes('package.json')) return 'node';
		if (projectPath.includes('Cargo.toml')) return 'rust';
		if (projectPath.includes('requirements.txt') || projectPath.includes('pyproject.toml')) return 'python';
		if (projectPath.includes('go.mod')) return 'go';
		return 'unknown';
	}
}

export const deploymentService = DeploymentService.getInstance();
