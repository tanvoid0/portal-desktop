<!-- Metrics Display Component -->
<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import Progress from '@/lib/components/ui/progress/progress.svelte';
	
	export interface Metrics {
		cpu_usage?: number | null;
		memory_usage?: number | null;
		cpu_limit?: number | null;
		memory_limit?: number | null;
		timestamp?: string;
	}
	
	interface Props {
		metrics: Metrics | null;
		title?: string;
		showDetails?: boolean;
	}
	
	let { metrics, title = 'Resource Metrics', showDetails = true }: Props = $props();
	
	function formatCPU(cpu: number | null | undefined): string {
		if (!cpu && cpu !== 0) return 'N/A';
		if (cpu < 0.001) {
			return `${(cpu * 1000).toFixed(0)}m`;
		}
		return cpu.toFixed(2);
	}
	
	function formatMemory(bytes: number | null | undefined): string {
		if (!bytes && bytes !== 0) return 'N/A';
		const gb = bytes / (1024 * 1024 * 1024);
		if (gb >= 1) {
			return `${gb.toFixed(2)} Gi`;
		}
		const mb = bytes / (1024 * 1024);
		if (mb >= 1) {
			return `${mb.toFixed(2)} Mi`;
		}
		const kb = bytes / 1024;
		if (kb >= 1) {
			return `${kb.toFixed(2)} Ki`;
		}
		return `${bytes.toFixed(0)} B`;
	}
	
	function getCPUPercentage(): number {
		if (!metrics?.cpu_usage && metrics?.cpu_usage !== 0) return 0;
		if (!metrics?.cpu_limit) return 0;
		return Math.min((metrics.cpu_usage / metrics.cpu_limit) * 100, 100);
	}
	
	function getMemoryPercentage(): number {
		if (!metrics?.memory_usage && metrics?.memory_usage !== 0) return 0;
		if (!metrics?.memory_limit) return 0;
		return Math.min((metrics.memory_usage / metrics.memory_limit) * 100, 100);
	}
	
	const cpuPercent = $derived(getCPUPercentage());
	const memoryPercent = $derived(getMemoryPercentage());
</script>

<Card>
	<CardHeader>
		<CardTitle class="text-sm font-medium">{title}</CardTitle>
	</CardHeader>
	<CardContent class="space-y-4">
		{#if !metrics || (!metrics.cpu_usage && metrics.cpu_usage !== 0 && !metrics.memory_usage && metrics.memory_usage !== 0)}
			<p class="text-sm text-muted-foreground">No metrics available</p>
		{:else}
			<!-- CPU Usage -->
			{#if metrics.cpu_usage !== null && metrics.cpu_usage !== undefined}
				<div class="space-y-2">
					<div class="flex justify-between items-center">
						<span class="text-sm font-medium">CPU</span>
						<span class="text-sm text-muted-foreground">
							{formatCPU(metrics.cpu_usage)}
							{#if metrics.cpu_limit}
								/ {formatCPU(metrics.cpu_limit)}
							{/if}
						</span>
					</div>
					{#if metrics.cpu_limit}
						<Progress value={cpuPercent} class="h-2" />
						<p class="text-xs text-muted-foreground">{cpuPercent.toFixed(1)}% used</p>
					{:else}
						<div class="h-2 bg-muted rounded-full">
							<div class="h-full bg-blue-500 rounded-full" style="width: {Math.min(cpuPercent, 100)}%"></div>
						</div>
					{/if}
				</div>
			{/if}
			
			<!-- Memory Usage -->
			{#if metrics.memory_usage !== null && metrics.memory_usage !== undefined}
				<div class="space-y-2">
					<div class="flex justify-between items-center">
						<span class="text-sm font-medium">Memory</span>
						<span class="text-sm text-muted-foreground">
							{formatMemory(metrics.memory_usage)}
							{#if metrics.memory_limit}
								/ {formatMemory(metrics.memory_limit)}
							{/if}
						</span>
					</div>
					{#if metrics.memory_limit}
						<Progress value={memoryPercent} class="h-2" />
						<p class="text-xs text-muted-foreground">{memoryPercent.toFixed(1)}% used</p>
					{:else}
						<div class="h-2 bg-muted rounded-full">
							<div class="h-full bg-green-500 rounded-full" style="width: {Math.min(memoryPercent, 100)}%"></div>
						</div>
					{/if}
				</div>
			{/if}
			
			{#if showDetails && metrics.timestamp}
				<div class="pt-2 border-t">
					<p class="text-xs text-muted-foreground">Last updated: {new Date(metrics.timestamp).toLocaleTimeString()}</p>
				</div>
			{/if}
		{/if}
	</CardContent>
</Card>

