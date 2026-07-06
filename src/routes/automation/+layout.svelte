<script lang="ts">
  import type { Snippet } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    Tabs,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";

  let { children }: { children: Snippet } = $props();

  const tabs = [
    { id: "run", label: "Quick Run", url: "/automation/run" },
    { id: "blocks", label: "Blocks", url: "/automation/blocks" },
    { id: "scripts", label: "Scripts", url: "/automation/scripts" },
    {
      id: "utilities",
      label: "Utilities",
      url: "/automation/utilities",
    },
  ] as const;

  const activeTab = $derived(
    tabs.find((tab) => $page.url.pathname.startsWith(tab.url))?.id ??
      "run",
  );
</script>

<div class="border-b bg-background px-6 pt-4">
  <Tabs value={activeTab}>
    <TabsList>
      {#each tabs as tab}
        <TabsTrigger value={tab.id} onclick={() => goto(tab.url)}>
          {tab.label}
        </TabsTrigger>
      {/each}
    </TabsList>
  </Tabs>
</div>

{@render children()}
