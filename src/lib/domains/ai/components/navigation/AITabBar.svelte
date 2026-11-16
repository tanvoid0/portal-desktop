<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import NavIcon from '$lib/components/shell/nav-icon.svelte';
  import { AI_TABS, getActiveAiTab } from '$lib/config/ai-tabs';
  import { agentTabPath } from '$lib/config/agents-tabs';
  import { isMobileShell } from '$lib/domains/mobile/shell';
  import { mobileConnection } from '$lib/domains/mobile/connection.svelte';

  interface Props {
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  const activeTab = $derived(getActiveAiTab(page.url.pathname));

  // On the mobile shell, only show tabs the paired device actually serves.
  // Code covers both the Portal workspace (cap `coder`) and agent backends
  // (cap `agents`) — a device with only one of those still needs the tab,
  // routed straight to whichever side it can reach.
  const tabs = $derived.by(() => {
    if (!isMobileShell) return AI_TABS;
    const caps = mobileConnection.status?.caps ?? [];
    const hasCoder = caps.includes('coder');
    const hasAgents = caps.includes('agents');
    return AI_TABS.filter((t) => t.id !== 'code' || hasCoder || hasAgents).map((t) =>
      t.id === 'code' && !hasCoder && hasAgents ? { ...t, url: agentTabPath() } : t
    );
  });

  function onTabChange(tabId: string | undefined) {
    if (!tabId) return;
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab || tab.id === activeTab) return;
    void goto(tab.url);
  }
</script>

<div class={className}>
  <Tabs value={activeTab} onValueChange={onTabChange}>
    <TabsList class="h-auto gap-1 p-1">
      {#each tabs as tab (tab.id)}
        <TabsTrigger value={tab.id} class="flex-1 gap-1.5 px-2 py-1.5 text-xs">
          <NavIcon icon={tab.icon} class="h-3.5 w-3.5" />
          <span class="group-data-[collapsible=icon]:hidden">{tab.label}</span>
        </TabsTrigger>
      {/each}
    </TabsList>
  </Tabs>
</div>
