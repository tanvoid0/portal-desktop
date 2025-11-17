<!--
	Service Log Viewer - FlyEnv-style log viewer with real-time streaming
	Displays service logs with filtering, search, and export capabilities
-->

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import Select from '$lib/components/ui/select.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { 
		Download, 
		Trash2, 
		Search, 
		Filter, 
		Play, 
		Square,
		AlertCircle,
		Info,
		AlertTriangle,
		X
	} from '@lucide/svelte';

	interface LogEntry {
		timestamp: string;
		level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
		message: string;
	}

	interface Props {
		serviceId: string;
		serviceName: string;
		isOpen: boolean;
		onClose: () => void;
	}

	let { serviceId, serviceName, isOpen, onClose }: Props = $props();

	// State
	let logs = $state<LogEntry[]>([]);
	let filteredLogs = $state<LogEntry[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let autoScroll = $state(true);
	let searchTerm = $state('');
	let selectedLevel = $state<string>('all');
	let isStreaming = $state(false);

	// Derived state
	let logLevels = $derived(() => {
		const levels = new Set(logs.map(log => log.level));
		return Array.from(levels).sort();
	});

	let filteredLogsCount = $derived(() => filteredLogs.length);
	let totalLogsCount = $derived(() => logs.length);

	// Initialize
	onMount(async () => {
		if (isOpen) {
			await loadLogs();
		}
	});

	// Watch for dialog open/close
	$effect(() => {
		if (isOpen) {
			loadLogs();
		}
	});

	async function loadLogs() {
		loading = true;
		error = null;
		
		try {
			const logData = await invoke('get_service_logs', { 
				serviceId, 
				lines: 1000 
			});
			logs = Array.isArray(logData) ? logData : [];
			applyFilters();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load logs';
			console.error('Failed to load service logs:', err);
		} finally {
			loading = false;
		}
	}

	function applyFilters() {
		filteredLogs = logs.filter(log => {
			// Level filter
			if (selectedLevel !== 'all' && log.level !== selectedLevel) {
				return false;
			}
			
			// Search filter
			if (searchTerm && !log.message.toLowerCase().includes(searchTerm.toLowerCase())) {
				return false;
			}
			
			return true;
		});
	}

	function handleSearch() {
		applyFilters();
	}

	function handleLevelChange(level: string) {
		selectedLevel = level;
		applyFilters();
	}

	function clearLogs() {
		logs = [];
		filteredLogs = [];
	}

	async function downloadLogs() {
		try {
			const logText = filteredLogs.map(log => 
				`[${log.timestamp}] ${log.level}: ${log.message}`
			).join('\n');
			
			const blob = new Blob([logText], { type: 'text/plain' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `${serviceName}-logs-${new Date().toISOString().split('T')[0]}.txt`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to download logs';
		}
	}

	function getLevelIcon(level: string) {
		switch (level) {
			case 'ERROR': return AlertCircle;
			case 'WARN': return AlertTriangle;
			case 'INFO': return Info;
			default: return Info;
		}
	}

	function getLevelColor(level: string) {
		switch (level) {
			case 'ERROR': return 'text-red-600';
			case 'WARN': return 'text-yellow-600';
			case 'INFO': return 'text-blue-600';
			case 'DEBUG': return 'text-gray-600';
			default: return 'text-gray-600';
		}
	}

	function getLevelBadgeVariant(level: string) {
		switch (level) {
			case 'ERROR': return 'destructive';
			case 'WARN': return 'secondary';
			case 'INFO': return 'default';
			case 'DEBUG': return 'outline';
			default: return 'outline';
		}
	}
</script>

<Dialog bind:open={isOpen} onOpenChange={(open) => !open && onClose()}>
	<DialogContent class="max-w-4xl h-[80vh] flex flex-col">
		<DialogHeader>
			<DialogTitle class="flex items-center gap-2">
				<Filter class="w-5 h-5" />
				Service Logs: {serviceName}
			</DialogTitle>
		</DialogHeader>

		<!-- Controls -->
		<div class="flex items-center gap-4 p-4 border-b">
			<div class="flex items-center gap-2 flex-1">
				<Search class="w-4 h-4 text-muted-foreground" />
				<Input 
					placeholder="Search logs..." 
					bind:value={searchTerm}
					oninput={handleSearch}
					class="max-w-sm"
				/>
			</div>
			
			<Select 
				options={[
					{ value: 'all', label: 'All Levels' },
					...logLevels().map(level => ({ value: level, label: level }))
				]}
				defaultValue={selectedLevel}
				onSelect={handleLevelChange}
				class="w-32"
			/>

			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={downloadLogs}>
					<Download class="w-4 h-4 mr-2" />
					Download
				</Button>
				<Button variant="outline" size="sm" onclick={clearLogs}>
					<Trash2 class="w-4 h-4 mr-2" />
					Clear
				</Button>
				<Button variant="outline" size="sm" onclick={loadLogs} disabled={loading}>
					<Play class="w-4 h-4 mr-2" />
					Refresh
				</Button>
			</div>
		</div>

		<!-- Log Stats -->
		<div class="flex items-center gap-4 p-2 border-b bg-muted/50">
			<Badge variant="outline">
				{filteredLogsCount} / {totalLogsCount} logs
			</Badge>
			{#if selectedLevel !== 'all'}
				<Badge variant="secondary">
					Filtered by: {selectedLevel}
				</Badge>
			{/if}
			{#if searchTerm}
				<Badge variant="secondary">
					Search: "{searchTerm}"
				</Badge>
			{/if}
		</div>

		<!-- Error Alert -->
		{#if error}
			<div class="p-4 border border-red-200 bg-red-50 rounded-md">
				<p class="text-sm text-red-600">{error}</p>
			</div>
		{/if}

		<!-- Logs Display -->
		<ScrollArea class="flex-1 p-4">
			{#if loading}
				<div class="flex items-center justify-center py-8">
					<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
					<span class="ml-2">Loading logs...</span>
				</div>
			{:else if filteredLogs.length === 0}
				<div class="text-center py-8 text-muted-foreground">
					<Filter class="w-12 h-12 mx-auto mb-4 opacity-50" />
					<p>No logs found</p>
					{#if searchTerm || selectedLevel !== 'all'}
						<p class="text-sm">Try adjusting your filters</p>
					{/if}
				</div>
			{:else}
				<div class="space-y-1 font-mono text-sm">
					{#each filteredLogs as log}
						<div class="flex items-start gap-3 p-2 hover:bg-muted/50 rounded">
							<div class="flex items-center gap-2 min-w-0 flex-shrink-0">
								{#if log.level === 'ERROR'}
									<AlertCircle class="w-4 h-4 {getLevelColor(log.level)}" />
								{:else if log.level === 'WARN'}
									<AlertTriangle class="w-4 h-4 {getLevelColor(log.level)}" />
								{:else if log.level === 'INFO'}
									<Info class="w-4 h-4 {getLevelColor(log.level)}" />
								{:else}
									<Info class="w-4 h-4 {getLevelColor(log.level)}" />
								{/if}
								<Badge variant={getLevelBadgeVariant(log.level)} class="text-xs">
									{log.level}
								</Badge>
							</div>
							<div class="flex-1 min-w-0">
								<div class="text-xs text-muted-foreground mb-1">
									{new Date(log.timestamp).toLocaleString()}
								</div>
								<div class="break-words">{log.message}</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</ScrollArea>
	</DialogContent>
</Dialog>
