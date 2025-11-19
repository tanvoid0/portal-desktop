<!-- Pods List Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType } from '$lib/domains/cloud/core/types';
	import PodsTable from '$lib/domains/cloud/components/workloads/PodsTable.svelte';
	import PodsStatistics from '$lib/domains/cloud/components/workloads/PodsStatistics.svelte';
	import PodsFilters from '$lib/domains/cloud/components/workloads/PodsFilters.svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card } from '@/lib/components/ui/card';
	import { useK8sKeyboard, KeyboardShortcutsPanel, useTableNavigation } from '$lib/domains/k8s-navigation';
	import type { ICloudResource } from '$lib/domains/cloud/core/types';
	
	let searchQuery = $state('');
	let statusFilter = $state('');
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.POD);
		}
	});
	
	const filteredPods = $derived(
		$cloudStore.resources[ResourceType.POD].filter(pod => {
			const matchesSearch = !searchQuery || pod.name.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesStatus = !statusFilter || pod.status === statusFilter;
			return matchesSearch && matchesStatus;
		})
	);
	
	const filteredPodsLength = $derived(filteredPods.length);
	
	// Unified keyboard handler - K8s-specific wrapper
	const tableNav = useTableNavigation({
		totalItems: () => filteredPodsLength,
		onSelect: () => {},
		onActivate: (index) => {
			const pods = filteredPods;
			const pod = pods[index];
			if (pod) {
				goto(`/cloud/workloads/pods/${pod.name}?namespace=${pod.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	const keyboard = useK8sKeyboard({
		tableNavigation: () => ({
			totalItems: filteredPodsLength,
			onSelect: () => {},
			onActivate: (index) => {
				const pods = filteredPods;
				const pod = pods[index];
				if (pod) {
					goto(`/cloud/workloads/pods/${pod.name}?namespace=${pod.namespace}`);
				}
			},
			enabled: $cloudStore.connection.isConnected
		}),
		resourceActions: {
			selectedIndex: tableNav.selectedIndex,
			resources: () => filteredPods,
			handlers: {
				onDescribe: (resource) => {
					goto(`/cloud/workloads/pods/${resource.name}?namespace=${resource.namespace}`);
				},
				onLogs: (resource) => {
					goto(`/cloud/workloads/pods/${resource.name}?namespace=${resource.namespace}&tab=logs`);
				},
				onYaml: (resource) => {
					goto(`/cloud/workloads/pods/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
				},
				onPortForward: (resource) => {
					goto(`/cloud/workloads/pods/${resource.name}?namespace=${resource.namespace}&tab=port-forward`);
				},
				onRefresh: () => {
					refreshResources();
				}
			},
			enabled: $cloudStore.connection.isConnected
		},
		enabled: $cloudStore.connection.isConnected,
		context: 'pods-page'
	});
	
	// Attach unified keyboard handler
	function handleKeydown(event: KeyboardEvent) {
		keyboard.handleKeydown(event);
	}
	
	function handleRefresh() {
		refreshResources();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="p-6 space-y-6">
	{#if $cloudStore.connection.isConnected}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Pods</h1>
				<p class="text-muted-foreground">
					Namespace: {$cloudStore.selectedNamespace || 'All'}
				</p>
			</div>
			<Button onclick={handleRefresh} variant="outline">
				Refresh
			</Button>
		</div>
		
		<PodsStatistics pods={$cloudStore.resources[ResourceType.POD]} />
		
		<Card class="p-4">
			<PodsFilters
				{searchQuery}
				{statusFilter}
				onSearchChange={(q) => searchQuery = q}
				onStatusFilterChange={(s) => statusFilter = s}
				onClear={() => { searchQuery = ''; statusFilter = ''; }}
			/>
		</Card>
		
		<Card class="p-4">
			<div class="k8s-navigable-table" data-selected-index={keyboard.tableNav?.selectedIndex ?? -1}>
				<PodsTable
					pods={filteredPods}
					emptyMessage={searchQuery || statusFilter ? 'No pods match your filters' : 'No pods found'}
				/>
			</div>
		</Card>
		
		<!-- Keyboard Shortcuts Panel (optional, can be toggled) -->
		<KeyboardShortcutsPanel
			shortcuts={keyboard.getShortcuts()}
			variant="panel"
			collapsible={true}
			showTitle={true}
		/>
	{/if}
</div>

