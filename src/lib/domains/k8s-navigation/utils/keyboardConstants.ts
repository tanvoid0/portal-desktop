// Constants for keyboard shortcuts and key mappings

export const KEY_CODES = {
	ARROW_UP: 'ArrowUp',
	ARROW_DOWN: 'ArrowDown',
	ARROW_LEFT: 'ArrowLeft',
	ARROW_RIGHT: 'ArrowRight',
	ENTER: 'Enter',
	ESCAPE: 'Escape',
	TAB: 'Tab',
	SPACE: ' ',
	BACKSPACE: 'Backspace',
	DELETE: 'Delete',
	HOME: 'Home',
	END: 'End',
	PAGE_UP: 'PageUp',
	PAGE_DOWN: 'PageDown',
} as const;

export const VIM_KEYS = {
	UP: 'k',
	DOWN: 'j',
	TOP: 'g',
	BOTTOM: 'G',
} as const;

export const ACTION_SHORTCUTS = {
	DESCRIBE: 'd',
	EDIT: 'e',
	LOGS: 'l',
	RESTART: 'r',
	SCALE: 's',
	YAML: 'y',
	DELETE: 'Delete',
	PORT_FORWARD: 'f',
	REFRESH: 'r',
} as const;

export const NAVIGATION_SHORTCUTS = {
	COMMAND_PALETTE: ['/', ':'],
	HELP: '?',
	NAMESPACE_SWITCH: ['n', ':'],
} as const;

export const RESOURCE_TYPE_SHORTCUTS = {
	OVERVIEW: '0',
	PODS: '1',
	SERVICES: '2',
	DEPLOYMENTS: '3',
	STATEFULSETS: '4',
	DAEMONSETS: '5',
	JOBS: '6',
	CRONJOBS: '7',
	CONFIGMAPS: '8',
	SECRETS: '9',
} as const;

export function isModifierKey(key: string): boolean {
	return ['Control', 'Alt', 'Shift', 'Meta', 'OS'].includes(key);
}

export function normalizeKey(key: string): string {
	// Normalize key names for consistent matching
	const normalized = key.toLowerCase();
	
	// Map common variations
	const keyMap: Record<string, string> = {
		'arrowup': KEY_CODES.ARROW_UP,
		'arrowdown': KEY_CODES.ARROW_DOWN,
		'arrowleft': KEY_CODES.ARROW_LEFT,
		'arrowright': KEY_CODES.ARROW_RIGHT,
		'enter': KEY_CODES.ENTER,
		'escape': KEY_CODES.ESCAPE,
		'esc': KEY_CODES.ESCAPE,
		'tab': KEY_CODES.TAB,
		'space': KEY_CODES.SPACE,
		'backspace': KEY_CODES.BACKSPACE,
		'delete': KEY_CODES.DELETE,
		'del': KEY_CODES.DELETE,
		'home': KEY_CODES.HOME,
		'end': KEY_CODES.END,
		'pageup': KEY_CODES.PAGE_UP,
		'pagedown': KEY_CODES.PAGE_DOWN,
	};
	
	return keyMap[normalized] || key;
}

