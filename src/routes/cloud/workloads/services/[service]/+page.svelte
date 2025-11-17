<!-- Service Detail Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { cloudStore, loadResources } from '$lib/domains/cloud/stores';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/lib/components/ui/tabs';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import { ArrowLeft, RefreshCw, FileCode } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import Loading from '@/lib/components/ui/loading.svelte';
	import YamlEditor from '$lib/domains/cloud/components/YamlEditor.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	
	const serviceName = $derived($page.params.service);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	let service = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	onMount(async () => {
		await loadService();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading) {
			loadYAML();
		}
	});
	
	async function loadService() {
		if (!serviceName || !$cloudStore.connection.isConnected) {
			error = 'Service name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.SERVICE, namespace);
			const resources = $cloudStore.resources[ResourceType.SERVICE] || [];
			service = resources.find(s => s.name === serviceName) || null;
			
			if (!service) {
				error = `Service "${serviceName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load service';
			console.error('Failed to load service:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!service) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'Service',
				namespace: service.namespace,
				name: service.name
			});
			
			yaml = yamlContent;
		} catch (err) {
			yamlError = err instanceof Error ? err.message : 'Failed to load YAML';
			console.error('Failed to load YAML:', err);
		} finally {
			yamlLoading = false;
		}
	}
	
	async function handleSaveYAML(yamlContent: string) {
		if (!service) return;
		
		try {
			const result = await invoke<string>('k8s_apply_resource_yaml', {
				namespace: service.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success(result);
			
			// Reload service to get updated data
			await loadService();
			await loadYAML();
		} catch (err) {
			const errorMsg = err instanceof Error ? err.message : 'Failed to apply YAML';
			toastActions.error(errorMsg);
			throw err;
		}
	}
	
	function handleTabChange(tab: string) {
		activeTab = tab;
		const url = new URL(window.location.href);
		url.searchParams.set('tab', tab);
		window.history.replaceState({}, '', url.toString());
	}
	
	const ports = $derived(() => {
		if (!service || !service.metadata?.ports) return [];
		const portArray = Array.isArray(service.metadata.ports) ? service.metadata.ports : [];
		return portArray.map((p: any) => ({
			name: typeof p === 'object' ? (p.name || 'Unnamed') : 'Unnamed',
			port: typeof p === 'object' ? (p.port || p.hostPort || 0) : p,
			targetPort: typeof p === 'object' ? (p.targetPort || p.containerPort || p.port || 0) : p,
			protocol: typeof p === 'object' ? (p.protocol || 'TCP') : 'TCP'
		}));
	});
	
	const selector = $derived(() => {
		if (!service || !service.metadata?.selector) return {};
		return service.metadata.selector || {};
	});
</script>


<div class="p-6 space-y-6">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading text="Loading service details..." />
		</div>
	{:else if error}
		<div class="text-center py-12 text-destructive">
			<p>{error}</p>
			<Button onclick={() => goto('/cloud/workloads/services')} class="mt-4">Back to Services</Button>
		</div>
	{:else if service}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Service: {service.name}</h1>
				<p class="text-muted-foreground">Namespace: {service.namespace}</p>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={loadService}>
					<RefreshCw class="mr-2 h-4 w-4" />
					Refresh
				</Button>
				<Button variant="outline" size="sm" onclick={() => goto('/cloud/workloads/services')}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back to Services
				</Button>
			</div>
		</div>
		
		<!-- Tabs -->
		<Tabs value={activeTab} onValueChange={handleTabChange}>
			<TabsList>
				<TabsTrigger value="overview">Overview</TabsTrigger>
				<TabsTrigger value="yaml">YAML</TabsTrigger>
			</TabsList>
			
			<!-- Overview Tab -->
			<TabsContent value="overview" class="space-y-4">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<Card>
						<CardHeader>
							<CardTitle>Service Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div>
								<p class="text-sm text-muted-foreground">Status</p>
								<Badge class="mt-1" variant="default">{service.status}</Badge>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Cluster IP</p>
								<p class="font-medium">{service.metadata?.cluster_ip || 'None'}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">External IP</p>
								<p class="font-medium">{service.metadata?.external_ip || 'None'}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Age</p>
								<p class="font-medium">{service.metadata?.age || 'N/A'}</p>
							</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader>
							<CardTitle>Ports</CardTitle>
						</CardHeader>
						<CardContent>
							{#if ports().length > 0}
								<div class="space-y-2">
									{#each ports() as port (port.port)}
										<div class="border rounded-md p-3">
											<div class="flex items-center justify-between">
												<div>
													<p class="font-medium">{port.name || 'Unnamed'}</p>
													<p class="text-sm text-muted-foreground">
														Port: {port.port} → {port.targetPort || port.port}
													</p>
													<p class="text-xs text-muted-foreground">Protocol: {port.protocol || 'TCP'}</p>
												</div>
											</div>
										</div>
									{/each}
								</div>
							{:else}
								<p class="text-muted-foreground">No ports configured</p>
							{/if}
						</CardContent>
					</Card>
				</div>
				
				<Card>
					<CardHeader>
						<CardTitle>Selector</CardTitle>
					</CardHeader>
					<CardContent>
						{#if Object.keys(selector).length > 0}
							<div class="flex flex-wrap gap-2">
								{#each Object.entries(selector) as [key, value]}
									<Badge variant="outline">{key}={value}</Badge>
								{/each}
							</div>
						{:else}
							<p class="text-muted-foreground">No selector configured</p>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
			
			<!-- YAML Tab -->
			<TabsContent value="yaml" class="h-[600px]">
				<Card class="h-full flex flex-col">
					<CardHeader>
						<CardTitle>Service YAML</CardTitle>
					</CardHeader>
					<CardContent class="flex-1 overflow-hidden">
						{#if yamlLoading}
							<div class="flex items-center justify-center h-full">
								<Loading text="Loading YAML..." />
							</div>
						{:else if yamlError}
							<div class="text-destructive text-center h-full flex items-center justify-center">
								<p>{yamlError}</p>
							</div>
						{:else if yaml}
							<YamlEditor
								value={yaml}
								onSave={handleSaveYAML}
								resourceName={service.name}
								resourceKind="Service"
								namespace={service.namespace}
							/>
						{:else}
							<div class="text-muted-foreground text-center h-full flex items-center justify-center">
								<p>No YAML available.</p>
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
		</Tabs>
	{/if}
</div>

