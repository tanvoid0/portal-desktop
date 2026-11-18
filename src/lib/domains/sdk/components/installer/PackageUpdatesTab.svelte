<script lang="ts">
  import { CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import ArrowUp from '~icons/fa6-solid/arrow-up';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import Loader2 from '~icons/fa6-solid/spinner';
  import InlineSpinner from '$lib/components/shared/InlineSpinner.svelte';
  import InlineEmptyState from '$lib/components/shared/InlineEmptyState.svelte';
  import PackageRowCard from './PackageRowCard.svelte';
  import type { PackageUpdate } from '$lib/domains/sdk/services/packageManagerService';

  interface Props {
    updates: PackageUpdate[];
    loading: boolean;
    upgradingId: string | null;
    onrefresh: () => void;
    onupgrade: (update: PackageUpdate) => void;
  }
  let { updates, loading, upgradingId, onrefresh, onupgrade }: Props = $props();
</script>

<div class="flex items-center justify-between">
  <CardTitle>Available Updates</CardTitle>
  <Button variant="outline" size="sm" onclick={onrefresh} disabled={loading}>
    <RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
    Check Updates
  </Button>
</div>
{#if loading}
  <InlineSpinner />
{:else if updates.length === 0}
  <InlineEmptyState
    icon={CheckCircle}
    iconClass="text-green-500"
    title="All packages are up to date"
    hint={'Click "Check Updates" to refresh'}
  />
{:else}
  <div class="space-y-2">
    {#each updates as update}
      <PackageRowCard
        name={update.name}
        source={update.source}
        subtitle="{update.current_version} → {update.available_version}"
      >
        {#snippet actions()}
          <Button
            variant="default"
            size="sm"
            onclick={() => onupgrade(update)}
            disabled={upgradingId === update.id}
          >
            {#if upgradingId === update.id}
              <Loader2 class="mr-1 h-4 w-4 animate-spin" />
              Upgrading...
            {:else}
              <ArrowUp class="mr-1 h-4 w-4" />
              Upgrade
            {/if}
          </Button>
        {/snippet}
      </PackageRowCard>
    {/each}
  </div>
{/if}
