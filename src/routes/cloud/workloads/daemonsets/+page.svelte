<!-- DaemonSets List Page -->
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
			await loadResources(ResourceType.DAEMONSET);
		}
	});
	
	const filteredDaemonSets = $derived(
		$cloudStore.resources[ResourceType.DAEMONSET].filter(daemonset => {
			const matchesSearch = !searchQuery || daemonset.name.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesStatus = !statusFilter || daemonset.status === statusFilter;
			return matchesSearch && matchesStatus;
		})
	);
	
	const daemonSetStats = $derived({
		total: $cloudStore.resources[ResourceType.DAEMONSET].length,
		running: $cloudStore.resources[ResourceType.DAEMONSET].filter((ds: any) => ds.status === 'running').length,
		pending: $cloudStore.resources[ResourceType.DAEMONSET].filter((ds: any) => ds.status === 'pending').length,
		failed: $cloudStore.resources[ResourceType.DAEMONSET].filter((ds: any) => ds.status === 'failed').length
	});
	
	const statusOptions = $derived(() => {
		const statuses = new Set($cloudStore.resources[ResourceType.DAEMONSET].map((ds: any) => ds.status));
		return Array.from(statuses).sort();
	});
	
	const daemonSetColumns = [
		{ key: 'name', label: 'Name', width: 'w-1/4' },
		{ key: 'status', label: 'Status', width: 'w-1/8' },
		{ key: 'desired', label: 'Desired', width: 'w-1/8' },
		{ key: 'current', label: 'Current', width: 'w-1/8' },
		{ key: 'ready', label: 'Ready', width: 'w-1/8' },
		{ key: 'age', label: 'Age', width: 'w-1/8' },
		{ key: 'namespace', label: 'Namespace', width: 'w-1/6' }
	];
	
	async function handleRefresh() {
		await refreshResources();
	}
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: filteredDaemonSets.length,
		onSelect: () => {},
		onActivate: (index) => {
			const daemonset = filteredDaemonSets[index];
			if (daemonset) {
				goto(`/cloud/workloads/daemonsets/${daemonset.name}?namespace=${daemonset.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: filteredDaemonSets,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/workloads/daemonsets/${resource.name}?namespace=${resource.namespace}`);
			},
			onEdit: (resource) => {
				goto(`/cloud/workloads/daemonsets/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onYaml: (resource) => {
				goto(`/cloud/workloads/daemonsets/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
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
			<h1 class="text-3xl font-bold">DaemonSets</h1>
			<p class="text-muted-foreground mt-1">Manage daemon workloads that run on every node</p>
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
				<div class="text-2xl font-bold">{daemonSetStats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Running</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600 dark:text-green-400">{daemonSetStats.running}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Pending</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-600 dark:text-yellow-400">{daemonSetStats.pending}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Failed</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600 dark:text-red-400">{daemonSetStats.failed}</div>
			</CardContent>
		</Card>
	</div>
	
	<!-- Filters -->
	<div class="flex gap-4">
		<Input
			placeholder="Search DaemonSets..."
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
	
	<!-- DaemonSets Table -->
	<div class="k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
		<BaseResourceTable
			resources={filteredDaemonSets}
			columns={daemonSetColumns}
			resourceType={ResourceType.DAEMONSET}
			onResourceClick={(resource) => {
				goto(`/cloud/workloads/daemonsets/${resource.name}?namespace=${resource.namespace}`);
			}}
		/>
	</div>
	
	{#if filteredDaemonSets.length === 0 && !$cloudStore.loading.resources[ResourceType.DAEMONSET]}
		<Card>
			<CardContent class="py-12 text-center">
				<p class="text-muted-foreground">No DaemonSets found</p>
			</CardContent>
		</Card>
	{/if}
</div>

