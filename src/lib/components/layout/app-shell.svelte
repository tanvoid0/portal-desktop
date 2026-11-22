<!--
	App shell — main layout rendered inside QueryClientProvider
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/state';
  import { afterNavigate } from '$app/navigation';
  import type { Snippet } from 'svelte';
  import { IsMobile } from '$lib/hooks/is-mobile.svelte.js';
  import { createDashboardOverviewQuery } from '$lib/domains/dashboard/queries/dashboardQueries';
  import { logger, themeStore, resolvedTheme } from '$lib/domains/shared';
  import { breadcrumbItems } from '$lib/domains/shared/stores/breadcrumbStore';
  import { terminalActions } from '$lib/domains/terminal';
  import { learningService } from '$lib/domains/learning';
  import { settingsActions } from '$lib/domains/settings/stores/settingsStore';
  import ToastContainer from '$lib/components/ui/toast-container.svelte';
  import ConfirmDialog from '$lib/components/ui/confirm-dialog.svelte';
  import UpdatePrompt from '$lib/domains/updates/components/UpdatePrompt.svelte';
  import PairRequestGate from '$lib/domains/devices/components/PairRequestGate.svelte';
  import VaultGate from '$lib/domains/credentials/components/VaultGate.svelte';
  import { vaultGate } from '$lib/domains/credentials/state/vault.svelte';
  import { runStartupUpdateCheck } from '$lib/domains/updates/services/startupUpdate';
  import { getCurrentVersion } from '$lib/domains/updates/services/updateService';
  import { getIdentifier } from '@tauri-apps/api/app';
  import DeviceAuthGuard from '$lib/components/DeviceAuthGuard.svelte';
  import AppTopBar from './app-top-bar.svelte';
  import BackgroundJobsTray from './BackgroundJobsTray.svelte';
  import AppSidebarHeader from './app-sidebar-header.svelte';
  import AppSidebarFooter from './app-sidebar-footer.svelte';
  import { startAppNotifications } from './appNotifications';
  import InterruptedMoveBanner from '$lib/domains/projects/components/InterruptedMoveBanner.svelte';
  import { Provider as SidebarProvider, Sidebar as SidebarRoot } from '$lib/components/ui/sidebar';
  import { SIDEBAR_COOKIE_NAME } from '$lib/components/ui/sidebar/constants';
  import type { NavSection } from '$lib/components/shell/nav-types';
  import NavSectionList from '$lib/components/shell/nav-section-list.svelte';
  import PageContainer from '$lib/components/shell/page-container.svelte';
  import { buildMainNavSections } from '$lib/config/main-nav';
  import AgentApprovalStack from '$lib/domains/agents/components/AgentApprovalStack.svelte';
  import { isMobileShell } from '$lib/domains/mobile/shell';
  import { mobileConnection } from '$lib/domains/mobile/connection.svelte';
  import { filterNavForCaps } from '$lib/domains/mobile/navCaps';
  import {
    isViewportFillRoute,
    isShellSidebarRoute,
    isMainSidebarHidden as shouldHideMainSidebar,
  } from '$lib/config/layout-breakpoints';

  const log = logger.createScoped('AppLayout');

  // Get children snippet from props for Svelte 5
  let { children }: { children: Snippet<[]> } = $props();

  let isSdkPage = $derived(page.url.pathname.startsWith('/sdk'));

  const dashboardQuery = createDashboardOverviewQuery(() => ({
    enabled: !isSdkPage,
  }));

  const dashboardOverview = $derived(dashboardQuery.data ?? null);

  let hideMainSidebar = $derived(shouldHideMainSidebar(page.url.pathname));
  let isFillViewport = $derived(isViewportFillRoute(page.url.pathname));
  let isDomainLayout = $derived(isShellSidebarRoute(page.url.pathname));
  const isMobileViewport = new IsMobile();
  // Starts collapsed on a phone-width viewport: the sidebar context mirrors
  // this `open` value into the mobile Sheet's open state at construction, so
  // defaulting to `true` meant the nav sidebar auto-opened as a full-page
  // overlay on first mobile visit, before anyone touched it.
  let sidebarOpen = $state(!isMobileViewport.current);

  // Main application navigation sections — derived from dashboard query cache.
  const navigationSections = $derived<NavSection[]>(
    isMobileShell
      ? filterNavForCaps(
          buildMainNavSections(dashboardOverview),
          mobileConnection.status?.caps ?? []
        )
      : buildMainNavSections(dashboardOverview)
  );

  let unsubscribe: (() => void) | undefined;
  let stopNotifications: (() => void) | undefined;

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

  // Build identity for the sidebar header. Debug builds run as `<identifier>.dev`
  // with their own data (portal-core `app_paths::DEV`), so say so loudly.
  let appVersion = $state<string | null>(null);
  let isDebugBuild = $state(false);

  onMount(() => {
    getCurrentVersion().then((v) => (appVersion = v));
    getIdentifier()
      .then((id) => (isDebugBuild = id.endsWith('.dev')))
      .catch((error) => log.warn('App identifier unavailable', { error }));
  });

  onMount(() => {
    vaultGate.promptIfLocked().catch((error) => log.warn('Vault startup check failed', { error }));
  });

  onMount(async () => {
    try {
      log.info('Initializing application');

      // Initialize sidebar open/closed state from cookie. Skipped on mobile:
      // the cookie is a desktop preference, and applying it there would
      // reopen the full-page overlay on revisit whenever it was last left
      // open on desktop.
      const cookieOpen = readSidebarCookieOpen();
      // Always show sidebar on the app root (`/`) regardless of persisted state.
      // This prevents the sidebar from appearing "missing" when users previously
      // closed it and then return to `/` (especially on mobile/offcanvas mode).
      if (!isMobileViewport.current && page.url.pathname !== '/' && cookieOpen !== null) {
        sidebarOpen = cookieOpen;
      }
      // Initialize backend log listener to receive logs from Rust backend
      logger.initBackendLogListener();

      stopNotifications = startAppNotifications();

      // Initialize theme first (now synchronous)
      themeStore.initialize();

      // Load saved settings so custom theme colors apply on startup
      if (!isMobileShell) void settingsActions.loadSettings().catch(() => {});

      // Check for a new release; prompts (or auto-installs, per settings) when one exists
      if (!isMobileShell) void runStartupUpdateCheck();

      // Initialize learning service (should be early to start collecting patterns)
      if (!isMobileShell) await learningService.initialize();

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

  onDestroy(() => {
    unsubscribe?.();
    stopNotifications?.();
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
    <div
      class="flex h-screen min-h-0 w-full flex-col overflow-hidden {isMobileShell
        ? 'pt-[env(safe-area-inset-top)] pr-[env(safe-area-inset-right)] pb-[env(safe-area-inset-bottom)] pl-[env(safe-area-inset-left)]'
        : ''}"
    >
      <AppTopBar />

      {#if !isMobileShell}
        <BackgroundJobsTray />
      {/if}

      {#if !isMobileShell}
        <InterruptedMoveBanner />
      {/if}

      <!-- Main Content Area -->
      <div class="flex h-full min-h-0 w-full flex-1 overflow-hidden">
        <!-- Sidebar Navigation (nested route groups manage their own sidebars) -->
        {#if !hideMainSidebar}
          <SidebarRoot collapsible="icon">
            <div class="flex h-full min-h-0 min-w-0 flex-col">
              <AppSidebarHeader {isSdkPage} {isDebugBuild} {appVersion} />

              <!-- Sidebar Content — nav renders immediately; the dashboard
                   query only supplies badge counts, so there is nothing to
                   wait for. -->
              <div class="min-h-0 flex-1 overflow-y-auto">
                <NavSectionList sections={navigationSections} currentPath={page.url.pathname} />
              </div>

              <AppSidebarFooter />
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

  <!-- Floating approval stack for agent runs not currently in view -->
  <AgentApprovalStack />

  <!-- Launch-time update prompt -->
  {#if !isMobileShell}
    <UpdatePrompt />

    <!-- Vault setup/unlock gate -->
    <VaultGate />

    <!-- Phone pairing request: Allow / Deny right here -->
    <PairRequestGate />
  {/if}
</DeviceAuthGuard>
