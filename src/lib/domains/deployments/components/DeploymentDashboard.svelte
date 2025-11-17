<!--
	Deployment Dashboard - Main interface for managing deployments
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { deploymentActions, deployments, deploymentStats, isLoadingDeployments, deploymentError, containers } from '../stores/deploymentStore';
	import { deploymentService } from '../services/deploymentService';
	import { logger } from '$lib/domains/shared';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import DeploymentCard from './DeploymentCard.svelte';
	import ContainerCard from './ContainerCard.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import Select from '$lib/components/ui/select.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { Plus, Search, Filter, Rocket, Container as ContainerIcon, Play, Square, AlertCircle, Loader2, RefreshCw, Box } from '@lucide/svelte';
	import type { DeploymentStatus } from '../types';

	type TabValue = 'deployments' | 'containers';

	let activeTab = $state<TabValue>('deployments');
	let searchQuery = $state('');
	let selectedStatus = $state<DeploymentStatus | null>(null);
	let containerSearchQuery = $state('');
	let isLoadingContainers = $state(false);

	// Reactive stores
	let deploymentList = $derived($deployments);
	let stats = $derived($deploymentStats);
	let loading = $derived($isLoadingDeployments);
	let errorMessage = $derived($deploymentError);

	// Reactive stores
	let containerList = $derived($containers);

	onMount(async () => {
		await loadDeployments();
		await loadContainers();
	});

	async function loadDeployments() {
		try {
			await deploymentActions.loadDeployments();
		} catch (err) {
			logger.error('Failed to load deployments', { context: 'DeploymentDashboard', 
				error: err
			});
			toast.error('Failed to load deployments');
		}
	}

	async function loadContainers() {
		isLoadingContainers = true;
		try {
			await deploymentActions.loadContainers();
		} catch (err) {
			logger.error('Failed to load containers', { context: 'DeploymentDashboard', 
				error: err
			});
			toast.error('Failed to load containers');
		} finally {
			isLoadingContainers = false;
		}
	}

	function handleSearch() {
		// Filter logic would be implemented here
		// For now, we'll just log the search
		logger.info('Searching deployments', { context: 'DeploymentDashboard', query: searchQuery });
	}

	function handleStatusFilter(status: DeploymentStatus | null) {
		selectedStatus = status;
		// Filter logic would be implemented here
		logger.info('Filtering by status', { context: 'DeploymentDashboard', status });
	}

	function handleCreateDeployment() {
		goto('/deployments/new');
	}

	async function handleRefresh() {
		if (activeTab === 'deployments') {
			try {
				await deploymentActions.refreshDeploymentStatuses();
				toast.success('Deployment statuses refreshed');
			} catch (err) {
				logger.error('Failed to refresh deployment statuses', { context: 'DeploymentDashboard', 
					error: err
				});
				toast.error('Failed to refresh deployment statuses');
			}
		} else {
			await loadContainers();
		}
	}

	async function handleContainerStart(containerId: string) {
		try {
			await deploymentService.startContainer(containerId);
			toast.success('Container started');
			await loadContainers();
		} catch (err) {
			logger.error('Failed to start container', { context: 'DeploymentDashboard', error: err });
			toast.error(err instanceof Error ? err.message : 'Failed to start container');
		}
	}

	async function handleContainerStop(containerId: string) {
		try {
			await deploymentService.stopContainer(containerId);
			toast.success('Container stopped');
			await loadContainers();
		} catch (err) {
			logger.error('Failed to stop container', { context: 'DeploymentDashboard', error: err });
			toast.error(err instanceof Error ? err.message : 'Failed to stop container');
		}
	}

	async function handleContainerRemove(containerId: string) {
		if (!confirm('Are you sure you want to remove this container? This action cannot be undone.')) {
			return;
		}
		try {
			await deploymentService.removeContainer(containerId);
			toast.success('Container removed');
			await loadContainers();
		} catch (err) {
			logger.error('Failed to remove container', { context: 'DeploymentDashboard', error: err });
			toast.error(err instanceof Error ? err.message : 'Failed to remove container');
		}
	}

	function getContainerStats() {
		const containersList = containerList;
		const running = containersList.filter((c: any) => {
			const status = c.status?.toLowerCase() || '';
			return status.includes('running') || status.includes('up');
		}).length;
		return {
			total: containersList.length,
			running,
			stopped: containersList.length - running
		};
	}

	function getStatusColor(status: DeploymentStatus): string {
		return deploymentService.getStatusColor(status);
	}

	function getStatusIcon(status: DeploymentStatus): string {
		return deploymentService.getStatusIcon(status);
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight flex items-center gap-2">
				<ContainerIcon class="h-8 w-8" />
				Deployments & Containers
			</h1>
			<p class="text-muted-foreground">
				Manage Docker containers and CLI command deployments
			</p>
		</div>
		<div class="flex gap-2">
			<Button variant="outline" onclick={handleRefresh} disabled={loading || isLoadingContainers}>
				{#if loading || isLoadingContainers}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
				{:else}
					<RefreshCw class="h-4 w-4 mr-2" />
				{/if}
				Refresh
			</Button>
			{#if activeTab === 'deployments'}
				<Button onclick={handleCreateDeployment}>
					<Plus class="h-4 w-4 mr-2" />
					New Deployment
				</Button>
			{/if}
		</div>
	</div>

	<!-- Error Alert -->
	{#if errorMessage}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertTitle>Error</AlertTitle>
			<AlertDescription>
				{errorMessage}
			</AlertDescription>
		</Alert>
	{/if}

	<!-- Tabs -->
	<Tabs bind:value={activeTab} class="w-full">
		<TabsList class="grid w-full grid-cols-2">
			<TabsTrigger value="deployments">
				<Rocket class="h-4 w-4 mr-2" />
				Deployments
			</TabsTrigger>
			<TabsTrigger value="containers">
				<Box class="h-4 w-4 mr-2" />
				Containers
			</TabsTrigger>
		</TabsList>

		<!-- Deployments Tab -->
		<TabsContent value="deployments" class="space-y-6 mt-6">
			<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-4">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Total Deployments</CardTitle>
				<ContainerIcon class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{stats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Running</CardTitle>
				<Badge variant="default" class="bg-green-100 text-green-800">
					<Play class="h-3 w-3 mr-1" />
					{stats.running}
				</Badge>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600">{stats.running}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Stopped</CardTitle>
				<Badge variant="outline">
					<Square class="h-3 w-3 mr-1" />
					{stats.stopped}
				</Badge>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-gray-600">{stats.stopped}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Building</CardTitle>
				<Badge variant="outline" class="bg-yellow-100 text-yellow-800">
					🔨 {stats.building}
				</Badge>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-600">{stats.building}</div>
			</CardContent>
		</Card>
	</div>

	<!-- Filters -->
	<div class="flex flex-col sm:flex-row gap-4">
		<div class="flex-1">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
				<Input
					placeholder="Search deployments..."
					bind:value={searchQuery}
					oninput={handleSearch}
					class="pl-10"
				/>
			</div>
		</div>
		<Select 
			options={[
				{ value: '', label: 'All Statuses' },
				{ value: 'Running', label: 'Running' },
				{ value: 'Stopped', label: 'Stopped' },
				{ value: 'Building', label: 'Building' },
				{ value: 'Error', label: 'Error' }
			]}
			defaultValue={selectedStatus || ''}
			placeholder="Filter by status"
			onSelect={(value) => handleStatusFilter(value ? value as DeploymentStatus : null)}
			class="w-[200px]"
		/>
	</div>

	<!-- Loading State -->
	{#if loading}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each Array(6) as _}
				<Card>
					<CardHeader>
						<div class="h-4 bg-muted animate-pulse rounded"></div>
						<div class="h-3 bg-muted animate-pulse rounded w-2/3"></div>
					</CardHeader>
					<CardContent>
						<div class="space-y-2">
							<div class="h-3 bg-muted animate-pulse rounded"></div>
							<div class="h-3 bg-muted animate-pulse rounded w-1/2"></div>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}

	<!-- Deployments Grid -->
	{#if !loading && deploymentList.length > 0}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each deploymentList as deployment}
				<DeploymentCard 
					{deployment}
					onStart={() => deploymentActions.startDeployment(deployment.id)}
					onStop={() => deploymentActions.stopDeployment(deployment.id)}
					onDelete={() => deploymentActions.deleteDeployment(deployment.id)}
				/>
			{/each}
		</div>
	{/if}

	<!-- Empty State -->
	{#if !loading && deploymentList.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<ContainerIcon class="h-12 w-12 text-muted-foreground mb-4" />
				<h3 class="text-lg font-semibold mb-2">No Deployments Found</h3>
				<p class="text-muted-foreground text-center mb-4">
					{searchQuery || selectedStatus ? 'Try adjusting your filters' : 'Create your first deployment to get started'}
				</p>
				{#if !searchQuery && !selectedStatus}
					<Button onclick={handleCreateDeployment}>
						<Plus class="h-4 w-4 mr-2" />
						New Deployment
					</Button>
				{/if}
			</CardContent>
		</Card>
		{/if}
		</TabsContent>

		<!-- Containers Tab -->
		<TabsContent value="containers" class="space-y-6 mt-6">
			<!-- Container Stats -->
			{@const containerStats = getContainerStats()}
			<div class="grid gap-4 md:grid-cols-3">
				<Card>
					<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle class="text-sm font-medium">Total Containers</CardTitle>
						<ContainerIcon class="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold">{containerStats.total}</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle class="text-sm font-medium">Running</CardTitle>
						<Badge variant="default" class="bg-green-100 text-green-800">
							<Play class="h-3 w-3 mr-1" />
							{containerStats.running}
						</Badge>
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold text-green-600">{containerStats.running}</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle class="text-sm font-medium">Stopped</CardTitle>
						<Badge variant="outline">
							<Square class="h-3 w-3 mr-1" />
							{containerStats.stopped}
						</Badge>
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold text-gray-600">{containerStats.stopped}</div>
					</CardContent>
				</Card>
			</div>

			<!-- Container Search -->
			<div class="flex flex-col sm:flex-row gap-4">
				<div class="flex-1">
					<div class="relative">
						<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
						<Input
							placeholder="Search containers..."
							bind:value={containerSearchQuery}
							class="pl-10"
						/>
					</div>
				</div>
			</div>

			<!-- Loading State -->
			{#if isLoadingContainers}
				<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
					{#each Array(6) as _}
						<Card>
							<CardHeader>
								<div class="h-4 bg-muted animate-pulse rounded"></div>
								<div class="h-3 bg-muted animate-pulse rounded w-2/3"></div>
							</CardHeader>
							<CardContent>
								<div class="space-y-2">
									<div class="h-3 bg-muted animate-pulse rounded"></div>
									<div class="h-3 bg-muted animate-pulse rounded w-1/2"></div>
								</div>
							</CardContent>
						</Card>
					{/each}
				</div>
			{/if}

			<!-- Containers Grid -->
			{#if !isLoadingContainers && containerList.length > 0}
				<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
					{#each containerList.filter(c => {
						if (!containerSearchQuery) return true;
						const query = containerSearchQuery.toLowerCase();
						return c.name?.toLowerCase().includes(query) || 
						       c.image?.toLowerCase().includes(query) ||
						       c.id?.toLowerCase().includes(query);
					}) as container}
						<ContainerCard 
							{container}
							onStart={handleContainerStart}
							onStop={handleContainerStop}
							onRemove={handleContainerRemove}
						/>
					{/each}
				</div>
			{/if}

			<!-- Empty State -->
			{#if !isLoadingContainers && containerList.length === 0}
				<Card>
					<CardContent class="flex flex-col items-center justify-center py-12">
						<ContainerIcon class="h-12 w-12 text-muted-foreground mb-4" />
						<h3 class="text-lg font-semibold mb-2">No Containers Found</h3>
						<p class="text-muted-foreground text-center mb-4">
							{containerSearchQuery ? 'Try adjusting your search' : 'No Docker containers are running or stopped'}
						</p>
					</CardContent>
				</Card>
			{/if}
		</TabsContent>
	</Tabs>
</div>
