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
	let clusterLoadError = $state<string | null>(null);
	let isLoadingClusters = $state(false);
	
	onMount(async () => {
		try {
			// #region agent log
			fetch('http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2',{method:'POST',headers:{'Content-Type':'application/json','X-Debug-Session-Id':'7cbddc'},body:JSON.stringify({sessionId:'7cbddc',runId:'pre-fix',hypothesisId:'H1',location:'CloudConnectionGuard.svelte:21',message:'onMount start',data:{},timestamp:Date.now()})}).catch(()=>{});
			// #endregion agent log
			// Initialize GCP provider
			await initializeProvider(CloudProviderType.GCP);
			// #region agent log
			fetch('http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2',{method:'POST',headers:{'Content-Type':'application/json','X-Debug-Session-Id':'7cbddc'},body:JSON.stringify({sessionId:'7cbddc',runId:'pre-fix',hypothesisId:'H1',location:'CloudConnectionGuard.svelte:24',message:'after initializeProvider',data:{},timestamp:Date.now()})}).catch(()=>{});
			// #endregion agent log
			await loadClustersList();
			// #region agent log
			fetch('http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2',{method:'POST',headers:{'Content-Type':'application/json','X-Debug-Session-Id':'7cbddc'},body:JSON.stringify({sessionId:'7cbddc',runId:'pre-fix',hypothesisId:'H1',location:'CloudConnectionGuard.svelte:25',message:'after loadClustersList',data:{clusterCount:clusters.length},timestamp:Date.now()})}).catch(()=>{});
			// #endregion agent log
			
			// If already connected, we're done
			if ($cloudStore.connection.isConnected && $cloudStore.currentCluster) {
				isInitializing = false;
				return;
			}
			
			// Attempt auto-connect
			if (clusters.length > 0 && !$cloudStore.connection.isConnecting) {
				// #region agent log
				fetch('http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2',{method:'POST',headers:{'Content-Type':'application/json','X-Debug-Session-Id':'7cbddc'},body:JSON.stringify({sessionId:'7cbddc',runId:'pre-fix',hypothesisId:'H4',location:'CloudConnectionGuard.svelte:43',message:'before attemptAutoConnect',data:{clusterCount:clusters.length},timestamp:Date.now()})}).catch(()=>{});
				// #endregion agent log
				await attemptAutoConnect();
				// #region agent log
				fetch('http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2',{method:'POST',headers:{'Content-Type':'application/json','X-Debug-Session-Id':'7cbddc'},body:JSON.stringify({sessionId:'7cbddc',runId:'pre-fix',hypothesisId:'H4',location:'CloudConnectionGuard.svelte:45',message:'after attemptAutoConnect',data:{},timestamp:Date.now()})}).catch(()=>{});
				// #endregion agent log
			}
		} catch (error) {
			console.error('Failed to initialize cloud connection:', error);
			autoConnectError = error instanceof Error ? error.message : 'Failed to initialize';
			// #region agent log
			fetch('http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2',{method:'POST',headers:{'Content-Type':'application/json','X-Debug-Session-Id':'7cbddc'},body:JSON.stringify({sessionId:'7cbddc',runId:'pre-fix',hypothesisId:'H1',location:'CloudConnectionGuard.svelte:37',message:'onMount error',data:{error:String(error)},timestamp:Date.now()})}).catch(()=>{});
			// #endregion agent log
		} finally {
			// #region agent log
			fetch('http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2',{method:'POST',headers:{'Content-Type':'application/json','X-Debug-Session-Id':'7cbddc'},body:JSON.stringify({sessionId:'7cbddc',runId:'pre-fix',hypothesisId:'H1',location:'CloudConnectionGuard.svelte:40',message:'onMount finally',data:{},timestamp:Date.now()})}).catch(()=>{});
			// #endregion agent log
			isInitializing = false;
			hasAttemptedAutoConnect = true;
		}
	});
	
	async function loadClustersList() {
		isLoadingClusters = true;
		clusterLoadError = null;
		try {
		clusters = await loadClusters(CloudProviderType.GCP);
			// If we got an empty array, it might mean kubeconfig isn't configured
			if (clusters.length === 0) {
				// Check if we're in Tauri environment - if so, kubeconfig might not be set up
				const { isTauriEnvironment } = await import('@/lib/utils/tauri');
				if (await isTauriEnvironment()) {
					clusterLoadError = 'No clusters found. Please ensure your kubeconfig is properly configured at ~/.kube/config or set KUBECONFIG environment variable.';
				} else {
					clusterLoadError = 'No clusters found. Kubernetes commands are only available in the desktop app.';
				}
			}
		} catch (error) {
			clusterLoadError = error instanceof Error ? error.message : 'Failed to load clusters';
			console.error('Failed to load clusters:', error);
			clusters = []; // Ensure clusters is set even on error
		} finally {
			isLoadingClusters = false;
		}
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
				{:else if clusterLoadError}
					<div class="rounded-md bg-destructive/10 border border-destructive/20 p-3">
						<p class="text-sm text-destructive font-medium">Failed to Load Clusters</p>
						<p class="text-xs text-muted-foreground mt-1">{clusterLoadError}</p>
					</div>
				{:else}
					<p class="text-sm text-muted-foreground">
						Please connect to a cluster to view cloud resources.
					</p>
				{/if}
				
				{#if clusters.length === 0}
					<div class="space-y-3">
						{#if !clusterLoadError}
						<p class="text-sm text-muted-foreground">
							No Kubernetes clusters found. Make sure your kubeconfig is configured.
						</p>
						{/if}
						<Button 
							variant="outline" 
							class="w-full" 
							onclick={handleRetry}
							disabled={isLoadingClusters}
						>
							{#if isLoadingClusters}
								Loading...
							{:else}
							Retry
							{/if}
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

