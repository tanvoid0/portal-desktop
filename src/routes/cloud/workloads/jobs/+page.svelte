<!-- Jobs List Page -->
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
	let statusFilter = $state('');
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadResources(ResourceType.JOB);
		}
	});
	
	const filteredJobs = $derived(
		$cloudStore.resources[ResourceType.JOB].filter(job => {
			const matchesSearch = !searchQuery || job.name.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesStatus = !statusFilter || job.status === statusFilter;
			return matchesSearch && matchesStatus;
		})
	);
	
	const jobStats = $derived({
		total: $cloudStore.resources[ResourceType.JOB].length,
		completed: $cloudStore.resources[ResourceType.JOB].filter((j: any) => j.status === 'succeeded' || j.status === 'completed').length,
		running: $cloudStore.resources[ResourceType.JOB].filter((j: any) => j.status === 'running').length,
		failed: $cloudStore.resources[ResourceType.JOB].filter((j: any) => j.status === 'failed').length
	});
	
	const statusOptions = $derived(() => {
		const statuses = new Set($cloudStore.resources[ResourceType.JOB].map((j: any) => j.status));
		return Array.from(statuses).sort();
	});
	
	const jobColumns = [
		{ key: 'name', label: 'Name', width: 'w-1/4' },
		{ key: 'status', label: 'Status', width: 'w-1/8' },
		{ key: 'completions', label: 'Completions', width: 'w-1/8' },
		{ key: 'succeeded', label: 'Succeeded', width: 'w-1/8' },
		{ key: 'failed', label: 'Failed', width: 'w-1/8' },
		{ key: 'age', label: 'Age', width: 'w-1/8' },
		{ key: 'namespace', label: 'Namespace', width: 'w-1/6' }
	];
	
	async function handleRefresh() {
		await refreshResources();
	}
	
	function handleJobClick(job: ICloudResource) {
		goto(`/cloud/workloads/jobs/${job.name}?namespace=${job.namespace}`);
	}
	
	// Table navigation
	const tableNav = useTableNavigation({
		totalItems: filteredJobs.length,
		onSelect: () => {},
		onActivate: (index) => {
			const job = filteredJobs[index];
			if (job) {
				goto(`/cloud/workloads/jobs/${job.name}?namespace=${job.namespace}`);
			}
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	// Resource actions
	const resourceActions = useResourceActions({
		selectedIndex: tableNav.selectedIndex,
		resources: filteredJobs,
		handlers: {
			onDescribe: (resource) => {
				goto(`/cloud/workloads/jobs/${resource.name}?namespace=${resource.namespace}`);
			},
			onEdit: (resource) => {
				goto(`/cloud/workloads/jobs/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
			},
			onYaml: (resource) => {
				goto(`/cloud/workloads/jobs/${resource.name}?namespace=${resource.namespace}&tab=yaml`);
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
			<h1 class="text-2xl font-bold">Jobs</h1>
			<p class="text-muted-foreground">Manage Kubernetes Jobs</p>
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
				<CardTitle class="text-sm font-medium">Total Jobs</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{jobStats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Completed</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600">{jobStats.completed}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Running</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">{jobStats.running}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium">Failed</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600">{jobStats.failed}</div>
			</CardContent>
		</Card>
	</div>
	
	<!-- Filters -->
	<div class="flex items-center gap-4">
		<div class="flex-1 relative">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
			<Input
				type="text"
				placeholder="Search jobs..."
				value={searchQuery}
				oninput={(e) => searchQuery = (e.target as HTMLInputElement).value}
				class="pl-10"
			/>
		</div>
		<Select
			options={[
				{ value: '', label: 'All Statuses' },
				...statusOptions().map(s => ({ value: s, label: s }))
			]}
			bind:value={statusFilter}
			placeholder="Filter by status"
			class="w-48"
		/>
	</div>
	
	<!-- Jobs Table -->
	<Card>
		<CardHeader>
			<CardTitle>Jobs ({filteredJobs.length})</CardTitle>
		</CardHeader>
		<CardContent>
			{#if filteredJobs.length === 0}
				<div class="text-center py-8 text-muted-foreground">
					<p>No jobs found</p>
					{#if searchQuery || statusFilter}
						<p class="text-xs mt-2">Try adjusting your filters</p>
					{/if}
				</div>
			{:else}
				<div class="k8s-navigable-table" data-selected-index={tableNav.selectedIndex}>
					<BaseResourceTable
						resources={filteredJobs}
						resourceType={ResourceType.JOB}
						columns={jobColumns}
						onResourceClick={handleJobClick}
					/>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

