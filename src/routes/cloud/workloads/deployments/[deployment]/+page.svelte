<!-- Deployment Detail Page -->
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
	import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/lib/components/ui/dialog';
	import { Label } from '@/lib/components/ui/label';
	import { Input } from '@/lib/components/ui/input';
	import { ArrowLeft, RefreshCw, FileCode, RotateCcw } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import Loading from '@/lib/components/ui/loading.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import YamlEditor from '$lib/domains/cloud/components/YamlEditor.svelte';
	
	const deploymentName = $derived($page.params.deployment);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	let deployment = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	// Scale dialog state
	let showScaleDialog = $state(false);
	let scaleReplicas = $state(1);
	let isScaling = $state(false);
	
	// Rollback state
	let isRollingBack = $state(false);
	
	onMount(async () => {
		await loadDeployment();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading) {
			loadYAML();
		}
	});
	
	async function loadDeployment() {
		if (!deploymentName || !$cloudStore.connection.isConnected) {
			error = 'Deployment name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.DEPLOYMENT, namespace);
			const resources = $cloudStore.resources[ResourceType.DEPLOYMENT] || [];
			deployment = resources.find(d => d.name === deploymentName) || null;
			
			if (!deployment) {
				error = `Deployment "${deploymentName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load deployment';
			console.error('Failed to load deployment:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!deployment) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'Deployment',
				namespace: deployment.namespace,
				name: deployment.name
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
		if (!deployment) return;
		
		try {
			const result = await invoke<string>('k8s_apply_resource_yaml', {
				namespace: deployment.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success(result);
			
			// Reload deployment to get updated data
			await loadDeployment();
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
	
	function handleScale() {
		if (!deployment) return;
		scaleReplicas = Number(deployment.metadata?.desired) || 1;
		showScaleDialog = true;
	}
	
	async function confirmScale() {
		if (!deployment) return;
		
		try {
			isScaling = true;
			await invoke('k8s_scale_deployment', {
				namespace: deployment.namespace,
				deploymentName: deployment.name,
				replicas: scaleReplicas
			});
			
			toastActions.success(`Scaled ${deployment.name} to ${scaleReplicas} replicas`);
			showScaleDialog = false;
			await loadDeployment();
		} catch (error) {
			toastActions.error(`Failed to scale deployment: ${error instanceof Error ? error.message : 'Unknown error'}`);
		} finally {
			isScaling = false;
		}
	}
	
	async function handleRollback() {
		if (!deployment) return;
		
		if (!confirm(`Are you sure you want to rollback deployment "${deployment.name}" to the previous revision?`)) {
			return;
		}
		
		try {
			isRollingBack = true;
			const result = await invoke<string>('k8s_rollback_deployment', {
				namespace: deployment.namespace,
				name: deployment.name
			});
			
			toastActions.success(result);
			await loadDeployment();
		} catch (error) {
			toastActions.error(`Failed to rollback deployment: ${error instanceof Error ? error.message : 'Unknown error'}`);
		} finally {
			isRollingBack = false;
		}
	}
	
	const labels = $derived(() => {
		if (!deployment || !deployment.metadata?.labels) return {};
		return deployment.metadata.labels || {};
	});
</script>


<div class="p-6 space-y-6">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading text="Loading deployment details..." />
		</div>
	{:else if error}
		<div class="text-center py-12 text-destructive">
			<p>{error}</p>
			<Button onclick={() => goto('/cloud/workloads/deployments')} class="mt-4">Back to Deployments</Button>
		</div>
	{:else if deployment}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Deployment: {deployment.name}</h1>
				<p class="text-muted-foreground">Namespace: {deployment.namespace}</p>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={loadDeployment}>
					<RefreshCw class="mr-2 h-4 w-4" />
					Refresh
				</Button>
				<Button variant="outline" size="sm" onclick={handleScale}>
					Scale
				</Button>
				<Button variant="outline" size="sm" onclick={handleRollback} disabled={isRollingBack}>
					{#if isRollingBack}
						<span class="i-lucide-loader-2 animate-spin mr-2 h-4 w-4"></span>
					{:else}
						<RotateCcw class="mr-2 h-4 w-4" />
					{/if}
					Rollback
				</Button>
				<Button variant="outline" size="sm" onclick={() => goto('/cloud/workloads/deployments')}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back to Deployments
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
							<CardTitle>Deployment Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div>
								<p class="text-sm text-muted-foreground">Status</p>
								<Badge class="mt-1" variant="default">{deployment.status}</Badge>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Desired Replicas</p>
								<p class="font-medium">{deployment.metadata?.desired || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Current Replicas</p>
								<p class="font-medium">{deployment.metadata?.current || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Available Replicas</p>
								<p class="font-medium">{deployment.metadata?.available || 0}/{deployment.metadata?.desired || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Up-to-date Replicas</p>
								<p class="font-medium">{deployment.metadata?.up_to_date || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Age</p>
								<p class="font-medium">{deployment.metadata?.age || 'N/A'}</p>
							</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader>
							<CardTitle>Replica Status</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-4">
								<div>
									<div class="flex justify-between mb-2">
										<span class="text-sm">Ready</span>
										<span class="font-medium">{deployment.metadata?.available || 0}/{deployment.metadata?.desired || 0}</span>
									</div>
									<div class="w-full bg-muted rounded-full h-2">
										<div 
											class="bg-green-600 h-2 rounded-full transition-all"
											style="width: {deployment.metadata?.desired ? (deployment.metadata.available / deployment.metadata.desired * 100) : 0}%"
										></div>
									</div>
								</div>
								<div>
									<div class="flex justify-between mb-2">
										<span class="text-sm">Up-to-date</span>
										<span class="font-medium">{deployment.metadata?.up_to_date || 0}/{deployment.metadata?.desired || 0}</span>
									</div>
									<div class="w-full bg-muted rounded-full h-2">
										<div 
											class="bg-blue-600 h-2 rounded-full transition-all"
											style="width: {deployment.metadata?.desired ? (deployment.metadata.up_to_date / deployment.metadata.desired * 100) : 0}%"
										></div>
									</div>
								</div>
							</div>
						</CardContent>
					</Card>
				</div>
				
				<Card>
					<CardHeader>
						<CardTitle>Labels</CardTitle>
					</CardHeader>
					<CardContent>
						{#if Object.keys(labels).length > 0}
							<div class="flex flex-wrap gap-2">
								{#each Object.entries(labels) as [key, value]}
									<Badge variant="outline">{key}={value}</Badge>
								{/each}
							</div>
						{:else}
							<p class="text-muted-foreground">No labels configured</p>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
			
			<!-- YAML Tab -->
			<TabsContent value="yaml" class="h-[600px]">
				<Card class="h-full flex flex-col">
					<CardHeader>
						<CardTitle>Deployment YAML</CardTitle>
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
								resourceName={deployment.name}
								resourceKind="Deployment"
								namespace={deployment.namespace}
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
		
		<!-- Scale Dialog -->
		<Dialog bind:open={showScaleDialog}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Scale Deployment: {deployment?.name}</DialogTitle>
					<DialogDescription>
						Enter the desired number of replicas for this deployment.
					</DialogDescription>
				</DialogHeader>
				<div class="grid gap-4 py-4">
					<div class="grid grid-cols-4 items-center gap-4">
						<Label for="replicas" class="text-right">
							Replicas
						</Label>
						<Input
							id="replicas"
							type="number"
							min="0"
							bind:value={scaleReplicas}
							class="col-span-3"
						/>
					</div>
				</div>
				<DialogFooter>
					<Button variant="outline" onclick={() => showScaleDialog = false} disabled={isScaling}>Cancel</Button>
					<Button onclick={confirmScale} disabled={isScaling}>
						{isScaling ? 'Scaling...' : 'Scale'}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	{/if}
</div>

