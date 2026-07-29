<!-- AI Layout - Home/Code tabs; session lists live in each page -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { afterNavigate } from '$app/navigation';
  import AITabBar from '$lib/domains/ai/components/navigation/AITabBar.svelte';
  import { setBreadcrumbs, clearBreadcrumbs } from '$lib/domains/shared/stores/breadcrumbStore';
  import { getAiTabBreadcrumb, isAiSectionRoute } from '$lib/config/ai-tabs';
  import { AI_CHAT_PATH } from '$lib/config/ai-nav';
  import { aiTopbar } from '$lib/domains/ai/state/aiTopbarStore.svelte.js';

  let { children }: { children: Snippet<[]> } = $props();

  function syncAiBreadcrumbs(pathname: string) {
    if (!isAiSectionRoute(pathname)) return;
    const tabCrumb = getAiTabBreadcrumb(pathname);
    setBreadcrumbs([{ label: 'AI', href: AI_CHAT_PATH, icon: 'sparkles' }, tabCrumb]);
  }

  afterNavigate(({ to, from }) => {
    if (to?.url.pathname && isAiSectionRoute(to.url.pathname)) {
      syncAiBreadcrumbs(to.url.pathname);
      return;
    }
    if (from?.url.pathname && isAiSectionRoute(from.url.pathname)) {
      clearBreadcrumbs();
    }
  });
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <!-- Pages park their action controls here so their own header stays readable. -->
  <div
    class="divider-edge-b divider-edge-full flex shrink-0 items-center gap-3 bg-background px-3 py-2 md:px-4"
  >
    <AITabBar class="w-[220px] shrink-0" />
    {#if aiTopbar.actions}
      <div class="flex min-w-0 flex-1 flex-wrap items-center justify-end gap-1.5">
        {@render aiTopbar.actions()}
      </div>
    {/if}
  </div>
  <div class="min-h-0 flex-1 overflow-hidden">
    {@render children()}
  </div>
</div>
