<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Terminal } from '@xterm/xterm';
	import { FitAddon } from '@xterm/addon-fit';
	import { TerminalService } from '@/lib/domains/terminal/services/terminalService';
	import type { TerminalProcess, TerminalOutput } from '@/lib/domains/terminal/types';
	import { Copy, Square, Maximize2, Minimize2 } from '@lucide/svelte';
	import { Button } from '@/lib/components/ui/button';
	import { toast } from 'svelte-sonner';

	interface Props {
		command: string;
		workingDirectory?: string;
		onComplete?: (exitCode: number | null) => void;
		onStart?: (processId: string) => void;
	}

	let { command, workingDirectory, onComplete, onStart }: Props = $props();

	let terminalElement: HTMLDivElement | null = $state(null);
	let terminal: Terminal | null = $state(null);
	let fitAddon: FitAddon | null = $state(null);
	let currentProcess: TerminalProcess | null = $state(null);
	let isConnected = $state(false);
	let isExpanded = $state(false);
	let unsubscribe: (() => void) | null = null;
	let outputBuffer = $state('');

	const terminalId = `embedded-terminal-${Date.now()}`;

	onMount(async () => {
		if (!terminalElement) return;

		// Initialize xterm.js terminal
		terminal = new Terminal({
			theme: {
				background: '#0c0c0c',
				foreground: '#cccccc',
				cursor: '#ffffff',
				selectionBackground: '#ffffff40',
			},
			fontSize: 13,
			fontFamily: 'Monaco, Consolas, "Courier New", monospace',
			cursorStyle: 'block',
			scrollback: 5000,
			allowTransparency: false,
		});

		fitAddon = new FitAddon();
		terminal.loadAddon(fitAddon);
		terminal.open(terminalElement);

		// Fit after a small delay to ensure container is sized
		setTimeout(() => {
			fitAddon?.fit();
		}, 100);

		// Set up input handler
		terminal.onData(onData);

		// Start the command
		await startCommand();

		// Handle window resize
		window.addEventListener('resize', handleResize);
	});

	onDestroy(() => {
		window.removeEventListener('resize', handleResize);

		if (unsubscribe) {
			unsubscribe();
		}

		if (currentProcess) {
			TerminalService.killProcess(currentProcess.id);
		}

		terminal?.dispose();
	});

	function handleResize() {
		fitAddon?.fit();
		if (currentProcess && terminal) {
			TerminalService.resizeTerminal(currentProcess.id, terminal.cols, terminal.rows);
		}
	}

	async function startCommand() {
		if (!terminal) return;

		try {
			// Show what command we're running
			terminal.write(`\x1b[90m$ ${command}\x1b[0m\r\n\r\n`);

			// Create a PTY process with bash
			// Default to /tmp if no working directory specified
			currentProcess = await TerminalService.createProcess(terminalId, {
				shell: 'bash',
				working_directory: workingDirectory || '/tmp',
				cols: terminal.cols || 80,
				rows: terminal.rows || 24,
			});

			isConnected = true;
			onStart?.(currentProcess.id);

			// Subscribe to output
			unsubscribe = await TerminalService.subscribeToOutput(currentProcess.id, handleOutput);

			// Use exec to replace the shell with the command (no echo, no extra shell)
			// stty -echo disables input echo, exec replaces shell with the command
			await TerminalService.sendInput(currentProcess.id, `stty -echo; exec ${command}\n`, terminalId);
		} catch (error) {
			console.error('Failed to start command:', error);
			terminal.write(`\x1b[1;31mError: Failed to start command\x1b[0m\r\n`);
			terminal.write(`${error}\r\n`);
		}
	}

	function handleOutput(output: TerminalOutput) {
		if (!terminal) return;
		terminal.write(output.content);
		outputBuffer += output.content;

		// Check for process exit
		if (output.content.includes('exit') || output.content.includes('logout')) {
			// Process may have exited
		}
	}

	function onData(data: string) {
		if (!isConnected || !currentProcess) return;

		// Normalize Enter key
		let chunk = data;
		if (chunk === '\n' || chunk === '\r') {
			chunk = '\r\n';
		}

		TerminalService.sendInput(currentProcess.id, chunk, terminalId).catch((error) => {
			console.error('Failed to send input:', error);
		});
	}

	async function handleStop() {
		if (currentProcess) {
			try {
				await TerminalService.killProcess(currentProcess.id);
				terminal?.write('\r\n\x1b[1;33mProcess terminated\x1b[0m\r\n');
				isConnected = false;
				onComplete?.(null);
			} catch (error) {
				console.error('Failed to kill process:', error);
			}
		}
	}

	function handleCopy() {
		if (outputBuffer) {
			navigator.clipboard.writeText(outputBuffer);
			toast.success('Output copied to clipboard');
		}
	}

	function toggleExpand() {
		isExpanded = !isExpanded;
		// Allow DOM to update, then refit terminal
		setTimeout(() => {
			if (fitAddon && terminal) {
				fitAddon.fit();
				if (currentProcess) {
					TerminalService.resizeTerminal(currentProcess.id, terminal.cols, terminal.rows);
				}
			}
		}, 150);
	}

	function handleClear() {
		terminal?.clear();
		outputBuffer = '';
	}
</script>

<div class="embedded-terminal rounded-lg overflow-hidden border border-gray-700" class:expanded={isExpanded}>
	<!-- Terminal Header -->
	<div class="flex items-center justify-between px-3 py-2 bg-gray-800 border-b border-gray-700">
		<div class="flex items-center gap-2">
			<div class="flex items-center gap-1">
				<div class="w-2.5 h-2.5 rounded-full" class:bg-green-500={isConnected} class:bg-yellow-500={!isConnected}></div>
				<span class="text-xs text-gray-400">{isConnected ? 'Running' : 'Stopped'}</span>
			</div>
		</div>
		<div class="flex items-center gap-1">
			<Button variant="ghost" size="sm" onclick={handleCopy} class="h-7 px-2" title="Copy output">
				<Copy class="h-3.5 w-3.5" />
			</Button>
			<Button variant="ghost" size="sm" onclick={toggleExpand} class="h-7 px-2" title={isExpanded ? 'Collapse' : 'Expand'}>
				{#if isExpanded}
					<Minimize2 class="h-3.5 w-3.5" />
				{:else}
					<Maximize2 class="h-3.5 w-3.5" />
				{/if}
			</Button>
			{#if isConnected}
				<Button variant="ghost" size="sm" onclick={handleStop} class="h-7 px-2 text-red-400 hover:text-red-300" title="Stop process">
					<Square class="h-3.5 w-3.5" />
				</Button>
			{/if}
		</div>
	</div>

	<!-- Terminal Container -->
	<div
		bind:this={terminalElement}
		class="terminal-content"
		class:h-64={!isExpanded}
		class:h-96={isExpanded}
	></div>
</div>

<style>
	.embedded-terminal {
		background: #0c0c0c;
	}

	.terminal-content {
		width: 100%;
		padding: 4px;
		overflow: hidden;
	}

	.terminal-content :global(.xterm) {
		height: 100% !important;
		width: 100% !important;
	}

	.terminal-content :global(.xterm-screen) {
		height: 100% !important;
	}

	.terminal-content :global(.xterm-viewport) {
		overflow-y: scroll !important;
		height: 100% !important;
	}

	.terminal-content :global(.xterm-viewport::-webkit-scrollbar) {
		width: 8px;
	}

	.terminal-content :global(.xterm-viewport::-webkit-scrollbar-track) {
		background: #1f2937;
	}

	.terminal-content :global(.xterm-viewport::-webkit-scrollbar-thumb) {
		background: #4b5563;
		border-radius: 4px;
	}

	.h-64 {
		height: 16rem;
	}

	.h-96 {
		height: 24rem;
	}
</style>
