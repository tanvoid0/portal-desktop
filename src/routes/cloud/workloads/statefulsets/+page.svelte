<!-- StatefulSets List Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import BaseResourceTable from '$lib/domains/cloud/core/components/BaseResourceTable.svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { RefreshCw } from '@lucide/svelte';
	import { useTableNavigation, useResourceActions } from '$lib/domains/k8s-navigation';
	
	let searchQuery = $state('');
	let statusFilter = $state('');
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.STATEFULSET);
		}
	});
	
	const filteredStatefulSets = $derived(
		$cloudStore.resources[ResourceType.STATEFULSET].filter(statefulset => {
			const matchesSearch = !searchQuery || statefulset.name.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesStatus = !statusFilter || statefulset.status === statusFilter;
			return matchesSearch && matchesStatus;
		})
	);
	
	const statefulSetStats = $derived({
		total: $cloudStore.resources[ResourceType.STATEFULSET].length,
		running: $cloudStore.resources[ResourceType.STATEFULSET].filter((ss: any) => ss.status === 'running').length,
		pending: $cloudStore.resources[ResourceType.STATEFULSET].filter((ss: any) => ss.status === 'pending').length,
		failed: $cloudStore.resources[ResourceType.STATEFULSET].filter((ss: any) => ss.status === 'failed').length
	});
	
	const statusOptions = $derived(() => {
		const statuses = new Set($cloudStore.resources[ResourceType.STATEFULSET].map((ss: any) => ss.status));
		return Array.from(statuses).sort();
	});
	
	const statefulSetColumns = [
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
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: filteredStatefulSets.length,
		onSelect: () => {},
		onActivate: (index) => {
			const statefulset = filteredStatefulSets[index];
			if (statefulset) {
				goto(`/cloud/workloads/statefulsets/${statefulset.name}?namespace=${statefulset.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: filteredStatefulSets,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/workloads/statefulsets/${resource.name}?namespace=${resource.namespace}`);
			},
			onEdit: (resource) => {
				goto(`/cloud/workloads/statefulsets/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onRestart: async (resource) => {
				// StatefulSets don't have a direct restart, but we can scale to 0 and back
				await refreshResources();
			},
			onYaml: (resource) => {
				goto(`/cloud/workloads/statefulsets/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
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


<div class="container mx-auto p-6 space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">StatefulSets</h1>
			<p class="text-muted-foreground mt-1">Manage stateful workloads with stable network identities</p>
		</div>
		<Button variant="outline" onclick={handleRefresh}>
			<RefreshCw class="h-4 w-4 mr-2" />
			Refresh
		</Button>
	</div>
	
	<!-- Statistics -->
	<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Total</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{statefulSetStats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Running</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600 dark:text-green-400">{statefulSetStats.running}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Pending</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-600 dark:text-yellow-400">{statefulSetStats.pending}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Failed</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600 dark:text-red-400">{statefulSetStats.failed}</div>
			</CardContent>
		</Card>
	</div>
	
	<!-- Filters -->
	<div class="flex gap-4">
		<Input
			placeholder="Search StatefulSets..."
			bind:value={searchQuery}
			class="max-w-sm"
		/>
		{#if statusOptions().length > 0}
			<select
				bind:value={statusFilter}
				class="px-3 py-2 border rounded-md bg-background text-foreground"
			>
				<option value="">All Statuses</option>
				{#each statusOptions() as status}
					<option value={status}>{status}</option>
				{/each}
			</select>
		{/if}
	</div>
	
	<!-- StatefulSets Table -->
	<div class="k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
		<BaseResourceTable
			resources={filteredStatefulSets}
			columns={statefulSetColumns}
			resourceType={ResourceType.STATEFULSET}
			onResourceClick={(resource) => {
				goto(`/cloud/workloads/statefulsets/${resource.name}?namespace=${resource.namespace}`);
			}}
		/>
	</div>
	
	{#if filteredStatefulSets.length === 0 && !$cloudStore.loading.resources[ResourceType.STATEFULSET]}
		<Card>
			<CardContent class="py-12 text-center">
				<p class="text-muted-foreground">No StatefulSets found</p>
			</CardContent>
		</Card>
	{/if}
</div>

