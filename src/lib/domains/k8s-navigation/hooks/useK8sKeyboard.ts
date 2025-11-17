// K8s-specific keyboard hook - wraps generic useKeyboard with K8s-specific functionality
// This hook combines table navigation, resource actions, and command palette for K8s pages

import { useTableNavigation, type TableNavigationOptions } from './useTableNavigation';
import { useResourceActions, type UseResourceActionsOptions } from './useResourceActions';
import { useCommandPalette, type UseCommandPaletteOptions } from './useCommandPalette';
import { useKeyboardShortcuts } from './useKeyboardShortcuts';
import { NAVIGATION_SHORTCUTS } from '../utils/keyboardConstants';
import type { Command, KeyboardShortcut } from '../types';

export interface K8sKeyboardConfig {
	// Table navigation
	tableNavigation?: TableNavigationOptions | (() => TableNavigationOptions);
	
	// Resource actions
	resourceActions?: UseResourceActionsOptions;
	
	// Command palette
	commandPalette?: UseCommandPaletteOptions;
	
	// Custom shortcuts
	customShortcuts?: KeyboardShortcut[];
	
	// Global shortcuts (help, etc.)
	globalShortcuts?: KeyboardShortcut[];
	
	// Enable/disable all keyboard handling
	enabled?: boolean;
	
	// Context identifier for this keyboard handler
	context?: string;
}

export interface K8sKeyboardReturn {
	// Table navigation
	tableNav?: ReturnType<typeof useTableNavigation>;
	
	// Resource actions
	resourceActions?: ReturnType<typeof useResourceActions>;
	
	// Command palette
	commandPalette?: ReturnType<typeof useCommandPalette>;
	
	// Custom shortcuts handler
	customShortcuts?: ReturnType<typeof useKeyboardShortcuts>;
	
	// Global shortcuts handler
	globalShortcuts?: ReturnType<typeof useKeyboardShortcuts>;
	
	// Unified keydown handler
	handleKeydown: (event: KeyboardEvent) => boolean;
	
	// Get all available shortcuts for display
	getShortcuts: () => Array<{ key: string; description: string; category?: string }>;
}

export function useK8sKeyboard(config: K8sKeyboardConfig): K8sKeyboardReturn {
	const {
		tableNavigation,
		resourceActions,
		commandPalette,
		customShortcuts: customShortcutsConfig,
		globalShortcuts: globalShortcutsConfig,
		enabled = true,
		context
	} = config;
	
	// Initialize hooks - support both direct values and getter functions for reactivity
	const tableNav = tableNavigation 
		? useTableNavigation(typeof tableNavigation === 'function' ? tableNavigation() : tableNavigation) 
		: undefined;
	const resourceActionsHook = resourceActions ? useResourceActions(resourceActions) : undefined;
	const commandPaletteHook = commandPalette ? useCommandPalette(commandPalette) : undefined;
	const customShortcuts = customShortcutsConfig ? useKeyboardShortcuts(customShortcutsConfig, { enabled }) : undefined;
	const globalShortcuts = globalShortcutsConfig ? useKeyboardShortcuts(globalShortcutsConfig, { enabled }) : undefined;
	
	// Unified keydown handler
	function handleKeydown(event: KeyboardEvent): boolean {
		if (!enabled) return false;
		
		// Priority order:
		// 1. Command palette (highest - can intercept all)
		if (commandPaletteHook?.handleKeydown(event)) {
			return true;
		}
		
		// 2. Custom shortcuts
		if (customShortcuts?.handleKeydown(event)) {
			return true;
		}
		
		// 3. Resource actions
		if (resourceActionsHook?.handleKeydown(event)) {
			return true;
		}
		
		// 4. Table navigation
		if (tableNav?.handleKeydown(event)) {
			return true;
		}
		
		// 5. Global shortcuts (lowest priority)
		if (globalShortcuts?.handleKeydown(event)) {
			return true;
		}
		
		return false;
	}
	
	// Get all available shortcuts for display
	function getShortcuts(): Array<{ key: string; description: string; category?: string }> {
		const shortcuts: Array<{ key: string; description: string; category?: string }> = [];
		
		// Table navigation shortcuts
		if (tableNav) {
			shortcuts.push(
				{ key: '↑/k', description: 'Move up', category: 'Navigation' },
				{ key: '↓/j', description: 'Move down', category: 'Navigation' },
				{ key: 'Enter', description: 'Activate selected', category: 'Navigation' },
				{ key: 'g', description: 'Go to top', category: 'Navigation' },
				{ key: 'G', description: 'Go to bottom', category: 'Navigation' }
			);
		}
		
		// Resource action shortcuts
		if (resourceActionsHook) {
			const actions = resourceActionsHook.actions;
			actions.forEach((action: any) => {
				shortcuts.push({
					key: action.shortcut,
					description: action.label,
					category: 'Actions'
				});
			});
		}
		
		// Custom shortcuts
		if (customShortcutsConfig) {
			customShortcutsConfig.forEach(shortcut => {
				shortcuts.push({
					key: shortcut.key,
					description: shortcut.description || '',
					category: 'Custom'
				});
			});
		}
		
		// Global shortcuts
		if (globalShortcutsConfig) {
			globalShortcutsConfig.forEach(shortcut => {
				shortcuts.push({
					key: shortcut.key,
					description: shortcut.description || '',
					category: 'Global'
				});
			});
		}
		
		// Command palette
		if (commandPaletteHook) {
			shortcuts.push(
				{ key: '/', description: 'Open command palette', category: 'Global' },
				{ key: ':', description: 'Open command palette', category: 'Global' }
			);
		}
		
		return shortcuts;
	}
	
	// Note: Individual hooks handle their own event listeners
	// This unified handler can be attached manually if needed
	
	return {
		tableNav,
		resourceActions: resourceActionsHook,
		commandPalette: commandPaletteHook,
		customShortcuts,
		globalShortcuts,
		handleKeydown,
		getShortcuts
	};
}

