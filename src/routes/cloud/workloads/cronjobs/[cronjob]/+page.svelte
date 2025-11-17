<!-- CronJob Detail Page -->
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
	
	const cronJobName = $derived($page.params.cronjob);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	
	let cronJob = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	onMount(async () => {
		await loadCronJob();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading && cronJob) {
			loadYAML();
		}
	});
	
	async function loadCronJob() {
		if (!cronJobName || !$cloudStore.connection.isConnected) {
			error = 'CronJob name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.CRONJOB, namespace);
			const resources = $cloudStore.resources[ResourceType.CRONJOB] || [];
			cronJob = resources.find(cj => cj.name === cronJobName) || null;
			
			if (!cronJob) {
				error = `CronJob "${cronJobName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load CronJob';
			console.error('Failed to load CronJob:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!cronJob) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'CronJob',
				namespace: cronJob.namespace,
				name: cronJob.name
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
		if (!cronJob) return;
		
		try {
			const result = await invoke<string>('k8s_apply_resource_yaml', {
				namespace: cronJob.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success(result);
			
			// Reload CronJob to get updated data
			await loadCronJob();
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
</script>


<div class="p-6 space-y-6">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading text="Loading CronJob details..." />
		</div>
	{:else if error}
		<div class="text-center py-12 text-destructive">
			<p>{error}</p>
			<Button onclick={() => goto('/cloud/workloads/cronjobs')} class="mt-4">Back to CronJobs</Button>
		</div>
	{:else if cronJob}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">CronJob: {cronJob.name}</h1>
				<p class="text-muted-foreground">Namespace: {cronJob.namespace}</p>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={loadCronJob}>
					<RefreshCw class="mr-2 h-4 w-4" />
					Refresh
				</Button>
				<Button variant="outline" size="sm" onclick={() => goto('/cloud/workloads/cronjobs')}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back to CronJobs
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
							<CardTitle>CronJob Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div>
								<p class="text-sm text-muted-foreground">Status</p>
								<Badge class="mt-1" variant={cronJob.metadata?.suspend ? 'secondary' : 'default'}>
									{cronJob.metadata?.suspend ? 'Suspended' : 'Active'}
								</Badge>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Schedule</p>
								<p class="font-medium font-mono">{cronJob.metadata?.schedule || 'N/A'}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Active Jobs</p>
								<p class="font-medium">{cronJob.metadata?.active || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Last Schedule Time</p>
								<p class="font-medium">{cronJob.metadata?.last_schedule_time || 'Never'}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Last Successful Time</p>
								<p class="font-medium">{cronJob.metadata?.last_successful_time || 'Never'}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Age</p>
								<p class="font-medium">{cronJob.metadata?.age || 'N/A'}</p>
							</div>
							{#if cronJob.metadata?.image}
								<div>
									<p class="text-sm text-muted-foreground">Image</p>
									<p class="font-medium">{cronJob.metadata.image}</p>
								</div>
							{/if}
						</CardContent>
					</Card>
				</div>
			</TabsContent>
			
			<!-- YAML Tab -->
			<TabsContent value="yaml" class="h-[600px]">
				<Card class="h-full flex flex-col">
					<CardHeader>
						<CardTitle>CronJob YAML</CardTitle>
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
								resourceName={cronJob.name}
								resourceKind="CronJob"
								namespace={cronJob.namespace}
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

