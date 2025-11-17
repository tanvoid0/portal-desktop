<!-- Ingress List Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import BaseResourceTable from '$lib/domains/cloud/core/components/BaseResourceTable.svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { RefreshCw, Search } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { useTableNavigation, useResourceActions } from '$lib/domains/k8s-navigation';
	
	let searchQuery = $state('');
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.INGRESS);
		}
	});
	
	const filteredIngresses = $derived(
		$cloudStore.resources[ResourceType.INGRESS].filter(ingress => {
			const matchesSearch = !searchQuery || ingress.name.toLowerCase().includes(searchQuery.toLowerCase());
			return matchesSearch;
		})
	);
	
	const ingressStats = $derived({
		total: $cloudStore.resources[ResourceType.INGRESS].length,
		withAddresses: $cloudStore.resources[ResourceType.INGRESS].filter((i: any) => 
			(i.metadata?.addresses || []).length > 0
		).length
	});
	
	const ingressColumns = [
		{ key: 'name', label: 'Name', width: 'w-1/4' },
		{ key: 'class', label: 'Class', width: 'w-1/6' },
		{ key: 'addresses', label: 'Addresses', width: 'w-1/4' },
		{ key: 'ports', label: 'Hosts', width: 'w-1/4' },
		{ key: 'age', label: 'Age', width: 'w-1/8' },
		{ key: 'namespace', label: 'Namespace', width: 'w-1/6' }
	];
	
	async function handleRefresh() {
		await refreshResources();
	}
	
	function handleIngressClick(ingress: ICloudResource) {
		goto(`/cloud/ingress/${ingress.name}?namespace=${ingress.namespace}`);
	}
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: filteredIngresses.length,
		onSelect: () => {},
		onActivate: (index) => {
			const ingress = filteredIngresses[index];
			if (ingress) {
				goto(`/cloud/ingress/${ingress.name}?namespace=${ingress.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: filteredIngresses,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/ingress/${resource.name}?namespace=${resource.namespace}`);
			},
			onEdit: (resource) => {
				goto(`/cloud/ingress/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onYaml: (resource) => {
				goto(`/cloud/ingress/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
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
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Ingress</h1>
			<p class="text-muted-foreground">Manage Kubernetes Ingress Resources</p>
		</div>
		<Button variant="outline" onclick={handleRefresh}>
			<RefreshCw class="mr-2 h-4 w-4" />
			Refresh
		</Button>
	</div>
	
	<!-- Statistics -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Total Ingress</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{ingressStats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">With Addresses</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">{ingressStats.withAddresses}</div>
			</CardContent>
		</Card>
	</div>
	
	<!-- Search -->
	<div class="flex items-center gap-4">
		<div class="flex-1 relative">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
			<Input
				type="text"
				placeholder="Search Ingress..."
				value={searchQuery}
				oninput={(e) => searchQuery = (e.target as HTMLInputElement).value}
				class="pl-10"
			/>
		</div>
	</div>
	
	<!-- Ingress Table -->
	<Card>
		<CardHeader>
			<CardTitle>Ingress ({filteredIngresses.length})</CardTitle>
		</CardHeader>
		<CardContent>
			{#if filteredIngresses.length === 0}
				<div class="text-center py-8 text-muted-foreground">
					<p>No Ingress resources found</p>
					{#if searchQuery}
						<p class="text-xs mt-2">Try adjusting your search</p>
					{/if}
				</div>
			{:else}
				<div class="k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
					<BaseResourceTable
						resources={filteredIngresses}
						resourceType={ResourceType.INGRESS}
						columns={ingressColumns}
						onResourceClick={handleIngressClick}
					/>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

