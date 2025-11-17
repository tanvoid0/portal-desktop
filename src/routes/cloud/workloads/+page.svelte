<!-- Cloud Overview Dashboard -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
	import { ResourceType } from '$lib/domains/cloud/core/types';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { goto } from '$app/navigation';
	import { Badge } from '@/lib/components/ui/badge';
	import Loading from '@/lib/components/ui/loading.svelte';
	import MetricsDisplay from '$lib/domains/cloud/components/MetricsDisplay.svelte';
	import { invoke } from '@tauri-apps/api/core';
	
	let isLoadingData = $state(false);
	let clusterMetrics = $state<Record<string, any> | null>(null);
	let metricsLoading = $state(false);
	
	onMount(async () => {
		if ($cloudStore.connection.isConnected) {
			await loadClusterData();
		}
	});
	
	// Reactive effect to load data when connection state changes
	$effect(() => {
		if ($cloudStore.connection.isConnected && !isLoadingData) {
			loadClusterData();
		}
	});
	
	async function loadClusterData() {
		if (!$cloudStore.connection.isConnected || isLoadingData) return;
		
		isLoadingData = true;
		try {
			await Promise.all([
				loadResources(ResourceType.POD),
				loadResources(ResourceType.SERVICE),
				loadResources(ResourceType.DEPLOYMENT),
				loadResources(ResourceType.STATEFULSET),
				loadResources(ResourceType.DAEMONSET),
				loadResources(ResourceType.JOB),
				loadResources(ResourceType.CRONJOB),
				loadResources(ResourceType.CONFIGMAP),
				loadResources(ResourceType.SECRET),
				loadResources(ResourceType.INGRESS),
				loadResources(ResourceType.NAMESPACE)
			]);
			
			// Load metrics
			await loadMetrics();
		} catch (error) {
			console.error('Failed to load cluster data:', error);
		} finally {
			isLoadingData = false;
		}
	}
	
	async function loadMetrics() {
		if (!$cloudStore.connection.isConnected) return;
		
		try {
			metricsLoading = true;
			const namespace = $cloudStore.selectedNamespace || undefined;
			const metrics = await invoke<Record<string, any>>('k8s_get_all_pods_metrics', {
				namespace: namespace || null
			});
			clusterMetrics = metrics;
		} catch (error) {
			console.error('Failed to load metrics:', error);
		} finally {
			metricsLoading = false;
		}
	}
	
	function formatCPU(millicores: number | null): string {
		if (millicores === null || millicores === undefined) return 'N/A';
		if (millicores >= 1000) {
			return `${(millicores / 1000).toFixed(2)} cores`;
		}
		return `${millicores.toFixed(0)}m`;
	}
	
	function formatMemory(bytes: number | null): string {
		if (bytes === null || bytes === undefined) return 'N/A';
		const gb = bytes / (1024 * 1024 * 1024);
		if (gb >= 1) {
			return `${gb.toFixed(2)} Gi`;
		}
		const mb = bytes / (1024 * 1024);
		if (mb >= 1) {
			return `${mb.toFixed(2)} Mi`;
		}
		const kb = bytes / 1024;
		if (kb >= 1) {
			return `${kb.toFixed(2)} Ki`;
		}
		return `${bytes.toFixed(0)} B`;
	}
	
	// Statistics
	const stats = $derived.by(() => {
		const pods = $cloudStore.resources[ResourceType.POD];
		const services = $cloudStore.resources[ResourceType.SERVICE];
		const deployments = $cloudStore.resources[ResourceType.DEPLOYMENT];
		const statefulsets = $cloudStore.resources[ResourceType.STATEFULSET];
		const daemonsets = $cloudStore.resources[ResourceType.DAEMONSET];
		const jobs = $cloudStore.resources[ResourceType.JOB];
		const cronjobs = $cloudStore.resources[ResourceType.CRONJOB];
		const configmaps = $cloudStore.resources[ResourceType.CONFIGMAP];
		const secrets = $cloudStore.resources[ResourceType.SECRET];
		const ingresses = $cloudStore.resources[ResourceType.INGRESS];
		const namespaces = $cloudStore.resources[ResourceType.NAMESPACE];
		
		return {
			pods: {
				total: pods.length,
				running: pods.filter((p: any) => p.status === 'running').length,
				pending: pods.filter((p: any) => p.status === 'pending').length,
				failed: pods.filter((p: any) => p.status === 'failed').length
			},
			services: {
				total: services.length
			},
			deployments: {
				total: deployments.length,
				running: deployments.filter((d: any) => d.status === 'running').length
			},
			statefulsets: {
				total: statefulsets.length,
				running: statefulsets.filter((ss: any) => ss.status === 'running').length
			},
			daemonsets: {
				total: daemonsets.length,
				running: daemonsets.filter((ds: any) => ds.status === 'running').length
			},
			jobs: {
				total: jobs.length
			},
			cronjobs: {
				total: cronjobs.length
			},
			configmaps: {
				total: configmaps.length
			},
			secrets: {
				total: secrets.length
			},
			ingresses: {
				total: ingresses.length
			},
			namespaces: {
				total: namespaces.length
			}
		};
	});
	
	// Aggregate metrics
	const aggregateMetrics = $derived.by(() => {
		if (!clusterMetrics) return null;
		
		let totalCPU = 0;
		let totalMemory = 0;
		let podCount = 0;
		
		for (const [podName, metrics] of Object.entries(clusterMetrics)) {
			if (!metrics || typeof metrics !== 'object') continue;
			if (metrics.cpu_usage !== null && metrics.cpu_usage !== undefined) {
				totalCPU += metrics.cpu_usage;
			}
			if (metrics.memory_usage !== null && metrics.memory_usage !== undefined) {
				totalMemory += metrics.memory_usage;
			}
			podCount++;
		}
		
		return {
			cpu_usage: podCount > 0 ? totalCPU : null,
			memory_usage: podCount > 0 ? totalMemory : null,
			pod_count: podCount
		};
	});
	
	async function handleRefresh() {
		await refreshResources();
		await loadMetrics();
	}
</script>

<div class="container mx-auto p-6 space-y-6">
	{#if $cloudStore.connection.isConnected && $cloudStore.currentCluster}
		<!-- Connected Cluster View -->
		<div class="space-y-6">
			<!-- Current Cluster Info -->
			<Card>
				<CardHeader>
					<div class="flex items-center justify-between">
						<div>
							<CardTitle>Connected Cluster</CardTitle>
							<p class="text-sm text-muted-foreground mt-1">
								{$cloudStore.currentCluster.name}
							</p>
						</div>
						<Badge variant="default" class="bg-green-500">
							Connected
						</Badge>
					</div>
				</CardHeader>
				<CardContent>
					<div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
						<div>
							<p class="text-muted-foreground">Context</p>
							<p class="font-medium">{$cloudStore.currentCluster.context || 'N/A'}</p>
						</div>
						<div>
							<p class="text-muted-foreground">Namespace</p>
							<p class="font-medium">{$cloudStore.selectedNamespace || 'default'}</p>
						</div>
						<div>
							<p class="text-muted-foreground">Server</p>
							<p class="font-medium truncate">{$cloudStore.currentCluster.server || 'N/A'}</p>
						</div>
						<div>
							<p class="text-muted-foreground">Version</p>
							<p class="font-medium">{$cloudStore.currentCluster.version || 'N/A'}</p>
						</div>
					</div>
				</CardContent>
			</Card>
			
			<!-- Metrics -->
			{#if aggregateMetrics}
				<MetricsDisplay
					metrics={aggregateMetrics}
					title="Cluster Resource Usage"
					showDetails={false}
				/>
			{/if}
			
			<!-- Resource Statistics -->
			<div>
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-xl font-semibold">Resource Statistics</h2>
					<Button variant="outline" onclick={handleRefresh} disabled={isLoadingData}>
						Refresh
					</Button>
				</div>
				
				<div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">Pods</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.pods.total}</div>
							<p class="text-sm text-muted-foreground mt-1">{stats.pods.running} running</p>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">Services</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.services.total}</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">Deployments</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.deployments.total}</div>
							<p class="text-sm text-muted-foreground mt-1">{stats.deployments.running} running</p>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">StatefulSets</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.statefulsets.total}</div>
							<p class="text-sm text-muted-foreground mt-1">{stats.statefulsets.running} running</p>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">DaemonSets</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.daemonsets.total}</div>
							<p class="text-sm text-muted-foreground mt-1">{stats.daemonsets.running} running</p>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">Jobs</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.jobs.total}</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">CronJobs</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.cronjobs.total}</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">ConfigMaps</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.configmaps.total}</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">Secrets</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.secrets.total}</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">Ingress</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.ingresses.total}</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader class="pb-2">
							<CardTitle class="text-sm font-medium">Namespaces</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stats.namespaces.total}</div>
						</CardContent>
					</Card>
				</div>
			</div>
			
			<!-- Quick Actions -->
			<Card>
				<CardHeader>
					<CardTitle>Quick Actions</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/workloads')}>
							<span class="text-lg mr-2">☸️</span>
							View Workloads
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/workloads/pods')}>
							<span class="text-lg mr-2">📦</span>
							View Pods
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/workloads/services')}>
							<span class="text-lg mr-2">🔗</span>
							View Services
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/workloads/deployments')}>
							<span class="text-lg mr-2">🚀</span>
							View Deployments
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/workloads/statefulsets')}>
							<span class="text-lg mr-2">🗄️</span>
							View StatefulSets
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/workloads/daemonsets')}>
							<span class="text-lg mr-2">👹</span>
							View DaemonSets
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/workloads/jobs')}>
							<span class="text-lg mr-2">⚙️</span>
							View Jobs
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/workloads/cronjobs')}>
							<span class="text-lg mr-2">⏰</span>
							View CronJobs
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/configmaps')}>
							<span class="text-lg mr-2">⚙️</span>
							View ConfigMaps
						</Button>
						<Button variant="outline" class="w-full justify-start" onclick={() => goto('/cloud/secrets')}>
							<span class="text-lg mr-2">🔐</span>
							View Secrets
						</Button>
					</div>
				</CardContent>
			</Card>
		</div>
	{:else if isLoadingData}
		<div class="text-center py-12">
			<Loading size="lg" text="Loading cluster data..." />
		</div>
	{:else}
		<Card>
			<CardHeader>
				<CardTitle>Not Connected</CardTitle>
			</CardHeader>
			<CardContent>
				<p class="text-muted-foreground">Please connect to a cluster to view resources.</p>
			</CardContent>
		</Card>
	{/if}
</div>

