<!--
	SDK Service Page
	Service management for a specific SDK
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { invokeClient } from '@/lib/utils/invokeClient';
	import { sdkConfigService, type ProcessedSDKConfig } from '$lib/domains/sdk/services/sdkConfigService';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { 
		AlertCircle,
		RefreshCw,
		ArrowLeft
	} from '@lucide/svelte';
	import ServiceCard from '$lib/domains/sdk/components/ServiceCard.svelte';
	import { goto } from '$app/navigation';

	// Get SDK ID from URL
	let sdkId = $derived($page.params.sdk);
	
	interface ServiceInfo {
		id: string;
		name: string;
		description: string;
		version: string;
		status: 'running' | 'stopped' | 'error' | 'starting' | 'stopping';
		port?: number;
		pid?: number;
		progress?: number;
	}

	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let sdkConfig = $state<ProcessedSDKConfig | null>(null);
	let service = $state<ServiceInfo | null>(null);
	let loadingStatus = $state(false);
	let togglingService = $state(false);

	// Initialize data
	$effect(() => {
		loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		
		try {
			if (!sdkId) {
				error = 'SDK ID is required';
				return;
			}
			
			const config = await sdkConfigService.getSDKConfig(sdkId);
			if (!config) {
				error = `SDK '${sdkId}' not found`;
				return;
			}
			
			sdkConfig = config;
			
			// Only show service management for services (databases, servers, containers, AI)
			if (['database', 'server', 'container', 'ai'].includes(config.category)) {
				await loadServiceStatus();
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load SDK configuration';
			console.error('Failed to load SDK config:', err);
		} finally {
			loading = false;
		}
	}

	async function loadServiceStatus() {
		if (!sdkConfig) return;
		
		loadingStatus = true;
		try {
			const status = await invokeClient.post<{
				running: boolean;
				pid?: number;
				port?: number;
				status: string;
			}>('get_service_status', { sdkType: sdkConfig.id });
			
			const rawVersion = sdkConfig.sdk_version || null;
			const version = rawVersion ? rawVersion.trim().replace(/^v/, '') : 'Unknown';
			
			service = {
				id: sdkConfig.id,
				name: sdkConfig.display_name,
				description: sdkConfig.description,
				version: version,
				status: status.running ? 'running' : 'stopped',
				port: status.port || sdkConfig.service_port || undefined,
				pid: status.pid || undefined
			};
		} catch (err) {
			console.error('Failed to load service status:', err);
			// Create service object even if status check fails
			if (sdkConfig) {
				const rawVersion = sdkConfig.sdk_version || null;
				const version = rawVersion ? rawVersion.trim().replace(/^v/, '') : 'Unknown';
				service = {
					id: sdkConfig.id,
					name: sdkConfig.display_name,
					description: sdkConfig.description,
					version: version,
					status: 'stopped',
					port: sdkConfig.service_port || undefined
				};
			}
		} finally {
			loadingStatus = false;
		}
	}

	async function toggleService(serviceInfo: ServiceInfo) {
		if (!sdkConfig || togglingService) return;
		
		togglingService = true;
		const wasRunning = serviceInfo.status === 'running';
		
		// Update UI immediately
		if (service) {
			service.status = wasRunning ? 'stopping' : 'starting';
		}
		
		try {
			if (wasRunning) {
				await invokeClient.post<string>('stop_service', { sdkType: sdkConfig.id });
			} else {
				await invokeClient.post<string>('start_service', { sdkType: sdkConfig.id });
			}
			
			// Reload status after a short delay
			await new Promise(resolve => setTimeout(resolve, 1000));
			await loadServiceStatus();
		} catch (err) {
			error = err instanceof Error ? err.message : `Failed to ${wasRunning ? 'stop' : 'start'} service`;
			console.error('Failed to toggle service:', err);
			
			// Revert status on error
			if (service) {
				service.status = wasRunning ? 'running' : 'stopped';
			}
		} finally {
			togglingService = false;
		}
	}

	function handleOpenUrl(serviceInfo: ServiceInfo) {
		if (serviceInfo.port) {
			window.open(`http://localhost:${serviceInfo.port}`, '_blank');
		}
	}
</script>

<div class="p-6">
	{#if loading}
		<div class="flex items-center justify-center h-64">
			<RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if error}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertDescription>{error}</AlertDescription>
		</Alert>
	{:else if sdkConfig}
		<div class="space-y-6">
			<!-- Header -->
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-4">
					<Button variant="ghost" size="sm" onclick={() => goto(`/sdk/${sdkId}`)}>
						<ArrowLeft class="h-4 w-4 mr-2" />
						Back
					</Button>
					<div>
						<h1 class="text-3xl font-bold">Service Management</h1>
						<p class="text-muted-foreground">{sdkConfig.display_name} Services</p>
					</div>
				</div>
			</div>

			<!-- Service Content -->
			{#if !['database', 'server', 'container', 'ai'].includes(sdkConfig.category)}
				<Card>
					<CardContent class="pt-6">
						<Alert>
							<AlertCircle class="h-4 w-4" />
							<AlertDescription>
								Service management is only available for databases, web servers, containers, and AI runtimes.
							</AlertDescription>
						</Alert>
					</CardContent>
				</Card>
			{:else if loadingStatus}
				<Card>
					<CardContent class="flex items-center justify-center py-8">
						<RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
					</CardContent>
				</Card>
			{:else if service}
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Service</CardTitle>
							<Button variant="outline" size="sm" onclick={loadServiceStatus} disabled={loadingStatus}>
								<RefreshCw class={`w-4 h-4 mr-2 ${loadingStatus ? 'animate-spin' : ''}`} />
								Refresh
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if service}
							<ServiceCard 
								{service} 
								availableVersions={[service.version]}
								onToggle={() => service && toggleService(service)}
								onVersionChange={() => {
									// Version change not supported for services
								}}
								onConfigure={() => {
									// Configuration not yet implemented
								}}
								onViewLogs={() => {
									// Logs not yet implemented
								}}
								onOpenUrl={() => service && handleOpenUrl(service)}
							/>
						{/if}
					</CardContent>
				</Card>
			{:else}
				<Card>
					<CardContent class="pt-6">
						<p class="text-muted-foreground">Unable to load service information.</p>
					</CardContent>
				</Card>
			{/if}
		</div>
	{/if}
</div>

