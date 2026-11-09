<!-- Cloud resource list — every list route in RESOURCE_DESCRIPTORS renders through here -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { toStore } from 'svelte/store';
  import { cloudStore, loadResources, refreshResources } from '$lib/domains/cloud/stores';
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import BaseResourceTable from '$lib/domains/cloud/core/components/BaseResourceTable.svelte';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import { resourceDetailPath } from '$lib/domains/cloud/core/resourceDescriptors';
  import { PageHeader, PageStats, PageFilters, PageLoading } from '$lib/components/shell';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Plus from '~icons/fa6-solid/plus';
  import { useTableNavigation, useResourceActions } from '$lib/domains/k8s-navigation';
  import { toastActions } from '$lib/utils/toast';
  import type { PageData } from './$types';

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const descriptor = $derived(data.descriptor);

  let searchQuery = $state('');
  let filterValue = $state('');
  let loadedSlug = $state('');

  const all = $derived($cloudStore.resources[descriptor.type] ?? []);
  const isLoading = $derived($cloudStore.loading.resources[descriptor.type] ?? false);

  const filtered = $derived.by(() => {
    const query = searchQuery.trim().toLowerCase();
    const match = descriptor.filter?.match;
    return all.filter(
      (resource) =>
        (!query || resource.name.toLowerCase().includes(query)) &&
        (!filterValue || !match || match(resource, filterValue))
    );
  });

  const stats = $derived.by(() => [
    { label: 'Total', value: all.length },
    ...(descriptor.stats?.(all) ?? []),
  ]);

  const filterOptions = $derived.by(() => {
    const filter = descriptor.filter;
    if (!filter) return [];
    const options = filter.options(all);
    return options.length > 0 ? [{ value: '', label: filter.placeholder }, ...options] : [];
  });

  // The route is reused across slugs, so load on the slug rather than on mount.
  $effect(() => {
    const slug = descriptor.slug;
    if (!$cloudStore.connection.isConnected || loadedSlug === slug) return;
    loadedSlug = slug;
    searchQuery = '';
    filterValue = '';
    void loadResources(descriptor.type);
  });

  function openDetail(resource: ICloudResource, tab?: string) {
    const path = resourceDetailPath(descriptor, resource.name, resource.namespace);
    void goto(tab ? `${path}&tab=${tab}` : path);
  }

  // Scale dialog — deployments only
  let scaleTarget = $state<ICloudResource | null>(null);
  let scaleReplicas = $state(1);
  let isScaling = $state(false);
  let showScaleDialog = $state(false);

  function openScaleDialog(resource: ICloudResource) {
    scaleTarget = resource;
    scaleReplicas = Number(resource.metadata?.replicas) || 1;
    showScaleDialog = true;
  }

  async function confirmScale() {
    if (!scaleTarget) return;
    const target = scaleTarget;
    try {
      isScaling = true;
      await k8sResourceService.scaleDeployment(target.namespace, target.name, scaleReplicas);
      toastActions.success(`Scaled ${target.name} to ${scaleReplicas} replicas`);
      showScaleDialog = false;
      scaleTarget = null;
      await loadResources(descriptor.type);
    } catch (err) {
      toastActions.error(
        `Failed to scale deployment: ${err instanceof Error ? err.message : 'Unknown error'}`
      );
    } finally {
      isScaling = false;
    }
  }

  const tableNav = useTableNavigation({
    totalItems: () => filtered.length,
    onActivate: (index) => {
      const resource = filtered[index];
      if (resource) openDetail(resource);
    },
    enabled: $cloudStore.connection.isConnected,
  });

  const selectedIndex = tableNav.selectedIndex;

  const resourceActions = useResourceActions({
    selectedIndex,
    // A store, not a getter — the hook only reads a getter once, so shortcuts would
    // otherwise act on the resource list as it was before the first load.
    resources: toStore(() => filtered),
    handlers: {
      onDescribe: (resource) => openDetail(resource),
      onEdit: (resource) => openDetail(resource, 'yaml'),
      onYaml: (resource) => openDetail(resource, 'yaml'),
      onLogs: (resource) => {
        if (descriptor.hasLogs) openDetail(resource, 'logs');
      },
      onPortForward: (resource) => {
        if (descriptor.hasLogs) openDetail(resource, 'port-forward');
      },
      onScale: (resource) => {
        if (descriptor.canScale) openScaleDialog(resource);
      },
      onRestart: descriptor.canRestart
        ? async (resource) => {
            try {
              await k8sResourceService.restartDeployment(resource.namespace, resource.name);
              toastActions.success(`Restarted ${resource.name}`);
              await loadResources(descriptor.type);
            } catch (error) {
              toastActions.error(
                `Failed to restart: ${error instanceof Error ? error.message : 'Unknown error'}`
              );
            }
          }
        : undefined,
      onRefresh: () => {
        void refreshResources();
      },
    },
    enabled: $cloudStore.connection.isConnected,
  });

  function handleKeydown(event: KeyboardEvent) {
    if (tableNav.handleKeydown(event)) return;
    if (resourceActions.handleKeydown(event)) return;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="space-y-6">
  <PageHeader title={descriptor.label} description={descriptor.description}>
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={() => void refreshResources()}>
        <RefreshCw class="mr-2 h-4 w-4" />
        Refresh
      </Button>
      {#if descriptor.canCreate}
        <Button size="sm" onclick={() => goto(`/cloud/${descriptor.slug}/new`)}>
          <Plus class="mr-2 h-4 w-4" />
          Create
        </Button>
      {/if}
    {/snippet}
  </PageHeader>

  <PageStats {stats} />

  <PageFilters bind:searchQuery searchPlaceholder="Search {descriptor.label}...">
    {#snippet filters()}
      {#if filterOptions.length > 0}
        <Select
          bind:value={filterValue}
          options={filterOptions}
          aria-label={descriptor.filter?.placeholder}
          class="w-[180px]"
        />
      {/if}
    {/snippet}
  </PageFilters>

  {#snippet scaleRowAction(resource: ICloudResource)}
    <Button
      variant="outline"
      size="sm"
      aria-label="Scale {resource.name}"
      onclick={(event) => {
        event.stopPropagation();
        openScaleDialog(resource);
      }}
    >
      Scale
    </Button>
  {/snippet}

  {#if isLoading && all.length === 0}
    <PageLoading message="Loading {descriptor.label}..." />
  {:else}
    <div class="k8s-navigable-table" data-selected-index={$selectedIndex}>
      <BaseResourceTable
        resources={filtered}
        columns={descriptor.columns}
        resourceType={descriptor.type}
        emptyMessage="No {descriptor.label} found"
        filteredEmptyMessage="No {descriptor.label} match your filters"
        isFiltered={Boolean(searchQuery || filterValue)}
        onResourceClick={(resource) => openDetail(resource)}
        onViewLogs={descriptor.hasLogs ? (resource) => openDetail(resource, 'logs') : undefined}
        rowActions={descriptor.canScale ? scaleRowAction : undefined}
      />
    </div>
  {/if}

  {#if descriptor.canScale}
    <Dialog bind:open={showScaleDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Scale {descriptor.label}</DialogTitle>
          <DialogDescription>
            Set the number of replicas for {scaleTarget?.name}
          </DialogDescription>
        </DialogHeader>
        <div class="space-y-2 py-4">
          <Label for="replicas">Replicas</Label>
          <Input
            id="replicas"
            type="number"
            min="0"
            bind:value={scaleReplicas}
            disabled={isScaling}
          />
        </div>
        <DialogFooter>
          <Button variant="outline" onclick={() => (showScaleDialog = false)} disabled={isScaling}>
            Cancel
          </Button>
          <Button onclick={confirmScale} disabled={isScaling}>
            {isScaling ? 'Scaling...' : 'Scale'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  {/if}
</div>
