<!--
	Command Input Component
	Enhanced input field with AI suggestions and command history
	Supports inline AI mode with /ai prefix or Ctrl+Space toggle
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Command, Sparkles, Terminal as TerminalIcon } from 'lucide-svelte';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { commandHistoryStore } from '../../stores/commandHistoryStore';
	import { cn } from '$lib/utils';

	interface Props {
		onSubmit: (command: string, isAIMode?: boolean) => void;
		onIntercept?: (command: string) => boolean;
		onStop?: () => void;
		disabled?: boolean;
		placeholder?: string;
	}

	let { onSubmit, onIntercept, onStop, disabled = false, placeholder = 'Enter command or /ai for AI mode...' }: Props = $props();

	let inputValue = $state('');
	let historyIndex = $state(-1);
	let suggestions = $state<string[]>([]);
	let showSuggestions = $state(false);
	let inputRef = $state<HTMLInputElement | null>(null);
	let isAIMode = $state(false);

	// Detect AI mode from input
	$effect(() => {
		isAIMode = inputValue.trim().startsWith('/ai ');
	});

	// Get command history for current tab
	const history = $derived.by(() => {
		// Use a default tab ID for AI terminal
		return commandHistoryStore.getTabHistory('ai-terminal') || [];
	});

	// Filter suggestions based on input
	$effect(() => {
		if (inputValue.trim()) {
			const filtered = history
				.filter(entry => entry.command.toLowerCase().includes(inputValue.toLowerCase()))
				.slice(0, 5)
				.map(entry => entry.command);
			suggestions = filtered;
			showSuggestions = filtered.length > 0;
		} else {
			suggestions = [];
			showSuggestions = false;
		}
	});

	function handleSubmit() {
		const command = inputValue.trim();
		if (!command || disabled) return;

		// Extract actual command and AI mode
		let actualCommand = command;
		let aiMode = false;

		if (command.startsWith('/ai ')) {
			actualCommand = command.substring(4).trim();
			aiMode = true;
		}

		// Check if command should be intercepted
		if (onIntercept && onIntercept(actualCommand)) {
			return; // Intercepted, don't submit
		}

		onSubmit(actualCommand, aiMode);
		inputValue = '';
		historyIndex = -1;
		showSuggestions = false;
	}

	function toggleAIMode() {
		if (isAIMode) {
			// Remove /ai prefix
			inputValue = inputValue.substring(4).trim();
		} else {
			// Add /ai prefix
			inputValue = '/ai ' + inputValue.trim();
		}
		// Focus input
		if (inputRef) {
			inputRef.focus();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		// Handle Ctrl+C to stop running command
		if (e.key === 'c' && e.ctrlKey) {
			e.preventDefault();
			if (onStop) {
				onStop();
			}
			return;
		}

		// Handle Ctrl+Space to toggle AI mode
		if (e.key === ' ' && e.ctrlKey) {
			e.preventDefault();
			toggleAIMode();
			return;
		}

		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (historyIndex < history.length - 1) {
				historyIndex++;
				inputValue = history[history.length - 1 - historyIndex].command;
				showSuggestions = false;
			}
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (historyIndex > 0) {
				historyIndex--;
				inputValue = history[history.length - 1 - historyIndex].command;
			} else if (historyIndex === 0) {
				historyIndex = -1;
				inputValue = '';
			}
		} else if (e.key === 'Escape') {
			showSuggestions = false;
			if (inputRef) {
				inputRef.blur();
			}
		} else if (e.key === 'Tab' && suggestions.length > 0 && showSuggestions) {
			e.preventDefault();
			inputValue = suggestions[0];
			showSuggestions = false;
		}
	}

	function selectSuggestion(suggestion: string) {
		inputValue = suggestion;
		showSuggestions = false;
		if (inputRef) {
			inputRef.focus();
		}
	}

	function focusInput() {
		if (inputRef) {
			inputRef.focus();
		}
	}
</script>

<div class="relative">
	<div class="flex items-center gap-2">
		<!-- Mode indicator -->
		<div class="flex-shrink-0">
			{#if isAIMode}
				<Sparkles class="h-4 w-4 text-purple-500" />
			{:else}
				<TerminalIcon class="h-4 w-4 text-muted-foreground" />
			{/if}
		</div>

		<!-- Input field -->
		<div class="flex-1 relative">
			<Input
				bind:ref={inputRef}
				bind:value={inputValue}
				onkeydown={handleKeydown}
				{disabled}
				{placeholder}
				class={cn(
					'font-mono text-sm',
					isAIMode ? 'border-purple-500 focus:border-purple-400 focus:ring-purple-400' : ''
				)}
			/>

			<!-- AI Mode Badge -->
			{#if isAIMode}
				<div class="absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none">
					<Badge variant="secondary" class="text-xs bg-purple-500/20 text-purple-300 border-purple-500/30">
						AI Mode
					</Badge>
				</div>
			{/if}

			<!-- Suggestions Dropdown -->
			{#if showSuggestions && suggestions.length > 0}
				<div
					class="absolute z-10 w-full mt-1 bg-popover border border-border rounded-lg shadow-lg max-h-48 overflow-y-auto suggestions-dropdown"
				>
					{#each suggestions as suggestion}
						<button
							type="button"
							onclick={() => selectSuggestion(suggestion)}
							class="w-full text-left px-3 py-2 hover:bg-accent text-sm font-mono text-foreground transition-colors"
						>
							{suggestion}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<!-- AI Mode Toggle Button -->
		<Button
			variant={isAIMode ? 'default' : 'outline'}
			size="sm"
			onclick={toggleAIMode}
			{disabled}
			class={cn(
				'flex-shrink-0',
				isAIMode && 'bg-purple-600 hover:bg-purple-700 border-purple-500'
			)}
			title="Toggle AI Mode (Ctrl+Space)"
		>
			<Sparkles class="h-4 w-4" />
		</Button>

		<!-- Submit Button -->
		<Button
			onclick={handleSubmit}
			{disabled}
			size="sm"
			class="flex-shrink-0"
		>
			Run
		</Button>
	</div>
</div>

<style>
	/* Suggestions scrollbar */
	:global(.suggestions-dropdown) {
		scrollbar-width: thin;
		scrollbar-color: #4b5563 #1f2937;
	}

	:global(.suggestions-dropdown)::-webkit-scrollbar {
		width: 6px;
	}

	:global(.suggestions-dropdown)::-webkit-scrollbar-track {
		background: #1f2937;
	}

	:global(.suggestions-dropdown)::-webkit-scrollbar-thumb {
		background: #4b5563;
		border-radius: 3px;
	}
</style>

