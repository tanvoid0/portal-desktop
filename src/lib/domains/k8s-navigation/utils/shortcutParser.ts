// Parse and match keyboard shortcuts

import { normalizeKey, isModifierKey } from './keyboardConstants';

// Re-export normalizeKey for use in other modules
export { normalizeKey };

export interface ParsedShortcut {
	key: string;
	ctrl?: boolean;
	alt?: boolean;
	shift?: boolean;
	meta?: boolean;
}

export function parseShortcut(shortcut: string): ParsedShortcut {
	const parts = shortcut.toLowerCase().split(/[+-\s]/).filter(Boolean);
	const parsed: ParsedShortcut = { key: '' };
	
	for (const part of parts) {
		switch (part) {
			case 'ctrl':
			case 'control':
				parsed.ctrl = true;
				break;
			case 'alt':
			case 'option':
				parsed.alt = true;
				break;
			case 'shift':
				parsed.shift = true;
				break;
			case 'meta':
			case 'cmd':
			case 'command':
				parsed.meta = true;
				break;
			default:
				parsed.key = normalizeKey(part);
		}
	}
	
	return parsed;
}

export function matchShortcut(
	event: KeyboardEvent,
	shortcut: ParsedShortcut | string
): boolean {
	const parsed = typeof shortcut === 'string' ? parseShortcut(shortcut) : shortcut;
	const eventKey = normalizeKey(event.key);
	
	// Check modifiers
	if (parsed.ctrl !== undefined && parsed.ctrl !== event.ctrlKey) return false;
	if (parsed.alt !== undefined && parsed.alt !== event.altKey) return false;
	if (parsed.shift !== undefined && parsed.shift !== event.shiftKey) return false;
	if (parsed.meta !== undefined && parsed.meta !== event.metaKey) return false;
	
	// Check key (ignore if it's a modifier key)
	if (isModifierKey(event.key)) return false;
	
	return eventKey === parsed.key;
}

export function formatShortcut(shortcut: ParsedShortcut | string): string {
	const parsed = typeof shortcut === 'string' ? parseShortcut(shortcut) : shortcut;
	const parts: string[] = [];
	
	if (parsed.ctrl) parts.push('Ctrl');
	if (parsed.alt) parts.push('Alt');
	if (parsed.shift) parts.push('Shift');
	if (parsed.meta) parts.push('Cmd');
	
	parts.push(parsed.key);
	
	return parts.join('+');
}

