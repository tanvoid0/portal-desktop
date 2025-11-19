<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import {
		Collapsible,
		CollapsibleContent,
		CollapsibleTrigger
	} from '$lib/components/ui/collapsible';
	import { ChevronUp, ChevronDown } from 'lucide-svelte';
	import type { AILog } from '../../types/index.js';

	interface Props {
		log: AILog;
	}

	let { log }: Props = $props();
	let isOpen = $state(false);

	function getLogTypeColor(type: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (type) {
			case 'error':
				return 'destructive';
			case 'request':
				return 'default';
			case 'response':
				return 'secondary';
			default:
				return 'outline';
		}
	}
</script>

<Card>
	<Collapsible bind:open={isOpen}>
		<CollapsibleTrigger class="w-full">
			<CardHeader class="pb-3">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Badge variant={getLogTypeColor(log.log_type)}>
							{log.log_type}
						</Badge>
						<Badge variant="outline">{log.provider}</Badge>
						<span class="text-sm text-muted-foreground">
							{new Date(log.timestamp).toLocaleString()}
						</span>
					</div>
					{#if isOpen}
						<ChevronUp class="h-4 w-4" />
					{:else}
						<ChevronDown class="h-4 w-4" />
					{/if}
				</div>
			</CardHeader>
		</CollapsibleTrigger>
		<CollapsibleContent>
			<CardContent class="space-y-4">
				{#if log.error_message}
					<div>
						<p class="text-sm font-medium mb-1">Error</p>
						<Card class="bg-destructive/10">
							<CardContent class="p-3">
								<p class="text-sm text-destructive">{log.error_message}</p>
							</CardContent>
						</Card>
					</div>
				{/if}
				{#if log.request_data}
					<div>
						<p class="text-sm font-medium mb-1">Request</p>
						<Card>
							<CardContent class="p-3">
								<pre class="text-xs whitespace-pre-wrap">{log.request_data}</pre>
							</CardContent>
						</Card>
					</div>
				{/if}
				{#if log.response_data}
					<div>
						<p class="text-sm font-medium mb-1">Response</p>
						<Card>
							<CardContent class="p-3">
								<pre class="text-xs whitespace-pre-wrap">{log.response_data}</pre>
							</CardContent>
						</Card>
					</div>
				{/if}
			</CardContent>
		</CollapsibleContent>
	</Collapsible>
</Card>

