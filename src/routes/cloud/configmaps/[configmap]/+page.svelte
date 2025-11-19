<!-- ConfigMap Detail Page -->
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
	import { ArrowLeft, RefreshCw, FileCode, Trash2 } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import Loading from '@/lib/components/ui/loading.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import YamlEditor from '$lib/domains/cloud/components/YamlEditor.svelte';
	
	const configMapName = $derived($page.params.configmap);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	let configMap = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	onMount(async () => {
		await loadConfigMap();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading && configMap) {
			loadYAML();
		}
	});
	
	async function loadConfigMap() {
		if (!configMapName || !$cloudStore.connection.isConnected) {
			error = 'ConfigMap name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.CONFIGMAP, namespace);
			const resources = $cloudStore.resources[ResourceType.CONFIGMAP] || [];
			configMap = resources.find(cm => cm.name === configMapName) || null;
			
			if (!configMap) {
				error = `ConfigMap "${configMapName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load ConfigMap';
			console.error('Failed to load ConfigMap:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!configMap) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'ConfigMap',
				namespace: configMap.namespace,
				name: configMap.name
			});
			
			// Convert JSON to YAML (basic conversion)
			// For now, we'll use JSON pretty-printed
			yaml = yamlContent;
		} catch (err) {
			yamlError = err instanceof Error ? err.message : 'Failed to load YAML';
			console.error('Failed to load YAML:', err);
		} finally {
			yamlLoading = false;
		}
	}
	
	async function handleSaveYAML(yamlContent: string) {
		if (!configMap) return;
		
		try {
			const result = await invoke<string>('k8s_apply_resource_yaml', {
				namespace: configMap.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success(result);
			
			// Reload ConfigMap to get updated data
			await loadConfigMap();
			await loadYAML();
		} catch (err) {
			const errorMsg = err instanceof Error ? err.message : 'Failed to apply YAML';
			toastActions.error(errorMsg);
			throw err;
		}
	}
	
	async function handleDelete() {
		if (!configMap) return;
		
		if (!confirm(`Are you sure you want to delete ConfigMap "${configMap.name}"? This action cannot be undone.`)) {
			return;
		}
		
		try {
			await invoke('k8s_delete_configmap', {
				namespace: configMap.namespace,
				name: configMap.name
			});
			
			toastActions.success(`ConfigMap "${configMap.name}" deleted successfully`);
			goto('/cloud/configmaps');
		} catch (err) {
			toastActions.error(err instanceof Error ? err.message : 'Failed to delete ConfigMap');
		}
	}
	
	function handleTabChange(tab: string) {
		activeTab = tab;
		const url = new URL(window.location.href);
		url.searchParams.set('tab', tab);
		window.history.replaceState({}, '', url.toString());
	}
	
	const dataEntries = $derived.by(() => {
		if (!configMap || !configMap.metadata?.data) return [];
		return Object.entries(configMap.metadata.data);
	});
</script>


<div class="p-6 space-y-6">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading text="Loading ConfigMap details..." />
		</div>
	{:else if error}
		<div class="text-center py-12 text-destructive">
			<p>{error}</p>
			<Button onclick={() => goto('/cloud/configmaps')} class="mt-4">Back to ConfigMaps</Button>
		</div>
	{:else if configMap}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">ConfigMap: {configMap.name}</h1>
				<p class="text-muted-foreground">Namespace: {configMap.namespace}</p>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={loadConfigMap}>
					<RefreshCw class="mr-2 h-4 w-4" />
					Refresh
				</Button>
				<Button variant="destructive" size="sm" onclick={handleDelete}>
					<Trash2 class="mr-2 h-4 w-4" />
					Delete
				</Button>
				<Button variant="outline" size="sm" onclick={() => goto('/cloud/configmaps')}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back to ConfigMaps
				</Button>
			</div>
		</div>
		
		<!-- Tabs -->
		<Tabs value={activeTab} onValueChange={handleTabChange}>
			<TabsList>
				<TabsTrigger value="overview">Overview</TabsTrigger>
				<TabsTrigger value="data">Data</TabsTrigger>
				<TabsTrigger value="yaml">YAML</TabsTrigger>
			</TabsList>
			
			<!-- Overview Tab -->
			<TabsContent value="overview" class="space-y-4">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<Card>
						<CardHeader>
							<CardTitle>ConfigMap Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div>
								<p class="text-sm text-muted-foreground">Name</p>
								<p class="font-medium">{configMap.name}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Namespace</p>
								<p class="font-medium">{configMap.namespace}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Data Keys</p>
								<p class="font-medium">{configMap.metadata?.dataCount || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Age</p>
								<p class="font-medium">{configMap.metadata?.age || 'N/A'}</p>
							</div>
						</CardContent>
					</Card>
					
					{#if configMap.metadata?.labels && Object.keys(configMap.metadata.labels).length > 0}
						<Card>
							<CardHeader>
								<CardTitle>Labels</CardTitle>
							</CardHeader>
							<CardContent>
								<div class="flex flex-wrap gap-2">
									{#each Object.entries(configMap.metadata.labels) as [key, value]}
										<Badge variant="outline">{key}={value}</Badge>
									{/each}
								</div>
							</CardContent>
						</Card>
					{/if}
				</div>
			</TabsContent>
			
			<!-- Data Tab -->
			<TabsContent value="data" class="space-y-4">
				<Card>
					<CardHeader>
						<CardTitle>Configuration Data</CardTitle>
					</CardHeader>
					<CardContent>
						{#if dataEntries.length === 0}
							<div class="text-center py-8 text-muted-foreground">
								<p>No data entries</p>
							</div>
						{:else}
							<div class="space-y-4">
								{#each dataEntries as [key, value]}
									<div class="border rounded-lg p-4">
										<div class="flex items-center justify-between mb-2">
											<span class="font-medium">{key}</span>
											<Button 
												variant="ghost" 
												size="sm"
												onclick={() => {
													navigator.clipboard.writeText(String(value));
													toastActions.success('Value copied to clipboard');
												}}
											>
												Copy
											</Button>
										</div>
										<pre class="text-sm bg-muted p-3 rounded overflow-auto whitespace-pre-wrap break-words">{value}</pre>
									</div>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
			
			<!-- YAML Tab -->
			<TabsContent value="yaml" class="space-y-4">
				{#if yamlLoading}
					<div class="flex items-center justify-center h-[600px]">
						<Loading text="Loading YAML..." />
					</div>
				{:else if yamlError}
					<div class="text-destructive text-center h-[600px] flex items-center justify-center">
						<p>{yamlError}</p>
					</div>
				{:else if yaml}
					<YamlEditor
						value={yaml}
						onSave={handleSaveYAML}
						resourceName={configMap.name}
						resourceKind="ConfigMap"
						namespace={configMap.namespace}
					/>
				{:else}
					<div class="text-muted-foreground text-center h-[600px] flex items-center justify-center">
						<p>No YAML available.</p>
					</div>
				{/if}
			</TabsContent>
		</Tabs>
	{/if}
</div>

