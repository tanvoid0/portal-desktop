<!--
	App shell — main layout rendered inside QueryClientProvider
-->

<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import ThemeToggle from "$lib/components/ui/theme-toggle.svelte";
  import Breadcrumb from "$lib/components/ui/breadcrumb.svelte";
  import { createDashboardOverviewQuery } from "$lib/domains/dashboard/queries/dashboardQueries";
  import { logger, themeStore, resolvedTheme } from "$lib/domains/shared";
  import {
    breadcrumbItems,
    breadcrumbSettings,
    homeItem,
    showHome,
  } from "$lib/domains/shared/stores/breadcrumbStore";
  import { terminalActions } from "$lib/domains/terminal/stores/terminalStore";
  import { learningService } from "$lib/domains/learning";
  import { settingsActions } from "$lib/domains/settings/stores/settingsStore";
  import ToastContainer from "$lib/components/ui/toast-container.svelte";
  import QRCodeDialog from "$lib/components/QRCodeDialog.svelte";
  import DeviceApprovalDialog from "$lib/components/DeviceApprovalDialog.svelte";
  import DeviceAuthGuard from "$lib/components/DeviceAuthGuard.svelte";
  import FloatingAvatar from "$lib/components/ai/FloatingAvatar.svelte";
  import { isTauriEnvironment } from "$lib/utils/tauri";
  import { InvokeClient } from "$lib/utils/invokeClient";
  import {
    ArrowLeft,
    ArrowRight,
    RefreshCw,
    Home,
    QrCode,
    Shield,
  } from "@lucide/svelte";
  import {
    Provider as SidebarProvider,
    Trigger as SidebarTrigger,
    Sidebar as SidebarRoot,
  } from "$lib/components/ui/sidebar";
  import { SIDEBAR_COOKIE_NAME } from "$lib/components/ui/sidebar/constants";
  import type { NavSection } from "$lib/components/shell/nav-types";
  import NavSectionList from "$lib/components/shell/nav-section-list.svelte";
  import { buildMainNavSections } from "$lib/config/main-nav";

  const log = logger.createScoped("AppLayout");

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

  let navigationLoading = $state(false);
  let navigationError: string | null = $state(null);
  let isSdkPage = $derived($page.url.pathname.startsWith("/sdk"));

  const dashboardQuery = createDashboardOverviewQuery(() => ({
    enabled: !isSdkPage,
  }));

  const dashboardOverview = $derived(dashboardQuery.data ?? null);

  let isMainSidebarHidden = $derived(() => {
    const path = $page.url.pathname;
    // Nested route groups manage their own sidebars.
    return (
      path.startsWith("/ai") ||
      path.startsWith("/cloud") ||
      path.startsWith("/settings") ||
      path.startsWith("/sdk")
    );
  });
  let sidebarOpen = $state(true);
  let qrCodeDialogOpen = $state(false);
  let deviceApprovalDialogOpen = $state(false);

  // Main application navigation sections — derived from dashboard query cache.
  const navigationSections = $derived<NavSection[]>(
    buildMainNavSections(dashboardOverview),
  );

  let unsubscribe: (() => void) | undefined;
  let popStateHandler: (() => void) | undefined;

  // Browser navigation state
  let canGoBack = $state(false);
  let canGoForward = $state(false);
  let hasNavigatedBack = $state(false);

  function updateNavigationState() {
    if (typeof window !== "undefined") {
      // Check if we can go back (history has more than 1 entry)
      canGoBack = window.history.length > 1;
    }
  }

  function goBack() {
    if (typeof window !== "undefined" && canGoBack) {
      hasNavigatedBack = true;
      canGoForward = true;
      window.history.back();
    }
  }

  function goForward() {
    if (typeof window !== "undefined" && canGoForward) {
      window.history.forward();
    }
  }

  function goHome() {
    hasNavigatedBack = false;
    canGoForward = false;
    goto("/");
  }

  function refresh() {
    if (typeof window !== "undefined") {
      window.location.reload();
    }
  }

  function handleSidebarOpenChange(open: boolean) {
    sidebarOpen = open;
  }

  // #region agent log
  let lastSidebarRectKey = "";
  $effect(() => {
    if (typeof document === "undefined") return;
    const path = $page.url.pathname;
    const key = `${path}:${sidebarOpen}:${isMainSidebarHidden()}`;
    if (key === lastSidebarRectKey) return;
    lastSidebarRectKey = key;

    const innerEl = document.querySelector(
      '[data-slot="sidebar-inner"]',
    ) as HTMLElement | null;
    const containerEl = document.querySelector(
      '[data-slot="sidebar-container"]',
    ) as HTMLElement | null;
    const wrapperEl = document.querySelector(
      '[data-slot="sidebar-wrapper"]',
    ) as HTMLElement | null;
    const sidebarEl = document.querySelector(
      '[data-slot="sidebar"]',
    ) as HTMLElement | null;
    const triggerEl = document.querySelector(
      '[data-slot="sidebar-trigger"]',
    ) as HTMLElement | null;

    const triggerRect = triggerEl ? triggerEl.getBoundingClientRect() : null;
    const sidebarRect = sidebarEl ? sidebarEl.getBoundingClientRect() : null;

    const isMobileMq =
      typeof window !== "undefined"
        ? window.matchMedia("(max-width: 767px)").matches
        : null;
    const rect = innerEl ? innerEl.getBoundingClientRect() : null;
    const computed = innerEl ? window.getComputedStyle(innerEl) : null;
    const sidebarDisplay = sidebarEl
      ? window.getComputedStyle(sidebarEl).display
      : null;
    const wrapperDisplay = wrapperEl
      ? window.getComputedStyle(wrapperEl).display
      : null;

    // Parse cookie value without triggering extra ingest calls.
    let cookieSidebar: boolean | null = null;
    try {
      const match = document.cookie.match(
        new RegExp(`(?:^|; )${SIDEBAR_COOKIE_NAME}=([^;]*)`),
      );
      if (match) {
        const raw = match[1];
        if (raw === "true") cookieSidebar = true;
        else if (raw === "false") cookieSidebar = false;
      }
    } catch {
      cookieSidebar = null;
    }

    // Surface useful information in the browser console too (line-by-line to avoid truncation).
    console.log("[SidebarDebug:summary]", {
      path,
      sidebarOpen,
      isMainSidebarHidden: isMainSidebarHidden(),
      cookieSidebar,
      isMobileMq,
    });
    console.log("[SidebarDebug:elements]", {
      foundInner: Boolean(innerEl),
      foundContainer: Boolean(containerEl),
      foundWrapper: Boolean(wrapperEl),
      foundSidebarEl: Boolean(sidebarEl),
      foundTriggerEl: Boolean(triggerEl),
    });
    console.log("[SidebarDebug:display]", {
      displayInner: computed?.display ?? null,
      displaySidebar: sidebarDisplay,
      displayWrapper: wrapperDisplay,
    });
    console.log("[SidebarDebug:rects]", {
      sidebarRect: sidebarRect
        ? {
            width: sidebarRect.width,
            height: sidebarRect.height,
            x: sidebarRect.x,
            y: sidebarRect.y,
          }
        : null,
      innerRect: rect
        ? {
            width: rect.width,
            height: rect.height,
            x: rect.x,
            y: rect.y,
          }
        : null,
      triggerRect: triggerRect
        ? {
            width: triggerRect.width,
            height: triggerRect.height,
            x: triggerRect.x,
            y: triggerRect.y,
          }
        : null,
    });

  });
  // #endregion

  function readSidebarCookieOpen(): boolean | null {
    if (typeof document === "undefined") return null;
    const match = document.cookie.match(
      new RegExp(`(?:^|; )${SIDEBAR_COOKIE_NAME}=([^;]*)`),
    );
    if (!match) return null;
    const raw = match[1];
    if (raw === "true") return true;
    if (raw === "false") return false;
    return null;
  }

  onMount(async () => {
    try {
      log.info("Initializing application");

      // Initialize sidebar open/closed state from cookie.
      const cookieOpen = readSidebarCookieOpen();
      // Always show sidebar on the app root (`/`) regardless of persisted state.
      // This prevents the sidebar from appearing "missing" when users previously
      // closed it and then return to `/` (especially on mobile/offcanvas mode).
      if ($page.url.pathname !== "/" && cookieOpen !== null) {
        sidebarOpen = cookieOpen;
      }
      // Initialize backend log listener to receive logs from Rust backend
      logger.initBackendLogListener();

      // Initialize navigation state
      updateNavigationState();

      // Listen to popstate events to track forward/back navigation
      if (typeof window !== "undefined") {
        popStateHandler = () => {
          updateNavigationState();
          // If we navigated to a new page (not via back/forward), disable forward
          if (!hasNavigatedBack) {
            canGoForward = false;
          }
        };
        window.addEventListener("popstate", popStateHandler);
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

      log.info("Application initialized successfully");
    } catch (error) {
      log.error("Failed to initialize application", error);
    }
  });

  // Update navigation state when page changes
  $effect(() => {
    updateNavigationState();
    // Reset forward state when navigating to a new page (not via back/forward)
    if (!hasNavigatedBack) {
      canGoForward = false;
    }
    // Reset hasNavigatedBack flag after navigation completes
    // This allows detecting new navigation vs back/forward
    const timeoutId = setTimeout(() => {
      if (hasNavigatedBack) {
        // Check if we're still in a back state or if we've navigated forward
        hasNavigatedBack = false;
      }
    }, 100);

    return () => clearTimeout(timeoutId);
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
    if (typeof window !== "undefined" && popStateHandler) {
      window.removeEventListener("popstate", popStateHandler);
    }
  });
</script>

<DeviceAuthGuard>
  <SidebarProvider open={sidebarOpen} onOpenChange={handleSidebarOpenChange}>
    <div class="flex h-screen min-h-0 w-full flex-col overflow-hidden">
      <!-- Top Bar -->
      <header
        class="flex-shrink-0 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
      >
        <div class="flex h-14 items-center gap-2 px-4">
          <SidebarTrigger />
          <!-- Navigation Buttons -->
          <div class="mr-2 flex items-center gap-1 border-r pr-2">
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              disabled={!canGoBack}
              onclick={goBack}
              title="Go back"
              aria-label="Go back"
            >
              <ArrowLeft class="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              disabled={!canGoForward}
              onclick={goForward}
              title="Go forward"
              aria-label="Go forward"
            >
              <ArrowRight class="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              onclick={refresh}
              title="Refresh page"
              aria-label="Refresh page"
            >
              <RefreshCw class="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              disabled={$page.url.pathname === "/"}
              onclick={goHome}
              title="Go to home"
              aria-label="Go to home"
            >
              <Home class="h-4 w-4" />
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
        {#if !isMainSidebarHidden()}
          <SidebarRoot collapsible="icon">
            <div class="flex h-full min-h-0 min-w-0 flex-col">
              <!-- Sidebar Header -->
              <button
                type="button"
                class="flex w-full items-center gap-3 border-b px-4 py-4 text-left transition-colors hover:bg-sidebar-accent/50 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:gap-0"
                onclick={() => goto("/")}
              >
                <div
                  class="flex h-8 w-8 items-center justify-center rounded-md bg-primary/10 text-primary"
                >
                  {#if isSdkPage}
                    <svg
                      class="h-4 w-4"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
                      />
                    </svg>
                  {:else}
                    <svg
                      class="h-4 w-4"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                      />
                    </svg>
                  {/if}
                </div>
                <div
                  class="min-w-0 flex-1 group-data-[collapsible=icon]:hidden"
                >
                  <h1
                    class="truncate text-sm font-semibold text-sidebar-foreground"
                  >
                    {isSdkPage ? "SDK Manager" : "Portal Desktop"}
                  </h1>
                  <p class="text-xs text-sidebar-foreground/60">
                    {isSdkPage
                      ? "Development Tools"
                      : "Development Environment"}
                  </p>
                </div>
              </button>

              <!-- Sidebar Content -->
              <div class="min-h-0 flex-1 overflow-y-auto">
                {#if navigationLoading}
                  <div class="flex items-center justify-center p-4">
                    <div
                      class="h-6 w-6 animate-spin rounded-full border-b-2 border-primary"
                    ></div>
                    <span
                      class="ml-2 text-sm text-muted-foreground group-data-[collapsible=icon]:hidden"
                      >Loading navigation...</span
                    >
                  </div>
                {:else if navigationError}
                  <div class="p-4">
                    <div class="text-sm text-destructive">
                      <p class="font-medium">Failed to load navigation</p>
                      <p class="mt-1 text-xs text-muted-foreground">
                        {navigationError}
                      </p>
                    </div>
                  </div>
                {:else}
                  <NavSectionList
                    sections={navigationSections}
                    currentPath={$page.url.pathname}
                    onNavigate={(url) => goto(url)}
                  />
                {/if}
              </div>

              <!-- Sidebar Footer -->
              <div class="flex-shrink-0 border-t p-4">
                <div class="flex items-center justify-between gap-2">
                  <ThemeToggle />
                  <div class="flex items-center gap-1">
                    {#if showQRButton}
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => (qrCodeDialogOpen = true)}
                        title="Share via QR Code"
                      >
                        <QrCode class="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => (deviceApprovalDialogOpen = true)}
                        title="Device approval requests"
                      >
                        <Shield class="h-4 w-4" />
                      </Button>
                    {/if}
                    <Button
                      variant="ghost"
                      size="sm"
                      onclick={() => goto("/settings")}
                      title="Settings"
                    >
                      <svg
                        class="h-4 w-4"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                        />
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                      </svg>
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          </SidebarRoot>
        {/if}

        <!-- Page Content -->
        <main class="min-h-0 min-w-0 flex-1 overflow-y-auto bg-background">
          {@render children()}
        </main>
      </div>
    </div>
  </SidebarProvider>

  <!-- Toast Container -->
  <ToastContainer />

  <!-- Floating Avatar Assistant -->
  <FloatingAvatar />

  <!-- QR Code Dialog -->
  <QRCodeDialog bind:open={qrCodeDialogOpen} />

  <!-- Device Approval Dialog -->
  {#if showQRButton}
    <DeviceApprovalDialog bind:open={deviceApprovalDialogOpen} />
  {/if}
</DeviceAuthGuard>
