/**
 * Pattern collector service for automatically recording learning events
 */

import { learningService } from './learningService';
import type { PatternType, EventType, EventOutcome } from '@/lib/domains/learning/types';
import { logger } from '@/lib/domains/shared/services/logger';

const log = logger.createScoped('PatternCollector');

export class PatternCollector {
	private enabled = true;
	private mlEnabled = true;

	/**
	 * Enable or disable automatic pattern collection
	 */
	setEnabled(enabled: boolean): void {
		this.enabled = enabled;
		log.info('Pattern collection', { enabled });
	}

	/**
	 * Update ML enabled state (checked before collecting patterns)
	 */
	async updateMLEnabledState(): Promise<void> {
		try {
			this.mlEnabled = await learningService.getMLEnabled();
			log.debug('ML enabled state updated', { enabled: this.mlEnabled });
		} catch (error) {
			log.warn('Failed to check ML enabled state, assuming enabled', error);
			this.mlEnabled = true;
		}
	}

	/**
	 * Check if collection should proceed
	 */
	private shouldCollect(): boolean {
		return this.enabled && this.mlEnabled;
	}

	/**
	 * Collect command pattern (from terminal)
	 */
	async collectCommandPattern(
		command: string,
		success: boolean,
		context?: string
	): Promise<void> {
		if (!this.shouldCollect()) {
			await this.updateMLEnabledState();
			if (!this.shouldCollect()) return;
		}

		try {
			// Learn the command pattern
			await learningService.learnPattern({
				pattern_type: 'command',
				pattern_data: { command },
				context,
			});

			// Record the event
			await learningService.recordEvent({
				event_type: 'command_executed',
				event_data: { command },
				outcome: success ? 'success' : 'failure',
				context,
			});

			log.debug('Command pattern collected', { command, success, context });
		} catch (error) {
			log.error('Failed to collect command pattern', error);
		}
	}

	/**
	 * Collect project setup pattern
	 */
	async collectProjectSetupPattern(
		framework: string | null,
		packageManager: string | null,
		context?: string
	): Promise<void> {
		if (!this.shouldCollect()) {
			await this.updateMLEnabledState();
			if (!this.shouldCollect()) return;
		}

		try {
			await learningService.learnPattern({
				pattern_type: 'framework',
				pattern_data: {
					framework,
					package_manager: packageManager,
				},
				context,
			});

			await learningService.recordEvent({
				event_type: 'project_created',
				event_data: {
					framework,
					package_manager: packageManager,
				},
				outcome: 'success',
				context,
			});

			log.debug('Project setup pattern collected', { framework, packageManager, context });
		} catch (error) {
			log.error('Failed to collect project setup pattern', error);
		}
	}

	/**
	 * Collect SDK preference
	 */
	async collectSDKPreference(
		sdkType: string,
		version: string,
		context?: string
	): Promise<void> {
		if (!this.shouldCollect()) {
			await this.updateMLEnabledState();
			if (!this.shouldCollect()) return;
		}

		try {
			await learningService.learnPreference({
				preference_type: 'sdk_version',
				context: context || `sdk_${sdkType}`,
				preference_value: {
					sdk_type: sdkType,
					version,
				},
				learned_from: 'user_selection',
			});

			log.debug('SDK preference collected', { sdk_type: sdkType, version, context });
		} catch (error) {
			log.error('Failed to collect SDK preference', error);
		}
	}

	/**
	 * Collect suggestion feedback
	 */
	async collectSuggestionFeedback(
		patternId: number,
		accepted: boolean
	): Promise<void> {
		if (!this.shouldCollect()) {
			await this.updateMLEnabledState();
			if (!this.shouldCollect()) return;
		}

		try {
			await learningService.recordPatternOutcome(patternId, accepted);

			await learningService.recordEvent({
				event_type: accepted ? 'suggestion_accepted' : 'suggestion_rejected',
				event_data: { pattern_id: patternId },
				outcome: accepted ? 'success' : 'failure',
			});

			log.debug('Suggestion feedback collected', { pattern_id: patternId, accepted });
		} catch (error) {
			log.error('Failed to collect suggestion feedback', error);
		}
	}

	/**
	 * Collect workflow pattern
	 */
	async collectWorkflowPattern(
		workflow: string[],
		success: boolean,
		context?: string
	): Promise<void> {
		if (!this.shouldCollect()) {
			await this.updateMLEnabledState();
			if (!this.shouldCollect()) return;
		}

		try {
			await learningService.learnPattern({
				pattern_type: 'workflow',
				pattern_data: { workflow },
				context,
			});

			await learningService.recordEvent({
				event_type: 'command_executed',
				event_data: { workflow },
				outcome: success ? 'success' : 'failure',
				context,
			});

			log.debug('Workflow pattern collected', { workflow, success, context });
		} catch (error) {
			log.error('Failed to collect workflow pattern', error);
		}
	}
}

export const patternCollector = new PatternCollector();

