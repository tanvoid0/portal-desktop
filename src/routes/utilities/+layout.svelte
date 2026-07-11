<script lang="ts">
  import type { Snippet } from "svelte";
  import { page } from "$app/stores";
  import UtilitiesNavigation from "$lib/domains/utilities/components/UtilitiesNavigation.svelte";
  import ShellSidebarLayout from "$lib/components/shell/shell-sidebar-layout.svelte";
  import PageContainer from "$lib/components/shell/page-container.svelte";
  import { Card } from "$lib/components/ui/card";
  import { Wrench } from "@lucide/svelte";

  let { children }: { children: Snippet<[]> } = $props();

  let currentSection = $derived(() => {
    const path = $page.url.pathname;
    if (path.startsWith("/utilities/environment")) return "environment" as const;
    if (path.startsWith("/utilities/disk")) return "disk" as const;
    return "disk" as const;
  });
</script>

<svelte:head>
  <title>Utilities - Portal Desktop</title>
</svelte:head>

<div class="flex h-full min-h-0 w-full flex-col overflow-hidden">
  <div
    class="divider-edge-b divider-edge-full flex-shrink-0 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
  >
    <div class="flex flex-col gap-3 px-4 py-4 md:flex-row md:items-center md:justify-between md:px-6 3xl:px-8">
      <div class="min-w-0">
        <h1 class="flex items-center gap-2 text-2xl font-bold tracking-tight">
          <Wrench class="h-6 w-6" />
          Utilities
        </h1>
        <p class="mt-1 text-sm text-muted-foreground">
          System tools for disk cleanup, environment management, and more
        </p>
      </div>
    </div>
  </div>

  <ShellSidebarLayout mobileTriggerLabel="Utilities menu">
    {#snippet sidebar()}
      <div class="p-4">
        <Card class="p-3">
          <UtilitiesNavigation currentSection={currentSection()} />
        </Card>
      </div>
    {/snippet}

    <div class="min-h-0 flex-1 overflow-y-auto">
      <PageContainer variant="readable" class="py-6">
        {@render children()}
      </PageContainer>
    </div>
  </ShellSidebarLayout>
</div>
