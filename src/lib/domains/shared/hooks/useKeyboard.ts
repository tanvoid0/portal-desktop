// Generic keyboard hook - reusable anywhere
// This is the base hook that can be customized for different domains

import { onMount, onDestroy } from 'svelte';

export interface KeyboardShortcut {
	key: string;
	modifiers?: {
		ctrl?: boolean;
		alt?: boolean;
		shift?: boolean;
		meta?: boolean;
	};
	description: string;
	action: () => void | Promise<void>;
	preventDefault?: boolean;
	stopPropagation?: boolean;
	context?: string; // Optional context to activate shortcut only in specific areas
}

export interface KeyboardConfig {
	shortcuts: KeyboardShortcut[];
	enabled?: boolean;
	context?: string;
	ignoreInputs?: boolean; // Whether to ignore shortcuts when typing in inputs
}

export interface KeyboardReturn {
	handleKeydown: (event: KeyboardEvent) => boolean;
	getShortcuts: () => Array<{ key: string; description: string; category?: string }>;
	enabled: boolean;
	setEnabled: (enabled: boolean) => void;
}

/**
 * Generic keyboard hook that can be used anywhere
 */
export function useKeyboard(config: KeyboardConfig): KeyboardReturn {
	const { shortcuts, enabled: initialEnabled = true, context, ignoreInputs = true } = config;
	
	let enabled = $state(initialEnabled);
	
	function matchesShortcut(event: KeyboardEvent, shortcut: KeyboardShortcut): boolean {
		// Check context
		if (shortcut.context && shortcut.context !== context) {
			return false;
		}
		
		// Check if typing in input
		if (ignoreInputs) {
			const target = event.target as HTMLElement;
			if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
				return false;
			}
		}
		
		// Parse event key
		const eventKey = parseEventKey(event);
		
		// Direct key match
		if (eventKey.toLowerCase() === shortcut.key.toLowerCase()) {
			// Check modifiers if specified
			if (shortcut.modifiers) {
				const { ctrl, alt, shift, meta } = shortcut.modifiers;
				
				if (ctrl !== undefined && event.ctrlKey !== ctrl) return false;
				if (alt !== undefined && event.altKey !== alt) return false;
				if (shift !== undefined && event.shiftKey !== shift) return false;
				if (meta !== undefined && event.metaKey !== meta) return false;
			} else {
				// If no modifiers specified, don't match if any modifiers are pressed
				if (event.ctrlKey || event.altKey || event.metaKey) {
					return false;
				}
			}
			
			return true;
		}
		
		return false;
	}
	
	function parseEventKey(event: KeyboardEvent): string {
		const parts: string[] = [];
		if (event.ctrlKey) parts.push('Ctrl');
		if (event.shiftKey) parts.push('Shift');
		if (event.altKey) parts.push('Alt');
		if (event.metaKey) parts.push('Meta');
		
		// Handle special keys
		const keyMap: Record<string, string> = {
			' ': 'Space',
			'ArrowUp': 'ArrowUp',
			'ArrowDown': 'ArrowDown',
			'ArrowLeft': 'ArrowLeft',
			'ArrowRight': 'ArrowRight',
			'Escape': 'Escape',
			'Enter': 'Enter',
			'Tab': 'Tab',
			'Delete': 'Delete',
			'Backspace': 'Backspace',
		};
		
		const key = keyMap[event.key] || event.key;
		if (key.length === 1 || key.startsWith('Arrow') || keyMap[event.key]) {
			parts.push(key);
		}
		
		return parts.join('+');
	}
	
	function handleKeydown(event: KeyboardEvent): boolean {
		if (!enabled) return false;
		
		for (const shortcut of shortcuts) {
			if (matchesShortcut(event, shortcut)) {
				if (shortcut.preventDefault) {
					event.preventDefault();
				}
				if (shortcut.stopPropagation) {
					event.stopPropagation();
				}
				shortcut.action();
				return true;
			}
		}
		
		return false;
	}
	
	function getShortcuts(): Array<{ key: string; description: string; category?: string }> {
		return shortcuts
			.filter(s => !context || s.context === context)
			.map(s => ({
				key: s.key,
				description: s.description,
				category: s.context || 'General'
			}));
	}
	
	function setEnabled(newEnabled: boolean) {
		enabled = newEnabled;
	}
	
	// Attach global listener
	onMount(() => {
		if (enabled) {
			window.addEventListener('keydown', handleKeydown);
		}
	});
	
	onDestroy(() => {
		if (enabled) {
			window.removeEventListener('keydown', handleKeydown);
		}
	});
	
	return {
		handleKeydown,
		getShortcuts,
		enabled: $derived(enabled),
		setEnabled
	};
}

