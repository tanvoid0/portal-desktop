// Hook for table row navigation with keyboard

import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
import { KEY_CODES, VIM_KEYS } from '../utils/keyboardConstants';
import type { NavigationState } from '../types';

export interface TableNavigationOptions {
	totalItems: number | (() => number);
	onSelect?: (index: number) => void;
	onActivate?: (index: number) => void;
	enabled?: boolean;
}

export function useTableNavigation(options: TableNavigationOptions) {
	const { onSelect, onActivate, enabled = true } = options;
	const getTotalItems = (): number => {
		return typeof options.totalItems === 'function' ? options.totalItems() : options.totalItems;
	};
	
	const selectedIndex = writable<number>(-1);
	
	const state: Readable<NavigationState> = derived(
		[selectedIndex],
		([$selectedIndex]) => ({
			selectedIndex: $selectedIndex,
			totalItems: getTotalItems()
		})
	);
	
	function selectIndex(index: number) {
		const total = getTotalItems();
		let newIndex: number;
		if (index < 0) {
			newIndex = -1;
		} else if (index >= total) {
			newIndex = total - 1;
		} else {
			newIndex = index;
		}
		selectedIndex.set(newIndex);
		onSelect?.(newIndex);
	}
	
	function moveUp() {
		const currentIndex = get(selectedIndex);
		if (currentIndex <= 0) {
			selectIndex(0);
		} else {
			selectIndex(currentIndex - 1);
		}
	}
	
	function moveDown() {
		const currentIndex = get(selectedIndex);
		const total = getTotalItems();
		if (currentIndex >= total - 1) {
			selectIndex(total - 1);
		} else {
			selectIndex(currentIndex + 1);
		}
	}
	
	function moveToTop() {
		selectIndex(0);
	}
	
	function moveToBottom() {
		const total = getTotalItems();
		selectIndex(total - 1);
	}
	
	function activate() {
		const currentIndex = get(selectedIndex);
		const total = getTotalItems();
		if (currentIndex >= 0 && currentIndex < total) {
			onActivate?.(currentIndex);
		}
	}
	
	function handleKeydown(event: KeyboardEvent): boolean {
		if (!enabled) return false;
		
		// Ignore if typing in input/textarea
		const target = event.target as HTMLElement;
		if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
			return false;
		}
		
		switch (event.key) {
			case KEY_CODES.ARROW_UP:
				event.preventDefault();
				moveUp();
				return true;
				
			case KEY_CODES.ARROW_DOWN:
				event.preventDefault();
				moveDown();
				return true;
				
			case KEY_CODES.ENTER:
				const currentIndex = get(selectedIndex);
				if (currentIndex >= 0) {
					event.preventDefault();
					activate();
					return true;
				}
				return false;
				
			case KEY_CODES.HOME:
				if (event.ctrlKey || event.metaKey) {
					event.preventDefault();
					moveToTop();
					return true;
				}
				return false;
				
			case KEY_CODES.END:
				if (event.ctrlKey || event.metaKey) {
					event.preventDefault();
					moveToBottom();
					return true;
				}
				return false;
				
			// Vim-style navigation
			case VIM_KEYS.UP:
				if (!event.ctrlKey && !event.metaKey && !event.altKey) {
					event.preventDefault();
					moveUp();
					return true;
				}
				return false;
				
			case VIM_KEYS.DOWN:
				if (!event.ctrlKey && !event.metaKey && !event.altKey) {
					event.preventDefault();
					moveDown();
					return true;
				}
				return false;
				
			case VIM_KEYS.TOP:
				if (!event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey) {
					event.preventDefault();
					moveToTop();
					return true;
				}
				return false;
				
			case VIM_KEYS.BOTTOM:
				if (!event.ctrlKey && !event.metaKey && !event.altKey && event.shiftKey) {
					event.preventDefault();
					moveToBottom();
					return true;
				}
				return false;
				
			default:
				return false;
		}
	}
	
	return {
		state,
		selectedIndex,
		selectIndex,
		moveUp,
		moveDown,
		moveToTop,
		moveToBottom,
		activate,
		handleKeydown
	};
}
