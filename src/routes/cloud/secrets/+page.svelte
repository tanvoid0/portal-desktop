<!-- Secrets List Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import BaseResourceTable from '$lib/domains/cloud/core/components/BaseResourceTable.svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { RefreshCw, Search, Plus, Eye, EyeOff } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { useTableNavigation, useResourceActions } from '$lib/domains/k8s-navigation';
	
	let searchQuery = $state('');
	let showSecrets = $state(false); // Toggle to show/hide secret values
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.SECRET);
		}
	});
	
	const filteredSecrets = $derived(
		$cloudStore.resources[ResourceType.SECRET].filter(secret => {
			const matchesSearch = !searchQuery || secret.name.toLowerCase().includes(searchQuery.toLowerCase());
			return matchesSearch;
		})
	);
	
	const secretStats = $derived({
		total: $cloudStore.resources[ResourceType.SECRET].length,
		withData: $cloudStore.resources[ResourceType.SECRET].filter((s: any) => (s.metadata?.dataCount || 0) > 0).length
	});
	
	const secretColumns = [
		{ key: 'name', label: 'Name', width: 'w-1/4' },
		{ key: 'type', label: 'Type', width: 'w-1/8' },
		{ key: 'dataCount', label: 'Data Keys', width: 'w-1/8' },
		{ key: 'age', label: 'Age', width: 'w-1/8' },
		{ key: 'namespace', label: 'Namespace', width: 'w-1/6' }
	];
	
	async function handleRefresh() {
		await refreshResources();
	}
	
	function handleSecretClick(secret: ICloudResource) {
		goto(`/cloud/secrets/${secret.name}?namespace=${secret.namespace}`);
	}
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: filteredSecrets.length,
		onSelect: () => {},
		onActivate: (index) => {
			const secret = filteredSecrets[index];
			if (secret) {
				goto(`/cloud/secrets/${secret.name}?namespace=${secret.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: filteredSecrets,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/secrets/${resource.name}?namespace=${resource.namespace}`);
			},
			onEdit: (resource) => {
				goto(`/cloud/secrets/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onYaml: (resource) => {
				goto(`/cloud/secrets/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
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
			<h1 class="text-2xl font-bold">Secrets</h1>
			<p class="text-muted-foreground">Manage Kubernetes Secrets</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={() => showSecrets = !showSecrets}>
				{#if showSecrets}
					<EyeOff class="mr-2 h-4 w-4" />
				{:else}
					<Eye class="mr-2 h-4 w-4" />
				{/if}
				{showSecrets ? 'Hide' : 'Show'} Secrets
			</Button>
			<Button variant="outline" onclick={handleRefresh}>
				<RefreshCw class="mr-2 h-4 w-4" />
				Refresh
			</Button>
			<Button onclick={() => goto('/cloud/secrets/new')}>
				<Plus class="mr-2 h-4 w-4" />
				Create Secret
			</Button>
		</div>
	</div>
	
	<!-- Warning -->
	{#if showSecrets}
		<div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
			<p class="text-sm text-yellow-800 dark:text-yellow-200">
				<strong>Warning:</strong> Secret values are now visible. Be careful when sharing your screen.
			</p>
		</div>
	{/if}
	
	<!-- Statistics -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Total Secrets</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{secretStats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">With Data</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">{secretStats.withData}</div>
			</CardContent>
		</Card>
	</div>
	
	<!-- Search -->
	<div class="flex items-center gap-4">
		<div class="flex-1 relative">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
			<Input
				type="text"
				placeholder="Search Secrets..."
				value={searchQuery}
				oninput={(e) => searchQuery = (e.target as HTMLInputElement).value}
				class="pl-10"
			/>
		</div>
	</div>
	
	<!-- Secrets Table -->
	<Card>
		<CardHeader>
			<CardTitle>Secrets ({filteredSecrets.length})</CardTitle>
		</CardHeader>
		<CardContent>
			{#if filteredSecrets.length === 0}
				<div class="text-center py-8 text-muted-foreground">
					<p>No Secrets found</p>
					{#if searchQuery}
						<p class="text-xs mt-2">Try adjusting your search</p>
					{/if}
				</div>
			{:else}
				<div class="k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
					<BaseResourceTable
						resources={filteredSecrets}
						resourceType={ResourceType.SECRET}
						columns={secretColumns}
						onResourceClick={handleSecretClick}
					/>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

