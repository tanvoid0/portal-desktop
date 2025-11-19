// Hook for namespace switching shortcuts

import { readable, get, type Readable } from 'svelte/store';
import { NAVIGATION_SHORTCUTS } from '../utils/keyboardConstants';
import type { NamespaceOption } from '../types';

export interface UseNamespaceShortcutsOptions {
	namespaces: NamespaceOption[] | (() => NamespaceOption[]);
	selectedNamespace: string;
	onSelect: (namespace: string) => void | Promise<void>;
	enabled?: boolean;
}

export function useNamespaceShortcuts(options: UseNamespaceShortcutsOptions) {
	const { selectedNamespace, onSelect, enabled = true } = options;
	
	// Get namespaces reactively - support both array and getter function
	const getNamespaces = typeof options.namespaces === 'function' 
		? options.namespaces 
		: () => options.namespaces;
	
	// Map namespaces to number shortcuts (0-9)
	const namespaceShortcuts: Readable<Map<number, NamespaceOption>> = readable(
		new Map<number, NamespaceOption>(),
		(set) => {
			// Compute shortcuts
			const namespacesResult = getNamespaces();
			const namespaces: NamespaceOption[] = Array.isArray(namespacesResult) ? namespacesResult : namespacesResult();
			const shortcuts = new Map<number, NamespaceOption>();
			const allOption: NamespaceOption = { value: '', label: 'All Namespaces' };
			shortcuts.set(0, allOption);
			
			namespaces.slice(0, 9).forEach((ns: NamespaceOption, index: number) => {
				shortcuts.set(index + 1, ns);
			});
			
			set(shortcuts);
		}
	);
	
	function selectByNumber(number: number) {
		const shortcuts = get(namespaceShortcuts);
		const namespace = shortcuts.get(number);
		if (namespace) {
			onSelect(namespace.value);
		}
	}
	
	function handleKeydown(event: KeyboardEvent): boolean {
		if (!enabled) return false;
		
		// Ignore if typing in input/textarea
		const target = event.target as HTMLElement;
		if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
			return false;
		}
		
		// Handle namespace switch shortcut (n or :)
		if ((NAVIGATION_SHORTCUTS.NAMESPACE_SWITCH as readonly string[]).includes(event.key) && !event.ctrlKey && !event.metaKey) {
			// This will be handled by command palette
			return false;
		}
		
		// Handle number shortcuts (0-9)
		if (!event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey) {
			const number = parseInt(event.key);
			if (!isNaN(number) && number >= 0 && number <= 9) {
				const shortcuts = get(namespaceShortcuts);
				const namespace = shortcuts.get(number);
				if (namespace) {
					event.preventDefault();
					selectByNumber(number);
					return true;
				}
			}
		}
		
		return false;
	}
	
	return {
		namespaceShortcuts,
		selectByNumber,
		handleKeydown
	};
}
