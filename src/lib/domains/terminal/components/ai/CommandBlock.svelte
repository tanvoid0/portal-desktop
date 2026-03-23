<!--
	Command Block Component
	Displays individual command blocks with streaming output
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { CommandBlock } from '../../stores/aiTerminalStore';
	import { Copy, Play, Edit2, X, Check, AlertCircle, Loader2, Square, FileText } from 'lucide-svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { cn } from '$lib/utils';

	interface Props {
		block: CommandBlock;
		onRerun?: (command: string) => void;
		onCopy?: (command: string) => void;
		onCopyOutput?: (output: string) => void;
		onEdit?: (command: string) => void;
		onProvideInput?: (blockId: string, input: string) => void;
		onStop?: (blockId: string) => void;
	}

	let { block, onRerun, onCopy, onCopyOutput, onEdit, onProvideInput, onStop }: Props = $props();

	let inputValue = $state('');
	let outputRef = $state<HTMLDivElement | null>(null);
	let inputRef = $state<HTMLInputElement | null>(null);
	let showInput = $derived(block.waitingForInput && block.status === 'paused');
	
	// Output is already parsed by the store, use it directly
	const displayOutput = $derived(block.output || '');

	// Auto-scroll output to bottom
	$effect(() => {
		if (outputRef && block.output) {
			outputRef.scrollTop = outputRef.scrollHeight;
		}
	});
	
	// Auto-focus input field when it becomes visible
	$effect(() => {
		if (showInput && inputRef) {
			// Small delay to ensure the input is rendered
			setTimeout(() => {
				inputRef?.focus();
			}, 100);
		}
	});

	function handleCopy() {
		if (onCopy) {
			onCopy(block.command);
		} else {
			navigator.clipboard.writeText(block.command);
		}
	}

	function handleCopyOutput() {
		if (onCopyOutput) {
			onCopyOutput(block.output);
		} else {
			navigator.clipboard.writeText(block.output);
		}
	}

	function handleRerun() {
		if (onRerun) {
			onRerun(block.command);
		}
	}

	function handleEdit() {
		if (onEdit) {
			onEdit(block.command);
		}
	}

	function handleStop() {
		if (onStop) {
			onStop(block.id);
		}
	}

	function handleSubmitInput() {
		if (onProvideInput && inputValue.trim()) {
			onProvideInput(block.id, inputValue);
			inputValue = '';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmitInput();
		}
	}

	function formatDuration(duration?: number): string {
		if (!duration) return '';
		if (duration < 1000) return `${duration}ms`;
		return `${(duration / 1000).toFixed(2)}s`;
	}

	function getStatusIcon() {
		switch (block.status) {
			case 'running':
				return Loader2;
			case 'completed':
				return Check;
			case 'failed':
				return AlertCircle;
			case 'paused':
				return X;
			default:
				return null;
		}
	}

	function getStatusColor() {
		switch (block.status) {
			case 'running':
				return 'text-blue-400';
			case 'completed':
				return 'text-green-400';
			case 'failed':
				return 'text-red-400';
			case 'paused':
				return 'text-yellow-400';
			default:
				return 'text-gray-400';
		}
	}
</script>

<div
	class={cn(
		'command-block border rounded-lg p-4 mb-4 transition-all bg-card',
		block.status === 'running' && 'border-blue-500 bg-blue-500/10',
		block.status === 'completed' && 'border-green-500/50 bg-green-500/5',
		block.status === 'failed' && 'border-red-500/50 bg-red-500/5',
		block.status === 'paused' && 'border-yellow-500 bg-yellow-500/10',
		block.status === 'pending' && 'border-border bg-muted/50'
	)}
>
	<!-- Command Header -->
	<div class="flex items-start justify-between mb-3">
		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2 mb-1">
				{#if getStatusIcon()}
					{@const Icon = getStatusIcon()}
					<Icon class={cn('h-4 w-4', getStatusColor())} />
				{/if}
				<code class="text-sm font-mono text-foreground break-all">{block.command}</code>
			</div>
			<div class="flex items-center gap-3 text-xs text-muted-foreground mt-1">
				<span>{block.startTime.toLocaleTimeString()}</span>
				{#if block.duration}
					<span>• {formatDuration(block.duration)}</span>
				{/if}
				{#if block.exitCode !== undefined}
					<span class={cn(block.exitCode === 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400')}>
						• Exit code: {block.exitCode}
					</span>
				{/if}
			</div>
		</div>

		<!-- Actions -->
		<div class="flex items-center gap-1 ml-2">
			{#if block.status === 'running'}
				<Button
					variant="ghost"
					size="sm"
					onclick={handleStop}
					class="h-7 w-7 p-0 text-red-400 hover:text-red-300 hover:bg-red-500/10"
					title="Stop command (Ctrl+C)"
				>
					<Square class="h-3 w-3 fill-current" />
				</Button>
			{/if}
			{#if block.status === 'completed' || block.status === 'failed'}
				<Button
					variant="ghost"
					size="sm"
					onclick={handleRerun}
					class="h-7 w-7 p-0"
					title="Rerun command"
				>
					<Play class="h-3 w-3" />
				</Button>
			{/if}
			<Button
				variant="ghost"
				size="sm"
				onclick={handleCopy}
				class="h-7 w-7 p-0"
				title="Copy command"
			>
				<Copy class="h-3 w-3" />
			</Button>
			{#if block.output && block.output.trim()}
				<Button
					variant="ghost"
					size="sm"
					onclick={handleCopyOutput}
					class="h-7 w-7 p-0"
					title="Copy output"
				>
					<FileText class="h-3 w-3" />
				</Button>
			{/if}
			{#if block.status === 'completed' || block.status === 'failed'}
				<Button
					variant="ghost"
					size="sm"
					onclick={handleEdit}
					class="h-7 w-7 p-0"
					title="Edit command"
				>
					<Edit2 class="h-3 w-3" />
				</Button>
			{/if}
		</div>
	</div>

	<!-- Output -->
	{#if block.output || block.status === 'running' || block.status === 'paused'}
		<div
			class={cn(
				'output-container bg-muted/50 dark:bg-black/50 rounded p-3 font-mono text-sm max-h-96 overflow-y-auto',
				'text-foreground whitespace-pre-wrap break-words'
			)}
			bind:this={outputRef}
		>
			{#if block.output || block.status === 'running'}
				{#if displayOutput}
					{displayOutput}
				{:else if block.status === 'running'}
					<span class="text-muted-foreground">Waiting for output...</span>
				{:else}
					<span class="text-muted-foreground">(No output)</span>
				{/if}
			{/if}
		</div>
	{/if}

	<!-- Input Prompt -->
	{#if showInput}
		<div class="mt-3 p-3 bg-yellow-500/10 border border-yellow-500/50 rounded">
			<div class="text-sm text-yellow-600 dark:text-yellow-400 mb-2 font-medium">{block.inputPrompt || 'Input required'}</div>
			<div class="flex gap-2">
				{#if block.inputType === 'password'}
					<Input
						bind:ref={inputRef}
						type="password"
						bind:value={inputValue}
						onkeydown={handleKeydown}
						placeholder="Enter password..."
						class="flex-1"
					/>
				{:else if block.inputType === 'confirm'}
					<Input
						bind:ref={inputRef}
						bind:value={inputValue}
						onkeydown={handleKeydown}
						placeholder="y/n"
						class="flex-1"
					/>
				{:else}
					<Input
						bind:ref={inputRef}
						bind:value={inputValue}
						onkeydown={handleKeydown}
						placeholder="Enter input..."
						class="flex-1"
					/>
				{/if}
				<Button onclick={handleSubmitInput} size="sm">
					Submit
				</Button>
			</div>
		</div>
	{/if}
</div>

<style>
	.command-block {
		animation: fadeIn 0.2s ease-in;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateY(-10px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.output-container {
		scrollbar-width: thin;
		scrollbar-color: hsl(var(--muted-foreground) / 0.3) hsl(var(--muted));
	}

	.output-container::-webkit-scrollbar {
		width: 6px;
	}

	.output-container::-webkit-scrollbar-track {
		background: hsl(var(--muted));
	}

	.output-container::-webkit-scrollbar-thumb {
		background: hsl(var(--muted-foreground) / 0.3);
		border-radius: 3px;
	}

	.output-container::-webkit-scrollbar-thumb:hover {
		background: hsl(var(--muted-foreground) / 0.5);
	}
</style>

