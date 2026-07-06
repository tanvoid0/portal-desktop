<!--
	SDK Layout - Sidebar + Main Container
	Provides consistent layout for all SDK pages
-->

<script lang="ts">
  import type { Snippet } from "svelte";
  import SDKSidebar from "$lib/domains/sdk/components/SDKSidebar.svelte";
  import ShellSidebarLayout from "$lib/components/shell/shell-sidebar-layout.svelte";
  import { page } from "$app/stores";

  // Get children snippet from props for Svelte 5
  let { children }: { children: Snippet<[]> } = $props();

  // Get current path for sidebar selection
  let currentPath = $derived($page.url.pathname);

  // Keep the sidebar reusable: pass the global navigation config.
  // For the SDK section we only show a single entry in the navigation list.
  const navigationItemIds = ["/sdk"];
</script>

<ShellSidebarLayout
  contentClass="flex h-full min-h-0 w-full overflow-hidden"
  sidebarClass="flex h-full min-h-0 flex-col"
  mainClass="min-h-0 min-w-0 flex-1 overflow-y-auto bg-background"
>
  {#snippet sidebar()}
    <div class="flex h-full min-h-0 flex-col">
      <SDKSidebar
        selectedSDK={currentPath.split("/").pop() || undefined}
        navigationItemIds={navigationItemIds}
      />
    </div>
  {/snippet}
  {@render children()}
</ShellSidebarLayout>
