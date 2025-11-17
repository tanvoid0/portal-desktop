<!-- Services List Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType } from '$lib/domains/cloud/core/types';
	import BaseResourceTable from '$lib/domains/cloud/core/components/BaseResourceTable.svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { goto } from '$app/navigation';
	import { useTableNavigation, useResourceActions } from '$lib/domains/k8s-navigation';
	
	let searchQuery = $state('');
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.SERVICE);
		}
	});
	
	const filteredServices = $derived(
		$cloudStore.resources[ResourceType.SERVICE].filter(service => {
			const matchesSearch = !searchQuery || service.name.toLowerCase().includes(searchQuery.toLowerCase());
			return matchesSearch;
		})
	);
	
	const serviceStats = $derived({
		total: $cloudStore.resources[ResourceType.SERVICE].length,
		clusterIP: $cloudStore.resources[ResourceType.SERVICE].filter((s: any) => s.metadata?.type === 'ClusterIP').length,
		loadBalancer: $cloudStore.resources[ResourceType.SERVICE].filter((s: any) => s.metadata?.type === 'LoadBalancer').length,
		nodePort: $cloudStore.resources[ResourceType.SERVICE].filter((s: any) => s.metadata?.type === 'NodePort').length
	});
	
	const serviceColumns = [
		{ key: 'name', label: 'Name', width: 'w-1/4' },
		{ key: 'type', label: 'Type', width: 'w-1/8' },
		{ key: 'clusterIP', label: 'Cluster IP', width: 'w-1/8' },
		{ key: 'ports', label: 'Ports', width: 'w-1/4' },
		{ key: 'age', label: 'Age', width: 'w-1/8' },
		{ key: 'namespace', label: 'Namespace', width: 'w-1/6' }
	];
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: filteredServices.length,
		onSelect: () => {},
		onActivate: (index) => {
			const service = filteredServices[index];
			if (service) {
				goto(`/cloud/workloads/services/${service.name}?namespace=${service.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: filteredServices,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/workloads/services/${resource.name}?namespace=${resource.namespace}`);
			},
			onYaml: (resource) => {
				goto(`/cloud/workloads/services/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onRefresh: () => {
				refreshResources();
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	function handleRefresh() {
		refreshResources();
	}
	
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
				<h1 class="text-2xl font-bold">Services</h1>
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
					<CardTitle class="text-sm font-medium">Total Services</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{serviceStats.total}</div>
				</CardContent>
			</Card>
			
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium">ClusterIP</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{serviceStats.clusterIP}</div>
				</CardContent>
			</Card>
			
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium">LoadBalancer</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{serviceStats.loadBalancer}</div>
				</CardContent>
			</Card>
			
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium">NodePort</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{serviceStats.nodePort}</div>
				</CardContent>
			</Card>
		</div>
		
		<!-- Filters -->
		<Card class="p-4">
			<div class="flex items-center gap-4">
				<Input
					placeholder="Search services..."
					value={searchQuery}
					oninput={(e) => searchQuery = e.currentTarget.value}
					class="max-w-sm"
				/>
				{#if searchQuery}
					<Button variant="ghost" size="sm" onclick={() => searchQuery = ''}>
						Clear
					</Button>
				{/if}
			</div>
		</Card>
		
		<!-- Services Table -->
		<Card class="p-4">
			<div class="k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
				<BaseResourceTable
					resources={filteredServices}
					resourceType={ResourceType.SERVICE}
					columns={serviceColumns}
					emptyMessage={searchQuery ? 'No services match your search' : 'No services found'}
					onResourceClick={(resource) => {
						goto(`/cloud/workloads/services/${resource.name}?namespace=${resource.namespace}`);
					}}
				/>
			</div>
		</Card>
	{/if}
</div>

