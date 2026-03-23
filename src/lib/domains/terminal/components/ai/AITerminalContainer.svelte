<!--
	AI Terminal Container
	Main container managing command blocks, terminal process, and AI integration
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { TerminalService } from '../../services/terminalService';
	import { CommandInterceptionService } from '../../services/commandInterceptionService';
	import {
		aiTerminalStore,
		type CommandBlock as CommandBlockType
	} from '../../stores/aiTerminalStore';
	import { commandHistoryStore } from '../../stores/commandHistoryStore';
	import { parseCommandOutput } from '../../utils/outputParser';
	import CommandBlock from './CommandBlock.svelte';
	import CommandInput from './CommandInput.svelte';
	import InlineAIAssistant from './InlineAIAssistant.svelte';
	import { Terminal, Monitor, Bot, Trash2, Plus, X, RotateCcw, Folder } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Sparkles } from 'lucide-svelte';
	import { cn } from '$lib/utils';
	import type { TerminalProcess, TerminalOutput } from '../../types';
	import { isTauriEnvironment } from '@/lib/utils/tauri';
	import { get } from 'svelte/store';
	import { aiChatService } from '$lib/domains/ai/services/aiChatService';

	const TAB_ID = 'ai-terminal';
	const isTauri = isTauriEnvironment();

	let currentProcess = $state<TerminalProcess | null>(null);
	let unsubscribe: (() => void) | null = null;
	let detectedShell = $state<'bash' | 'zsh' | 'powershell' | 'cmd' | 'fish' | 'sh' | undefined>(undefined);
	let workingDirectory = $state<string>(navigator.userAgent.includes('Windows') ? 'C:\\' : '/home/tan');
	
	// Use reactive store subscription with $state for reactivity
	let storeState = $state(get(aiTerminalStore));

	// Subscribe to store updates - update state reactively
	const storeUnsubscribe = aiTerminalStore.subscribe((state) => {
		storeState = state;
	});

	// Reactive derived values from store state
	const viewMode = $derived(storeState.viewMode);
	const commandBlocks = $derived(storeState.commandBlocks);
	const isConnected = $derived(storeState.isConnected);
	const activeBlockId = $derived(storeState.activeBlockId);

	// Check if any command is currently running or paused
	const hasRunningCommand = $derived(
		commandBlocks.some(b => b.status === 'running' || b.status === 'paused')
	);

	onMount(() => {
		console.log('AITerminalContainer mounted');
		// Initialize terminal process
		initializeTerminal();

		// Add keyboard shortcuts
		const handleKeyboard = (e: KeyboardEvent) => {
			// Ctrl+L to clear screen
			if (e.ctrlKey && e.key === 'l') {
				e.preventDefault();
				clearScreen();
			}
		};

		window.addEventListener('keydown', handleKeyboard);

		return () => {
			window.removeEventListener('keydown', handleKeyboard);
		};
	});

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
		if (storeUnsubscribe) {
			storeUnsubscribe();
		}
		// Don't kill process - let it run in background
	});

	async function initializeTerminal() {
		if (!isTauri) {
			console.warn('AI Terminal requires Tauri environment');
			return;
		}

		try {
			const shellCommand = navigator.userAgent.includes('Windows') ? 'powershell.exe' : 'zsh';
			const process = await TerminalService.createProcess(TAB_ID, {
				shell: shellCommand,
				working_directory: navigator.userAgent.includes('Windows') ? 'C:\\' : '/home/tan',
				cols: 80,
				rows: 24
			});
			
			// Set detected shell based on command
			if (shellCommand.includes('powershell')) {
				detectedShell = 'powershell';
			} else if (shellCommand.includes('zsh')) {
				detectedShell = 'zsh';
			} else if (shellCommand.includes('bash')) {
				detectedShell = 'bash';
			}

			currentProcess = process;
			aiTerminalStore.setCurrentProcess(process);
			aiTerminalStore.setConnected(true);

			// Update working directory
			workingDirectory = process.working_directory;

			// Subscribe to output
			unsubscribe = await TerminalService.subscribeToOutput(process.id, handleOutput);

			// Resize terminal
			if (process) {
				await TerminalService.resizeTerminal(process.id, 80, 24);
			}
		} catch (error) {
			console.error('Failed to initialize terminal:', error);
			aiTerminalStore.setConnected(false);
		}
	}

	function handleOutput(output: TerminalOutput) {
		console.log('[AITerminal] Received output:', {
			type: output.output_type,
			content: output.content?.substring(0, 100),
			hasContent: !!output.content,
			activeBlockId
		});

		if (!activeBlockId) {
			console.warn('[AITerminal] No active block ID');
			return;
		}

		// Append raw output to active block
		aiTerminalStore.appendOutput(activeBlockId, output.content);

		// Detect shell type from output if not already detected
		if (!detectedShell && output.content) {
			const shellMatch = output.content.match(/(zsh|bash|powershell|cmd|fish|sh)/i);
			if (shellMatch) {
				detectedShell = shellMatch[1].toLowerCase() as any;
				console.log('[AITerminal] Detected shell:', detectedShell);
			}
		}

		// Check for input prompts
		// Use raw output for detection (prompts might be in raw format)
		const activeBlock = aiTerminalStore.getActiveBlock();
		if (activeBlock) {
			console.log('[AITerminal] Active block rawOutput length:', activeBlock.rawOutput?.length);

			// IMPORTANT: Check the raw, unparsed output for prompts FIRST
			// Password prompts often appear in raw terminal output before parsing
			const rawInterception = CommandInterceptionService.checkForInputPrompt(activeBlock.rawOutput);

			if (rawInterception.needsInput && rawInterception.prompt) {
				console.log('[AITerminal] Input prompt detected:', rawInterception.prompt.type);
				// Extract prompt text from raw output (most reliable)
				const promptText = CommandInterceptionService.extractPromptText(
					activeBlock.rawOutput,
					rawInterception.prompt
				);

				// Set the block as waiting for input
				aiTerminalStore.setBlockWaitingForInput(
					activeBlockId,
					promptText,
					rawInterception.prompt.type
				);

				// Parse and display the output (keeping the prompt visible in display)
				aiTerminalStore.parseAndSetOutput(activeBlockId, activeBlock.command, detectedShell);
			} else {
				// No prompt detected, update parsed output for display
				const parsedOutput = parseCommandOutput(activeBlock.rawOutput, activeBlock.command, detectedShell);
				if (parsedOutput !== activeBlock.output) {
					console.log('[AITerminal] Updating parsed output');
					aiTerminalStore.parseAndSetOutput(activeBlockId, activeBlock.command, detectedShell);
				}
			}
		}

		// Check for process exit or command completion
		if (output.output_type === 'exit') {
			console.log('[AITerminal] Process exit detected');
			// Get exit code from backend
			TerminalService.getProcess(output.process_id).then((process) => {
				console.log('[AITerminal] Exit code:', process?.exit_code);
				if (activeBlockId) {
					const block = aiTerminalStore.getActiveBlock();
					if (block) {
						// Parse the final output when command completes
						aiTerminalStore.parseAndSetOutput(activeBlockId, block.command, detectedShell);
					}
					aiTerminalStore.completeBlock(activeBlockId, process?.exit_code);
				}
			});
		} else {
			// Check if we see a prompt (command completed)
			// Look for common prompt patterns in the output
			const promptPatterns = [
				/\n[^\s]+@[^\s]+:[^\s]+[%$#>❯]\s*$/,
				/\nPS [A-Z]:[\\][^>]*>\s*$/,
				/\n[A-Z]:[\\][^>]*>\s*$/,
				/\n[^\s]+[%$#>❯]\s*$/,
				/[%$#>❯]\s*$/ // More lenient - just look for prompt char at end
			];

			const hasPrompt = activeBlock ? promptPatterns.some(pattern => {
				const match = pattern.test(activeBlock.rawOutput);
				if (match) {
					console.log('[AITerminal] Prompt pattern matched:', pattern);
				}
				return match;
			}) : false;

			if (hasPrompt && activeBlockId) {
				const block = aiTerminalStore.getActiveBlock();
				if (block && block.status === 'running') {
					console.log('[AITerminal] Command appears complete, will mark as done');
					// Command appears to have completed, parse the output
					// Use a small delay to ensure all output chunks are received
					setTimeout(() => {
						const currentBlock = aiTerminalStore.getActiveBlock();
						if (currentBlock && currentBlock.id === activeBlockId && currentBlock.status === 'running') {
							console.log('[AITerminal] Marking command as completed');
							aiTerminalStore.parseAndSetOutput(activeBlockId, currentBlock.command, detectedShell);
							aiTerminalStore.completeBlock(activeBlockId, 0);
						}
					}, 300); // Small delay to ensure all output is received
				}
			}
		}
	}

	async function executeCommand(command: string, isAIMode: boolean = false) {
		if (!currentProcess || !isTauri) {
			console.warn('Terminal not connected');
			return;
		}

		// Check if there's already a running command
		// Note: The input should already be disabled via hasRunningCommand, but this provides extra safety
		if (hasRunningCommand) {
			console.warn('Command already running, cannot execute new command');
			return;
		}

		// If AI mode is enabled, handle it as an AI query
		if (isAIMode) {
			await handleAIQuery(command);
			return;
		}

		// Check if command should be intercepted before execution
		if (CommandInterceptionService.shouldInterceptBeforeExecution(command)) {
			const reason = CommandInterceptionService.getInterceptionReason(command);
			if (reason) {
				// For now, just log - in full implementation, show confirmation dialog
				const confirmed = confirm(reason);
				if (!confirmed) {
					return;
				}
			}
		}

		// Create command block
		const blockId = aiTerminalStore.addCommandBlock(command);
		aiTerminalStore.updateBlockStatus(blockId, 'running');

		// Track command for history
		TerminalService.startCommandTracking(command, TAB_ID);

		// Save to history store (note: full entry will be updated when command completes)
		commandHistoryStore.addEntry(TAB_ID, {
			command,
			output: '',
			duration: 0
		});

		// Send command to terminal
		try {
			await TerminalService.sendInput(currentProcess.id, command + '\n', TAB_ID);
		} catch (error) {
			console.error('Failed to execute command:', error);
			aiTerminalStore.updateBlockStatus(blockId, 'failed');
			aiTerminalStore.completeBlock(blockId, 1);
		}
	}

	/**
	 * Handle AI query (natural language question to AI)
	 */
	async function handleAIQuery(query: string) {
		console.log('Processing AI query:', query);

		// Create a command block to display the AI interaction
		const blockId = aiTerminalStore.addCommandBlock(`/ai ${query}`);
		aiTerminalStore.updateBlockStatus(blockId, 'running');

		try {
			// Stream the AI response
			await aiChatService.streamMessage(
				query,
				[], // No history for now - could be extended to maintain conversation context
				{
					onChunk: (chunk: string) => {
						// Append each chunk as it arrives
						aiTerminalStore.appendOutput(blockId, chunk);
					},
					onComplete: (fullMessage: string) => {
						// Mark as complete when done
						console.log('AI response complete:', fullMessage);
						aiTerminalStore.updateBlockStatus(blockId, 'completed');
						aiTerminalStore.completeBlock(blockId, 0); // Exit code 0 for success
					},
					onError: (error: Error) => {
						// Handle errors
						console.error('AI query failed:', error);
						const errorMessage = `\n\nError: ${error.message}`;
						aiTerminalStore.appendOutput(blockId, errorMessage);
						aiTerminalStore.updateBlockStatus(blockId, 'failed');
						aiTerminalStore.completeBlock(blockId, 1); // Exit code 1 for error
					}
				}
			);
		} catch (error) {
			console.error('Failed to process AI query:', error);
			const errorMessage = error instanceof Error ? error.message : 'Unknown error';
			aiTerminalStore.appendOutput(blockId, `\n\nError: ${errorMessage}`);
			aiTerminalStore.updateBlockStatus(blockId, 'failed');
			aiTerminalStore.completeBlock(blockId, 1);
		}
	}

	function handleProvideInput(blockId: string, input: string) {
		if (!currentProcess) return;

		const block = commandBlocks.find((b: CommandBlockType) => b.id === blockId);
		if (!block || !block.waitingForInput) return;

		// Normalize input based on type
		const normalizedInput = CommandInterceptionService.normalizeInput(
			input,
			block.inputType || 'text'
		);

		// Provide input to terminal
		TerminalService.sendInput(currentProcess.id, normalizedInput, TAB_ID).catch(
			(error) => {
				console.error('Failed to provide input:', error);
			}
		);

		// Update block status
		aiTerminalStore.provideInput(blockId, input);
		aiTerminalStore.updateBlockStatus(blockId, 'running');
	}

	function handleRerunCommand(command: string) {
		executeCommand(command);
	}

	function handleCopyCommand(command: string) {
		navigator.clipboard.writeText(command);
	}

	function handleCopyOutput(output: string) {
		navigator.clipboard.writeText(output);
	}

	function handleEditCommand(command: string) {
		// In a full implementation, this would open an edit dialog
		// For now, just execute the edited command
		executeCommand(command);
	}

	function handleInterceptCommand(command: string): boolean {
		// Check if command should be intercepted
		if (CommandInterceptionService.shouldInterceptBeforeExecution(command)) {
			const reason = CommandInterceptionService.getInterceptionReason(command);
			if (reason) {
				return !confirm(reason);
			}
		}
		return false;
	}

	function setViewMode(mode: 'terminal' | 'ai-terminal' | 'ai-only') {
		aiTerminalStore.setViewMode(mode);
	}

	function clearBlocks() {
		aiTerminalStore.clearBlocks();
	}

	function clearScreen() {
		// Clear all command blocks
		aiTerminalStore.clearBlocks();
	}

	async function handleStopCommand(blockId?: string) {
		if (!currentProcess) return;

		// Send Ctrl+C interrupt signal
		try {
			await TerminalService.sendInput(currentProcess.id, '\x03', TAB_ID);
			
			// If a specific block ID is provided, mark it as stopped
			if (blockId) {
				aiTerminalStore.updateBlockStatus(blockId, 'failed');
				aiTerminalStore.completeBlock(blockId, 130); // 130 is typical exit code for SIGINT
			} else if (activeBlockId) {
				// Stop the currently active block
				aiTerminalStore.updateBlockStatus(activeBlockId, 'failed');
				aiTerminalStore.completeBlock(activeBlockId, 130);
			}
		} catch (error) {
			console.error('Failed to stop command:', error);
		}
	}
</script>

<div class="flex flex-col h-full w-full bg-background overflow-hidden" data-testid="ai-terminal-container">
	<!-- Header -->
	<div class="flex items-center justify-between px-4 py-3 border-b border-border bg-card">
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-2">
				<Terminal class="h-5 w-5 text-primary" />
				<Sparkles class="h-4 w-4 text-purple-500" />
			</div>
			<div>
				<h1 class="text-lg font-semibold text-foreground">Terminal</h1>
				<p class="text-xs text-muted-foreground">Run commands or ask AI questions</p>
			</div>
			<Badge variant={isConnected ? 'default' : 'secondary'} class="gap-1.5">
				<div
					class={cn(
						'w-1.5 h-1.5 rounded-full',
						isConnected ? 'bg-green-500' : 'bg-yellow-500'
					)}
				></div>
				{isConnected ? 'Connected' : 'Disconnected'}
			</Badge>
		</div>

		<!-- Actions -->
		<div class="flex items-center gap-2">
			<!-- Working Directory -->
			<div class="flex items-center gap-1.5 px-2 py-1 rounded bg-muted/50 text-xs text-muted-foreground">
				<Folder class="h-3 w-3" />
				<span class="font-mono">{workingDirectory}</span>
			</div>

			<!-- View Mode Toggle -->
			<Button
				variant={viewMode === 'terminal' ? 'secondary' : 'ghost'}
				size="sm"
				onclick={() => setViewMode('terminal')}
				title="Terminal only"
			>
				<Monitor class="h-4 w-4" />
			</Button>
			<Button
				variant={viewMode === 'ai-terminal' ? 'secondary' : 'ghost'}
				size="sm"
				onclick={() => setViewMode('ai-terminal')}
				title="Terminal + AI"
			>
				<Terminal class="h-4 w-4" />
				<Bot class="h-3 w-3 ml-1" />
			</Button>
			<Button
				variant={viewMode === 'ai-only' ? 'secondary' : 'ghost'}
				size="sm"
				onclick={() => setViewMode('ai-only')}
				title="AI only"
			>
				<Bot class="h-4 w-4" />
			</Button>

			<!-- Clear Screen Button -->
			<Button
				variant="ghost"
				size="sm"
				onclick={clearScreen}
				title="Clear screen (Ctrl+L)"
			>
				<RotateCcw class="h-4 w-4" />
			</Button>
		</div>
	</div>

	<!-- Main Content -->
	<div class="flex flex-1 min-h-0">
		<!-- Terminal View -->
		{#if viewMode === 'terminal' || viewMode === 'ai-terminal'}
			<div
				class={cn(
					'flex flex-col h-full bg-background',
					viewMode === 'ai-terminal' ? 'w-2/3' : 'w-full'
				)}
			>
				<!-- Command Blocks -->
				<div class="flex-1 overflow-y-auto p-4 command-blocks-container">
					{#if commandBlocks.length === 0}
						<div class="flex items-center justify-center h-full text-muted-foreground">
							<div class="text-center max-w-md">
								<div class="flex items-center justify-center gap-2 mb-3">
									<Terminal class="h-12 w-12 text-muted-foreground/50" />
									<Sparkles class="h-8 w-8 text-purple-500/50" />
								</div>
								<p class="text-lg font-medium mb-2">Welcome to AI-Enhanced Terminal</p>
								<p class="text-sm text-muted-foreground mb-4">
									Run any command directly or ask AI questions about your system
								</p>

								<div class="space-y-2 text-left bg-card/50 rounded-lg p-4 border border-border/50">
									<p class="text-sm font-medium text-foreground mb-2">Quick Tips:</p>
									<div class="space-y-1 text-xs text-muted-foreground">
										<p>• <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">ls</kbd> or <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">dir</kbd> - List files</p>
										<p>• <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">cd folder</kbd> - Change directory</p>
										<p>• <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">git status</kbd> - Check git status</p>
										<p>• <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">/ai how do I...</kbd> - Ask AI questions</p>
										<p>• <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">Ctrl+Space</kbd> - Toggle AI mode</p>
										<p>• <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">Ctrl+L</kbd> - Clear screen</p>
										<p>• <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">Ctrl+C</kbd> - Stop running command</p>
									</div>
								</div>
							</div>
						</div>
					{:else}
						{#each commandBlocks as block}
							<CommandBlock
								{block}
								onRerun={handleRerunCommand}
								onCopy={handleCopyCommand}
								onCopyOutput={handleCopyOutput}
								onEdit={handleEditCommand}
								onProvideInput={handleProvideInput}
								onStop={() => handleStopCommand(block.id)}
							/>
						{/each}
					{/if}
				</div>

				<!-- Command Input -->
				<div class="border-t border-border p-4 bg-card/50 backdrop-blur-sm">
					<CommandInput
						onSubmit={executeCommand}
						onIntercept={handleInterceptCommand}
						onStop={() => handleStopCommand()}
						disabled={!isConnected || !isTauri || hasRunningCommand}
						placeholder={hasRunningCommand ? 'Command running... (Ctrl+C to stop)' : 'Enter command or /ai for AI mode...'}
					/>
				</div>
			</div>
		{/if}

		<!-- AI Assistant View -->
		{#if viewMode === 'ai-terminal' || viewMode === 'ai-only'}
			<div
				class={cn(
					'flex flex-col h-full border-l border-border bg-muted/30',
					viewMode === 'ai-terminal' ? 'w-1/3' : 'w-full'
				)}
			>
				<InlineAIAssistant
					{commandBlocks}
					onSuggestCommand={executeCommand}
					onExplainOutput={(blockId) => {
						const block = commandBlocks.find((b: CommandBlockType) => b.id === blockId);
						if (block) {
							console.log('Explain output for:', block.command);
						}
					}}
				/>
			</div>
		{/if}
	</div>
</div>

<style>
	/* Scrollbar styling */
	:global(.command-blocks-container) {
		scrollbar-width: thin;
		scrollbar-color: hsl(var(--muted-foreground) / 0.3) hsl(var(--background));
	}

	:global(.command-blocks-container)::-webkit-scrollbar {
		width: 8px;
	}

	:global(.command-blocks-container)::-webkit-scrollbar-track {
		background: hsl(var(--background));
	}

	:global(.command-blocks-container)::-webkit-scrollbar-thumb {
		background: hsl(var(--muted-foreground) / 0.3);
		border-radius: 4px;
	}

	:global(.command-blocks-container)::-webkit-scrollbar-thumb:hover {
		background: hsl(var(--muted-foreground) / 0.5);
	}

	kbd {
		font-family: ui-monospace, monospace;
		font-weight: 600;
	}
</style>


