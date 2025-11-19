// Hook for registering and handling keyboard shortcuts

import { derived, get, type Readable } from 'svelte/store';
import { matchShortcut, type ParsedShortcut } from '../utils/shortcutParser';
import type { KeyboardShortcut } from '../types';

export interface UseKeyboardShortcutsOptions {
	enabled?: boolean;
	preventDefault?: boolean;
}

export function useKeyboardShortcuts(
	shortcuts: KeyboardShortcut[],
	options: UseKeyboardShortcutsOptions = {}
) {
	const { enabled = true, preventDefault = true } = options;
	
	const parsedShortcuts: Readable<Array<KeyboardShortcut & { parsed: ParsedShortcut }>> = derived(
		[],
		() => shortcuts.map(shortcut => ({
			...shortcut,
			parsed: {
				key: shortcut.key,
				ctrl: shortcut.modifiers?.ctrl,
				alt: shortcut.modifiers?.alt,
				shift: shortcut.modifiers?.shift,
				meta: shortcut.modifiers?.meta
			} as ParsedShortcut
		}))
	);
	
	function handleKeydown(event: KeyboardEvent): boolean {
		if (!enabled) return false;
		
		// Ignore if typing in input/textarea/contenteditable
		const target = event.target as HTMLElement;
		if (
			target.tagName === 'INPUT' ||
			target.tagName === 'TEXTAREA' ||
			target.isContentEditable
		) {
			// Allow shortcuts with modifiers even in inputs
			if (!event.ctrlKey && !event.metaKey && !event.altKey) {
				return false;
			}
		}
		
		// Try to match shortcuts
		const currentShortcuts = get(parsedShortcuts);
		for (const shortcut of currentShortcuts) {
			if (matchShortcut(event, shortcut.parsed)) {
				if (preventDefault) {
					event.preventDefault();
					event.stopPropagation();
				}
				shortcut.action();
				return true;
			}
		}
		
		return false;
	}
	
	const enabledStore: Readable<boolean> = derived([], () => enabled);
	
	return {
		handleKeydown,
		enabled: enabledStore
	};
}
