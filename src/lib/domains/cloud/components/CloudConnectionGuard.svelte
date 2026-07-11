<!-- CloudConnectionGuard - Middleware-like component that ensures cluster connection -->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import {
    cloudStore,
    loadClusters,
    initializeProvider,
    connectToCluster,
  } from "../stores";
  import { CloudProviderType, type ICluster } from "../core/types";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import Loading from "$lib/components/ui/loading.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { invokeClient } from "$lib/utils/invokeClient";
  import { isTauriEnvironment } from "$lib/utils/tauri";
  import { toast } from "$lib/utils/toast";

  interface KubeSetupToolStatus {
    installed: boolean;
    command: string;
  }

  interface KubeSetupTools {
    kubectl: KubeSetupToolStatus;
    gcloud: KubeSetupToolStatus;
    aws: KubeSetupToolStatus;
    az: KubeSetupToolStatus;
    minikube: KubeSetupToolStatus;
    kind: KubeSetupToolStatus;
  }

  interface KubeSetupTarget {
    id: string;
    label: string;
    provider: string;
    metadata: Record<string, string>;
  }

  interface KubeSetupDetectionResult {
    tools: KubeSetupTools;
    targets: KubeSetupTarget[];
    errors: string[];
  }

  interface GenerateKubeconfigRequest {
    provider: string;
    cluster_name?: string;
    region?: string;
    zone?: string;
    project?: string;
    resource_group?: string;
    profile?: string;
  }

  let { children }: { children?: Snippet<[]> } = $props();

  let isInitializing = $state(true);
  let clusters = $state<ICluster[]>([]);
  let hasAttemptedAutoConnect = $state(false);
  let autoConnectError = $state<string | null>(null);
  let clusterLoadError = $state<string | null>(null);
  let isLoadingClusters = $state(false);
  let setupDetection = $state<KubeSetupDetectionResult | null>(null);
  let isLoadingSetupOptions = $state(false);
  let setupDetectionError = $state<string | null>(null);
  let isGeneratingKubeconfig = $state(false);
  let kubeconfigActionOutput = $state<string | null>(null);
  let awsClusterName = $state("");
  let awsRegion = $state("");
  let gkeClusterName = $state("");
  let gkeLocation = $state("");
  let gkeProject = $state("");
  let aksClusterName = $state("");
  let aksResourceGroup = $state("");
  const isExpectedNoClustersState = $derived(
    !!clusterLoadError && clusterLoadError.startsWith("No clusters found"),
  );
  const canOfferAutoSetup = $derived(
    isExpectedNoClustersState && isTauriEnvironment(),
  );
  const detectedTargets = $derived(setupDetection?.targets ?? []);
  const installedToolLabels = $derived.by(() => {
    if (!setupDetection) return [];

    return Object.entries(setupDetection.tools)
      .filter(([, value]) => value.installed)
      .map(([key]) => key);
  });

  onMount(async () => {
    try {
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H1",
            location: "CloudConnectionGuard.svelte:21",
            message: "onMount start",
            data: {},
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log
      // Initialize GCP provider
      await initializeProvider(CloudProviderType.GCP);
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H1",
            location: "CloudConnectionGuard.svelte:24",
            message: "after initializeProvider",
            data: {},
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log
      await loadClustersList();
      if (canOfferAutoSetup) {
        await loadSetupDetection();
      }
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H1",
            location: "CloudConnectionGuard.svelte:25",
            message: "after loadClustersList",
            data: { clusterCount: clusters.length },
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log

      // If already connected, we're done
      if ($cloudStore.connection.isConnected && $cloudStore.currentCluster) {
        isInitializing = false;
        return;
      }

      // Attempt auto-connect
      if (clusters.length > 0 && !$cloudStore.connection.isConnecting) {
        // #region agent log
        fetch(
          "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
          {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              "X-Debug-Session-Id": "7cbddc",
            },
            body: JSON.stringify({
              sessionId: "7cbddc",
              runId: "pre-fix",
              hypothesisId: "H4",
              location: "CloudConnectionGuard.svelte:43",
              message: "before attemptAutoConnect",
              data: { clusterCount: clusters.length },
              timestamp: Date.now(),
            }),
          },
        ).catch(() => {});
        // #endregion agent log
        await attemptAutoConnect();
        // #region agent log
        fetch(
          "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
          {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              "X-Debug-Session-Id": "7cbddc",
            },
            body: JSON.stringify({
              sessionId: "7cbddc",
              runId: "pre-fix",
              hypothesisId: "H4",
              location: "CloudConnectionGuard.svelte:45",
              message: "after attemptAutoConnect",
              data: {},
              timestamp: Date.now(),
            }),
          },
        ).catch(() => {});
        // #endregion agent log
      }
    } catch (error) {
      console.error("Failed to initialize cloud connection:", error);
      autoConnectError =
        error instanceof Error ? error.message : "Failed to initialize";
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H1",
            location: "CloudConnectionGuard.svelte:37",
            message: "onMount error",
            data: { error: String(error) },
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log
    } finally {
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H1",
            location: "CloudConnectionGuard.svelte:40",
            message: "onMount finally",
            data: {},
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
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
        if (isTauriEnvironment()) {
          clusterLoadError =
            "No clusters found. Add or export a kubeconfig, then retry. Expected locations: ~/.kube/config (macOS/Linux) or %USERPROFILE%\\.kube\\config (Windows), or set KUBECONFIG.";
        } else {
          clusterLoadError =
            "No clusters found. Kubernetes commands are only available in the desktop app.";
        }
      }
    } catch (error) {
      clusterLoadError =
        error instanceof Error ? error.message : "Failed to load clusters";
      console.error("Failed to load clusters:", error);
      clusters = []; // Ensure clusters is set even on error
    } finally {
      isLoadingClusters = false;
    }
  }

  async function loadSetupDetection() {
    if (!isTauriEnvironment()) return;

    isLoadingSetupOptions = true;
    setupDetectionError = null;
    try {
      setupDetection = await invokeClient.post<KubeSetupDetectionResult>(
        "k8s_detect_setup_tools",
        undefined,
        { localhostStrategy: "error" },
      );
    } catch (error) {
      setupDetectionError =
        error instanceof Error
          ? error.message
          : "Failed to detect Kubernetes setup tools";
    } finally {
      isLoadingSetupOptions = false;
    }
  }

  async function generateKubeconfig(request: GenerateKubeconfigRequest) {
    isGeneratingKubeconfig = true;
    kubeconfigActionOutput = null;
    setupDetectionError = null;

    try {
      const result = await invokeClient.post<{
        success: boolean;
        command: string;
        stdout: string;
        stderr: string;
      }>("k8s_generate_kubeconfig", {
        request,
      });

      kubeconfigActionOutput = [result.command, result.stdout, result.stderr]
        .filter(Boolean)
        .join("\n\n");

      toast.success("Kubeconfig generated", {
        description: "Retrying cluster detection.",
      });

      await loadClustersList();
      await attemptAutoConnect();
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to generate kubeconfig";
      kubeconfigActionOutput = message;
      toast.error("Kubeconfig setup failed", { description: message });
    } finally {
      isGeneratingKubeconfig = false;
    }
  }

  async function handleDetectedTargetSetup(target: KubeSetupTarget) {
    await generateKubeconfig({
      provider: target.provider,
      cluster_name: target.metadata.cluster_name,
      profile: target.metadata.profile,
      region: target.metadata.region,
      zone: target.metadata.location,
      project: target.metadata.project,
      resource_group: target.metadata.resource_group,
    });
  }

  async function handleAwsSetup() {
    if (!awsClusterName || !awsRegion) {
      toast.error("AWS setup needs cluster name and region");
      return;
    }

    await generateKubeconfig({
      provider: "aws",
      cluster_name: awsClusterName,
      region: awsRegion,
    });
  }

  async function handleManualGkeSetup() {
    if (!gkeClusterName || !gkeLocation) {
      toast.error("GKE setup needs cluster name and location");
      return;
    }

    const isZone = /[a-z]+-[a-z]+\d-[a-z]$/.test(gkeLocation);
    await generateKubeconfig({
      provider: "gcloud",
      cluster_name: gkeClusterName,
      project: gkeProject || undefined,
      zone: isZone ? gkeLocation : undefined,
      region: isZone ? undefined : gkeLocation,
    });
  }

  async function handleManualAksSetup() {
    if (!aksClusterName || !aksResourceGroup) {
      toast.error("AKS setup needs cluster name and resource group");
      return;
    }

    await generateKubeconfig({
      provider: "az",
      cluster_name: aksClusterName,
      resource_group: aksResourceGroup,
    });
  }

  async function attemptAutoConnect() {
    try {
      // Try to reconnect to previously connected cluster first
      const previousCluster = $cloudStore.currentCluster;
      if (
        previousCluster &&
        clusters.some((c) => c.id === previousCluster.id)
      ) {
        await connectToCluster(CloudProviderType.GCP, previousCluster.id);
        return;
      }

      // Fall back to first available cluster
      if (clusters.length > 0) {
        await connectToCluster(CloudProviderType.GCP, clusters[0].id);
      }
    } catch (error) {
      console.warn("Auto-connect failed:", error);
      autoConnectError =
        error instanceof Error ? error.message : "Auto-connect failed";
    }
  }

  async function handleConnect(clusterId: string) {
    try {
      autoConnectError = null;
      await connectToCluster(CloudProviderType.GCP, clusterId);
    } catch (error) {
      autoConnectError =
        error instanceof Error ? error.message : "Failed to connect";
    }
  }

  async function handleRetry() {
    await loadClustersList();
    if (canOfferAutoSetup) {
      await loadSetupDetection();
    }
    await attemptAutoConnect();
  }

  const showConnectionUI = $derived(
    !isInitializing &&
      !$cloudStore.connection.isConnected &&
      hasAttemptedAutoConnect,
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
          <div
            class="rounded-md border border-destructive/20 bg-destructive/10 p-3"
          >
            <p class="text-sm font-medium text-destructive">
              Connection Failed
            </p>
            <p class="mt-1 text-xs text-muted-foreground">{autoConnectError}</p>
          </div>
        {:else if clusterLoadError}
          <div
            class={`rounded-md border p-3 ${isExpectedNoClustersState
              ? "border-primary/20 bg-primary/5"
              : "border-destructive/20 bg-destructive/10"}`}
          >
            <p
              class={`text-sm font-medium ${isExpectedNoClustersState
                ? "text-foreground"
                : "text-destructive"}`}
            >
              {isExpectedNoClustersState
                ? "No Kubernetes clusters configured yet"
                : "Failed to Load Clusters"}
            </p>
            <p class="mt-1 text-xs text-muted-foreground">{clusterLoadError}</p>
            {#if isExpectedNoClustersState}
              <ul class="mt-3 list-inside list-disc space-y-1 text-xs text-muted-foreground">
                <li>Create a kubeconfig using your cluster provider CLI.</li>
                <li>Save it to the default kubeconfig location, or set KUBECONFIG.</li>
                <li>Reopen this page and click Retry.</li>
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
                No Kubernetes clusters found. Make sure your kubeconfig is
                configured.
              </p>
            {/if}

            {#if canOfferAutoSetup}
              <div class="rounded-md border bg-muted/30 p-3">
                <div class="mb-2 flex items-center justify-between gap-2">
                  <p class="text-sm font-medium">Automatic setup</p>
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={loadSetupDetection}
                    disabled={isLoadingSetupOptions || isGeneratingKubeconfig}
                  >
                    Refresh tools
                  </Button>
                </div>

                {#if isLoadingSetupOptions}
                  <p class="text-xs text-muted-foreground">
                    Detecting installed Kubernetes tools...
                  </p>
                {:else}
                  {#if installedToolLabels.length > 0}
                    <p class="text-xs text-muted-foreground">
                      Detected tools: {installedToolLabels.join(", ")}
                    </p>
                  {/if}

                  {#if setupDetectionError}
                    <p class="mt-2 text-xs text-destructive">
                      {setupDetectionError}
                    </p>
                  {/if}

                  {#if detectedTargets.length > 0}
                    <div class="mt-3 space-y-2">
                      <p class="text-xs font-medium text-foreground">
                        Discovered clusters
                      </p>
                      {#each detectedTargets as target (target.id)}
                        <Button
                          variant="outline"
                          class="w-full justify-between"
                          onclick={() => handleDetectedTargetSetup(target)}
                          disabled={isGeneratingKubeconfig}
                        >
                          <span class="truncate">{target.label}</span>
                          <span class="text-xs text-muted-foreground">
                            Use
                          </span>
                        </Button>
                      {/each}
                    </div>
                  {/if}

                  {#if setupDetection?.tools.aws.installed}
                    <div class="divider-edge-t divider-edge-full mt-4 space-y-2 pt-3">
                      <p class="text-xs font-medium text-foreground">
                        AWS EKS
                      </p>
                      <div class="grid gap-2">
                        <div class="grid gap-1">
                          <Label for="aws-cluster-name">Cluster name</Label>
                          <Input id="aws-cluster-name" bind:value={awsClusterName} />
                        </div>
                        <div class="grid gap-1">
                          <Label for="aws-region">Region</Label>
                          <Input id="aws-region" bind:value={awsRegion} placeholder="eu-west-1" />
                        </div>
                        <Button
                          variant="outline"
                          onclick={handleAwsSetup}
                          disabled={isGeneratingKubeconfig}
                        >
                          Generate with AWS CLI
                        </Button>
                      </div>
                    </div>
                  {/if}

                  {#if setupDetection?.tools.gcloud.installed && !detectedTargets.some((target) => target.provider === "gcloud")}
                    <div class="divider-edge-t divider-edge-full mt-4 space-y-2 pt-3">
                      <p class="text-xs font-medium text-foreground">Google GKE</p>
                      <div class="grid gap-2">
                        <div class="grid gap-1">
                          <Label for="gke-cluster-name">Cluster name</Label>
                          <Input id="gke-cluster-name" bind:value={gkeClusterName} />
                        </div>
                        <div class="grid gap-1">
                          <Label for="gke-location">Zone or region</Label>
                          <Input id="gke-location" bind:value={gkeLocation} placeholder="europe-west1-b" />
                        </div>
                        <div class="grid gap-1">
                          <Label for="gke-project">Project ID</Label>
                          <Input id="gke-project" bind:value={gkeProject} />
                        </div>
                        <Button
                          variant="outline"
                          onclick={handleManualGkeSetup}
                          disabled={isGeneratingKubeconfig}
                        >
                          Generate with gcloud
                        </Button>
                      </div>
                    </div>
                  {/if}

                  {#if setupDetection?.tools.az.installed && !detectedTargets.some((target) => target.provider === "az")}
                    <div class="divider-edge-t divider-edge-full mt-4 space-y-2 pt-3">
                      <p class="text-xs font-medium text-foreground">Azure AKS</p>
                      <div class="grid gap-2">
                        <div class="grid gap-1">
                          <Label for="aks-cluster-name">Cluster name</Label>
                          <Input id="aks-cluster-name" bind:value={aksClusterName} />
                        </div>
                        <div class="grid gap-1">
                          <Label for="aks-resource-group">Resource group</Label>
                          <Input id="aks-resource-group" bind:value={aksResourceGroup} />
                        </div>
                        <Button
                          variant="outline"
                          onclick={handleManualAksSetup}
                          disabled={isGeneratingKubeconfig}
                        >
                          Generate with Azure CLI
                        </Button>
                      </div>
                    </div>
                  {/if}

                  {#if kubeconfigActionOutput}
                    <pre class="mt-3 max-h-48 overflow-auto rounded-md bg-background p-3 text-xs text-muted-foreground">{kubeconfigActionOutput}</pre>
                  {/if}
                {/if}
              </div>
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
                    {#if clusterTyped.status === "connected"}
                      <Badge variant="default" class="bg-green-500"
                        >Connected</Badge
                      >
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
