<!-- CloudConnectionGuard - Middleware-like component that ensures cluster connection -->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import { cloudStore, loadClusters, initializeProvider, connectToCluster } from '../stores';
	import { CloudProviderType, type ICluster } from '../core/types';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import Loading from '@/lib/components/ui/loading.svelte';
	import { Badge } from '@/lib/components/ui/badge';
	
	let { children }: { children: Snippet<[]> } = $props();
	
	let isInitializing = $state(true);
	let clusters = $state<ICluster[]>([]);
	let hasAttemptedAutoConnect = $state(false);
	let autoConnectError = $state<string | null>(null);
	
	onMount(async () => {
		try {
			// Initialize GCP provider
			await initializeProvider(CloudProviderType.GCP);
			await loadClustersList();
			
			// If already connected, we're done
			if ($cloudStore.connection.isConnected && $cloudStore.currentCluster) {
				isInitializing = false;
				return;
			}
			
			// Attempt auto-connect
			if (clusters.length > 0 && !$cloudStore.connection.isConnecting) {
				await attemptAutoConnect();
			}
		} catch (error) {
			console.error('Failed to initialize cloud connection:', error);
			autoConnectError = error instanceof Error ? error.message : 'Failed to initialize';
		} finally {
			isInitializing = false;
			hasAttemptedAutoConnect = true;
		}
	});
	
	async function loadClustersList() {
		clusters = await loadClusters(CloudProviderType.GCP);
	}
	
	async function attemptAutoConnect() {
		try {
			// Try to reconnect to previously connected cluster first
			const previousCluster = $cloudStore.currentCluster;
			if (previousCluster && clusters.some(c => c.id === previousCluster.id)) {
				await connectToCluster(CloudProviderType.GCP, previousCluster.id);
				return;
			}
			
			// Fall back to first available cluster
			if (clusters.length > 0) {
				await connectToCluster(CloudProviderType.GCP, clusters[0].id);
			}
		} catch (error) {
			console.warn('Auto-connect failed:', error);
			autoConnectError = error instanceof Error ? error.message : 'Auto-connect failed';
		}
	}
	
	async function handleConnect(clusterId: string) {
		try {
			autoConnectError = null;
			await connectToCluster(CloudProviderType.GCP, clusterId);
		} catch (error) {
			autoConnectError = error instanceof Error ? error.message : 'Failed to connect';
		}
	}
	
	async function handleRetry() {
		await loadClustersList();
		await attemptAutoConnect();
	}
	
	const showConnectionUI = $derived(
		!isInitializing && 
		!$cloudStore.connection.isConnected && 
		hasAttemptedAutoConnect
	);
	
	// Watch for connection state changes after mount
	$effect(() => {
		// If connection becomes available, refresh clusters list
		if ($cloudStore.connection.isConnected && clusters.length === 0) {
			loadClustersList();
		}
	});
</script>

{#if isInitializing}
	<div class="flex items-center justify-center min-h-[400px]">
		<Loading size="lg" text="Connecting to cluster..." />
	</div>
{:else if showConnectionUI}
	<!-- Connection Required UI -->
	<div class="flex items-center justify-center min-h-[400px] p-6">
		<Card class="max-w-md w-full">
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<span>Cluster Connection Required</span>
				</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				{#if autoConnectError}
					<div class="rounded-md bg-destructive/10 border border-destructive/20 p-3">
						<p class="text-sm text-destructive font-medium">Connection Failed</p>
						<p class="text-xs text-muted-foreground mt-1">{autoConnectError}</p>
					</div>
				{:else}
					<p class="text-sm text-muted-foreground">
						Please connect to a cluster to view cloud resources.
					</p>
				{/if}
				
				{#if clusters.length === 0}
					<div class="space-y-3">
						<p class="text-sm text-muted-foreground">
							No Kubernetes clusters found. Make sure your kubeconfig is configured.
						</p>
						<Button variant="outline" class="w-full" onclick={handleRetry}>
							Retry
						</Button>
					</div>
				{:else}
					<div class="space-y-3">
						<p class="text-sm font-medium">Available Clusters:</p>
						<div class="space-y-2">
							{#each clusters as cluster (cluster.id)}
								{@const clusterTyped = cluster as ICluster}
								<Button
									variant="outline"
									class="w-full justify-between"
									onclick={() => handleConnect(clusterTyped.id)}
									disabled={$cloudStore.connection.isConnecting}
								>
									<div class="flex items-center gap-2">
										<span>{clusterTyped.name}</span>
										{#if clusterTyped.status === 'connected'}
											<Badge variant="default" class="bg-green-500">Connected</Badge>
										{/if}
									</div>
									{#if $cloudStore.connection.isConnecting && $cloudStore.currentCluster?.id === clusterTyped.id}
										<span class="text-xs">Connecting...</span>
									{/if}
								</Button>
							{/each}
						</div>
						{#if autoConnectError}
							<Button variant="outline" class="w-full" onclick={handleRetry}>
								Retry Auto-Connect
							</Button>
						{/if}
					</div>
				{/if}
			</CardContent>
		</Card>
	</div>
{:else}
	<!-- Render children when connected -->
	{@render children()}
{/if}

