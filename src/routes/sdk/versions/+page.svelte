<!--
	SDK Versions Page
	Installed and installable versions for every SDK, grouped by SDK.
-->

<script lang="ts">
  import { useQueryClient } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { PageHeader } from '$lib/components/shell';
  import VersionsPanel from '$lib/domains/sdk/components/VersionsPanel.svelte';

  const queryClient = useQueryClient();

  function refresh() {
    void queryClient.invalidateQueries({ queryKey: ['sdk'] });
  }
</script>

<svelte:head>
  <title>SDK Versions - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader title="SDK Versions" description="Installed SDK versions and switching between them">
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={refresh}>Refresh</Button>
    {/snippet}
  </PageHeader>

  <VersionsPanel />
</div>
