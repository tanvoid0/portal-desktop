<!-- Log Entry Component - Displays formatted log entries -->
<script lang="ts">
	import { Badge } from '@/lib/components/ui/badge';
	import { Button } from '@/lib/components/ui/button';
	import { Copy } from '@lucide/svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import type { K8sLog } from '../../types/k8s';
	
	interface Props {
		log: K8sLog;
		viewMode?: 'detailed' | 'compact' | 'raw';
		onFilterBySeverity?: (severity: string) => void;
	}
	
	let {
		log,
		viewMode = 'detailed',
		onFilterBySeverity
	}: Props = $props();
	
	// Parse structured log if it's JSON
	let parsedLog = $derived.by(() => {
		try {
			const parsed = JSON.parse(log.message);
			return {
				isStructured: true,
				data: parsed,
				rawMessage: log.message
			};
		} catch {
			return {
				isStructured: false,
				data: null,
				rawMessage: log.message
			};
		}
	});
	
	// Extract structured fields
	let structuredFields = $derived.by(() => {
		if (!parsedLog.isStructured || !parsedLog.data) {
			return null;
		}
		
		const data = parsedLog.data;
		return {
			severity: data.severity || data.level || data.logLevel || log.level,
			time: data.time || data.timestamp || data.ts || log.timestamp,
			message: data.message || data.msg || data.text || log.message,
			logger: data.logger || data.component || data.service,
			requestId: data.requestId || data.request_id || data.trace_id,
			traceId: data['logging.googleapis.com/trace'] || data.requestId || data.request_id || data.trace_id,
			error: data.error || data.err || data.exception,
			stackTrace: data.stackTrace || data.stack_trace || data.stack,
			method: data.method || data.httpMethod,
			url: data.url || data.path || data.endpoint,
			statusCode: data.statusCode || data.status_code || data.status,
			duration: data.duration || data.elapsed || data.responseTime
		};
	});
	
	function getSeverityColor(severity: string): string {
		const s = severity?.toUpperCase();
		switch (s) {
			case 'ERROR': case 'FATAL': case 'CRITICAL':
				return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200 border-l-4 border-red-400';
			case 'WARN': case 'WARNING':
				return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 border-l-4 border-yellow-400';
			case 'INFO': case 'INFORMATION':
				return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 border-l-4 border-blue-400';
			case 'DEBUG': case 'TRACE':
				return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200 border-l-4 border-gray-400';
			default:
				return 'bg-slate-100 text-slate-800 dark:bg-slate-900 dark:text-slate-200 border-l-4 border-slate-400';
		}
	}
	
	function getSeverityBadgeVariant(severity: string): "default" | "secondary" | "destructive" | "outline" {
		const s = severity?.toUpperCase();
		switch (s) {
			case 'ERROR': case 'FATAL': case 'CRITICAL':
				return 'destructive';
			case 'WARN': case 'WARNING':
				return 'outline';
			default:
				return 'default';
		}
	}
	
	function formatTimestamp(timestamp: string): string {
		try {
			return new Date(timestamp).toLocaleString();
		} catch {
			return timestamp;
		}
	}
	
	async function copyToClipboard(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			toastActions.success('Copied to clipboard');
		} catch (err) {
			toastActions.error('Failed to copy');
		}
	}
	
	function handleSeverityClick(severity: string) {
		if (onFilterBySeverity) {
			onFilterBySeverity(severity);
		}
	}
	
	let showFullMessage = $state(false);
	let showFullError = $state(false);
	let showFullStackTrace = $state(false);
</script>

<div class="rounded-md p-3 mb-2 {getSeverityColor(structuredFields?.severity || log.level)}">
	{#if viewMode === 'raw'}
		<!-- Raw Mode -->
		<div class="text-xs font-mono whitespace-pre-wrap break-words">
			<span class="text-slate-500 dark:text-slate-400">[{formatTimestamp(log.timestamp)}]</span>
			<span class="text-slate-600 dark:text-slate-300">[{log.level}]</span>
			<span class="ml-2">{parsedLog.rawMessage}</span>
			<Button
				variant="ghost"
				size="sm"
				onclick={() => copyToClipboard(`[${formatTimestamp(log.timestamp)}] [${log.level}] ${parsedLog.rawMessage}`)}
				class="ml-2 h-6 w-6 p-0 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
				title="Copy log"
			>
				<Copy class="h-3 w-3" />
			</Button>
		</div>
	{:else if viewMode === 'compact'}
		<!-- Compact Mode -->
		<div class="flex items-center space-x-2 text-sm">
			<Button
				variant="ghost"
				onclick={() => handleSeverityClick(structuredFields?.severity || log.level)}
				class="cursor-pointer hover:opacity-80 p-0 h-auto"
			>
				<Badge variant={getSeverityBadgeVariant(structuredFields?.severity || log.level)}>
					{structuredFields?.severity || log.level}
				</Badge>
			</Button>
			<span class="text-slate-500 dark:text-slate-400 text-xs font-mono">
				{formatTimestamp(structuredFields?.time || log.timestamp)}
			</span>
			<span class="flex-1 min-w-0 break-words">
				{structuredFields?.message || parsedLog.rawMessage}
			</span>
			<Button
				variant="ghost"
				size="sm"
				onclick={() => copyToClipboard(structuredFields?.message || parsedLog.rawMessage)}
				class="h-6 w-6 p-0 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 flex-shrink-0"
				title="Copy message"
			>
				<Copy class="h-3 w-3" />
			</Button>
		</div>
	{:else}
		<!-- Detailed Mode -->
		<div class="space-y-2">
			<!-- Header -->
			<div class="flex items-center justify-between">
				<div class="flex items-center space-x-2 flex-wrap">
					<Button
						variant="ghost"
						onclick={() => handleSeverityClick(structuredFields?.severity || log.level)}
						class="cursor-pointer hover:opacity-80 p-0 h-auto"
					>
						<Badge variant={getSeverityBadgeVariant(structuredFields?.severity || log.level)}>
							{structuredFields?.severity || log.level}
						</Badge>
					</Button>
					<span class="text-xs text-slate-500 dark:text-slate-400 font-mono">
						{formatTimestamp(structuredFields?.time || log.timestamp)}
					</span>
					<span class="text-xs text-slate-500 dark:text-slate-400">
						{log.pod}
						{#if log.container && log.container !== 'app'}
							/{log.container}
						{/if}
					</span>
				</div>
				<Button
					variant="ghost"
					size="sm"
					onclick={() => copyToClipboard(log.message)}
					class="h-8 w-8 p-0 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
					title="Copy log"
				>
					<Copy class="h-4 w-4" />
				</Button>
			</div>
			
			<!-- Message -->
			<div class="text-sm break-words">
				{#if structuredFields?.message}
					<div class="p-2 bg-white dark:bg-slate-800 rounded border font-mono">
						{#if structuredFields.message.length > 200 && !showFullMessage}
							<div>
								<span>{structuredFields.message.slice(0, 200)}...</span>
								<Button
									variant="link"
									size="sm"
									onclick={() => showFullMessage = true}
									class="ml-2 h-auto p-0 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-xs underline"
								>
									Show more
								</Button>
							</div>
						{:else}
							<div>
								{structuredFields.message}
								{#if structuredFields.message.length > 200 && showFullMessage}
									<Button
										variant="link"
										size="sm"
										onclick={() => showFullMessage = false}
										class="ml-2 h-auto p-0 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-xs underline"
									>
										Show less
									</Button>
								{/if}
							</div>
						{/if}
					</div>
				{:else}
					<div class="p-2 bg-white dark:bg-slate-800 rounded border font-mono whitespace-pre-wrap">
						{#if parsedLog.rawMessage.length > 200 && !showFullMessage}
							<div>
								<span>{parsedLog.rawMessage.slice(0, 200)}...</span>
								<Button
									variant="link"
									size="sm"
									onclick={() => showFullMessage = true}
									class="ml-2 h-auto p-0 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-xs underline"
								>
									Show more
								</Button>
							</div>
						{:else}
							<div>
								{parsedLog.rawMessage}
								{#if parsedLog.rawMessage.length > 200 && showFullMessage}
									<Button
										variant="link"
										size="sm"
										onclick={() => showFullMessage = false}
										class="ml-2 h-auto p-0 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-xs underline"
									>
										Show less
									</Button>
								{/if}
							</div>
						{/if}
					</div>
				{/if}
			</div>
			
			<!-- Structured Fields -->
			{#if structuredFields}
				<div class="space-y-1 text-xs">
					{#if structuredFields.logger || structuredFields.requestId || structuredFields.traceId}
						<div class="flex items-center space-x-4 flex-wrap gap-2">
							{#if structuredFields.logger}
								<div>
									<span class="font-medium text-slate-600 dark:text-slate-400">Logger:</span>
									<span class="ml-1">{structuredFields.logger}</span>
								</div>
							{/if}
							{#if structuredFields.requestId}
								<div>
									<span class="font-medium text-slate-600 dark:text-slate-400">Request ID:</span>
									<span class="ml-1 font-mono">{structuredFields.requestId}</span>
								</div>
							{/if}
							{#if structuredFields.traceId}
								<div>
									<span class="font-medium text-slate-600 dark:text-slate-400">Trace ID:</span>
									<span class="ml-1 font-mono">{structuredFields.traceId}</span>
								</div>
							{/if}
						</div>
					{/if}
					
					{#if structuredFields.method || structuredFields.url || structuredFields.statusCode}
						<div class="flex items-center space-x-4 flex-wrap gap-2">
							{#if structuredFields.method}
								<div>
									<span class="font-medium text-slate-600 dark:text-slate-400">Method:</span>
									<span class="ml-1">{structuredFields.method}</span>
								</div>
							{/if}
							{#if structuredFields.url}
								<div>
									<span class="font-medium text-slate-600 dark:text-slate-400">URL:</span>
									<span class="ml-1 font-mono">{structuredFields.url}</span>
								</div>
							{/if}
							{#if structuredFields.statusCode}
								<div>
									<span class="font-medium text-slate-600 dark:text-slate-400">Status:</span>
									<span class="ml-1">{structuredFields.statusCode}</span>
								</div>
							{/if}
						</div>
					{/if}
				</div>
			{/if}
			
			<!-- Error -->
			{#if structuredFields?.error}
				<div class="text-sm">
					<span class="font-medium text-red-600 dark:text-red-400">Error:</span>
					<div class="mt-1 p-2 bg-red-50 dark:bg-red-900/20 rounded border border-red-200 dark:border-red-800 font-mono text-red-900 dark:text-red-100 break-words">
						{#if String(structuredFields.error).length > 200 && !showFullError}
							<div>
								<span>{String(structuredFields.error).slice(0, 200)}...</span>
								<Button
									variant="link"
									size="sm"
									onclick={() => showFullError = true}
									class="ml-2 h-auto p-0 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-xs underline"
								>
									Show more
								</Button>
							</div>
						{:else}
							<div>
								{String(structuredFields.error)}
								{#if String(structuredFields.error).length > 200 && showFullError}
									<Button
										variant="link"
										size="sm"
										onclick={() => showFullError = false}
										class="ml-2 h-auto p-0 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-xs underline"
									>
										Show less
									</Button>
								{/if}
							</div>
						{/if}
					</div>
				</div>
			{/if}
			
			<!-- Stack Trace -->
			{#if structuredFields?.stackTrace}
				<div class="text-sm">
					<span class="font-medium text-red-600 dark:text-red-400">Stack Trace:</span>
					<div class="mt-1 p-2 bg-red-50 dark:bg-red-900/20 rounded border border-red-200 dark:border-red-800 font-mono text-xs text-red-900 dark:text-red-100 whitespace-pre-wrap break-words">
						{#if String(structuredFields.stackTrace).length > 300 && !showFullStackTrace}
							<div>
								<span>{String(structuredFields.stackTrace).slice(0, 300)}...</span>
								<Button
									variant="link"
									size="sm"
									onclick={() => showFullStackTrace = true}
									class="ml-2 h-auto p-0 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-xs underline"
								>
									Show more
								</Button>
							</div>
						{:else}
							<div>
								{String(structuredFields.stackTrace)}
								{#if String(structuredFields.stackTrace).length > 300 && showFullStackTrace}
									<Button
										variant="link"
										size="sm"
										onclick={() => showFullStackTrace = false}
										class="ml-2 h-auto p-0 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 text-xs underline"
									>
										Show less
									</Button>
								{/if}
							</div>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>

