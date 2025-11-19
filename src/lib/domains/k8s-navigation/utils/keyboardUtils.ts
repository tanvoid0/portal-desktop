// Utility functions for keyboard shortcuts

import { parseShortcut, formatShortcut, normalizeKey } from './shortcutParser';
import type { KeyboardShortcut } from '../types';

/**
 * Convert a KeyboardEvent to a string representation for comparison
 */
function eventToShortcutString(event: KeyboardEvent): string {
	const parts: string[] = [];
	if (event.ctrlKey) parts.push('Ctrl');
	if (event.altKey) parts.push('Alt');
	if (event.shiftKey) parts.push('Shift');
	if (event.metaKey) parts.push('Cmd');
	parts.push(normalizeKey(event.key));
	return parts.join('+');
}

/**
 * Check if a keyboard event matches a shortcut
 */
export function matchesShortcut(event: KeyboardEvent, shortcut: KeyboardShortcut): boolean {
	const eventKey = eventToShortcutString(event);
	const shortcutKey = shortcut.key;
	
	// Direct match
	if (eventKey === shortcutKey) {
		return true;
	}
	
	// Check modifiers if specified
	if (shortcut.modifiers) {
		const { ctrl, alt, shift, meta } = shortcut.modifiers;
		
		if (ctrl !== undefined && event.ctrlKey !== ctrl) return false;
		if (alt !== undefined && event.altKey !== alt) return false;
		if (shift !== undefined && event.shiftKey !== shift) return false;
		if (meta !== undefined && event.metaKey !== meta) return false;
	}
	
	return false;
}

/**
 * Check if user is typing in an input field
 */
export function isTypingInInput(event: KeyboardEvent): boolean {
	const target = event.target as HTMLElement;
	return target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
}

/**
 * Prevent default and stop propagation for a keyboard event
 */
export function preventKeyboardEvent(event: KeyboardEvent, preventDefault = true, stopPropagation = true) {
	if (preventDefault) {
		event.preventDefault();
	}
	if (stopPropagation) {
		event.stopPropagation();
	}
}

/**
 * Create a keyboard shortcut object
 */
export function createShortcut(
	key: string,
	description: string,
	action: () => void | Promise<void>,
	modifiers?: KeyboardShortcut['modifiers']
): KeyboardShortcut {
	return {
		key,
		description,
		action,
		modifiers
	};
}

/**
 * Group shortcuts by category
 */
export function groupShortcutsByCategory(
	shortcuts: Array<{ key: string; description: string; category?: string }>
): Map<string, Array<{ key: string; description: string }>> {
	const groups = new Map<string, Array<{ key: string; description: string }>>();
	
	shortcuts.forEach(shortcut => {
		const category = shortcut.category || 'Other';
		if (!groups.has(category)) {
			groups.set(category, []);
		}
		groups.get(category)!.push({ key: shortcut.key, description: shortcut.description });
	});
	
	return groups;
}

/**
 * Filter shortcuts based on context
 */
export function filterShortcutsByContext(
	shortcuts: KeyboardShortcut[],
	context?: string
): KeyboardShortcut[] {
	if (!context) return shortcuts;
	
	// This can be extended to filter based on context
	// For now, return all shortcuts
	return shortcuts;
}

