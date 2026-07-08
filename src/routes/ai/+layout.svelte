<!-- AI Layout - Handles sidebar navigation for all AI subpages -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import ShellSidebarLayout from "$lib/components/shell/shell-sidebar-layout.svelte";
  import AINavigation from "$lib/domains/ai/components/navigation/AINavigation.svelte";
  import { isViewportFillRoute } from "$lib/config/layout-breakpoints";
  import { page } from "$app/stores";

  let { children }: { children: Snippet<[]> } = $props();

  let isFillViewport = $derived(isViewportFillRoute($page.url.pathname));
</script>

<ShellSidebarLayout mobileTriggerLabel="AI menu">
  {#snippet sidebar()}
    <div class="p-4">
      <AINavigation />
    </div>
  {/snippet}

  {#if isFillViewport}
    <div class="flex h-full min-h-0 flex-col overflow-hidden">
      {@render children()}
    </div>
  {:else}
    <div class="min-h-0 flex-1 overflow-y-auto">
      {@render children()}
    </div>
  {/if}
</ShellSidebarLayout>
