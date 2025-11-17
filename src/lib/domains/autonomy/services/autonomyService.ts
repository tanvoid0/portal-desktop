/**
 * Autonomy service for managing autonomous actions
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '@/lib/domains/shared/services/logger';

const log = logger.createScoped('AutonomyService');

export type AutonomyLevel = 'observation' | 'conservative' | 'balanced' | 'aggressive';

export interface AutonomousActionRequest {
	action_type: string;
	action_data: Record<string, unknown>;
	context: string;
}

export interface AutonomousActionResult {
	action_id: string;
	executed: boolean;
	requires_approval: boolean;
	classification: {
		safety_level: string;
		confidence: number;
		reason: string;
	};
	message: string;
}

export interface ApprovalStats {
	[key: string]: {
		approved: number;
		total: number;
		approval_rate: number;
	};
}

class AutonomyService {
	private initialized = false;

	/**
	 * Initialize the autonomy service
	 */
	async initialize(): Promise<void> {
		if (this.initialized) return;

		try {
			log.info('Initializing autonomy service');
			
			// Check current autonomy settings
			const enabled = await this.getEnabled();
			const level = await this.getLevel();
			
			log.info('Autonomy service initialized', { enabled, level });
			this.initialized = true;
		} catch (error) {
			log.error('Failed to initialize autonomy service', error);
			throw error;
		}
	}

	/**
	 * Evaluate if an action should be executed autonomously
	 */
	async evaluateAction(
		request: AutonomousActionRequest
	): Promise<AutonomousActionResult> {
		try {
			log.info('Evaluating autonomous action', { action_type: request.action_type });

			const result = await invoke<AutonomousActionResult>('evaluate_autonomous_action', {
				actionType: request.action_type,
				actionData: request.action_data,
				context: request.context,
			});

			log.info('Action evaluation complete', {
				action_id: result.action_id,
				executed: result.executed,
				requires_approval: result.requires_approval,
			});

			return result;
		} catch (error) {
			log.error('Failed to evaluate autonomous action', error);
			throw error;
		}
	}

	/**
	 * Record the outcome of an autonomous action
	 */
	async recordActionOutcome(
		actionId: string,
		actionType: string,
		context: string,
		success: boolean,
		feedback?: string
	): Promise<void> {
		try {
			log.info('Recording action outcome', { action_id: actionId, success });

			await invoke('record_autonomous_action_outcome', {
				actionId,
				actionType,
				context,
				success,
				feedback: feedback ?? null,
			});

			log.info('Action outcome recorded successfully', { action_id: actionId });
		} catch (error) {
			log.error('Failed to record action outcome', error);
			throw error;
		}
	}

	/**
	 * Get current autonomy level
	 */
	async getLevel(): Promise<AutonomyLevel> {
		try {
			const level = await invoke<string>('get_autonomy_level');
			return level.toLowerCase() as AutonomyLevel;
		} catch (error) {
			log.error('Failed to get autonomy level', error);
			return 'balanced'; // Default
		}
	}

	/**
	 * Set autonomy level
	 */
	async setLevel(level: AutonomyLevel): Promise<void> {
		try {
			log.info('Setting autonomy level', { level });
			await invoke('set_autonomy_level', { level });
			log.info('Autonomy level updated successfully');
		} catch (error) {
			log.error('Failed to set autonomy level', error);
			throw error;
		}
	}

	/**
	 * Check if autonomy is enabled
	 */
	async getEnabled(): Promise<boolean> {
		try {
			return await invoke<boolean>('get_autonomy_enabled');
		} catch (error) {
			log.error('Failed to get autonomy enabled state', error);
			return false; // Default to disabled for safety
		}
	}

	/**
	 * Enable or disable autonomy
	 */
	async setEnabled(enabled: boolean): Promise<void> {
		try {
			log.info('Setting autonomy enabled state', { enabled });
			await invoke('set_autonomy_enabled', { enabled });
			log.info('Autonomy enabled state updated successfully');
		} catch (error) {
			log.error('Failed to set autonomy enabled state', error);
			throw error;
		}
	}

	/**
	 * Get approval statistics
	 */
	async getApprovalStats(): Promise<ApprovalStats> {
		try {
			return await invoke<ApprovalStats>('get_approval_stats');
		} catch (error) {
			log.error('Failed to get approval stats', error);
			throw error;
		}
	}

	/**
	 * Cleanup resources
	 */
	cleanup(): void {
		this.initialized = false;
	}
}

export const autonomyService = new AutonomyService();

