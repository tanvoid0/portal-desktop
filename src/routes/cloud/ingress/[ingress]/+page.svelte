<!-- Ingress Detail Page -->
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
	import { ArrowLeft, RefreshCw } from '@lucide/svelte';
	import Loading from '@/lib/components/ui/loading.svelte';
	import YamlEditor from '$lib/domains/cloud/components/YamlEditor.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	
	const ingressName = $derived($page.params.ingress);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	
	let ingress = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	onMount(async () => {
		await loadIngress();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading && ingress) {
			loadYAML();
		}
	});
	
	async function loadIngress() {
		if (!ingressName || !$cloudStore.connection.isConnected) {
			error = 'Ingress name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.INGRESS, namespace);
			const resources = $cloudStore.resources[ResourceType.INGRESS] || [];
			ingress = resources.find(i => i.name === ingressName) || null;
			
			if (!ingress) {
				error = `Ingress "${ingressName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load Ingress';
			console.error('Failed to load Ingress:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!ingress) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'Ingress',
				namespace: ingress.namespace,
				name: ingress.name
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
		if (!ingress) return;
		
		try {
			const result = await invoke<string>('k8s_apply_resource_yaml', {
				namespace: ingress.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success(result);
			
			// Reload Ingress to get updated data
			await loadIngress();
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
	
	const addresses = $derived(() => {
		if (!ingress || !ingress.metadata?.addresses) return [];
		return ingress.metadata.addresses;
	});
	
	const hosts = $derived(() => {
		if (!ingress || !ingress.metadata?.ports) return [];
		return ingress.metadata.ports;
	});
</script>


<div class="p-6 space-y-6">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading text="Loading Ingress details..." />
		</div>
	{:else if error}
		<div class="text-center py-12 text-destructive">
			<p>{error}</p>
			<Button onclick={() => goto('/cloud/ingress')} class="mt-4">Back to Ingress</Button>
		</div>
	{:else if ingress}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Ingress: {ingress.name}</h1>
				<p class="text-muted-foreground">Namespace: {ingress.namespace}</p>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={loadIngress}>
					<RefreshCw class="mr-2 h-4 w-4" />
					Refresh
				</Button>
				<Button variant="outline" size="sm" onclick={() => goto('/cloud/ingress')}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back to Ingress
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
							<CardTitle>Ingress Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div>
								<p class="text-sm text-muted-foreground">Name</p>
								<p class="font-medium">{ingress.name}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Namespace</p>
								<p class="font-medium">{ingress.namespace}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Ingress Class</p>
								<p class="font-medium">{ingress.metadata?.class || 'N/A'}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Age</p>
								<p class="font-medium">{ingress.metadata?.age || 'N/A'}</p>
							</div>
						</CardContent>
					</Card>
					
					{#if addresses().length > 0 || hosts().length > 0}
						<Card>
							<CardHeader>
								<CardTitle>Network Information</CardTitle>
							</CardHeader>
							<CardContent class="space-y-3">
								{#if addresses().length > 0}
									<div>
										<p class="text-sm text-muted-foreground">Addresses</p>
										<div class="flex flex-wrap gap-2 mt-1">
											{#each addresses() as addr}
												<Badge variant="outline">{addr}</Badge>
											{/each}
										</div>
									</div>
								{/if}
								{#if hosts().length > 0}
									<div>
										<p class="text-sm text-muted-foreground">Hosts</p>
										<div class="flex flex-wrap gap-2 mt-1">
											{#each hosts() as host}
												<Badge variant="secondary">{host}</Badge>
											{/each}
										</div>
									</div>
								{/if}
							</CardContent>
						</Card>
					{/if}
					
					{#if ingress.metadata?.labels && Object.keys(ingress.metadata.labels).length > 0}
						<Card>
							<CardHeader>
								<CardTitle>Labels</CardTitle>
							</CardHeader>
							<CardContent>
								<div class="flex flex-wrap gap-2">
									{#each Object.entries(ingress.metadata.labels) as [key, value]}
										<Badge variant="outline">{key}={value}</Badge>
									{/each}
								</div>
							</CardContent>
						</Card>
					{/if}
				</div>
			</TabsContent>
			
			<!-- YAML Tab -->
			<TabsContent value="yaml" class="h-[600px]">
				<Card class="h-full flex flex-col">
					<CardHeader>
						<CardTitle>Ingress YAML</CardTitle>
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
								resourceName={ingress.name}
								resourceKind="Ingress"
								namespace={ingress.namespace}
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

