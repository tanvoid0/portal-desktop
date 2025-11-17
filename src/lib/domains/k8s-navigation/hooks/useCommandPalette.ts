// Hook for command palette state and logic

import { fuzzySearch } from '../utils/fuzzySearch';
import { NAVIGATION_SHORTCUTS } from '../utils/keyboardConstants';
import type { Command } from '../types';

// Re-export Command type for convenience
export type { Command };

export interface UseCommandPaletteOptions {
	commands: Command[];
	maxResults?: number;
}

export function useCommandPalette(options: UseCommandPaletteOptions) {
	const { commands, maxResults = 10 } = options;
	
	let isOpen = $state(false);
	let query = $state('');
	let selectedIndex = $state(0);
	
	const filteredCommands = $derived(() => {
		if (!query.trim()) {
			return commands.slice(0, maxResults);
		}
		
		const results = fuzzySearch(
			query,
			commands,
			(cmd) => cmd.label,
			(cmd) => cmd.keywords || []
		);
		
		return results
			.slice(0, maxResults)
			.map(result => result.item);
	});
	
	const selectedCommand = $derived(() => {
		if (selectedIndex >= 0 && selectedIndex < filteredCommands().length) {
			return filteredCommands()[selectedIndex];
		}
		return null;
	});
	
	function open() {
		isOpen = true;
		query = '';
		selectedIndex = 0;
	}
	
	function close() {
		isOpen = false;
		query = '';
		selectedIndex = 0;
	}
	
	function setQuery(newQuery: string) {
		query = newQuery;
		selectedIndex = 0;
	}
	
	function selectNext() {
		if (selectedIndex < filteredCommands().length - 1) {
			selectedIndex++;
		}
	}
	
	function selectPrevious() {
		if (selectedIndex > 0) {
			selectedIndex--;
		}
	}
	
	async function executeSelected() {
		const command = selectedCommand();
		if (command) {
			await command.action();
			close();
		}
	}
	
	function handleKeydown(event: KeyboardEvent): boolean {
		// Open command palette
		if (NAVIGATION_SHORTCUTS.COMMAND_PALETTE.includes(event.key) && !isOpen) {
			// Only if not in input
			const target = event.target as HTMLElement;
			if (target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
				event.preventDefault();
				open();
				return true;
			}
		}
		
		if (!isOpen) return false;
		
		switch (event.key) {
			case 'Escape':
				event.preventDefault();
				close();
				return true;
				
			case 'ArrowDown':
				event.preventDefault();
				selectNext();
				return true;
				
			case 'ArrowUp':
				event.preventDefault();
				selectPrevious();
				return true;
				
			case 'Enter':
				event.preventDefault();
				executeSelected();
				return true;
				
			default:
				return false;
		}
	}
	
	return {
		isOpen: $derived(isOpen),
		query: $derived(query),
		selectedIndex: $derived(selectedIndex),
		filteredCommands,
		selectedCommand,
		open,
		close,
		setQuery,
		selectNext,
		selectPrevious,
		executeSelected,
		handleKeydown
	};
}

