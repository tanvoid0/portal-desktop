<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import Trash2 from '~icons/fa6-solid/trash';
  import ManagerVersionsCard from './ManagerVersionsCard.svelte';
  import ManagerVersionRow from './ManagerVersionRow.svelte';
  import ManagerSwitchButton from './ManagerSwitchButton.svelte';

  interface Props {
    versions: string[];
    current: string | null;
    loading: boolean;
    error: string | null;
    switchingVersion: string | null;
    onrefresh: () => void;
    onswitch: (version: string) => void;
    onuninstall: (version: string) => void;
  }
  let {
    versions,
    current,
    loading,
    error,
    switchingVersion,
    onrefresh,
    onswitch,
    onuninstall,
  }: Props = $props();
</script>

<ManagerVersionsCard
  title="Installed Versions"
  {loading}
  {error}
  empty={versions.length === 0}
  emptyText="No versions installed"
  {onrefresh}
>
  <div class="space-y-2">
    {#each versions as version}
      <ManagerVersionRow {version}>
        {#snippet badge()}
          {#if current === version}
            <Badge variant="default" class="bg-blue-100 text-xs text-blue-800">Current</Badge>
          {/if}
        {/snippet}
        {#snippet actions()}
          {#if current !== version}
            <ManagerSwitchButton
              switching={switchingVersion === version}
              onclick={() => onswitch(version)}
            />
          {/if}
          <Button variant="outline" size="sm" onclick={() => onuninstall(version)}>
            <Trash2 class="mr-1 h-4 w-4" />
            Uninstall
          </Button>
        {/snippet}
      </ManagerVersionRow>
    {/each}
  </div>
</ManagerVersionsCard>
