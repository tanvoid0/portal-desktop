<!-- Secret Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { ResourceDetail } from '$lib/domains/cloud/state/resourceDetail.svelte';
  import { ResourceType } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Trash2 from '~icons/fa6-solid/trash';
  import Eye from '~icons/fa6-solid/eye';
  import EyeOff from '~icons/fa6-solid/eye-slash';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import Loading from '$lib/components/ui/loading.svelte';
  import { PageLoading, PageError } from '$lib/components/shell';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import ResourceDetailHeader from '$lib/domains/cloud/components/ResourceDetailHeader.svelte';
  import LabelsCard from '$lib/domains/cloud/components/LabelsCard.svelte';

  const r = new ResourceDetail(ResourceType.SECRET, 'Secret', () => page.params.secret);
  const secret = $derived(r.resource);
  let showSecretValues = $state<Record<string, boolean>>({});

  onMount(r.load);

  async function handleDelete() {
    if (!secret) return;

    const confirmed = await confirmAction(
      `Are you sure you want to delete Secret "${secret.name}"? This action cannot be undone.`,
      'Delete secret'
    );
    if (!confirmed) return;

    try {
      await k8sResourceService.deleteSecret(secret.namespace, secret.name);

      toastActions.success(`Secret "${secret.name}" deleted successfully`);
      goto('/cloud/secrets');
    } catch (err) {
      toastActions.error(err instanceof Error ? err.message : 'Failed to delete Secret');
    }
  }

  function toggleSecretValue(key: string) {
    showSecretValues[key] = !showSecretValues[key];
    showSecretValues = { ...showSecretValues };
  }

  function decodeBase64(base64: string): string {
    try {
      // The data is already base64-encoded from the backend
      // We need to decode it to show the actual value
      const decoded = atob(base64);
      return decoded;
    } catch (error) {
      return '[Decode Error]';
    }
  }

  const dataEntries = $derived.by(() => {
    if (!secret || !secret.metadata?.data) return [];
    return Object.entries(secret.metadata.data);
  });

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
    toastActions.success('Copied to clipboard');
  }
</script>

<div class="space-y-6 p-6">
  {#if r.isLoading}
    <PageLoading message="Loading secret..." />
  {:else if r.error}
    <PageError title="Failed to load secret" message={r.error} onRetry={r.load} />
  {:else if secret}
    <ResourceDetailHeader
      kind="Secret"
      name={secret.name}
      namespace={secret.namespace}
      backHref="/cloud/secrets"
      backLabel="Back to Secrets"
      onRefresh={r.load}
    >
      <Button variant="destructive" size="sm" onclick={handleDelete}>
        <Trash2 class="mr-2 h-4 w-4" />
        Delete
      </Button>
    </ResourceDetailHeader>

    <!-- Warning -->
    <div
      class="rounded-lg border border-yellow-200 bg-yellow-50 p-4 dark:border-yellow-800 dark:bg-yellow-900/20"
    >
      <p class="text-sm text-yellow-800 dark:text-yellow-200">
        <strong>Security Warning:</strong> Secret values contain sensitive information. Be careful when
        viewing or editing.
      </p>
    </div>

    <!-- Tabs -->
    <Tabs value={r.activeTab} onValueChange={r.setTab}>
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="data">Data</TabsTrigger>
        <TabsTrigger value="yaml">YAML</TabsTrigger>
      </TabsList>

      <!-- Overview Tab -->
      <TabsContent value="overview" class="space-y-4">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Secret Information</CardTitle>
            </CardHeader>
            <CardContent class="space-y-3">
              <div>
                <p class="text-sm text-muted-foreground">Name</p>
                <p class="font-medium">{secret.name}</p>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Namespace</p>
                <p class="font-medium">{secret.namespace}</p>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Type</p>
                <Badge variant="outline">{secret.metadata?.type || 'Opaque'}</Badge>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Data Keys</p>
                <p class="font-medium">{secret.metadata?.dataCount || 0}</p>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Age</p>
                <p class="font-medium">{secret.metadata?.age || 'N/A'}</p>
              </div>
            </CardContent>
          </Card>

          {#if secret.metadata?.labels && Object.keys(secret.metadata.labels).length > 0}
            <LabelsCard items={secret.metadata.labels} />
          {/if}
        </div>
      </TabsContent>

      <!-- Data Tab -->
      <TabsContent value="data" class="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>Secret Data</CardTitle>
          </CardHeader>
          <CardContent>
            {#if dataEntries.length === 0}
              <div class="py-8 text-center text-muted-foreground">
                <p>No data entries</p>
              </div>
            {:else}
              <div class="space-y-4">
                {#each dataEntries as [key, base64Value]}
                  <div class="rounded-lg border p-4">
                    <div class="mb-2 flex items-center justify-between">
                      <span class="font-medium">{key}</span>
                      <div class="flex items-center gap-2">
                        <Button variant="ghost" size="sm" onclick={() => toggleSecretValue(key)}>
                          {#if showSecretValues[key]}
                            <EyeOff class="mr-1 h-4 w-4" />
                          {:else}
                            <Eye class="mr-1 h-4 w-4" />
                          {/if}
                          {showSecretValues[key] ? 'Hide' : 'Show'}
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onclick={() => {
                            const value = showSecretValues[key]
                              ? decodeBase64(String(base64Value))
                              : String(base64Value);
                            copyToClipboard(value);
                          }}
                        >
                          Copy
                        </Button>
                      </div>
                    </div>
                    <pre
                      class="overflow-auto rounded bg-muted p-3 text-sm break-words whitespace-pre-wrap">
											{#if showSecretValues[key]}
                        {decodeBase64(String(base64Value))}
                      {:else}
                        {'*'.repeat(20)} (hidden)
                      {/if}
										</pre>
                  </div>
                {/each}
              </div>
            {/if}
          </CardContent>
        </Card>
      </TabsContent>

      <!-- YAML Tab -->
      <TabsContent value="yaml" class="space-y-4">
        <div class="h-[600px]">
          <ResourceYamlTab
            resource={secret}
            kind="Secret"
            active={r.activeTab === 'yaml'}
            onApplied={r.load}
          />
        </div>
      </TabsContent>
    </Tabs>
  {/if}
</div>
