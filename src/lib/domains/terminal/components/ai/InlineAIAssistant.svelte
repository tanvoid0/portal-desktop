<!--
	Inline AI Assistant Component
	AI assistant panel that appears alongside terminal
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Bot, Send, Sparkles, X } from 'lucide-svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { cn } from '$lib/utils';
	import type { CommandBlock } from '../../stores/aiTerminalStore';

	interface Props {
		commandBlocks: CommandBlock[];
		onSuggestCommand?: (command: string) => void;
		onExplainOutput?: (blockId: string) => void;
		onAskQuestion?: (question: string) => void;
	}

	let {
		commandBlocks,
		onSuggestCommand,
		onExplainOutput,
		onAskQuestion
	}: Props = $props();

	let messages = $state<Array<{ role: 'user' | 'assistant'; content: string; timestamp: Date }>>(
		[]
	);
	let inputValue = $state('');
	let isLoading = $state(false);
	let suggestions = $state<string[]>([]);

	// Generate suggestions based on recent commands
	$effect(() => {
		if (commandBlocks.length > 0) {
			const recentCommands = commandBlocks.slice(-3).map(b => b.command);
			suggestions = generateSuggestions(recentCommands);
		}
	});

	function generateSuggestions(commands: string[]): string[] {
		// Simple suggestion generation based on commands
		const suggestions: string[] = [];
		
		if (commands.some(c => c.includes('git'))) {
			suggestions.push('Explain the git commands');
			suggestions.push('Suggest next git steps');
		}
		if (commands.some(c => c.includes('npm') || c.includes('yarn'))) {
			suggestions.push('Explain package management');
			suggestions.push('Check for updates');
		}
		if (commands.some(c => c.includes('ls') || c.includes('dir'))) {
			suggestions.push('List files in detail');
			suggestions.push('Find specific files');
		}
		
		return suggestions.slice(0, 3);
	}

	async function handleSend() {
		const question = inputValue.trim();
		if (!question || isLoading) return;

		// Add user message
		messages = [...messages, { role: 'user', content: question, timestamp: new Date() }];
		inputValue = '';
		isLoading = true;

		// Simulate AI response (replace with actual AI integration)
		setTimeout(() => {
			const response = generateAIResponse(question);
			messages = [
				...messages,
				{ role: 'assistant', content: response, timestamp: new Date() }
			];
			isLoading = false;
		}, 1000);

		if (onAskQuestion) {
			onAskQuestion(question);
		}
	}

	function generateAIResponse(question: string): string {
		// Placeholder AI response - replace with actual AI integration
		const lowerQuestion = question.toLowerCase();
		
		if (lowerQuestion.includes('explain') || lowerQuestion.includes('what')) {
			return `I can help explain that. Based on your recent commands, I can see you're working with terminal commands. Would you like me to explain any specific command or suggest next steps?`;
		}
		
		if (lowerQuestion.includes('suggest') || lowerQuestion.includes('next')) {
			return `Based on your command history, here are some suggestions:\n\n1. Check command output for errors\n2. Verify file permissions\n3. Review recent changes\n\nWould you like me to generate specific commands?`;
		}
		
		return `I understand you're asking: "${question}". I can help with:\n\n- Explaining command outputs\n- Suggesting next commands\n- Debugging errors\n- Optimizing workflows\n\nWhat would you like help with?`;
	}

	function handleSuggestion(suggestion: string) {
		inputValue = suggestion;
		handleSend();
	}

	function handleExplainOutput(blockId: string) {
		const block = commandBlocks.find(b => b.id === blockId);
		if (!block) return;

		const question = `Explain the output of: ${block.command}`;
		inputValue = question;
		handleSend();

		if (onExplainOutput) {
			onExplainOutput(blockId);
		}
	}

	function handleSuggestCommand() {
		const suggestion = 'Suggest a command based on my recent activity';
		inputValue = suggestion;
		handleSend();

		if (onSuggestCommand) {
			onSuggestCommand(suggestion);
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	}
</script>

<div class="flex flex-col h-full bg-gray-900 border-l border-gray-700">
	<!-- Header -->
	<div class="flex items-center justify-between p-4 border-b border-gray-700">
		<div class="flex items-center gap-2">
			<Bot class="h-5 w-5 text-blue-400" />
			<h3 class="text-sm font-semibold text-gray-200">AI Assistant</h3>
		</div>
		<Sparkles class="h-4 w-4 text-blue-400" />
	</div>

	<!-- Messages -->
	<div class="flex-1 overflow-y-auto p-4 space-y-4">
		{#if messages.length === 0}
			<div class="text-center text-gray-500 text-sm py-8">
				<Bot class="h-12 w-12 mx-auto mb-3 text-gray-600" />
				<p class="mb-2">AI Assistant is ready to help!</p>
				<p class="text-xs">Ask questions, get explanations, or request command suggestions.</p>
			</div>
		{/if}

		{#each messages as message}
			<div
				class={cn(
					'flex',
					message.role === 'user' ? 'justify-end' : 'justify-start'
				)}
			>
				<div
					class={cn(
						'max-w-[80%] rounded-lg p-3 text-sm',
						message.role === 'user'
							? 'bg-blue-600 text-white'
							: 'bg-gray-800 text-gray-200'
					)}
				>
					<div class="whitespace-pre-wrap break-words">{message.content}</div>
					<div
						class={cn(
							'text-xs mt-1',
							message.role === 'user' ? 'text-blue-100' : 'text-gray-400'
						)}
					>
						{message.timestamp.toLocaleTimeString()}
					</div>
				</div>
			</div>
		{/each}

		{#if isLoading}
			<div class="flex justify-start">
				<div class="bg-gray-800 rounded-lg p-3 text-sm text-gray-400">
					<div class="flex items-center gap-2">
						<div class="animate-spin h-4 w-4 border-2 border-gray-400 border-t-transparent rounded-full"></div>
						<span>Thinking...</span>
					</div>
				</div>
			</div>
		{/if}
	</div>

	<!-- Suggestions -->
	{#if suggestions.length > 0 && messages.length === 0}
		<div class="px-4 pb-2">
			<div class="text-xs text-gray-500 mb-2">Suggestions:</div>
			<div class="flex flex-wrap gap-2">
				{#each suggestions as suggestion}
					<button
						type="button"
						onclick={() => handleSuggestion(suggestion)}
						class="text-xs px-2 py-1 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded border border-gray-700"
					>
						{suggestion}
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Input -->
	<div class="p-4 border-t border-gray-700">
		<div class="flex items-center gap-2">
			<Input
				bind:value={inputValue}
				onkeydown={handleKeydown}
				placeholder="Ask AI assistant..."
				class="flex-1 bg-gray-800 border-gray-700 text-sm"
				disabled={isLoading}
			/>
			<Button onclick={handleSend} size="sm" disabled={isLoading || !inputValue.trim()}>
				<Send class="h-4 w-4" />
			</Button>
		</div>
	</div>
</div>

<style>
	/* Scrollbar styling */
	:global(.ai-assistant-messages) {
		scrollbar-width: thin;
		scrollbar-color: #4b5563 #1f2937;
	}

	:global(.ai-assistant-messages)::-webkit-scrollbar {
		width: 6px;
	}

	:global(.ai-assistant-messages)::-webkit-scrollbar-track {
		background: #1f2937;
	}

	:global(.ai-assistant-messages)::-webkit-scrollbar-thumb {
		background: #4b5563;
		border-radius: 3px;
	}
</style>

