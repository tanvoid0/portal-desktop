<!-- App header: sidebar trigger, back/forward/refresh buttons and breadcrumb. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { Button } from '$lib/components/ui/button';
  import Breadcrumb from '$lib/components/ui/breadcrumb.svelte';
  import { breadcrumbItems, homeItem, showHome } from '$lib/domains/shared/stores/breadcrumbStore';
  import { Trigger as SidebarTrigger } from '$lib/components/ui/sidebar';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import ArrowRight from '~icons/fa6-solid/arrow-right';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Bell from '~icons/fa6-solid/bell';
  import { Badge } from '$lib/components/ui/badge';
  import { notificationInbox } from '$lib/domains/notifications/state/notificationInbox.svelte';

  const unread = $derived(notificationInbox.unread);
  const bellLabel = $derived(unread ? `Notifications, ${unread} unread` : 'Notifications');

  // Browser navigation state
  let canGoBack = $state(false);
  let canGoForward = $state(false);

  function updateNavigationState() {
    if (typeof window !== 'undefined') {
      // Navigation API (Chromium/WebView2) reports real back/forward availability
      const nav = (
        window as unknown as { navigation?: { canGoBack: boolean; canGoForward: boolean } }
      ).navigation;
      canGoBack = nav ? nav.canGoBack : window.history.length > 1;
      canGoForward = nav?.canGoForward ?? false;
    }
  }

  function goBack() {
    if (typeof window !== 'undefined' && canGoBack) {
      window.history.back();
    }
  }

  function goForward() {
    if (typeof window !== 'undefined' && canGoForward) {
      window.history.forward();
    }
  }

  function refresh() {
    if (typeof window !== 'undefined') {
      window.location.reload();
    }
  }

  // Listen to popstate events to track forward/back navigation
  onMount(() => {
    updateNavigationState();
    window.addEventListener('popstate', updateNavigationState);
    return () => window.removeEventListener('popstate', updateNavigationState);
  });

  // Update navigation state when page changes
  $effect(() => {
    void page.url.href;
    updateNavigationState();
  });
</script>

<header
  class="divider-edge-b divider-edge-full flex-shrink-0 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
>
  <div class="flex h-10 items-center gap-2 px-3 md:px-4 3xl:px-6">
    <SidebarTrigger />
    <!-- Navigation Buttons -->
    <div class="divider-edge-r mr-2 flex items-center gap-1 pr-2">
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7"
        disabled={!canGoBack}
        onclick={goBack}
        title="Go back"
        aria-label="Go back"
      >
        <ArrowLeft class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7"
        disabled={!canGoForward}
        onclick={goForward}
        title="Go forward"
        aria-label="Go forward"
      >
        <ArrowRight class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7"
        onclick={refresh}
        title="Refresh page"
        aria-label="Refresh page"
      >
        <RefreshCw class="h-3.5 w-3.5" />
      </Button>
    </div>
    <Breadcrumb
      items={$breadcrumbItems}
      showHome={$showHome}
      homeItem={$homeItem}
      class="min-w-0 flex-1 overflow-hidden"
    />
    <Button
      href="/notifications"
      variant="ghost"
      size="icon"
      class="relative h-7 w-7 shrink-0"
      title={bellLabel}
      aria-label={bellLabel}
    >
      <Bell class="h-3.5 w-3.5" />
      {#if unread}
        <Badge
          variant="destructive"
          class="absolute -top-0.5 -right-1.5 h-4 min-w-4 justify-center rounded-full px-1 py-0 text-[10px] leading-none tabular-nums"
        >
          {unread > 99 ? '99+' : unread}
        </Badge>
      {/if}
    </Button>
  </div>
</header>
