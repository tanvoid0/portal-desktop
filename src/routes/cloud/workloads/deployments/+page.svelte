<!-- Deployments List Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import BaseResourceTable from '$lib/domains/cloud/core/components/BaseResourceTable.svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import Select from '@/lib/components/ui/select.svelte';
	import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/lib/components/ui/dialog';
	import { Label } from '@/lib/components/ui/label';
	import { invoke } from '@tauri-apps/api/core';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { goto } from '$app/navigation';
	import { get } from 'svelte/store';
	import { useTableNavigation, useResourceActions } from '$lib/domains/k8s-navigation';
	
	let searchQuery = $state('');
	let statusFilter = $state('');
	let scalingDeployment = $state<ICloudResource | null>(null);
	let scaleReplicas = $state(1);
	let isScaling = $state(false);
	let showScaleDialog = $state(false);
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.DEPLOYMENT);
		}
	});
	
	const filteredDeployments = $derived(
		$cloudStore.resources[ResourceType.DEPLOYMENT].filter(deployment => {
			const matchesSearch = !searchQuery || deployment.name.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesStatus = !statusFilter || deployment.status === statusFilter;
			return matchesSearch && matchesStatus;
		})
	);
	
	const filteredDeploymentsLength = $derived(filteredDeployments.length);
	
	const deploymentStats = $derived({
		total: $cloudStore.resources[ResourceType.DEPLOYMENT].length,
		running: $cloudStore.resources[ResourceType.DEPLOYMENT].filter((d: any) => d.status === 'running').length,
		pending: $cloudStore.resources[ResourceType.DEPLOYMENT].filter((d: any) => d.status === 'pending').length,
		failed: $cloudStore.resources[ResourceType.DEPLOYMENT].filter((d: any) => d.status === 'failed').length
	});
	
	const statusOptions = $derived(() => {
		const statuses = new Set($cloudStore.resources[ResourceType.DEPLOYMENT].map((d: any) => d.status));
		return Array.from(statuses).sort();
	});
	
	const deploymentColumns = [
		{ key: 'name', label: 'Name', width: 'w-1/4' },
		{ key: 'status', label: 'Status', width: 'w-1/8' },
		{ key: 'replicas', label: 'Replicas', width: 'w-1/8' },
		{ key: 'ready', label: 'Ready', width: 'w-1/8' },
		{ key: 'age', label: 'Age', width: 'w-1/8' },
		{ key: 'namespace', label: 'Namespace', width: 'w-1/6' }
	];
	
	async function handleRefresh() {
		await refreshResources();
	}
	
	function handleScale(deployment: ICloudResource) {
		scalingDeployment = deployment;
		scaleReplicas = Number(deployment.metadata?.replicas) || 1;
		showScaleDialog = true;
	}
	
	async function confirmScale() {
		if (!scalingDeployment) return;
		
		try {
			isScaling = true;
			await invoke('k8s_scale_deployment', {
				namespace: scalingDeployment.namespace,
				deploymentName: scalingDeployment.name,
				replicas: scaleReplicas
			});
			
			toastActions.success(`Scaled ${scalingDeployment.name} to ${scaleReplicas} replicas`);
			showScaleDialog = false;
			scalingDeployment = null;
			
			// Refresh deployments
			await loadResources(ResourceType.DEPLOYMENT);
		} catch (error) {
			toastActions.error(`Failed to scale deployment: ${error instanceof Error ? error.message : 'Unknown error'}`);
		} finally {
			isScaling = false;
		}
	}
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: () => filteredDeploymentsLength,
		onSelect: (index) => {
			// Visual selection handled by CSS
		},
		onActivate: (index) => {
			const deployments = filteredDeployments;
			const deployment = deployments[index];
			if (deployment) {
				goto(`/cloud/workloads/deployments/${deployment.name}?namespace=${deployment.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: () => filteredDeployments,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/workloads/deployments/${resource.name}?namespace=${resource.namespace}`);
			},
			onEdit: (resource) => {
				goto(`/cloud/workloads/deployments/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onRestart: async (resource) => {
				try {
					await invoke('k8s_rollback_deployment', {
						namespace: resource.namespace,
						deploymentName: resource.name
					});
					toastActions.success(`Restarted ${resource.name}`);
					await loadResources(ResourceType.DEPLOYMENT);
				} catch (error) {
					toastActions.error(`Failed to restart: ${error instanceof Error ? error.message : 'Unknown error'}`);
				}
			},
			onScale: (resource) => {
				handleScale(resource);
			},
			onYaml: (resource) => {
				goto(`/cloud/workloads/deployments/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onRefresh: () => {
				refreshResources();
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	function handleKeydown(event: KeyboardEvent) {
		if (tableNav.handleKeydown(event)) return;
		if (resourceActions.handleKeydown(event)) return;
	}
</script>

<svelte:window onkeydown={handleKeydown} />


<div class="p-6 space-y-6">
	{#if $cloudStore.connection.isConnected}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Deployments</h1>
				<p class="text-muted-foreground">
					Namespace: {$cloudStore.selectedNamespace || 'All'}
				</p>
			</div>
			<Button onclick={handleRefresh} variant="outline">
				Refresh
			</Button>
		</div>
		
		<!-- Statistics -->
		<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium">Total Deployments</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{deploymentStats.total}</div>
				</CardContent>
			</Card>
			
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium">Running</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-green-600 dark:text-green-400">{deploymentStats.running}</div>
				</CardContent>
			</Card>
			
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium">Pending</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-yellow-600 dark:text-yellow-400">{deploymentStats.pending}</div>
				</CardContent>
			</Card>
			
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium">Failed</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-red-600 dark:text-red-400">{deploymentStats.failed}</div>
				</CardContent>
			</Card>
		</div>
		
		<!-- Filters -->
		<Card class="p-4">
			<div class="flex items-center gap-4">
				<Input
					placeholder="Search deployments..."
					value={searchQuery}
					oninput={(e) => searchQuery = e.currentTarget.value}
					class="max-w-sm"
				/>
				<Select
					options={[
						{ value: '', label: 'All Statuses' },
						...statusOptions().map(s => ({ value: s, label: s }))
					]}
					bind:value={statusFilter}
					placeholder="All Statuses"
				/>
				{#if searchQuery || statusFilter}
					<Button variant="ghost" size="sm" onclick={() => { searchQuery = ''; statusFilter = ''; }}>
						Clear
					</Button>
				{/if}
			</div>
		</Card>
		
		<!-- Deployments Table -->
		<Card class="p-4">
			{#if filteredDeployments.length === 0}
				<div class="text-center py-8 text-muted-foreground">
					{searchQuery || statusFilter ? 'No deployments match your filters' : 'No deployments found'}
				</div>
			{:else}
				<div class="overflow-x-auto k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
					<table class="w-full">
						<thead>
							<tr class="border-b">
								{#each deploymentColumns as column}
									<th class="text-left p-2 font-medium">{column.label}</th>
								{/each}
								<th class="text-left p-2 font-medium">Actions</th>
							</tr>
						</thead>
						<tbody>
							{#each filteredDeployments as deployment, index}
								{@const selectedIdx = get(tableNav.selectedIndex)}
								{@const isSelected = index === selectedIdx}
								<tr 
									class="border-b hover:bg-muted/50 {isSelected ? 'bg-accent' : ''}"
									data-selected={isSelected}
									data-index={index}
								>
									<td class="p-2">
										<div class="flex items-center space-x-2">
											<div class="w-2 h-2 rounded-full {deployment.status === 'running' ? 'bg-green-500' : deployment.status === 'pending' ? 'bg-yellow-500' : 'bg-red-500'}"></div>
											<button 
												class="font-medium hover:underline cursor-pointer"
												onclick={() => goto(`/cloud/workloads/deployments/${deployment.name}?namespace=${deployment.namespace}`)}
											>
												{deployment.name}
											</button>
										</div>
									</td>
									<td class="p-2">
										<span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium">
											{deployment.status}
										</span>
									</td>
									<td class="p-2 text-sm">{deployment.metadata?.replicas || 'N/A'}</td>
									<td class="p-2 text-sm">{deployment.metadata?.readyReplicas || 0}/{deployment.metadata?.replicas || 0}</td>
									<td class="p-2 text-sm">{deployment.metadata?.age || 'N/A'}</td>
									<td class="p-2 text-sm">{deployment.namespace}</td>
									<td class="p-2">
										<Button variant="outline" size="sm" onclick={() => handleScale(deployment)}>
											Scale
										</Button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</Card>
		
		<!-- Scale Dialog -->
		<Dialog bind:open={showScaleDialog}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Scale Deployment</DialogTitle>
					<DialogDescription>
						Set the number of replicas for {scalingDeployment?.name}
					</DialogDescription>
				</DialogHeader>
				<div class="space-y-4 py-4">
					<div class="space-y-2">
						<Label for="replicas">Replicas</Label>
						<Input
							id="replicas"
							type="number"
							min="0"
							bind:value={scaleReplicas}
							disabled={isScaling}
						/>
					</div>
				</div>
				<DialogFooter>
					<Button variant="outline" onclick={() => { showScaleDialog = false; }} disabled={isScaling}>
						Cancel
					</Button>
					<Button onclick={confirmScale} disabled={isScaling}>
						{isScaling ? 'Scaling...' : 'Scale'}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	{/if}
</div>
