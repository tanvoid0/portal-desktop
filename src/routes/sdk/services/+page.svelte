<!--
	SDK Services Page
	Shows running SDK services and allows management
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Play, Square, Settings, ExternalLink, AlertCircle } from '@lucide/svelte';

	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let services = $state<any[]>([]);

	// Initialize data
	onMount(async () => {
		await loadServices();
	});

	async function loadServices() {
		loading = true;
		error = null;
		
		try {
			// Load available SDKs
			const availableSDKs = await invoke<any[]>('get_all_available_sdks');
			
			// Load service status for each SDK type
			const servicePromises = availableSDKs.map(async (sdk: any) => {
				try {
					// Ensure we have a valid sdkType
					const sdkType = sdk.id || sdk.name || sdk.type;
					if (!sdkType) {
						console.warn('Skipping SDK without valid id/name/type:', sdk);
						return {
							...sdk,
							services: []
						};
					}
					
					const serviceStatus = await invoke('get_service_status', { sdkType });
					return {
						...sdk,
						services: serviceStatus || []
					};
				} catch (err) {
					console.warn('Failed to load service status for SDK:', sdk, err);
					return {
						...sdk,
						services: []
					};
				}
			});
			
			const sdkWithServices = await Promise.all(servicePromises);
			
			// Flatten services from all SDKs
			services = sdkWithServices.flatMap((sdk: any) => {
				const sdkServices = Array.isArray(sdk.services) ? sdk.services : [];
				return sdkServices.map((service: any) => ({
					...service,
					sdkType: sdk.id || sdk.name || sdk.type,
					sdkName: sdk.name || sdk.title || 'Unknown SDK'
				}));
			});
			
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load SDK services';
			console.error('Failed to load SDK services:', err);
		} finally {
			loading = false;
		}
	}

	async function toggleService(service: any) {
		try {
			if (service.status === 'running') {
				// Stop service
				service.status = 'stopping';
				await invoke('stop_sdk_service', { 
					sdkType: service.sdkType, 
					pid: service.pid 
				});
				service.status = 'stopped';
				service.pid = null;
			} else {
				// Start service
				service.status = 'starting';
				const config = {
					port: service.port,
					host: 'localhost',
					data_dir: null,
					config_file: null,
					environment: {}
				};
				const pid = await invoke('start_sdk_service', { 
					sdkType: service.sdkType, 
					version: service.version,
					config
				});
				service.status = 'running';
				service.pid = pid;
			}
		} catch (err) {
			service.status = 'error';
			error = err instanceof Error ? err.message : 'Failed to toggle service';
		}
	}

	async function configureService(service: any) {
		// FUTURE: Open configuration dialog for service settings
		console.log('Configure service:', service.id);
	}

	async function viewServiceLogs(service: any) {
		// FUTURE: Open log viewer with service logs
		console.log('View logs for:', service.id);
	}

	async function openServiceUrl(service: any) {
		if (service.port) {
			window.open(`http://localhost:${service.port}`, '_blank');
		}
	}
</script>

<svelte:head>
	<title>SDK Services - Portal Desktop</title>
</svelte:head>

<div class="space-y-6 p-6 w-full max-w-none">
	<!-- Header -->
	<div class="flex items-center gap-4">
		<div class="flex-1">
			<h1 class="text-3xl font-bold">SDK Services</h1>
			<p class="text-muted-foreground">
				Manage running SDK services and their configurations
			</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={loadServices} disabled={loading}>
				Refresh
			</Button>
		</div>
	</div>

	<!-- Error Alert -->
	{#if error}
		<div class="p-4 border border-red-200 bg-red-50 rounded-md">
			<p class="text-sm text-red-600">{error}</p>
		</div>
	{/if}

	<!-- Services List -->
	<div class="space-y-4">
		{#each services as service}
			<Card>
				<CardContent class="p-6">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="flex items-center gap-2">
								<h3 class="text-lg font-semibold">{service.sdkName}</h3>
								<Badge variant="outline">{service.version}</Badge>
								{#if service.status === 'running'}
									<Badge variant="default" class="bg-green-100 text-green-800">
										<Play class="w-3 h-3 mr-1" />
										Running
									</Badge>
								{:else if service.status === 'stopped'}
									<Badge variant="outline" class="text-gray-500">
										<Square class="w-3 h-3 mr-1" />
										Stopped
									</Badge>
								{:else if service.status === 'error'}
									<Badge variant="destructive">
										<AlertCircle class="w-3 h-3 mr-1" />
										Error
									</Badge>
								{/if}
							</div>
							{#if service.port}
								<div class="text-sm text-muted-foreground">
									Port: {service.port}
								</div>
							{/if}
						</div>
						
						<div class="flex items-center gap-2">
							<Button 
								variant={service.status === 'running' ? 'outline' : 'default'}
								size="sm"
								onclick={() => toggleService(service)}
								disabled={service.status === 'starting' || service.status === 'stopping'}
							>
								{#if service.status === 'running'}
									<Square class="w-4 h-4 mr-1" />
									Stop
								{:else if service.status === 'starting'}
									Starting...
								{:else if service.status === 'stopping'}
									Stopping...
								{:else}
									<Play class="w-4 h-4 mr-1" />
									Start
								{/if}
							</Button>
							
							{#if service.status === 'running' && service.port}
								<Button 
									variant="outline" 
									size="sm"
									onclick={() => openServiceUrl(service)}
								>
									<ExternalLink class="w-4 h-4 mr-1" />
									Open
								</Button>
							{/if}
							
							<Button 
								variant="outline" 
								size="sm"
								onclick={() => configureService(service)}
							>
								<Settings class="w-4 h-4 mr-1" />
								Configure
							</Button>
							
							<Button 
								variant="outline" 
								size="sm"
								onclick={() => viewServiceLogs(service)}
							>
								View Logs
							</Button>
						</div>
					</div>
					
					{#if service.pid}
						<p class="text-sm text-muted-foreground mt-2">PID: {service.pid}</p>
					{/if}
				</CardContent>
			</Card>
		{/each}
		
		{#if services.length === 0}
			<Card>
				<CardContent class="p-6 text-center">
					<p class="text-muted-foreground">No SDK services found</p>
					<p class="text-sm text-muted-foreground mt-1">Install and configure SDKs to see their services here</p>
				</CardContent>
			</Card>
		{/if}
	</div>
</div>
