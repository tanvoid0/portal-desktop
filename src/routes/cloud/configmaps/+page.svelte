<!-- ConfigMaps List Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import BaseResourceTable from '$lib/domains/cloud/core/components/BaseResourceTable.svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { RefreshCw, Search, Plus } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { useTableNavigation, useResourceActions } from '$lib/domains/k8s-navigation';
	
	let searchQuery = $state('');
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.CONFIGMAP);
		}
	});
	
	const filteredConfigMaps = $derived(
		$cloudStore.resources[ResourceType.CONFIGMAP].filter(cm => {
			const matchesSearch = !searchQuery || cm.name.toLowerCase().includes(searchQuery.toLowerCase());
			return matchesSearch;
		})
	);
	
	const configMapStats = $derived({
		total: $cloudStore.resources[ResourceType.CONFIGMAP].length,
		withData: $cloudStore.resources[ResourceType.CONFIGMAP].filter((cm: any) => (cm.metadata?.dataCount || 0) > 0).length
	});
	
	const configMapColumns = [
		{ key: 'name', label: 'Name', width: 'w-1/4' },
		{ key: 'dataCount', label: 'Data Keys', width: 'w-1/8' },
		{ key: 'age', label: 'Age', width: 'w-1/8' },
		{ key: 'namespace', label: 'Namespace', width: 'w-1/6' }
	];
	
	async function handleRefresh() {
		await refreshResources();
	}
	
	function handleConfigMapClick(configMap: ICloudResource) {
		goto(`/cloud/configmaps/${configMap.name}?namespace=${configMap.namespace}`);
	}
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: filteredConfigMaps.length,
		onSelect: () => {},
		onActivate: (index) => {
			const configMap = filteredConfigMaps[index];
			if (configMap) {
				goto(`/cloud/configmaps/${configMap.name}?namespace=${configMap.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: filteredConfigMaps,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/configmaps/${resource.name}?namespace=${resource.namespace}`);
			},
			onEdit: (resource) => {
				goto(`/cloud/configmaps/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onYaml: (resource) => {
				goto(`/cloud/configmaps/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
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
			<h1 class="text-2xl font-bold">ConfigMaps</h1>
			<p class="text-muted-foreground">Manage Kubernetes ConfigMaps</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={handleRefresh}>
				<RefreshCw class="mr-2 h-4 w-4" />
				Refresh
			</Button>
			<Button onclick={() => goto('/cloud/configmaps/new')}>
				<Plus class="mr-2 h-4 w-4" />
				Create ConfigMap
			</Button>
		</div>
	</div>
	
	<!-- Statistics -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Total ConfigMaps</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{configMapStats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">With Data</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">{configMapStats.withData}</div>
			</CardContent>
		</Card>
	</div>
	
	<!-- Search -->
	<div class="flex items-center gap-4">
		<div class="flex-1 relative">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
			<Input
				type="text"
				placeholder="Search ConfigMaps..."
				value={searchQuery}
				oninput={(e) => searchQuery = (e.target as HTMLInputElement).value}
				class="pl-10"
			/>
		</div>
	</div>
	
	<!-- ConfigMaps Table -->
	<Card>
		<CardHeader>
			<CardTitle>ConfigMaps ({filteredConfigMaps.length})</CardTitle>
		</CardHeader>
		<CardContent>
			{#if filteredConfigMaps.length === 0}
				<div class="text-center py-8 text-muted-foreground">
					<p>No ConfigMaps found</p>
					{#if searchQuery}
						<p class="text-xs mt-2">Try adjusting your search</p>
					{/if}
				</div>
			{:else}
				<div class="k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
					<BaseResourceTable
						resources={filteredConfigMaps}
						resourceType={ResourceType.CONFIGMAP}
						columns={configMapColumns}
						onResourceClick={handleConfigMapClick}
					/>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

