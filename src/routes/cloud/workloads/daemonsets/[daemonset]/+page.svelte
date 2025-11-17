<!-- DaemonSet Detail Page -->
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
	import { invoke } from '@tauri-apps/api/core';
	import Loading from '@/lib/components/ui/loading.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import YamlEditor from '$lib/domains/cloud/components/YamlEditor.svelte';
	
	const daemonSetName = $derived($page.params.daemonset);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	let daemonSet = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	onMount(async () => {
		await loadDaemonSet();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading) {
			loadYAML();
		}
	});
	
	async function loadDaemonSet() {
		if (!daemonSetName || !$cloudStore.connection.isConnected) {
			error = 'DaemonSet name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.DAEMONSET, namespace);
			const resources = $cloudStore.resources[ResourceType.DAEMONSET] || [];
			daemonSet = resources.find(ds => ds.name === daemonSetName) || null;
			
			if (!daemonSet) {
				error = `DaemonSet "${daemonSetName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load DaemonSet';
			console.error('Failed to load DaemonSet:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!daemonSet) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'DaemonSet',
				namespace: daemonSet.namespace,
				name: daemonSet.name
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
		if (!daemonSet) return;
		
		try {
			await invoke<string>('k8s_apply_resource_yaml', {
				namespace: daemonSet.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success('DaemonSet updated successfully');
			await loadDaemonSet();
			await loadYAML();
		} catch (err) {
			const errorMsg = err instanceof Error ? err.message : 'Failed to update DaemonSet';
			toastActions.error(errorMsg);
			throw err;
		}
	}
	
	function handleTabChange(newTab: string) {
		activeTab = newTab;
		goto(`/cloud/workloads/daemonsets/${daemonSetName}?namespace=${namespace}&tab=${newTab}`);
	}
</script>


<div class="container mx-auto p-6 space-y-6">
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/cloud/workloads/daemonsets')}>
			<ArrowLeft class="h-4 w-4 mr-2" />
			Back
		</Button>
		<Button variant="outline" size="sm" onclick={loadDaemonSet}>
			<RefreshCw class="h-4 w-4 mr-2" />
			Refresh
		</Button>
	</div>
	
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading size="lg" text="Loading DaemonSet..." />
		</div>
	{:else if error}
		<Card>
			<CardContent class="py-12 text-center">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{:else if daemonSet}
		<div class="space-y-6">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-3xl font-bold">{daemonSet.name}</h1>
					<p class="text-muted-foreground mt-1">Namespace: {daemonSet.namespace}</p>
				</div>
				<Badge variant={daemonSet.status === 'running' ? 'default' : 'secondary'}>
					{daemonSet.status}
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
								<CardTitle>DaemonSet Information</CardTitle>
							</CardHeader>
							<CardContent class="space-y-3">
								<div>
									<p class="text-sm text-muted-foreground">Desired</p>
									<p class="text-lg font-semibold">{daemonSet.metadata?.desired || 0}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Current</p>
									<p class="text-lg font-semibold">{daemonSet.metadata?.current || 0}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Ready</p>
									<p class="text-lg font-semibold">{daemonSet.metadata?.ready || 0}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Up to Date</p>
									<p class="text-lg font-semibold">{daemonSet.metadata?.up_to_date || 0}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Available</p>
									<p class="text-lg font-semibold">{daemonSet.metadata?.available || 0}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Age</p>
									<p class="text-lg font-semibold">{daemonSet.metadata?.age || 'N/A'}</p>
								</div>
							</CardContent>
						</Card>
						
						{#if daemonSet.metadata?.labels && Object.keys(daemonSet.metadata.labels).length > 0}
							<Card>
								<CardHeader>
									<CardTitle>Labels</CardTitle>
								</CardHeader>
								<CardContent>
									<div class="flex flex-wrap gap-2">
										{#each Object.entries(daemonSet.metadata.labels) as [key, value]}
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

