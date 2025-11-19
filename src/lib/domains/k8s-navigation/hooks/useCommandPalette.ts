// Hook for command palette state and logic

import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
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
	
	const isOpen = writable(false);
	const query = writable('');
	const selectedIndex = writable(0);
	
	const filteredCommands: Readable<Command[]> = derived(
		[query],
		([$query]) => {
			if (!$query.trim()) {
				return commands.slice(0, maxResults);
			}
			
			const results = fuzzySearch(
				$query,
				commands,
				(cmd) => cmd.label,
				(cmd) => cmd.keywords || []
			);
			
			return results
				.slice(0, maxResults)
				.map(result => result.item);
		}
	);
	
	const selectedCommand: Readable<Command | null> = derived(
		[selectedIndex, filteredCommands],
		([$selectedIndex, $filteredCommands]) => {
			if ($selectedIndex >= 0 && $selectedIndex < $filteredCommands.length) {
				return $filteredCommands[$selectedIndex];
			}
			return null;
		}
	);
	
	function open() {
		isOpen.set(true);
		query.set('');
		selectedIndex.set(0);
	}
	
	function close() {
		isOpen.set(false);
		query.set('');
		selectedIndex.set(0);
	}
	
	function setQuery(newQuery: string) {
		query.set(newQuery);
		selectedIndex.set(0);
	}
	
	function selectNext() {
		const currentIndex = get(selectedIndex);
		const currentFiltered = get(filteredCommands);
		if (currentIndex < currentFiltered.length - 1) {
			selectedIndex.set(currentIndex + 1);
		}
	}
	
	function selectPrevious() {
		const currentIndex = get(selectedIndex);
		if (currentIndex > 0) {
			selectedIndex.set(currentIndex - 1);
		}
	}
	
	async function executeSelected() {
		const command = get(selectedCommand);
		if (command) {
			await command.action();
			close();
		}
	}
	
	function handleKeydown(event: KeyboardEvent): boolean {
		const $isOpen = get(isOpen);
		
		// Open command palette
		if (NAVIGATION_SHORTCUTS.COMMAND_PALETTE.includes(event.key as '/' | ':') && !$isOpen) {
			// Only if not in input
			const target = event.target as HTMLElement;
			if (target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
				event.preventDefault();
				open();
				return true;
			}
		}
		
		if (!$isOpen) return false;
		
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
		isOpen,
		query,
		selectedIndex,
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
