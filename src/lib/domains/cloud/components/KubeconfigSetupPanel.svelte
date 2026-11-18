<!-- Automatic kubeconfig setup: detect installed cloud CLIs and generate a kubeconfig -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { invokeClient } from '$lib/utils/invokeClient';
  import { isTauriEnvironment } from '$lib/utils/tauri';
  import { toast } from '$lib/utils/toast';

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

  interface Props {
    /** Called after a kubeconfig was generated so the parent can reload clusters and connect. */
    onGenerated: () => Promise<void>;
  }
  let { onGenerated }: Props = $props();

  let setupDetection = $state<KubeSetupDetectionResult | null>(null);
  let isLoadingSetupOptions = $state(false);
  let setupDetectionError = $state<string | null>(null);
  let isGeneratingKubeconfig = $state(false);
  let kubeconfigActionOutput = $state<string | null>(null);
  let awsClusterName = $state('');
  let awsRegion = $state('');
  let gkeClusterName = $state('');
  let gkeLocation = $state('');
  let gkeProject = $state('');
  let aksClusterName = $state('');
  let aksResourceGroup = $state('');

  const detectedTargets = $derived(setupDetection?.targets ?? []);
  const installedToolLabels = $derived.by(() => {
    if (!setupDetection) return [];

    return Object.entries(setupDetection.tools)
      .filter(([, value]) => value.installed)
      .map(([key]) => key);
  });

  onMount(loadSetupDetection);

  /** Re-run tool detection (parent calls this on Retry). */
  export async function reload() {
    await loadSetupDetection();
  }

  async function loadSetupDetection() {
    if (!isTauriEnvironment()) return;

    isLoadingSetupOptions = true;
    setupDetectionError = null;
    try {
      setupDetection = await invokeClient.post<KubeSetupDetectionResult>(
        'k8s_detect_setup_tools',
        undefined,
        { localhostStrategy: 'error' }
      );
    } catch (error) {
      setupDetectionError =
        error instanceof Error ? error.message : 'Failed to detect Kubernetes setup tools';
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
      }>('k8s_generate_kubeconfig', {
        request,
      });

      kubeconfigActionOutput = [result.command, result.stdout, result.stderr]
        .filter(Boolean)
        .join('\n\n');

      toast.success('Kubeconfig generated', {
        description: 'Retrying cluster detection.',
      });

      await onGenerated();
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to generate kubeconfig';
      kubeconfigActionOutput = message;
      toast.error('Kubeconfig setup failed', { description: message });
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
      toast.error('AWS setup needs cluster name and region');
      return;
    }

    await generateKubeconfig({
      provider: 'aws',
      cluster_name: awsClusterName,
      region: awsRegion,
    });
  }

  async function handleManualGkeSetup() {
    if (!gkeClusterName || !gkeLocation) {
      toast.error('GKE setup needs cluster name and location');
      return;
    }

    const isZone = /[a-z]+-[a-z]+\d-[a-z]$/.test(gkeLocation);
    await generateKubeconfig({
      provider: 'gcloud',
      cluster_name: gkeClusterName,
      project: gkeProject || undefined,
      zone: isZone ? gkeLocation : undefined,
      region: isZone ? undefined : gkeLocation,
    });
  }

  async function handleManualAksSetup() {
    if (!aksClusterName || !aksResourceGroup) {
      toast.error('AKS setup needs cluster name and resource group');
      return;
    }

    await generateKubeconfig({
      provider: 'az',
      cluster_name: aksClusterName,
      resource_group: aksResourceGroup,
    });
  }

  async function refreshTools() {
    const before = installedToolLabels.join(',') + detectedTargets.length;
    await loadSetupDetection();
    if (setupDetectionError)
      toast.error('Tool detection failed', { description: setupDetectionError });
    else if (before === installedToolLabels.join(',') + detectedTargets.length)
      toast.info('No new tools detected');
    else toast.success('Tools updated');
  }
</script>

<div class="rounded-md border bg-muted/30 p-3">
  <div class="mb-2 flex items-center justify-between gap-2">
    <p class="text-sm font-medium">Automatic setup</p>
    <Button
      variant="outline"
      size="sm"
      onclick={refreshTools}
      disabled={isLoadingSetupOptions || isGeneratingKubeconfig}
    >
      {isLoadingSetupOptions ? 'Refreshing...' : 'Refresh tools'}
    </Button>
  </div>

  {#if isLoadingSetupOptions}
    <p class="text-xs text-muted-foreground">Detecting installed Kubernetes tools...</p>
  {:else}
    {#if installedToolLabels.length > 0}
      <p class="text-xs text-muted-foreground">
        Detected tools: {installedToolLabels.join(', ')}
      </p>
    {/if}

    {#each installedToolLabels.filter((t) => !['aws', 'gcloud', 'az', 'kubectl'].includes(t) && !detectedTargets.some((x) => x.provider === t)) as tool (tool)}
      <p class="mt-1 text-xs text-muted-foreground">
        no {tool} clusters found. Add a cluster context to your kubeconfig, then click Retry.
      </p>
    {/each}

    {#if setupDetectionError}
      <p class="mt-2 text-xs text-destructive">
        {setupDetectionError}
      </p>
    {/if}

    {#if detectedTargets.length > 0}
      <div class="mt-3 space-y-2">
        <p class="text-xs font-medium text-foreground">Discovered clusters</p>
        {#each detectedTargets as target (target.id)}
          <Button
            variant="outline"
            class="w-full justify-between"
            onclick={() => handleDetectedTargetSetup(target)}
            disabled={isGeneratingKubeconfig}
          >
            <span class="truncate">{target.label}</span>
            <span class="text-xs text-muted-foreground"> Use </span>
          </Button>
        {/each}
      </div>
    {/if}

    {#if setupDetection?.tools.aws.installed}
      <div class="divider-edge-t divider-edge-full mt-4 space-y-2 pt-3">
        <p class="text-xs font-medium text-foreground">AWS EKS</p>
        <div class="grid gap-2">
          <div class="grid gap-1">
            <Label for="aws-cluster-name">Cluster name</Label>
            <Input id="aws-cluster-name" bind:value={awsClusterName} />
          </div>
          <div class="grid gap-1">
            <Label for="aws-region">Region</Label>
            <Input id="aws-region" bind:value={awsRegion} placeholder="eu-west-1" />
          </div>
          <Button variant="outline" onclick={handleAwsSetup} disabled={isGeneratingKubeconfig}>
            Generate with AWS CLI
          </Button>
        </div>
      </div>
    {/if}

    {#if setupDetection?.tools.gcloud.installed && !detectedTargets.some((target) => target.provider === 'gcloud')}
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

    {#if setupDetection?.tools.az.installed && !detectedTargets.some((target) => target.provider === 'az')}
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
      <pre
        class="mt-3 max-h-48 overflow-auto rounded-md bg-background p-3 text-xs text-muted-foreground">{kubeconfigActionOutput}</pre>
    {/if}
  {/if}
</div>
