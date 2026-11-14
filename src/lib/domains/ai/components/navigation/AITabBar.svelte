<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import NavIcon from '$lib/components/shell/nav-icon.svelte';
  import { AI_TABS, getActiveAiTab } from '$lib/config/ai-tabs';
  import { isMobileShell } from '$lib/domains/mobile/shell';
  import { mobileConnection } from '$lib/domains/mobile/connection.svelte';

  interface Props {
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  const activeTab = $derived(getActiveAiTab(page.url.pathname));

  // On the mobile shell, only show tabs the paired device actually serves.
  const tabs = $derived.by(() => {
    if (!isMobileShell) return AI_TABS;
    const caps = mobileConnection.status?.caps ?? [];
    return AI_TABS.filter((t) => t.id !== 'code' || caps.includes('coder')).filter(
      (t) => t.id !== 'agents' || caps.includes('agents')
    );
  });

  function onTabChange(tabId: string | undefined) {
    if (!tabId) return;
    const tab = AI_TABS.find((t) => t.id === tabId);
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
