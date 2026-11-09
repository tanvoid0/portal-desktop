<!--
	SDK Layout - Sidebar + Main Container
	Provides consistent layout for all SDK pages
-->

<script lang="ts">
  import type { Snippet } from 'svelte';
  import { setContext } from 'svelte';
  import { createQuery } from '@tanstack/svelte-query';
  import SDKSidebar from '$lib/domains/sdk/components/SDKSidebar.svelte';
  import ShellSidebarLayout from '$lib/components/shell/shell-sidebar-layout.svelte';
  import PageContainer from '$lib/components/shell/page-container.svelte';
  import { page } from '$app/state';
  import {
    sdkConfigsQuery,
    sdkManagersQuery,
    SDK_QUERIES_CONTEXT,
    type SdkQueries,
  } from '$lib/domains/sdk/queries/sdkQueries';

  let { children }: { children: Snippet<[]> } = $props();

  // Created once here, for the lifetime of the whole /sdk section — see SDK_QUERIES_CONTEXT's
  // doc comment for why this must not be duplicated per-page.
  const configsQuery = createQuery(sdkConfigsQuery);
  const managersQuery = createQuery(sdkManagersQuery);
  setContext<SdkQueries>(SDK_QUERIES_CONTEXT, { configsQuery, managersQuery });

  let currentPath = $derived(page.url.pathname);
  const navigationItemIds = ['/sdk', '/sdk/managers', '/sdk/services', '/sdk/versions'];
  // The last path segment is only an SDK/manager id on a route like /sdk/python or
  // /sdk/managers/nvm — on the nav pages themselves (/sdk, /sdk/managers, ...) it's one of
  // these keywords, which matches no sidebar item and must not be treated as "selected".
  const NAV_KEYWORDS = new Set(['sdk', 'managers', 'services', 'versions']);
  const selectedSDK = $derived.by(() => {
    const segments = currentPath.split('/').filter(Boolean);
    const last = segments.at(-1);
    return last && !NAV_KEYWORDS.has(last) ? last : undefined;
  });
</script>

<ShellSidebarLayout sidebarClass="flex h-full min-h-0 flex-col" mobileTriggerLabel="SDK menu">
  {#snippet sidebar()}
    <SDKSidebar {selectedSDK} {navigationItemIds} />
  {/snippet}
  <div class="min-h-0 flex-1 overflow-y-auto bg-background">
    <PageContainer variant="full" class="py-4 md:py-6">
      {@render children()}
    </PageContainer>
  </div>
</ShellSidebarLayout>
