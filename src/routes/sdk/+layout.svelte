<!--
	SDK Layout - Sidebar + Main Container
	Provides consistent layout for all SDK pages
-->

<script lang="ts">
  import type { Snippet } from "svelte";
  import SDKSidebar from "$lib/domains/sdk/components/SDKSidebar.svelte";
  import ShellSidebarLayout from "$lib/components/shell/shell-sidebar-layout.svelte";
  import PageContainer from "$lib/components/shell/page-container.svelte";
  import { page } from "$app/stores";

  let { children }: { children: Snippet<[]> } = $props();

  let currentPath = $derived($page.url.pathname);
  const navigationItemIds = ["/sdk"];
</script>

<ShellSidebarLayout
  sidebarClass="flex h-full min-h-0 flex-col"
  mobileTriggerLabel="SDK menu"
>
  {#snippet sidebar()}
    <SDKSidebar
      selectedSDK={currentPath.split("/").pop() || undefined}
      {navigationItemIds}
    />
  {/snippet}
  <div class="min-h-0 flex-1 overflow-y-auto bg-background">
    <PageContainer variant="full" class="py-4 md:py-6">
      {@render children()}
    </PageContainer>
  </div>
</ShellSidebarLayout>
