<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import Download from '~icons/fa6-solid/download';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import ManagerVersionsCard from './ManagerVersionsCard.svelte';
  import ManagerVersionRow from './ManagerVersionRow.svelte';
  import ManagerSwitchButton from './ManagerSwitchButton.svelte';

  interface Props {
    versions: string[];
    installed: string[];
    supportsInstallation: boolean;
    loading: boolean;
    installingVersion: string | null;
    switchingVersion: string | null;
    onrefresh: () => void;
    oninstall: (version: string) => void;
    onswitch: (version: string) => void;
  }
  let {
    versions,
    installed,
    supportsInstallation,
    loading,
    installingVersion,
    switchingVersion,
    onrefresh,
    oninstall,
    onswitch,
  }: Props = $props();
</script>

<ManagerVersionsCard
  title="Available Versions"
  {loading}
  empty={versions.length === 0}
  emptyText="No versions available"
  {onrefresh}
>
  <div class="max-h-[600px] space-y-2 overflow-y-auto">
    {#each versions as version}
      {@const isInstalled = installed.includes(version)}
      <ManagerVersionRow {version}>
        {#snippet badge()}
          {#if isInstalled}
            <Badge variant="secondary" class="text-xs">Installed</Badge>
          {/if}
        {/snippet}
        {#snippet actions()}
          {#if !isInstalled && supportsInstallation}
            <Button
              variant="default"
              size="sm"
              onclick={() => oninstall(version)}
              disabled={installingVersion === version}
            >
              {#if installingVersion === version}
                <RefreshCw class="mr-1 h-4 w-4 animate-spin" />
                Installing...
              {:else}
                <Download class="mr-1 h-4 w-4" />
                Install
              {/if}
            </Button>
          {:else if isInstalled}
            <ManagerSwitchButton
              switching={switchingVersion === version}
              onclick={() => onswitch(version)}
            />
          {/if}
        {/snippet}
      </ManagerVersionRow>
    {/each}
  </div>
</ManagerVersionsCard>
