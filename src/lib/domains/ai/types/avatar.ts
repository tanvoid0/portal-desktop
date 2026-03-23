/**
 * Avatar Assistant Types
 */

export type AvatarState = 'idle' | 'thinking' | 'suggesting' | 'error' | 'success';

export type AvatarExpression = 'neutral' | 'thinking' | 'happy' | 'concerned';

export interface AvatarPosition {
	x: number;
	y: number;
}

export interface AvatarSuggestion {
	id: string;
	message: string;
	timestamp: Date;
	context?: {
		error?: string;
		page?: string;
		toastType?: 'error' | 'warning' | 'info' | 'success';
	};
}

export interface AvatarConfig {
	enabled: boolean;
	position?: AvatarPosition;
	animationSpeed?: 'slow' | 'normal' | 'fast';
	suggestionFrequency?: 'low' | 'medium' | 'high';
}

export interface AvatarStoreState {
	state: AvatarState;
	expression: AvatarExpression;
	currentSuggestion: AvatarSuggestion | null;
	position: AvatarPosition;
	config: AvatarConfig;
	isVisible: boolean;
}

