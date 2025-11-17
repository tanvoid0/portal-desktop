/**
 * Learning store using Svelte 5 runes
 */

import { writable, derived } from 'svelte/store';
import type { Suggestion, MLIntensity } from '@/lib/domains/learning/types';

interface LearningState {
	suggestions: Suggestion[];
	mlIntensity: MLIntensity;
	loading: boolean;
	error: string | null;
}

const initialState: LearningState = {
	suggestions: [],
	mlIntensity: 'medium',
	loading: false,
	error: null,
};

function createLearningStore() {
	const { subscribe, set, update } = writable<LearningState>(initialState);

	return {
		subscribe,

		// Actions
		setLoading: (loading: boolean) => {
			update((state) => ({ ...state, loading }));
		},

		setError: (error: string | null) => {
			update((state) => ({ ...state, error }));
		},

		setSuggestions: (suggestions: Suggestion[]) => {
			update((state) => ({ ...state, suggestions }));
		},

		addSuggestion: (suggestion: Suggestion) => {
			update((state) => ({
				...state,
				suggestions: [...state.suggestions, suggestion],
			}));
		},

		removeSuggestion: (patternId: number) => {
			update((state) => ({
				...state,
				suggestions: state.suggestions.filter((s) => s.pattern_id !== patternId),
			}));
		},

		setMLIntensity: (intensity: MLIntensity) => {
			update((state) => ({ ...state, mlIntensity: intensity }));
		},

		reset: () => {
			set(initialState);
		},
	};
}

export const learningStore = createLearningStore();

// Derived stores
export const hasSuggestions = derived(
	learningStore,
	($store) => $store.suggestions.length > 0
);

export const isLoading = derived(learningStore, ($store) => $store.loading);

export const hasError = derived(learningStore, ($store) => $store.error !== null);

