// TypeScript types for k8s-navigation domain

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
}

export interface Command {
	id: string;
	label: string;
	description?: string;
	keywords?: string[];
	action: () => void | Promise<void>;
	category?: 'navigation' | 'namespace' | 'action' | 'resource';
}

export interface NavigationState {
	selectedIndex: number;
	totalItems: number;
}

// TableNavigationOptions is now exported from useTableNavigation.ts

export interface ResourceAction {
	key: string;
	label: string;
	shortcut: string;
	action: (resource: any) => void | Promise<void>;
	enabled?: (resource: any) => boolean;
}

export interface NamespaceOption {
	value: string;
	label: string;
	shortcut?: number; // 0-9 for quick selection
}

