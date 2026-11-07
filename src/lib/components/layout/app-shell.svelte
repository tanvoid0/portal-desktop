<!--
	App shell — main layout rendered inside QueryClientProvider
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/state';
  import { goto, afterNavigate } from '$app/navigation';
  import type { Snippet } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import ThemeToggle from '$lib/components/ui/theme-toggle.svelte';
  import Breadcrumb from '$lib/components/ui/breadcrumb.svelte';
  import { createDashboardOverviewQuery } from '$lib/domains/dashboard/queries/dashboardQueries';
  import { logger, themeStore, resolvedTheme } from '$lib/domains/shared';
  import {
    breadcrumbItems,
    breadcrumbSettings,
    homeItem,
    showHome,
  } from '$lib/domains/shared/stores/breadcrumbStore';
  import { terminalActions } from '$lib/domains/terminal';
  import { learningService } from '$lib/domains/learning';
  import { settingsActions } from '$lib/domains/settings/stores/settingsStore';
  import ToastContainer from '$lib/components/ui/toast-container.svelte';
  import ConfirmDialog from '$lib/components/ui/confirm-dialog.svelte';
  import QRCodeDialog from '$lib/components/QRCodeDialog.svelte';
  import DeviceApprovalDialog from '$lib/components/DeviceApprovalDialog.svelte';
  import DeviceAuthGuard from '$lib/components/DeviceAuthGuard.svelte';
  import { isTauriEnvironment } from '$lib/utils/tauri';
  import { InvokeClient } from '$lib/utils/invokeClient';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import ArrowRight from '~icons/fa6-solid/arrow-right';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import QrCode from '~icons/fa6-solid/qrcode';
  import Shield from '~icons/fa6-solid/shield';
  import Gear from '~icons/fa6-solid/gear';
  import {
    Provider as SidebarProvider,
    Trigger as SidebarTrigger,
    Sidebar as SidebarRoot,
  } from '$lib/components/ui/sidebar';
  import { SIDEBAR_COOKIE_NAME } from '$lib/components/ui/sidebar/constants';
  import type { NavSection } from '$lib/components/shell/nav-types';
  import NavSectionList from '$lib/components/shell/nav-section-list.svelte';
  import SidebarTooltip from '$lib/components/shell/sidebar-tooltip.svelte';
  import PageContainer from '$lib/components/shell/page-container.svelte';
  import { buildMainNavSections } from '$lib/config/main-nav';
  import {
    isViewportFillRoute,
    isShellSidebarRoute,
    isMainSidebarHidden as shouldHideMainSidebar,
  } from '$lib/config/layout-breakpoints';

  const log = logger.createScoped('AppLayout');

  // Get children snippet from props for Svelte 5
  let { children }: { children: Snippet<[]> } = $props();

  // Show QR code button in Tauri environment or when accessed via network IP (not localhost)
  let isTauri = isTauriEnvironment();
  let isLocalhost = $state(false);
  let showQRButton = $derived(isTauri || !isLocalhost);

  // Check if accessing from localhost
  $effect(() => {
    isLocalhost = InvokeClient.isLocalhost();
  });

  let isSdkPage = $derived(page.url.pathname.startsWith('/sdk'));

  const dashboardQuery = createDashboardOverviewQuery(() => ({
    enabled: !isSdkPage,
  }));

  const dashboardOverview = $derived(dashboardQuery.data ?? null);

  let hideMainSidebar = $derived(shouldHideMainSidebar(page.url.pathname));
  let isFillViewport = $derived(isViewportFillRoute(page.url.pathname));
  let isDomainLayout = $derived(isShellSidebarRoute(page.url.pathname));
  let sidebarOpen = $state(true);
  let qrCodeDialogOpen = $state(false);
  let deviceApprovalDialogOpen = $state(false);

  // Main application navigation sections — derived from dashboard query cache.
  const navigationSections = $derived<NavSection[]>(buildMainNavSections(dashboardOverview));

  let unsubscribe: (() => void) | undefined;
  let popStateHandler: (() => void) | undefined;

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

  function handleSidebarOpenChange(open: boolean) {
    sidebarOpen = open;
  }

  function readSidebarCookieOpen(): boolean | null {
    if (typeof document === 'undefined') return null;
    const match = document.cookie.match(new RegExp(`(?:^|; )${SIDEBAR_COOKIE_NAME}=([^;]*)`));
    if (!match) return null;
    const raw = match[1];
    if (raw === 'true') return true;
    if (raw === 'false') return false;
    return null;
  }

  onMount(async () => {
    try {
      log.info('Initializing application');

      // Initialize sidebar open/closed state from cookie.
      const cookieOpen = readSidebarCookieOpen();
      // Always show sidebar on the app root (`/`) regardless of persisted state.
      // This prevents the sidebar from appearing "missing" when users previously
      // closed it and then return to `/` (especially on mobile/offcanvas mode).
      if (page.url.pathname !== '/' && cookieOpen !== null) {
        sidebarOpen = cookieOpen;
      }
      // Initialize backend log listener to receive logs from Rust backend
      logger.initBackendLogListener();

      // Initialize navigation state
      updateNavigationState();

      // Listen to popstate events to track forward/back navigation
      if (typeof window !== 'undefined') {
        popStateHandler = updateNavigationState;
        window.addEventListener('popstate', popStateHandler);
      }

      // Initialize theme first (now synchronous)
      themeStore.initialize();

      // Load saved settings so custom theme colors apply on startup
      void settingsActions.loadSettings().catch(() => {});

      // Initialize learning service (should be early to start collecting patterns)
      await learningService.initialize();

      // Dashboard overview loads via TanStack Query (skipped on /sdk/* routes).

      // Sync terminal theme with global theme
      unsubscribe = resolvedTheme.subscribe((theme) => {
        terminalActions.updateSettings({ theme });
      });

      log.info('Application initialized successfully');
    } catch (error) {
      log.error('Failed to initialize application', error);
    }
  });

  // The app scrolls inside <main>, not the window, so SvelteKit's own scroll reset never
  // reaches it. Reset on forward navigation to a different path; back/forward (popstate) and
  // same-path navigations (tab switches with reset:false) keep position, and so do sidebars.
  let mainScroller = $state<HTMLElement | null>(null);
  afterNavigate(({ type, from, to }) => {
    if (type === 'popstate' || !from?.url || !to?.url || from.url.pathname === to.url.pathname)
      return;
    mainScroller?.querySelectorAll('*').forEach((el) => {
      if (el.scrollTop > 0 && !el.closest('[data-sidebar]')) el.scrollTop = 0;
    });
  });

  // Update navigation state when page changes
  $effect(() => {
    void page.url.href;
    updateNavigationState();
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
    if (typeof window !== 'undefined' && popStateHandler) {
      window.removeEventListener('popstate', popStateHandler);
    }
  });
</script>

<!-- Fallback title for routes that set no <title> of their own (the last breadcrumb names the page). -->
<svelte:head>
  <title>{$breadcrumbItems.at(-1)?.label ?? 'Home'} - Portal Desktop</title>
</svelte:head>

<DeviceAuthGuard>
  <!-- Skip link: first Tab stop, jumps past the header and sidebar into the page. -->
  <a
    href="#main-content"
    class="sr-only z-50 rounded-md bg-background px-3 py-2 text-sm font-medium text-foreground shadow focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
    onclick={(e) => {
      e.preventDefault();
      mainScroller?.focus();
    }}
  >
    Skip to content
  </a>
  <SidebarProvider open={sidebarOpen} onOpenChange={handleSidebarOpenChange}>
    <div class="flex h-screen min-h-0 w-full flex-col overflow-hidden">
      <!-- Top Bar -->
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
            class="flex-1"
          />
        </div>
      </header>

      <!-- Main Content Area -->
      <div class="flex h-full min-h-0 w-full flex-1 overflow-hidden">
        <!-- Sidebar Navigation (nested route groups manage their own sidebars) -->
        {#if !hideMainSidebar}
          <SidebarRoot collapsible="icon">
            <div class="flex h-full min-h-0 min-w-0 flex-col">
              <!-- Sidebar Header -->
              <SidebarTooltip label={isSdkPage ? 'SDK Manager' : 'Portal Desktop'}>
                {#snippet trigger({ props })}
                  <Button
                    {...props}
                    type="button"
                    variant="ghost"
                    class="divider-edge-b divider-edge-full divider-edge-sidebar h-auto w-full justify-start gap-3 px-4 py-4 text-left transition-colors group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:gap-0 hover:bg-sidebar-accent/50"
                    onclick={() => goto('/')}
                  >
                    <div
                      class="flex h-8 w-8 items-center justify-center rounded-md bg-primary/10 text-primary"
                    >
                      {#if isSdkPage}
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
                          />
                        </svg>
                      {:else}
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                          />
                        </svg>
                      {/if}
                    </div>
                    <div class="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
                      <span class="block truncate text-sm font-semibold text-sidebar-foreground">
                        {isSdkPage ? 'SDK Manager' : 'Portal Desktop'}
                      </span>
                      <span class="block text-xs text-sidebar-foreground/60">
                        {isSdkPage ? 'Development Tools' : 'Development Environment'}
                      </span>
                    </div>
                  </Button>
                {/snippet}
              </SidebarTooltip>

              <!-- Sidebar Content — nav renders immediately; the dashboard
                   query only supplies badge counts, so there is nothing to
                   wait for. -->
              <div class="min-h-0 flex-1 overflow-y-auto">
                <NavSectionList sections={navigationSections} currentPath={page.url.pathname} />
              </div>

              <!-- Sidebar Footer — stacks into a single icon column when collapsed,
                   otherwise the row overflows the 3rem rail and clips the buttons. -->
              <div
                class="divider-edge-t divider-edge-full divider-edge-sidebar flex-shrink-0 p-4 group-data-[collapsible=icon]:p-2"
              >
                <div
                  class="flex items-center justify-between gap-2 group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:gap-1"
                >
                  <ThemeToggle />
                  <div
                    class="flex items-center gap-1 group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:gap-1"
                  >
                    {#if showQRButton}
                      <SidebarTooltip label="Share via QR Code">
                        {#snippet trigger({ props })}
                          <Button
                            {...props}
                            variant="ghost"
                            size="sm"
                            class="h-8 w-8 px-0"
                            onclick={() => (qrCodeDialogOpen = true)}
                            aria-label="Share via QR Code"
                          >
                            <QrCode class="h-4 w-4" />
                          </Button>
                        {/snippet}
                      </SidebarTooltip>
                      <SidebarTooltip label="Device approval requests">
                        {#snippet trigger({ props })}
                          <Button
                            {...props}
                            variant="ghost"
                            size="sm"
                            class="h-8 w-8 px-0"
                            onclick={() => (deviceApprovalDialogOpen = true)}
                            aria-label="Device approval requests"
                          >
                            <Shield class="h-4 w-4" />
                          </Button>
                        {/snippet}
                      </SidebarTooltip>
                    {/if}
                    <SidebarTooltip label="Settings">
                      {#snippet trigger({ props })}
                        <Button
                          {...props}
                          variant="ghost"
                          size="sm"
                          class="h-8 w-8 px-0"
                          onclick={() => goto('/settings')}
                          aria-label="Settings"
                        >
                          <Gear class="h-4 w-4" />
                        </Button>
                      {/snippet}
                    </SidebarTooltip>
                  </div>
                </div>
              </div>
            </div>
          </SidebarRoot>
        {/if}

        <!-- Page Content -->
        <main
          id="main-content"
          tabindex="-1"
          bind:this={mainScroller}
          class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-background outline-none"
        >
          {#if isFillViewport}
            {@render children()}
          {:else if isDomainLayout}
            <div class="flex h-full min-h-0 flex-1 flex-col overflow-hidden">
              {@render children()}
            </div>
          {:else}
            <div class="h-full min-h-0 flex-1 overflow-y-auto">
              <PageContainer variant="full" class="py-4 md:py-6">
                {@render children()}
              </PageContainer>
            </div>
          {/if}
        </main>
      </div>
    </div>
  </SidebarProvider>

  <!-- Toast Container -->
  <ToastContainer />

  <!-- Global confirmation dialog -->
  <ConfirmDialog />

  <!-- QR Code Dialog -->
  <QRCodeDialog bind:open={qrCodeDialogOpen} />

  <!-- Device Approval Dialog -->
  {#if showQRButton}
    <DeviceApprovalDialog bind:open={deviceApprovalDialogOpen} />
  {/if}
</DeviceAuthGuard>
