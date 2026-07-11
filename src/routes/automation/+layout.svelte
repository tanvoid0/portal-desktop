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

<div class="divider-edge-b divider-edge-full bg-background px-[var(--content-gutter)] pt-4">
  <Tabs value={activeTab}>
    <TabsList class="h-auto w-full justify-start overflow-x-auto">
      {#each tabs as tab}
        <TabsTrigger value={tab.id} class="shrink-0" onclick={() => goto(tab.url)}>
          {tab.label}
        </TabsTrigger>
      {/each}
    </TabsList>
  </Tabs>
</div>

{@render children()}
