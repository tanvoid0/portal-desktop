/**
 * Block Library Store - Svelte store for block library state
 */

import { writable, derived } from 'svelte/store';
import type { Block } from '../types';
import { blockLibraryService } from '../services/blockLibraryService';

interface BlockLibraryState {
	blocks: Block[];
	loading: boolean;
	error: string | null;
	searchQuery: string;
	selectedCategory: Block['category'] | null;
}

const initialState: BlockLibraryState = {
	blocks: [],
	loading: false,
	error: null,
	searchQuery: '',
	selectedCategory: null,
};

function createBlockLibraryStore() {
	const { subscribe, set, update } = writable<BlockLibraryState>(initialState);

	return {
		subscribe,

		/**
		 * Load all blocks
		 */
		async loadBlocks() {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				const blocks = await blockLibraryService.getBlocks();
				update((state) => ({
					...state,
					blocks,
					loading: false,
				}));
			} catch (error) {
				update((state) => ({
					...state,
					error: error instanceof Error ? error.message : 'Failed to load blocks',
					loading: false,
				}));
			}
		},

		/**
		 * Set search query
		 */
		setSearchQuery(query: string) {
			update((state) => ({ ...state, searchQuery: query }));
		},

		/**
		 * Set selected category
		 */
		setSelectedCategory(category: Block['category'] | null) {
			update((state) => ({ ...state, selectedCategory: category }));
		},

		/**
		 * Add or update a block
		 */
		upsertBlock(block: Block) {
			update((state) => {
				const index = state.blocks.findIndex((b) => b.id === block.id);
				if (index >= 0) {
					const blocks = [...state.blocks];
					blocks[index] = block;
					return { ...state, blocks };
				} else {
					return { ...state, blocks: [...state.blocks, block] };
				}
			});
		},

		/**
		 * Remove a block
		 */
		removeBlock(blockId: string) {
			update((state) => ({
				...state,
				blocks: state.blocks.filter((b) => b.id !== blockId),
			}));
		},

		/**
		 * Clear error
		 */
		clearError() {
			update((state) => ({ ...state, error: null }));
		},

		/**
		 * Reset store
		 */
		reset() {
			set(initialState);
		},
	};
}

export const blockLibraryStore = createBlockLibraryStore();

// Derived stores
export const blocks = derived(blockLibraryStore, ($store) => $store.blocks);
export const filteredBlocks = derived(
	[blockLibraryStore],
	([$store]) => {
		let filtered = $store.blocks;

		// Filter by category
		if ($store.selectedCategory) {
			filtered = filtered.filter((b) => b.category === $store.selectedCategory);
		}

		// Filter by search query
		if ($store.searchQuery) {
			const query = $store.searchQuery.toLowerCase();
			filtered = filtered.filter(
				(b) =>
					b.name.toLowerCase().includes(query) ||
					b.description.toLowerCase().includes(query) ||
					b.tags.some((tag) => tag.toLowerCase().includes(query))
			);
		}

		return filtered;
	}
);
export const blockLibraryLoading = derived(blockLibraryStore, ($store) => $store.loading);
export const blockLibraryError = derived(blockLibraryStore, ($store) => $store.error);

