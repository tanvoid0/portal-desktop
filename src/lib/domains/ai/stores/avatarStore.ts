/**
 * Avatar Store - State management for the floating avatar assistant
 */

import { writable, derived } from 'svelte/store';
import type { AvatarStoreState, AvatarState, AvatarExpression, AvatarSuggestion, AvatarPosition, AvatarConfig } from '../types/avatar';

const defaultConfig: AvatarConfig = {
	enabled: true,
	animationSpeed: 'normal',
	suggestionFrequency: 'medium'
};

const getDefaultPosition = (): AvatarPosition => {
	if (typeof window === 'undefined') {
		return { x: 100, y: 100 };
	}
	// Position in bottom right corner
	const AVATAR_SIZE = 64;
	const PADDING = 20;
	return {
		x: window.innerWidth - AVATAR_SIZE - PADDING,
		y: window.innerHeight - AVATAR_SIZE - PADDING
	};
};

const defaultPosition: AvatarPosition = getDefaultPosition();

const initialState: AvatarStoreState = {
	state: 'idle',
	expression: 'neutral',
	currentSuggestion: null,
	position: defaultPosition,
	config: defaultConfig,
	isVisible: true
};

export const avatarStore = writable<AvatarStoreState>(initialState);

// Derived stores for convenience
export const avatarState = derived(avatarStore, ($store) => $store.state);
export const avatarExpression = derived(avatarStore, ($store) => $store.expression);
export const currentSuggestion = derived(avatarStore, ($store) => $store.currentSuggestion);
export const avatarPosition = derived(avatarStore, ($store) => $store.position);
export const avatarConfig = derived(avatarStore, ($store) => $store.config);
export const isAvatarVisible = derived(avatarStore, ($store) => $store.isVisible);

export const avatarActions = {
	/**
	 * Update avatar state
	 */
	setState(state: AvatarState) {
		avatarStore.update((current) => ({
			...current,
			state
		}));
	},

	/**
	 * Update avatar expression
	 */
	setExpression(expression: AvatarExpression) {
		avatarStore.update((current) => ({
			...current,
			expression
		}));
	},

	/**
	 * Set current suggestion
	 */
	setSuggestion(suggestion: AvatarSuggestion | null) {
		avatarStore.update((current) => ({
			...current,
			currentSuggestion: suggestion,
			state: suggestion ? 'suggesting' : 'idle'
		}));
	},

	/**
	 * Dismiss current suggestion
	 */
	dismissSuggestion() {
		avatarStore.update((current) => ({
			...current,
			currentSuggestion: null,
			state: 'idle',
			expression: 'neutral'
		}));
	},

	/**
	 * Update avatar position
	 */
	setPosition(position: AvatarPosition) {
		avatarStore.update((current) => ({
			...current,
			position
		}));
	},

	/**
	 * Update avatar configuration
	 */
	setConfig(config: Partial<AvatarConfig>) {
		avatarStore.update((current) => ({
			...current,
			config: {
				...current.config,
				...config
			}
		}));
	},

	/**
	 * Set avatar visibility
	 */
	setVisible(visible: boolean) {
		avatarStore.update((current) => ({
			...current,
			isVisible: visible
		}));
	},

	/**
	 * Reset avatar to initial state
	 */
	reset() {
		avatarStore.set(initialState);
	}
};

