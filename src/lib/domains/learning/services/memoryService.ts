/**
 * Memory management service for learning data
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '@/lib/domains/shared/services/logger';

const log = logger.createScoped('MemoryService');

export interface MemoryStats {
	total_events: number;
	total_patterns: number;
	total_preferences: number;
	events_retention_days: number;
	patterns_retention_days: number;
	max_events: number;
	max_patterns_per_type: number;
}

export interface CleanupStats {
	events_deleted: number;
	patterns_deleted: number;
	patterns_consolidated: number;
}

export interface CleanupPreview {
	events_to_delete: number;
	patterns_to_delete: number;
	events_over_limit: number;
}

class MemoryService {
	/**
	 * Perform automatic cleanup of learning data
	 */
	async cleanup(): Promise<CleanupStats> {
		try {
			log.info('Cleaning up learning data');
			const stats = await invoke<CleanupStats>('cleanup_learning_data');
			log.info('Cleanup completed', stats);
			return stats;
		} catch (error) {
			log.error('Failed to cleanup learning data', error);
			throw error;
		}
	}

	/**
	 * Get memory usage statistics
	 */
	async getStats(): Promise<MemoryStats> {
		try {
			const stats = await invoke<MemoryStats>('get_memory_stats');
			log.debug('Memory stats retrieved', stats);
			return stats;
		} catch (error) {
			log.error('Failed to get memory stats', error);
			throw error;
		}
	}

	/**
	 * Get preview of what would be cleaned (for user authorization)
	 */
	async getCleanupPreview(): Promise<CleanupPreview> {
		try {
			const preview = await invoke<CleanupPreview>('get_cleanup_preview');
			log.debug('Cleanup preview retrieved', preview);
			return preview;
		} catch (error) {
			log.error('Failed to get cleanup preview', error);
			throw error;
		}
	}
}

export const memoryService = new MemoryService();

