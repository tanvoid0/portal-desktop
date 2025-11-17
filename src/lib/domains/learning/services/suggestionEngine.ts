/**
 * Intelligent suggestion engine
 */

import { learningService } from './learningService';
import type { PatternType, Suggestion } from '@/lib/domains/learning/types';
import { logger } from '@/lib/domains/shared/services/logger';

const log = logger.createScoped('SuggestionEngine');

export class SuggestionEngine {
	/**
	 * Get contextual suggestions for a specific pattern type and context
	 */
	async getContextualSuggestions(
		patternType: PatternType,
		context?: string
	): Promise<Suggestion[]> {
		try {
			const suggestions = await learningService.getSuggestions(patternType, context);
			
			// Sort by relevance (success_rate * frequency)
			const sorted = suggestions.sort((a, b) => {
				const scoreA = a.success_rate * a.frequency;
				const scoreB = b.success_rate * b.frequency;
				return scoreB - scoreA;
			});

			log.info('Contextual suggestions retrieved', {
				pattern_type: patternType,
				context,
				count: sorted.length,
			});

			return sorted;
		} catch (error) {
			log.error('Failed to get contextual suggestions', error);
			return [];
		}
	}

	/**
	 * Get best suggestion (highest confidence)
	 */
	async getBestSuggestion(
		patternType: PatternType,
		context?: string
	): Promise<Suggestion | null> {
		const suggestions = await this.getContextualSuggestions(patternType, context);
		return suggestions.length > 0 ? suggestions[0] : null;
	}

	/**
	 * Record suggestion acceptance
	 */
	async recordSuggestionAccepted(patternId: number): Promise<void> {
		try {
			await learningService.recordPatternOutcome(patternId, true);
			log.info('Suggestion acceptance recorded', { pattern_id: patternId });
		} catch (error) {
			log.error('Failed to record suggestion acceptance', error);
		}
	}

	/**
	 * Record suggestion rejection
	 */
	async recordSuggestionRejected(patternId: number): Promise<void> {
		try {
			await learningService.recordPatternOutcome(patternId, false);
			log.info('Suggestion rejection recorded', { pattern_id: patternId });
		} catch (error) {
			log.error('Failed to record suggestion rejection', error);
		}
	}
}

export const suggestionEngine = new SuggestionEngine();

