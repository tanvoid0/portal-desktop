<!-- StatefulSet Detail Page -->
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
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import YamlEditor from '$lib/domains/cloud/components/YamlEditor.svelte';
	
	const statefulSetName = $derived($page.params.statefulset);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	let statefulSet = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	onMount(async () => {
		await loadStatefulSet();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading) {
			loadYAML();
		}
	});
	
	async function loadStatefulSet() {
		if (!statefulSetName || !$cloudStore.connection.isConnected) {
			error = 'StatefulSet name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.STATEFULSET, namespace);
			const resources = $cloudStore.resources[ResourceType.STATEFULSET] || [];
			statefulSet = resources.find(ss => ss.name === statefulSetName) || null;
			
			if (!statefulSet) {
				error = `StatefulSet "${statefulSetName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load StatefulSet';
			console.error('Failed to load StatefulSet:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!statefulSet) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'StatefulSet',
				namespace: statefulSet.namespace,
				name: statefulSet.name
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
		if (!statefulSet) return;
		
		try {
			await invoke<string>('k8s_apply_resource_yaml', {
				namespace: statefulSet.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success('StatefulSet updated successfully');
			await loadStatefulSet();
			await loadYAML();
		} catch (err) {
			const errorMsg = err instanceof Error ? err.message : 'Failed to update StatefulSet';
			toastActions.error(errorMsg);
			throw err;
		}
	}
	
	function handleTabChange(newTab: string) {
		activeTab = newTab;
		goto(`/cloud/workloads/statefulsets/${statefulSetName}?namespace=${namespace}&tab=${newTab}`);
	}
</script>


<div class="container mx-auto p-6 space-y-6">
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/cloud/workloads/statefulsets')}>
			<ArrowLeft class="h-4 w-4 mr-2" />
			Back
		</Button>
		<Button variant="outline" size="sm" onclick={loadStatefulSet}>
			<RefreshCw class="h-4 w-4 mr-2" />
			Refresh
		</Button>
	</div>
	
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading size="lg" text="Loading StatefulSet..." />
		</div>
	{:else if error}
		<Card>
			<CardContent class="py-12 text-center">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{:else if statefulSet}
		<div class="space-y-6">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-3xl font-bold">{statefulSet.name}</h1>
					<p class="text-muted-foreground mt-1">Namespace: {statefulSet.namespace}</p>
				</div>
				<Badge variant={statefulSet.status === 'running' ? 'default' : 'secondary'}>
					{statefulSet.status}
				</Badge>
			</div>
			
			<Tabs value={activeTab} onValueChange={handleTabChange}>
				<TabsList>
					<TabsTrigger value="overview">Overview</TabsTrigger>
					<TabsTrigger value="yaml">YAML</TabsTrigger>
				</TabsList>
				
				<TabsContent value="overview" class="space-y-4">
					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						<Card>
							<CardHeader>
								<CardTitle>StatefulSet Information</CardTitle>
							</CardHeader>
							<CardContent class="space-y-3">
								<div>
									<p class="text-sm text-muted-foreground">Desired Replicas</p>
									<p class="text-lg font-semibold">{statefulSet.metadata?.desired || 0}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Current Replicas</p>
									<p class="text-lg font-semibold">{statefulSet.metadata?.current || 0}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Ready Replicas</p>
									<p class="text-lg font-semibold">{statefulSet.metadata?.ready || 0}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Age</p>
									<p class="text-lg font-semibold">{statefulSet.metadata?.age || 'N/A'}</p>
								</div>
							</CardContent>
						</Card>
						
						{#if statefulSet.metadata?.labels && Object.keys(statefulSet.metadata.labels).length > 0}
							<Card>
								<CardHeader>
									<CardTitle>Labels</CardTitle>
								</CardHeader>
								<CardContent>
									<div class="flex flex-wrap gap-2">
										{#each Object.entries(statefulSet.metadata.labels) as [key, value]}
											<Badge variant="outline">{key}={value}</Badge>
										{/each}
									</div>
								</CardContent>
							</Card>
						{/if}
					</div>
				</TabsContent>
				
				<TabsContent value="yaml">
					{#if yamlLoading}
						<div class="flex items-center justify-center min-h-[400px]">
							<Loading size="lg" text="Loading YAML..." />
						</div>
					{:else if yamlError}
						<Card>
							<CardContent class="py-12 text-center">
								<p class="text-destructive">{yamlError}</p>
							</CardContent>
						</Card>
					{:else if yaml}
						<YamlEditor
							value={yaml}
							onSave={handleSaveYAML}
						/>
					{/if}
				</TabsContent>
			</Tabs>
		</div>
	{/if}
</div>

