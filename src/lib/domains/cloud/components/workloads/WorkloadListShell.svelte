<script lang="ts">
  import type { Snippet } from "svelte";
  import {
    PageHeader,
    PageLoading,
    PageEmpty,
  } from "$lib/components/shell";
  import { Button } from "$lib/components/ui/button";
  import { RefreshCw } from "@lucide/svelte";
  import { cloudStore } from "$lib/domains/cloud/stores";
  import type { ResourceType } from "$lib/domains/cloud/core/types";

  interface Props {
    title: string;
    description: string;
    resourceType: ResourceType;
    isLoading?: boolean;
    itemCount?: number;
    emptyTitle?: string;
    emptyDescription?: string;
    onRefresh?: () => void | Promise<void>;
    children: Snippet;
    filters?: Snippet;
    stats?: Snippet;
  }

  let {
    title,
    description,
    resourceType,
    isLoading = false,
    itemCount = 0,
    emptyTitle = "No resources found",
    emptyDescription = "Connect to a cluster to load resources",
    onRefresh,
    children,
    filters,
    stats,
  }: Props = $props();
</script>

<div class="space-y-6 p-6">
  <PageHeader {title} {description}>
    {#snippet actions()}
      {#if onRefresh}
        <Button variant="outline" size="sm" onclick={() => onRefresh?.()}>
          <RefreshCw class="mr-2 h-4 w-4" />
          Refresh
        </Button>
      {/if}
    {/snippet}
  </PageHeader>

  {#if !$cloudStore.connection.isConnected}
    <PageEmpty
      title="Not connected"
      description="Connect to a Kubernetes cluster from Cloud settings"
    />
  {:else if isLoading}
    <PageLoading message={`Loading ${title.toLowerCase()}...`} />
  {:else if itemCount === 0}
    <PageEmpty title={emptyTitle} description={emptyDescription} />
  {:else}
    {#if stats}
      <div>{@render stats()}</div>
    {/if}
    {#if filters}
      <div>{@render filters()}</div>
    {/if}
    {@render children()}
  {/if}
</div>
