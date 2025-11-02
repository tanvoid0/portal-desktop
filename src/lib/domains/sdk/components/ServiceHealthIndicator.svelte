<!--
	Service Health Indicator - FlyEnv-style health monitoring
	Shows service health status with resource usage and alerts
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import { Button } from '$lib/components/ui/button';
	import { 
		Heart, 
		HeartOff, 
		Activity, 
		MemoryStick, 
		Cpu, 
		Wifi, 
		WifiOff,
		AlertTriangle,
		CheckCircle,
		XCircle,
		RefreshCw
	} from 'lucide-svelte';

	interface HealthData {
		is_healthy: boolean;
		last_check: string;
		memory_usage?: number;
		cpu_usage?: number;
		port_open: boolean;
		error_message?: string;
	}

	interface Props {
		serviceId: string;
		serviceName: string;
		showDetails?: boolean;
		refreshInterval?: number; // in seconds
	}

	let { 
		serviceId, 
		serviceName, 
		showDetails = false, 
		refreshInterval = 30 
	}: Props = $props();

	// State
	let health = $state<HealthData | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let lastUpdate = $state<string | null>(null);

	// Derived state
	let healthStatus = $derived(() => {
		if (!health) return 'unknown';
		if (health.is_healthy) return 'healthy';
		if (health.error_message) return 'error';
		return 'unhealthy';
	});

	let statusColor = $derived(() => {
		switch (healthStatus()) {
			case 'healthy': return 'text-green-600';
			case 'error': return 'text-red-600';
			case 'unhealthy': return 'text-yellow-600';
			default: return 'text-gray-600';
		}
	});

	let statusIcon = $derived(() => {
		switch (healthStatus()) {
			case 'healthy': return CheckCircle;
			case 'error': return XCircle;
			case 'unhealthy': return AlertTriangle;
			default: return HeartOff;
		}
	});

	let statusText = $derived(() => {
		switch (healthStatus()) {
			case 'healthy': return 'Healthy';
			case 'error': return 'Error';
			case 'unhealthy': return 'Unhealthy';
			default: return 'Unknown';
		}
	});

	// Initialize
	onMount(() => {
		loadHealth();
		
		// Set up auto-refresh
		if (refreshInterval > 0) {
			const interval = setInterval(loadHealth, refreshInterval * 1000);
			return () => clearInterval(interval);
		}
	});

	async function loadHealth() {
		loading = true;
		error = null;
		
		try {
			const healthData = await invoke('get_service_health', { serviceId });
			health = healthData as HealthData;
			lastUpdate = new Date().toISOString();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load health data';
			console.error('Failed to load service health:', err);
		} finally {
			loading = false;
		}
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function formatCpuUsage(cpu: number): string {
		return `${cpu.toFixed(1)}%`;
	}

	function getMemoryUsagePercentage(): number {
		if (!health?.memory_usage) return 0;
		// Assume 1GB total memory for visualization
		const totalMemory = 1024 * 1024 * 1024; // 1GB
		return Math.min((health.memory_usage / totalMemory) * 100, 100);
	}
</script>

<Card class="w-full">
	<CardHeader class="pb-3">
		<div class="flex items-center justify-between">
			<CardTitle class="flex items-center gap-2">
				{@const StatusIcon = statusIcon()}
				<StatusIcon class="w-5 h-5 {statusColor()}" />
				Service Health
			</CardTitle>
			<div class="flex items-center gap-2">
				<Badge variant={healthStatus() === 'healthy' ? 'default' : 'destructive'}>
					{statusText()}
				</Badge>
				<Button 
					variant="ghost" 
					size="sm" 
					onclick={loadHealth} 
					disabled={loading}
				>
					<RefreshCw class="w-4 h-4" />
				</Button>
			</div>
		</div>
	</CardHeader>

	<CardContent class="space-y-4">
		<!-- Health Status -->
		<div class="flex items-center gap-4">
			<div class="flex items-center gap-2">
				{#if health?.port_open}
					<Wifi class="w-4 h-4 text-green-600" />
				{:else}
					<WifiOff class="w-4 h-4 text-red-600" />
				{/if}
				<span class="text-sm">Port Status</span>
			</div>
			
			{#if health?.error_message}
				<div class="flex-1">
					<Badge variant="destructive" class="text-xs">
						{health.error_message}
					</Badge>
				</div>
			{/if}
		</div>

		<!-- Resource Usage (if details are shown) -->
		{#if showDetails && health}
			<div class="space-y-3">
				<!-- Memory Usage -->
				{#if health.memory_usage !== undefined}
					<div class="space-y-2">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								<MemoryStick class="w-4 h-4 text-blue-600" />
								<span class="text-sm font-medium">Memory Usage</span>
							</div>
							<span class="text-sm text-muted-foreground">
								{formatBytes(health.memory_usage)}
							</span>
						</div>
						<Progress value={getMemoryUsagePercentage()} class="h-2" />
					</div>
				{/if}

				<!-- CPU Usage -->
				{#if health.cpu_usage !== undefined}
					<div class="space-y-2">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								<Cpu class="w-4 h-4 text-purple-600" />
								<span class="text-sm font-medium">CPU Usage</span>
							</div>
							<span class="text-sm text-muted-foreground">
								{formatCpuUsage(health.cpu_usage)}
							</span>
						</div>
						<Progress value={health.cpu_usage} class="h-2" />
					</div>
				{/if}
			</div>
		{/if}

		<!-- Last Check Time -->
		{#if lastUpdate}
			<div class="text-xs text-muted-foreground">
				Last checked: {new Date(lastUpdate).toLocaleString()}
			</div>
		{/if}

		<!-- Error Display -->
		{#if error}
			<div class="p-3 border border-red-200 bg-red-50 rounded-md">
				<div class="flex items-center gap-2">
					<XCircle class="w-4 h-4 text-red-600" />
					<span class="text-sm text-red-600">{error}</span>
				</div>
			</div>
		{/if}

		<!-- Loading State -->
		{#if loading}
			<div class="flex items-center gap-2 text-sm text-muted-foreground">
				<div class="animate-spin rounded-full h-4 w-4 border-b-2 border-primary"></div>
				Checking health...
			</div>
		{/if}
	</CardContent>
</Card>
