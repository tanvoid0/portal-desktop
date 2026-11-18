<!-- CloudConnectionGuard - Middleware-like component that ensures cluster connection -->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';
  import { cloudStore, loadClusters, initializeProvider, connectToCluster } from '../stores';
  import { CloudProviderType, type ICluster } from '../core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Loading from '$lib/components/ui/loading.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { isTauriEnvironment } from '$lib/utils/tauri';
  import { toast } from '$lib/utils/toast';
  import KubeconfigSetupPanel from './KubeconfigSetupPanel.svelte';

  let { children }: { children?: Snippet<[]> } = $props();

  let isInitializing = $state(true);
  let clusters = $state<ICluster[]>([]);
  let hasAttemptedAutoConnect = $state(false);
  let autoConnectError = $state<string | null>(null);
  let clusterLoadError = $state<string | null>(null);
  let isLoadingClusters = $state(false);
  let setupPanel = $state<KubeconfigSetupPanel | null>(null);
  const isExpectedNoClustersState = $derived(
    !!clusterLoadError && clusterLoadError.startsWith('No clusters found')
  );
  const canOfferAutoSetup = $derived(isExpectedNoClustersState && isTauriEnvironment());

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
    isLoadingClusters = true;
    clusterLoadError = null;
    try {
      clusters = await loadClusters(CloudProviderType.GCP);
      // If we got an empty array, it might mean kubeconfig isn't configured
      if (clusters.length === 0) {
        if (isTauriEnvironment()) {
          clusterLoadError =
            'No clusters found. Add or export a kubeconfig, then retry. Expected locations: ~/.kube/config (macOS/Linux) or %USERPROFILE%\\.kube\\config (Windows), or set KUBECONFIG.';
        } else {
          clusterLoadError =
            'No clusters found. Kubernetes commands are only available in the desktop app.';
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
      if (previousCluster && clusters.some((c) => c.id === previousCluster.id)) {
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

  let isRetrying = $state(false);

  async function handleRetry() {
    isRetrying = true;
    autoConnectError = null;
    try {
      await loadClustersList();
      if (canOfferAutoSetup) {
        await setupPanel?.reload();
      }
      await attemptAutoConnect();
      const error = autoConnectError ?? (isExpectedNoClustersState ? null : clusterLoadError);
      if (error) toast.error('Retry failed', { description: error });
      else if (clusters.length === 0) toast.info('No clusters found');
      else toast.success(`Found ${clusters.length} cluster(s)`);
    } finally {
      isRetrying = false;
    }
  }

  const showConnectionUI = $derived(
    !isInitializing && !$cloudStore.connection.isConnected && hasAttemptedAutoConnect
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
  <div class="flex min-h-[400px] items-center justify-center">
    <Loading size="lg" text="Connecting to cluster..." />
  </div>
{:else if showConnectionUI}
  <!-- Connection Required UI -->
  <div class="flex min-h-[400px] items-center justify-center p-6">
    <Card class="w-full max-w-md">
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <span>Cluster Connection Required</span>
        </CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        {#if autoConnectError}
          <div class="rounded-md border border-destructive/20 bg-destructive/10 p-3">
            <p class="text-sm font-medium text-destructive">Connection Failed</p>
            <p class="mt-1 text-xs text-muted-foreground">{autoConnectError}</p>
          </div>
        {:else if clusterLoadError}
          <div
            class={`rounded-md border p-3 ${
              isExpectedNoClustersState
                ? 'border-primary/20 bg-primary/5'
                : 'border-destructive/20 bg-destructive/10'
            }`}
          >
            <p
              class={`text-sm font-medium ${
                isExpectedNoClustersState ? 'text-foreground' : 'text-destructive'
              }`}
            >
              {isExpectedNoClustersState
                ? 'No Kubernetes clusters configured yet'
                : 'Failed to Load Clusters'}
            </p>
            <p class="mt-1 text-xs text-muted-foreground">{clusterLoadError}</p>
            {#if isExpectedNoClustersState}
              <ul class="mt-3 list-inside list-disc space-y-1 text-xs text-muted-foreground">
                <li>Create a kubeconfig using your cluster provider CLI.</li>
                <li>Save it to the default kubeconfig location, or set KUBECONFIG.</li>
                <li>Then click Retry below.</li>
              </ul>
            {/if}
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

            {#if canOfferAutoSetup}
              <KubeconfigSetupPanel
                bind:this={setupPanel}
                onGenerated={async () => {
                  await loadClustersList();
                  await attemptAutoConnect();
                }}
              />
            {/if}

            <Button
              variant="outline"
              class="w-full"
              onclick={handleRetry}
              disabled={isLoadingClusters || isRetrying}
            >
              {#if isLoadingClusters || isRetrying}
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
  {#if children}
    {@render children()}
  {/if}
{/if}
