/**
 * Learning service for managing learning and suggestions
 */

import { invoke } from '@tauri-apps/api/core';
import type {
	MLIntensity,
	PatternType,
	LearnedPattern,
	UserPreference,
	LearningEvent,
	Suggestion,
	RecordLearningEventRequest,
	LearnPatternRequest,
	LearnPreferenceRequest,
} from '@/lib/domains/learning/types';
import { learningStore } from '@/lib/domains/learning/stores/learningStore';
import { logger } from '@/lib/domains/shared/services/logger';
import { cache } from '@/lib/domains/shared/services/cache';
import { memoryService } from './memoryService';

const log = logger.createScoped('LearningService');

class LearningService {
	private initialized = false;
	private cleanupInterval: ReturnType<typeof setInterval> | null = null;

	/**
	 * Initialize the learning service
	 */
	async initialize(): Promise<void> {
		if (this.initialized) return;

		try {
			log.info('Initializing learning service');
			
			// Schedule automatic cleanup every 24 hours
			this.scheduleAutomaticCleanup();
			
			this.initialized = true;
			log.info('Learning service initialized successfully');
		} catch (error) {
			log.error('Failed to initialize learning service', error);
			throw error;
		}
	}

	/**
	 * Schedule automatic memory cleanup
	 */
	private scheduleAutomaticCleanup(): void {
		// Run cleanup every 24 hours
		const CLEANUP_INTERVAL_MS = 24 * 60 * 60 * 1000; // 24 hours

		this.cleanupInterval = setInterval(async () => {
			try {
				log.info('Running scheduled memory cleanup');
				
				// Check what would be cleaned
				const preview = await memoryService.getCleanupPreview();
				
				// Only clean automatically if minimal impact (small amounts of low-quality data)
				// For larger cleanups, we'll need user authorization
				const totalToDelete = preview.events_to_delete + preview.patterns_to_delete;
				const needsAuthorization = totalToDelete > 50; // Threshold for authorization
				
				if (needsAuthorization) {
					log.info('Cleanup needs authorization', {
						events_to_delete: preview.events_to_delete,
						patterns_to_delete: preview.patterns_to_delete,
					});
					// Store preview for user to review
					// The UI should show a notification/prompt
					return;
				}
				
				// Small cleanup, proceed automatically
				await memoryService.cleanup();
			} catch (error) {
				log.error('Scheduled cleanup failed', error);
			}
		}, CLEANUP_INTERVAL_MS);

		// Run initial cleanup after 1 hour (to not block startup)
		setTimeout(async () => {
			try {
				log.info('Running initial memory cleanup');
				
				// Always check preview first
				const preview = await memoryService.getCleanupPreview();
				const totalToDelete = preview.events_to_delete + preview.patterns_to_delete;
				
				// Only auto-clean if minimal
				if (totalToDelete <= 50) {
					await memoryService.cleanup();
				} else {
					log.info('Initial cleanup skipped - needs authorization', preview);
				}
			} catch (error) {
				log.error('Initial cleanup failed', error);
			}
		}, 60 * 60 * 1000); // 1 hour
	}

	/**
	 * Cleanup resources
	 */
	cleanup(): void {
		if (this.cleanupInterval) {
			clearInterval(this.cleanupInterval);
			this.cleanupInterval = null;
		}
		this.initialized = false;
	}

	/**
	 * Record a learning event
	 */
	async recordEvent(request: RecordLearningEventRequest): Promise<number> {
		try {
			log.info('Recording learning event', { event_type: request.event_type });
			
			const eventId = await invoke<number>('record_learning_event', {
				eventType: request.event_type,
				eventData: request.event_data,
				outcome: request.outcome ?? null,
				context: request.context ?? null,
			});

			log.info('Learning event recorded', { event_id: eventId });
			return eventId;
		} catch (error) {
			log.error('Failed to record learning event', error);
			throw error;
		}
	}

	/**
	 * Learn a pattern from user behavior
	 */
	async learnPattern(request: LearnPatternRequest): Promise<number> {
		try {
			log.info('Learning pattern', { pattern_type: request.pattern_type });
			
			const patternId = await invoke<number>('learn_pattern', {
				patternType: request.pattern_type,
				patternData: request.pattern_data,
				context: request.context ?? null,
			});

			log.info('Pattern learned', { pattern_id: patternId });
			return patternId;
		} catch (error) {
			log.error('Failed to learn pattern', error);
			throw error;
		}
	}

	/**
	 * Record pattern outcome (success/failure)
	 */
	async recordPatternOutcome(patternId: number, success: boolean): Promise<void> {
		try {
			log.info('Recording pattern outcome', { pattern_id: patternId, success });
			
			await invoke('record_pattern_outcome', {
				patternId,
				success,
			});

			log.info('Pattern outcome recorded', { pattern_id: patternId });
		} catch (error) {
			log.error('Failed to record pattern outcome', error);
			throw error;
		}
	}

	/**
	 * Get suggestions based on context
	 */
	async getSuggestions(
		patternType: PatternType,
		context?: string
	): Promise<Suggestion[]> {
		try {
			log.info('Getting suggestions', { pattern_type: patternType, context });
			
			// Check cache first
			const cacheKey = `suggestions:${patternType}:${context ?? 'global'}`;
			const cached = cache.get<Suggestion[]>(cacheKey);
			if (cached) {
				log.info('Returning cached suggestions', { count: cached.length });
				return cached;
			}

			const suggestions = await invoke<Suggestion[]>('get_suggestions', {
				patternType,
				context: context ?? null,
			});

			// Cache for 2 minutes
			cache.set(cacheKey, suggestions, 2 * 60 * 1000);
			
			log.info('Suggestions retrieved', { count: suggestions.length });
			return suggestions;
		} catch (error) {
			log.error('Failed to get suggestions', error);
			throw error;
		}
	}

	/**
	 * Learn user preference
	 */
	async learnPreference(request: LearnPreferenceRequest): Promise<number> {
		try {
			log.info('Learning preference', { preference_type: request.preference_type });
			
			const preferenceId = await invoke<number>('learn_preference', {
				preferenceType: request.preference_type,
				context: request.context ?? null,
				preferenceValue: request.preference_value,
				learnedFrom: request.learned_from ?? null,
			});

			log.info('Preference learned', { preference_id: preferenceId });
			return preferenceId;
		} catch (error) {
			log.error('Failed to learn preference', error);
			throw error;
		}
	}

	/**
	 * Mark pattern as important (never auto-delete)
	 */
	async markPatternImportant(patternId: number, isImportant: boolean): Promise<void> {
		try {
			log.info('Marking pattern as important', { pattern_id: patternId, is_important: isImportant });
			
			await invoke('mark_pattern_important', {
				patternId,
				isImportant,
			});
			
			log.info('Pattern importance updated successfully', { pattern_id: patternId });
		} catch (error) {
			log.error('Failed to mark pattern as important', error);
			throw error;
		}
	}

	/**
	 * Get user preference
	 */
	async getPreference(
		preferenceType: string,
		context?: string
	): Promise<Record<string, unknown> | null> {
		try {
			log.info('Getting preference', { preference_type: preferenceType, context });
			
			const preference = await invoke<Record<string, unknown> | null>('get_preference', {
				preferenceType,
				context: context ?? null,
			});

			if (preference) {
				log.info('Preference retrieved', { preference_type: preferenceType });
			}
			
			return preference;
		} catch (error) {
			log.error('Failed to get preference', error);
			throw error;
		}
	}

	/**
	 * Get ML intensity setting
	 */
	async getMLIntensity(): Promise<MLIntensity> {
		try {
			const intensity = await invoke<string>('get_ml_intensity');
			return intensity as MLIntensity;
		} catch (error) {
			log.error('Failed to get ML intensity', error);
			return 'medium' as MLIntensity; // Default
		}
	}

	/**
	 * Set ML intensity
	 */
	async setMLIntensity(intensity: MLIntensity): Promise<void> {
		try {
			log.info('Setting ML intensity', { intensity });
			await invoke('set_ml_intensity', { intensity });
			learningStore.setMLIntensity(intensity);
			log.info('ML intensity updated successfully');
		} catch (error) {
			log.error('Failed to set ML intensity', error);
			throw error;
		}
	}

	/**
	 * Check if ML learning is enabled
	 */
	async getMLEnabled(): Promise<boolean> {
		try {
			const enabled = await invoke<boolean>('get_ml_enabled');
			return enabled;
		} catch (error) {
			log.error('Failed to get ML enabled state', error);
			return true; // Default to enabled
		}
	}

	/**
	 * Enable or disable ML learning
	 */
	async setMLEnabled(enabled: boolean): Promise<void> {
		try {
			log.info('Setting ML enabled state', { enabled });
			await invoke('set_ml_enabled', { enabled });
			// Pattern collector will check enabled state on next collection attempt
			log.info('ML enabled state updated successfully');
		} catch (error) {
			log.error('Failed to set ML enabled state', error);
			throw error;
		}
	}

	/**
	 * Get all learned patterns
	 */
	async getAllPatterns(limit?: number): Promise<LearnedPattern[]> {
		try {
			log.debug('Getting all patterns', { limit });
			const patterns = await invoke<LearnedPattern[]>('get_all_patterns', { limit: limit ?? null });
			return patterns;
		} catch (error) {
			log.error('Failed to get all patterns', error);
			throw error;
		}
	}

	/**
	 * Get recent learning events
	 */
	async getRecentEvents(limit: number = 50): Promise<LearningEvent[]> {
		try {
			log.debug('Getting recent events', { limit });
			const events = await invoke<LearningEvent[]>('get_recent_events', { limit });
			return events;
		} catch (error) {
			log.error('Failed to get recent events', error);
			throw error;
		}
	}

	/**
	 * Get all user preferences
	 */
	async getAllPreferences(): Promise<UserPreference[]> {
		try {
			log.debug('Getting all preferences');
			const preferences = await invoke<UserPreference[]>('get_all_preferences');
			return preferences;
		} catch (error) {
			log.error('Failed to get all preferences', error);
			throw error;
		}
	}
}

export const learningService = new LearningService();

