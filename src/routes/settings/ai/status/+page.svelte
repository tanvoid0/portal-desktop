<!--
  AI › Status & Logs — live state of the local agent-platform daemon plus a tail
  of its output. Behind portal's platform-health gating: when the daemon is not
  usable, the banner takes over and the status query stays idle.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { createQuery } from '@tanstack/svelte-query';
  import { PageHeader } from '$lib/components/shell';
  import { Button } from '$lib/components/ui/button';
  import PlatformStatusBanner from '$lib/domains/ai/components/PlatformStatusBanner.svelte';
  import PlatformStatusCards from '$lib/domains/ai/components/platform/PlatformStatusCards.svelte';
  import DaemonLogViewer from '$lib/domains/ai/components/platform/DaemonLogViewer.svelte';
  import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';
  import { platformSystemService, aiProviderService } from '$lib/domains/ai';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import RefreshCw from '~icons/fa6-solid/rotate';

  let token = $state<string | null>(null);

  onMount(async () => {
    await aiPlatform.refresh();
    try {
      const cfg = await aiProviderService.getProviderConfig('AgentPlatform');
      token = cfg.api_key ?? null;
    } catch {
      token = null;
    }
  });

  const statusQuery = createQuery(() => ({
    queryKey: queryKeys.ai.platformStatus,
    queryFn: () => platformSystemService.getStatus(),
    enabled: aiPlatform.known && !aiPlatform.blocked,
    refetchInterval: 5000,
  }));

  const status = $derived(statusQuery.data);

  function refreshAll() {
    aiPlatform.refresh();
    statusQuery.refetch();
  }
</script>

<svelte:head>
  <title>AI Status & Logs - Portal Desktop</title>
</svelte:head>

<div class="space-y-5">
  <PageHeader
    title="Status & Logs"
    description="Live state of the local agent-platform daemon and its output."
  >
    {#snippet actions()}
      <Button
        variant="outline"
        size="sm"
        class="gap-2"
        disabled={aiPlatform.checking}
        onclick={refreshAll}
      >
        <RefreshCw class="h-3.5 w-3.5 {aiPlatform.checking ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    {/snippet}
  </PageHeader>

  <PlatformStatusBanner blocks="Status and logs" />

  <PlatformStatusCards {status} {token} pending={statusQuery.isPending && !aiPlatform.blocked} />

  <DaemonLogViewer />
</div>
