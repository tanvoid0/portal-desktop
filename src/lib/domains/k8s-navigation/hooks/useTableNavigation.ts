// Hook for table row navigation with keyboard

import { KEY_CODES, VIM_KEYS } from '../utils/keyboardConstants';
import type { NavigationState } from '../types';

export interface TableNavigationOptions {
	totalItems: number;
	onSelect?: (index: number) => void;
	onActivate?: (index: number) => void;
	enabled?: boolean;
}

export function useTableNavigation(options: TableNavigationOptions) {
	const { totalItems, onSelect, onActivate, enabled = true } = options;
	
	let selectedIndex = $state<number>(-1);
	
	const state = $derived<NavigationState>({
		selectedIndex,
		totalItems
	});
	
	function selectIndex(index: number) {
		if (index < 0) {
			selectedIndex = -1;
		} else if (index >= totalItems) {
			selectedIndex = totalItems - 1;
		} else {
			selectedIndex = index;
		}
		onSelect?.(selectedIndex);
	}
	
	function moveUp() {
		if (selectedIndex <= 0) {
			selectIndex(0);
		} else {
			selectIndex(selectedIndex - 1);
		}
	}
	
	function moveDown() {
		if (selectedIndex >= totalItems - 1) {
			selectIndex(totalItems - 1);
		} else {
			selectIndex(selectedIndex + 1);
		}
	}
	
	function moveToTop() {
		selectIndex(0);
	}
	
	function moveToBottom() {
		selectIndex(totalItems - 1);
	}
	
	function activate() {
		if (selectedIndex >= 0 && selectedIndex < totalItems) {
			onActivate?.(selectedIndex);
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
				if (selectedIndex >= 0) {
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
		selectedIndex: $derived(selectedIndex),
		selectIndex,
		moveUp,
		moveDown,
		moveToTop,
		moveToBottom,
		activate,
		handleKeydown
	};
}

