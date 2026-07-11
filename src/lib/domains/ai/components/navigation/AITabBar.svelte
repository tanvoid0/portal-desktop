<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Tabs, TabsList, TabsTrigger } from "$lib/components/ui/tabs";
  import NavIcon from "$lib/components/shell/nav-icon.svelte";
  import { AI_TABS, getActiveAiTab } from "$lib/config/ai-tabs";

  interface Props {
    class?: string;
  }

  let { class: className = "" }: Props = $props();

  const activeTab = $derived(getActiveAiTab($page.url.pathname));

  function onTabChange(tabId: string | undefined) {
    if (!tabId) return;
    const tab = AI_TABS.find((t) => t.id === tabId);
    if (!tab || tab.id === activeTab) return;
    void goto(tab.url);
  }
</script>

<div class={className}>
  <Tabs value={activeTab} onValueChange={onTabChange}>
    <TabsList class="h-auto w-full justify-stretch gap-1 p-1">
      {#each AI_TABS as tab (tab.id)}
        <TabsTrigger value={tab.id} class="flex-1 gap-1.5 px-2 py-1.5 text-xs">
          <NavIcon icon={tab.icon} class="h-3.5 w-3.5" />
          <span class="group-data-[collapsible=icon]:hidden">{tab.label}</span>
        </TabsTrigger>
      {/each}
    </TabsList>
  </Tabs>
</div>
