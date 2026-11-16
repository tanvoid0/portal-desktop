<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { createQuery } from '@tanstack/svelte-query';
  import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { engineTabsFor } from '$lib/config/agents-tabs';
  import { agentBackendsQuery } from '$lib/domains/agents/queries/agentsQueries.js';
  import { isMobileShell } from '$lib/domains/mobile/shell';
  import { mobileConnection } from '$lib/domains/mobile/connection.svelte';

  interface Props {
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  const backendsQuery = createQuery(agentBackendsQuery);

  // Data-driven: a new backend (e.g. Codex) shows up here with no frontend edit.
  const tabs = $derived(
    engineTabsFor(
      backendsQuery.data ?? [],
      isMobileShell ? (mobileConnection.status?.caps ?? []) : null
    )
  );

  // Selected tab comes from the URL; each tab's value is its route.
  const active = $derived(page.url.pathname.replace(/\/$/, ''));

  function onTabChange(url: string | undefined) {
    if (url && url !== active) void goto(url);
  }
</script>

{#if tabs.length > 1}
  <div class={className}>
    <Tabs value={active} onValueChange={onTabChange}>
      <TabsList
        class="h-auto max-w-full justify-start gap-0.5 overflow-x-auto bg-transparent p-0"
        aria-label="Coding engine"
      >
        {#each tabs as tab (tab.id)}
          <TabsTrigger
            value={tab.url}
            class="shrink-0 px-2.5 py-1 text-xs whitespace-nowrap transition-none data-[state=active]:bg-muted data-[state=active]:shadow-none"
          >
            {tab.label}
          </TabsTrigger>
        {/each}
      </TabsList>
    </Tabs>
  </div>
{/if}
