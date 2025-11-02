<!--
	Terminal - Global Terminal (Default Page)
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { TerminalTabContainer, terminalActions } from '$lib/domains/terminal';
	import type { TerminalSettings } from '$lib/domains/terminal';

	let settings: TerminalSettings = {
		theme: 'dark',
		fontSize: 14,
		fontFamily: 'Monaco, Consolas, "Courier New", monospace',
		cursorStyle: 'block',
		scrollbackLines: 1000,
		bellSound: false,
		autoClose: true,
		confirmClose: true,
		defaultShell: navigator.userAgent.includes('Windows') ? 'cmd.exe' : 'bash',
		workingDirectory: navigator.userAgent.includes('Windows') ? 'C:\\' : '/'
	};

	onMount(() => {
		// Clean up stale terminal data on app startup
		terminalActions.cleanupStaleData();
	});
</script>

<div class="h-screen w-full bg-gray-900 flex flex-col">
	<!-- Header -->
	<div class="terminal-header bg-gray-800 border-b border-gray-700 px-4 py-2">
		<div class="flex items-center space-x-4">
			<h1 class="text-lg font-semibold text-gray-100">Global Terminal</h1>
			<div class="flex items-center space-x-2">
				<div class="w-2 h-2 bg-green-500 rounded-full"></div>
				<span class="text-sm text-gray-400">System Terminal</span>
			</div>
		</div>
	</div>

	<!-- Terminal Content -->
	<div class="flex-1 min-h-0">
		<TerminalTabContainer {settings} />
	</div>
</div>
