<!-- AI Layout - Handles sidebar navigation for all AI subpages -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import ShellSidebarLayout from "$lib/components/shell/shell-sidebar-layout.svelte";
  import AINavigation from "$lib/domains/ai/components/navigation/AINavigation.svelte";

  let { children }: { children: Snippet<[]> } = $props();
</script>

<!--
  ShellSidebarLayout expects a flex content wrapper so the sidebar + main render side-by-side.
  Without it, the main area can be clipped/hidden due to overflow constraints.
-->
<ShellSidebarLayout
  contentClass="flex h-full min-h-0 w-full overflow-hidden"
  mainClass="min-h-0 min-w-0 flex-1 overflow-hidden"
>
  {#snippet sidebar()}
    <div class="min-h-0 flex-1 overflow-y-auto p-4">
      <AINavigation />
    </div>
  {/snippet}

  {@render children()}
</ShellSidebarLayout>
