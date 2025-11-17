/**
 * Learning domain types
 */

export type MLIntensity = 'fast' | 'light' | 'medium' | 'deep';

export type PatternType = 'command' | 'workflow' | 'config' | 'framework' | 'code';

export type EventType = 
	| 'command_executed'
	| 'project_created'
	| 'suggestion_accepted'
	| 'suggestion_rejected'
	| 'preference_updated'
	| 'pattern_learned';

export type EventOutcome = 'success' | 'failure' | 'ignored';

export interface LearnedPattern {
	id: number;
	pattern_type: PatternType;
	pattern_data: Record<string, unknown>;
	context: string | null;
	frequency: number;
	last_used: string | null;
	success_rate: number;
	is_important?: boolean;
	created_at: string;
}

export interface UserPreference {
	id: number;
	preference_type: string;
	context: string | null;
	preference_value: Record<string, unknown>;
	confidence: number;
	learned_from: string | null;
	is_important?: boolean;
	created_at: string;
	updated_at: string;
}

export interface LearningEvent {
	id: number;
	event_type: EventType;
	event_data: Record<string, unknown>;
	outcome: EventOutcome | null;
	context: string | null;
	created_at: string;
}

export interface Suggestion {
	pattern_id: number;
	pattern_data: Record<string, unknown>;
	frequency: number;
	success_rate: number;
	context: string | null;
}

export interface RecordLearningEventRequest {
	event_type: EventType;
	event_data: Record<string, unknown>;
	outcome?: EventOutcome;
	context?: string;
}

export interface LearnPatternRequest {
	pattern_type: PatternType;
	pattern_data: Record<string, unknown>;
	context?: string;
}

export interface LearnPreferenceRequest {
	preference_type: string;
	context?: string;
	preference_value: Record<string, unknown>;
	learned_from?: string;
}

