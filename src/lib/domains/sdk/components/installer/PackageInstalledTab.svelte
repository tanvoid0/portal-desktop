<script lang="ts">
  import { CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Trash2 from '~icons/fa6-solid/trash';
  import ArrowUp from '~icons/fa6-solid/arrow-up';
  import PackageIcon from '~icons/fa6-solid/box-open';
  import Info from '~icons/fa6-solid/circle-info';
  import Loader2 from '~icons/fa6-solid/spinner';
  import InlineSpinner from '$lib/components/shared/InlineSpinner.svelte';
  import InlineEmptyState from '$lib/components/shared/InlineEmptyState.svelte';
  import PackageRowCard from './PackageRowCard.svelte';
  import type { InstalledPackage } from '$lib/domains/sdk/services/packageManagerService';

  interface Props {
    packages: InstalledPackage[];
    loading: boolean;
    upgradingId: string | null;
    uninstallingId: string | null;
    onrefresh: () => void;
    ondetails: (pkg: InstalledPackage) => void;
    onupgrade: (pkg: InstalledPackage) => void;
    onuninstall: (pkg: InstalledPackage) => void;
  }
  let {
    packages,
    loading,
    upgradingId,
    uninstallingId,
    onrefresh,
    ondetails,
    onupgrade,
    onuninstall,
  }: Props = $props();
</script>

<div class="flex items-center justify-between">
  <CardTitle>Installed Packages</CardTitle>
  <Button variant="outline" size="sm" onclick={onrefresh} disabled={loading}>
    <RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
    Refresh
  </Button>
</div>
{#if loading}
  <InlineSpinner />
{:else if packages.length === 0}
  <InlineEmptyState
    icon={PackageIcon}
    title="No packages installed"
    hint="Click Refresh to load installed packages"
  />
{:else}
  <div class="space-y-2">
    {#each packages as pkg}
      <PackageRowCard name={pkg.name} source={pkg.source} subtitle="Version: {pkg.version}">
        {#snippet actions()}
          <Button variant="outline" size="sm" onclick={() => ondetails(pkg)}>
            <Info class="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={() => onupgrade(pkg)}
            disabled={upgradingId === pkg.id}
          >
            {#if upgradingId === pkg.id}
              <Loader2 class="mr-1 h-4 w-4 animate-spin" />
            {:else}
              <ArrowUp class="mr-1 h-4 w-4" />
            {/if}
            Upgrade
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={() => onuninstall(pkg)}
            disabled={uninstallingId === pkg.id}
          >
            {#if uninstallingId === pkg.id}
              <Loader2 class="mr-1 h-4 w-4 animate-spin" />
            {:else}
              <Trash2 class="mr-1 h-4 w-4" />
            {/if}
            Uninstall
          </Button>
        {/snippet}
      </PackageRowCard>
    {/each}
  </div>
{/if}
