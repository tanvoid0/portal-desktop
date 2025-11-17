<!-- CronJobs List Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import BaseResourceTable from '$lib/domains/cloud/core/components/BaseResourceTable.svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import Select from '@/lib/components/ui/select.svelte';
	import { RefreshCw, Search } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { useTableNavigation, useResourceActions } from '$lib/domains/k8s-navigation';
	
	let searchQuery = $state('');
	let suspendFilter = $state('');
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.CRONJOB);
		}
	});
	
	const filteredCronJobs = $derived(
		$cloudStore.resources[ResourceType.CRONJOB].filter(cj => {
			const matchesSearch = !searchQuery || cj.name.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesSuspend = !suspendFilter || 
				(suspendFilter === 'active' && !cj.metadata?.suspend) ||
				(suspendFilter === 'suspended' && cj.metadata?.suspend);
			return matchesSearch && matchesSuspend;
		})
	);
	
	const cronJobStats = $derived({
		total: $cloudStore.resources[ResourceType.CRONJOB].length,
		active: $cloudStore.resources[ResourceType.CRONJOB].filter((cj: any) => !cj.metadata?.suspend).length,
		suspended: $cloudStore.resources[ResourceType.CRONJOB].filter((cj: any) => cj.metadata?.suspend).length,
		withActiveJobs: $cloudStore.resources[ResourceType.CRONJOB].filter((cj: any) => (cj.metadata?.active || 0) > 0).length
	});
	
	const cronJobColumns = [
		{ key: 'name', label: 'Name', width: 'w-1/4' },
		{ key: 'schedule', label: 'Schedule', width: 'w-1/6' },
		{ key: 'suspend', label: 'Suspend', width: 'w-1/8' },
		{ key: 'active', label: 'Active', width: 'w-1/8' },
		{ key: 'lastSchedule', label: 'Last Schedule', width: 'w-1/6' },
		{ key: 'age', label: 'Age', width: 'w-1/8' },
		{ key: 'namespace', label: 'Namespace', width: 'w-1/6' }
	];
	
	async function handleRefresh() {
		await refreshResources();
	}
	
	function handleCronJobClick(cronJob: ICloudResource) {
		goto(`/cloud/workloads/cronjobs/${cronJob.name}?namespace=${cronJob.namespace}`);
	}
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: filteredCronJobs.length,
		onSelect: () => {},
		onActivate: (index) => {
			const cronJob = filteredCronJobs[index];
			if (cronJob) {
				goto(`/cloud/workloads/cronjobs/${cronJob.name}?namespace=${cronJob.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: filteredCronJobs,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/workloads/cronjobs/${resource.name}?namespace=${resource.namespace}`);
			},
			onEdit: (resource) => {
				goto(`/cloud/workloads/cronjobs/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onYaml: (resource) => {
				goto(`/cloud/workloads/cronjobs/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
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
			<h1 class="text-2xl font-bold">CronJobs</h1>
			<p class="text-muted-foreground">Manage Kubernetes CronJobs</p>
		</div>
		<Button variant="outline" onclick={handleRefresh}>
			<RefreshCw class="mr-2 h-4 w-4" />
			Refresh
		</Button>
	</div>
	
	<!-- Statistics -->
	<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Total CronJobs</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{cronJobStats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Active</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600">{cronJobStats.active}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Suspended</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-600">{cronJobStats.suspended}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">With Active Jobs</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">{cronJobStats.withActiveJobs}</div>
			</CardContent>
		</Card>
	</div>
	
	<!-- Filters -->
	<div class="flex items-center gap-4">
		<div class="flex-1 relative">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
			<Input
				type="text"
				placeholder="Search CronJobs..."
				value={searchQuery}
				oninput={(e) => searchQuery = (e.target as HTMLInputElement).value}
				class="pl-10"
			/>
		</div>
		<Select
			options={[
				{ value: '', label: 'All Statuses' },
				{ value: 'active', label: 'Active' },
				{ value: 'suspended', label: 'Suspended' }
			]}
			bind:value={suspendFilter}
			placeholder="Filter by status"
			class="w-48"
		/>
	</div>
	
	<!-- CronJobs Table -->
	<Card>
		<CardHeader>
			<CardTitle>CronJobs ({filteredCronJobs.length})</CardTitle>
		</CardHeader>
		<CardContent>
			{#if filteredCronJobs.length === 0}
				<div class="text-center py-8 text-muted-foreground">
					<p>No CronJobs found</p>
					{#if searchQuery || suspendFilter}
						<p class="text-xs mt-2">Try adjusting your filters</p>
					{/if}
				</div>
			{:else}
				<div class="k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
					<BaseResourceTable
						resources={filteredCronJobs}
						resourceType={ResourceType.CRONJOB}
						columns={cronJobColumns}
						onResourceClick={handleCronJobClick}
					/>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

