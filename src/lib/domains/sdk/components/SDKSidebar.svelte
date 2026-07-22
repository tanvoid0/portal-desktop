<!--
	SDK Sidebar - FlyEnv-style sidebar with language and database categories
	Shows all available SDKs with toggle switches and selection states
-->

<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Separator } from "$lib/components/ui/separator";
  import {
    Settings,
    Database,
    Code,
    Globe,
    Container,
    Package,
    Download,
    ArrowLeft,
  } from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { logger } from "$lib/domains/shared";
  import {
    sdkConfigService,
    type ProcessedSDKConfig,
  } from "$lib/domains/sdk/services/sdkConfigService";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import {
    useSidebar,
    MenuButton as SidebarMenuButton,
  } from "$lib/components/ui/sidebar";

  import SDKCategorySection from "./sidebar/SDKCategorySection.svelte";
  import type { SDKItem } from "./sidebar/sdkSidebarTypes";

  interface Props {
    selectedSDK?: string;
    selectedView?: string;
    navigationItemIds?: string[];
    onSDKSelect?: (sdk: SDKItem) => void;
    onViewSelect?: (view: string) => void;
  }

  interface SDKManager {
    id: string;
    name: string;
    display_name: string;
    installed: boolean;
    version: string | null;
    supports_installation: boolean;
    supports_version_switching: boolean;
    install_command: string | null;
    website: string | null;
  }

  const {
    selectedSDK,
    selectedView = "overview",
    navigationItemIds,
    onSDKSelect,
    onViewSelect,
  }: Props = $props();

  // State
  let sdkConfigs = $state<ProcessedSDKConfig[]>([]);
  let loading = $state(true);
  let sdkManagers = $state<SDKManager[]>([]);
  let managersLoading = $state(true);
  let currentPath = $derived($page.url.pathname);
  const sidebar = useSidebar();
  let serviceToggleInFlight = $state<Set<string>>(new Set());
  let openSections = $state<Record<string, boolean>>({
    manager: false,
    language: false,
    database: false,
    web: false,
    container: false,
    ai: false,
  });
  let lastSelectedItemId = $state<string | null>(null);

  // Load SDK configs
  $effect(() => {
    loadSDKConfigs();
    loadSDKManagers();
  });

  async function loadSDKConfigs() {
    try {
      loading = true;
      const configs = await sdkConfigService.getAllSDKConfigs();
      sdkConfigs = configs;
      logger.info("SDK configs loaded", {
        context: "SDKSidebar",
        data: { count: configs.length },
      });
    } catch (error) {
      logger.error("Failed to load SDK configs", {
        context: "SDKSidebar",
        error,
      });
    } finally {
      loading = false;
    }
  }

  async function loadSDKManagers() {
    try {
      managersLoading = true;
      const managers = await invoke<SDKManager[]>("get_all_sdk_managers");
      sdkManagers = Array.isArray(managers) ? managers : [];
    } catch (error) {
      logger.error("Failed to load SDK managers", {
        context: "SDKSidebar",
        error,
      });
      sdkManagers = [];
    } finally {
      managersLoading = false;
    }
  }

  async function refreshSDKConfigsSilent() {
    try {
      const configs = await sdkConfigService.getAllSDKConfigs();
      sdkConfigs = configs;
    } catch (error) {
      logger.error("Failed to refresh SDK configs", {
        context: "SDKSidebar",
        error,
      });
    }
  }

  // Navigation items - using actual routes
  type NavigationItem = {
    id: string;
    name: string;
    icon: string;
    description: string;
  };

  const navigationItems: NavigationItem[] = [
    {
      id: "/sdk",
      name: "Overview",
      icon: "lucide:layout-dashboard",
      description: "SDK overview and statistics",
    },
    {
      id: "/sdk/manager",
      name: "SDK Managers",
      icon: "lucide:settings",
      description: "Manage SDK version managers",
    },
    {
      id: "/sdk/software-installer",
      name: "Software Installer",
      icon: "lucide:package",
      description: "Install and manage software via package managers",
    },
    {
      id: "/sdk/installations",
      name: "Installations",
      icon: "lucide:download",
      description: "View installed SDKs and versions",
    },
  ];

  // Allow the parent layout to trim the sidebar navigation per-section.
  // This keeps the sidebar reusable and avoids page-specific nav markup.
  let resolvedNavigationItems = $derived.by((): NavigationItem[] =>
    navigationItemIds?.length
      ? navigationItems.filter((item) => navigationItemIds.includes(item.id))
      : navigationItems,
  );

  function isNavigationItemActive(itemId: string) {
    const normalizedCurrentPath =
      currentPath.endsWith("/") && currentPath !== "/"
        ? currentPath.slice(0, -1)
        : currentPath;

    const normalizedItemId =
      itemId.endsWith("/") && itemId !== "/" ? itemId.slice(0, -1) : itemId;

    // Overview should be active for the whole SDK section.
    if (normalizedItemId === "/sdk") {
      return (
        normalizedCurrentPath === normalizedItemId ||
        normalizedCurrentPath.startsWith("/sdk/")
      );
    }

    // Mark the parent route as active for nested SDK-manager pages.
    if (normalizedItemId === "/sdk/manager") {
      return (
        normalizedCurrentPath === normalizedItemId ||
        normalizedCurrentPath.startsWith(`${normalizedItemId}/`)
      );
    }

    if (normalizedItemId === "/sdk/software-installer") {
      return normalizedCurrentPath === normalizedItemId;
    }

    if (normalizedItemId === "/sdk/installations") {
      return normalizedCurrentPath === normalizedItemId;
    }

    return normalizedCurrentPath === normalizedItemId;
  }

  // Convert SDK configs to SDKItem format
  let languageSDKs = $derived.by(() => {
    return sdkConfigs
      .filter((config) => config.category === "language")
      .map((config) => {
        // Get version, preferring SDK version over manager version
        const rawVersion =
          config.sdk_version ||
          config.sdk_managers.find((m) => m.installed)?.version ||
          null;
        // Format version (remove 'v' prefix if present, but keep it clean)
        const version = rawVersion ? rawVersion.trim().replace(/^v/, "") : null;

        return {
          id: config.id,
          name: config.name,
          displayName: config.display_name,
          icon: config.icon,
          category: "language",
          installed:
            config.sdk_installed ||
            config.sdk_managers.some((m) => m.installed) ||
            false,
          enabled: true,
          version: version || undefined,
          description: config.description,
          hasToggle: true,
          hasService: config.service_config != null,
          serviceRunning: config.service_running ?? null,
          port: config.service_port ?? null,
        };
      });
  });

  let managerSDKs = $derived.by(() => {
    return sdkManagers.map((m) => {
      const rawVersion = m.version ? m.version.trim() : null;
      const version = rawVersion ? rawVersion.replace(/^v/, "") : null;

      return {
        id: m.id,
        name: m.name ?? m.id,
        displayName: m.display_name ?? m.name ?? m.id,
        icon: getSDKIcon(m.id),
        category: "manager",
        installed: m.installed,
        enabled: true,
        version: version || undefined,
        description: undefined,
        hasToggle: false,
        hasService: false,
        serviceRunning: null,
        port: null,
      } as SDKItem;
    });
  });

  let databaseSDKs = $derived.by(() => {
    return sdkConfigs
      .filter((config) => config.category === "database")
      .map((config) => {
        // Get version, preferring SDK version
        const rawVersion = config.sdk_version || null;
        const version = rawVersion ? rawVersion.trim().replace(/^v/, "") : null;

        return {
          id: config.id,
          name: config.name,
          displayName: config.display_name,
          icon: config.icon,
          category: "database",
          installed: config.sdk_installed || false,
          enabled: true,
          version: version || undefined,
          description: config.description,
          hasToggle: true,
          hasService: config.service_config != null,
          serviceRunning: config.service_running ?? null,
          port: config.service_port ?? null,
        } as SDKItem;
      });
  });

  let webServerSDKs = $derived.by(() => {
    return sdkConfigs
      .filter((config) => config.category === "server")
      .map((config) => {
        // Get version, preferring SDK version
        const rawVersion = config.sdk_version || null;
        const version = rawVersion ? rawVersion.trim().replace(/^v/, "") : null;

        return {
          id: config.id,
          name: config.name,
          displayName: config.display_name,
          icon: config.icon,
          category: "server",
          installed: config.sdk_installed || false,
          enabled: true,
          version: version || undefined,
          description: config.description,
          hasToggle: true,
          hasService: config.service_config != null,
          serviceRunning: config.service_running ?? null,
          port: config.service_port ?? null,
        } as SDKItem;
      });
  });

  let containerSDKs = $derived.by(() => {
    return sdkConfigs
      .filter((config) => config.category === "container")
      .map((config) => {
        // Get version, preferring SDK version
        const rawVersion = config.sdk_version || null;
        const version = rawVersion ? rawVersion.trim().replace(/^v/, "") : null;

        return {
          id: config.id,
          name: config.name,
          displayName: config.display_name,
          icon: config.icon,
          category: "container",
          installed: config.sdk_installed || false,
          enabled: true,
          version: version || undefined,
          description: config.description,
          hasToggle: true,
          hasService: config.service_config != null,
          serviceRunning: config.service_running ?? null,
          port: config.service_port ?? null,
        } as SDKItem;
      });
  });

  let aiSDKs = $derived.by(() => {
    return sdkConfigs
      .filter((config) => config.category === "ai")
      .map((config) => {
        // Get version, preferring SDK version
        const rawVersion = config.sdk_version || null;
        const version = rawVersion ? rawVersion.trim().replace(/^v/, "") : null;

        return {
          id: config.id,
          name: config.name,
          displayName: config.display_name,
          icon: config.icon,
          category: "ai",
          installed: config.sdk_installed || false,
          enabled: true,
          version: version || undefined,
          description: config.description,
          hasToggle: true,
          hasService: config.service_config != null,
          serviceRunning: config.service_running ?? null,
          port: config.service_port ?? null,
        } as SDKItem;
      });
  });

  function isServiceToggleDisabled(sdk: SDKItem) {
    return (
      !sdk.hasService ||
      !sdk.installed ||
      sdk.serviceRunning == null ||
      serviceToggleInFlight.has(sdk.id)
    );
  }

  function isSelectedInCategory(category: string) {
    if (!selectedItem) return false;
    const listForCategory =
      category === "manager"
        ? managerSDKs
        : category === "language"
        ? languageSDKs
        : category === "database"
          ? databaseSDKs
          : category === "web"
            ? webServerSDKs
            : category === "container"
              ? containerSDKs
              : aiSDKs;
    return listForCategory.some((s) => s.id === selectedItem);
  }

  function toggleSection(category: string) {
    if (isSelectedInCategory(category)) return;
    openSections = { ...openSections, [category]: !openSections[category] };
  }

  async function setServiceRunning(sdk: SDKItem, running: boolean) {
    if (isServiceToggleDisabled(sdk)) return;

    try {
      // Clone to trigger reactivity.
      serviceToggleInFlight = new Set(serviceToggleInFlight);
      serviceToggleInFlight.add(sdk.id);

      const cmd = running ? "start_service" : "stop_service";
      await invoke(cmd, { sdkType: sdk.id });
    } catch (error) {
      logger.error("Failed to toggle service", {
        context: "SDKSidebar",
        data: { sdkId: sdk.id, running },
        error,
      });
    } finally {
      serviceToggleInFlight = new Set(serviceToggleInFlight);
      serviceToggleInFlight.delete(sdk.id);
      await refreshSDKConfigsSilent();
    }
  }

  // Reactive state
  let allSDKs = $derived([
    ...managerSDKs,
    ...languageSDKs,
    ...databaseSDKs,
    ...webServerSDKs,
    ...containerSDKs,
    ...aiSDKs,
  ]);

  let selectedItem = $state<string | null>(null);

  $effect(() => {
    selectedItem = selectedSDK || null;
  });

  // Keep the currently selected SDK's category expanded.
  $effect(() => {
    if (!selectedItem) return;
    if (selectedItem === lastSelectedItemId) return;

    if (managerSDKs.some((s) => s.id === selectedItem)) {
      openSections = { ...openSections, manager: true };
    }
    if (languageSDKs.some((s) => s.id === selectedItem)) {
      openSections = { ...openSections, language: true };
    }
    if (databaseSDKs.some((s) => s.id === selectedItem)) {
      openSections = { ...openSections, database: true };
    }
    if (webServerSDKs.some((s) => s.id === selectedItem)) {
      openSections = { ...openSections, web: true };
    }
    if (containerSDKs.some((s) => s.id === selectedItem)) {
      openSections = { ...openSections, container: true };
    }
    if (aiSDKs.some((s) => s.id === selectedItem)) {
      openSections = { ...openSections, ai: true };
    }

    lastSelectedItemId = selectedItem;
  });

  // If nothing is selected yet, show something by default.
  $effect(() => {
    if (loading || managersLoading) return;
    if (selectedItem) return;
    if (openSections.language) return;
    const hasManagers = managerSDKs.length > 0;
    const hasLanguageSDKs = languageSDKs.length > 0;
    openSections = {
      ...openSections,
      language: hasLanguageSDKs,
      manager: hasManagers,
    };
  });

  // Category icons
  const categoryIcons = {
    language: Code,
    database: Database,
    web: Globe,
    container: Container,
    package: Package,
  };

  let sidebarLoading = $derived(loading || managersLoading);

  // Map SDK types to their route paths - use dynamic route
  function getSDKRoute(sdkId: string, category: string): string {
    // Normalize SDK ID for routing
    const normalizedId = sdkId.toLowerCase().trim();

    // SDK Managers go to /sdk/manager/[name] (singular, matches route structure)
    if (category === "manager") {
      return `/sdk/manager/${normalizedId}`;
    }

    // Use the new dynamic route for all SDKs (language, database, ai, server, container, etc.)
    return `/sdk/${normalizedId}`;
  }

  function handleSDKClick(sdk: SDKItem) {
    selectedItem = sdk.id;
    onSDKSelect?.(sdk);

    // Get the appropriate route for this SDK
    const route = getSDKRoute(sdk.id, sdk.category);

    // Navigate to the appropriate route
    goto(route);

    logger.info("SDK selected", {
      context: "SDKSidebar",
      data: {
        sdkId: sdk.id,
        sdkName: sdk.displayName,
        category: sdk.category,
        route,
      },
    });
  }

  function getCategoryIcon(category: string) {
    return categoryIcons[category as keyof typeof categoryIcons] || Code;
  }

  function getCategoryName(category: string) {
    const names: Record<string, string> = {
      language: "Language & Runtime",
      database: "Database Server",
      web: "Web Server",
      container: "Container Platform",
      package: "Package Manager",
    };
    return names[category] || category;
  }

  function getSDKIcon(sdkType: string): string {
    const iconMap: Record<string, string> = {
      // Language & Runtime
      java: "devicon-java-plain",
      node: "devicon-nodejs-plain",
      python: "devicon-python-plain",
      rust: "devicon-rust-plain",
      go: "devicon-go-plain",
      php: "devicon-php-plain",
      ruby: "devicon-ruby-plain",
      bun: "devicon-bun-plain",
      deno: "devicon-deno-plain",
      gradle: "devicon-gradle-plain",
      kotlin: "devicon-kotlin-plain",
      scala: "devicon-scala-plain",
      erlang: "devicon-erlang-plain",
      perl: "devicon-perl-plain",

      // Database
      mysql: "devicon-mysql-plain",
      postgresql: "devicon-postgresql-plain",
      mongodb: "devicon-mongodb-plain",
      mariadb: "devicon-mariadb-plain",

      // Web Server
      nginx: "devicon-nginx-original",
      apache: "devicon-apache-plain",
      caddy: "devicon-caddy-plain",

      // Container
      docker: "devicon-docker-plain",
      kubernetes: "devicon-kubernetes-plain",
      podman: "devicon-podman-plain",

      // Package Managers
      npm: "devicon-npm-original-wordmark",
      yarn: "devicon-yarn-plain",
      pip: "devicon-python-plain",
      cargo: "devicon-rust-plain",
      composer: "devicon-composer-plain",
      gem: "devicon-ruby-plain",

      // SDK Managers
      nvm: "devicon-nodejs-plain",
      pyenv: "devicon-python-plain",
      rustup: "devicon-rust-plain",
      sdkman: "devicon-sdkman-plain",
      goenv: "devicon-go-plain",
      rbenv: "devicon-ruby-plain",
      phpenv: "devicon-php-plain",
    };

    return iconMap[sdkType.toLowerCase()] || "devicon-devicon-plain";
  }

  function getSDKIconColor(sdkId: string): string {
    const colorMap: Record<string, string> = {
      // Language & Runtime - colored icons
      java: "text-orange-600",
      node: "text-green-600",
      nodejs: "text-green-600",
      python: "text-blue-600",
      rust: "text-orange-600",
      go: "text-blue-500",
      php: "text-purple-600",
      ruby: "text-red-600",
      bun: "text-yellow-600",
      deno: "text-gray-800",
      gradle: "text-blue-500",
      kotlin: "text-purple-600",
      scala: "text-red-600",
      erlang: "text-red-500",
      perl: "text-blue-700",

      // Database
      mysql: "text-blue-600",
      postgresql: "text-blue-700",
      mongodb: "text-green-600",
      mariadb: "text-blue-500",

      // Web Server
      nginx: "text-green-600",
      apache: "text-red-600",
      caddy: "text-blue-600",

      // Container
      docker: "text-blue-500",
      kubernetes: "text-blue-600",
      podman: "text-blue-600",

      // Package Managers
      npm: "text-red-600",
      yarn: "text-blue-600",
      pip: "text-blue-600",
      cargo: "text-orange-600",
      composer: "text-gray-700",
      gem: "text-red-600",

      // SDK Managers
      nvm: "text-green-600",
      pyenv: "text-blue-600",
      rustup: "text-orange-600",
      sdkman: "text-blue-600",
      goenv: "text-blue-500",
      rbenv: "text-red-600",
      phpenv: "text-purple-600",
    };

    return colorMap[sdkId.toLowerCase()] || "";
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  {#if sidebar.state === "collapsed"}
    <!-- Header (icon-only) -->
    <div class="divider-edge-b divider-edge-full flex-shrink-0 p-1">
      <SidebarMenuButton
        size="sm"
        tooltipContent="Back to Portal Desktop"
        onclick={() => goto("/")}
      >
        <ArrowLeft class="h-4 w-4" />
      </SidebarMenuButton>
    </div>

    <!-- Sidebar Content -->
    <ScrollArea class="min-h-0 flex-1 overflow-hidden">
      <div class="space-y-3 p-1">
        <!-- Navigation Section (icon-only) -->
        <div class="space-y-0.5">
          {#each resolvedNavigationItems as item}
            <SidebarMenuButton
              size="sm"
              isActive={isNavigationItemActive(item.id)}
              tooltipContent={item.name}
              onclick={() => goto(item.id)}
            >
              <div class="flex h-4 w-4 items-center justify-center">
                {#if item.icon === "lucide:layout-dashboard"}
                  <Settings class="h-4 w-4" />
                {:else if item.icon === "lucide:settings"}
                  <Settings class="h-4 w-4" />
                {:else if item.icon === "lucide:download"}
                  <Download class="h-4 w-4" />
                {:else if item.icon === "lucide:package"}
                  <Package class="h-4 w-4" />
                {/if}
              </div>
            </SidebarMenuButton>
          {/each}
        </div>

        <Separator />

        {#if sidebarLoading}
          <div class="py-4 text-center text-muted-foreground">
            <div class="mb-1 text-xl">⏳</div>
            <h3 class="text-sm font-medium">Loading SDKs...</h3>
          </div>
        {:else if
          managerSDKs.length === 0 &&
            languageSDKs.length === 0 &&
            databaseSDKs.length === 0 &&
            webServerSDKs.length === 0 &&
            containerSDKs.length === 0 &&
            aiSDKs.length === 0}
          <div class="py-4 text-center text-muted-foreground">
            <div class="mb-1 text-xl">🔍</div>
            <h3 class="mb-1 text-sm font-medium">No SDKs Detected</h3>
            <p class="text-xs">
              Install SDK managers like NVM, Pyenv, or SDKMAN to get started.
            </p>
          </div>
        {:else}
          {#if managerSDKs.length > 0}
            <SDKCategorySection
              variant="collapsed"
              title="SDK Managers"
              iconComponent={Settings}
              items={managerSDKs}
              selectedItemId={selectedItem}
              collapsible={true}
              isOpen={openSections.manager}
              onToggle={() => toggleSection("manager")}
              getSDKIconColor={getSDKIconColor}
              isServiceToggleDisabled={isServiceToggleDisabled}
              onSDKClick={(sdk) => handleSDKClick(sdk)}
              onServiceToggle={(sdk, next) =>
                setServiceRunning(sdk, next)}
            />
            {#if languageSDKs.length > 0}
              <Separator />
            {/if}
          {/if}

          {#if languageSDKs.length > 0}
            <SDKCategorySection
              variant="collapsed"
              title={getCategoryName("language")}
              iconComponent={getCategoryIcon("language")}
              items={languageSDKs}
              selectedItemId={selectedItem}
              collapsible={true}
              isOpen={openSections.language}
              onToggle={() => toggleSection("language")}
              getSDKIconColor={getSDKIconColor}
              isServiceToggleDisabled={isServiceToggleDisabled}
              onSDKClick={(sdk) => handleSDKClick(sdk)}
              onServiceToggle={(sdk, next) =>
                setServiceRunning(sdk, next)}
            />
          {/if}

          {#if databaseSDKs.length > 0}
            <Separator />
            <SDKCategorySection
              variant="collapsed"
              title={getCategoryName("database")}
              iconComponent={getCategoryIcon("database")}
              items={databaseSDKs}
              selectedItemId={selectedItem}
              collapsible={true}
              isOpen={openSections.database}
              onToggle={() => toggleSection("database")}
              getSDKIconColor={getSDKIconColor}
              isServiceToggleDisabled={isServiceToggleDisabled}
              onSDKClick={(sdk) => handleSDKClick(sdk)}
              onServiceToggle={(sdk, next) =>
                setServiceRunning(sdk, next)}
            />
          {/if}

          {#if webServerSDKs.length > 0}
            <Separator />
            <SDKCategorySection
              variant="collapsed"
              title={getCategoryName("web")}
              iconComponent={getCategoryIcon("web")}
              items={webServerSDKs}
              selectedItemId={selectedItem}
              collapsible={true}
              isOpen={openSections.web}
              onToggle={() => toggleSection("web")}
              getSDKIconColor={getSDKIconColor}
              isServiceToggleDisabled={isServiceToggleDisabled}
              onSDKClick={(sdk) => handleSDKClick(sdk)}
              onServiceToggle={(sdk, next) =>
                setServiceRunning(sdk, next)}
            />
          {/if}

          {#if containerSDKs.length > 0}
            <Separator />
            <SDKCategorySection
              variant="collapsed"
              title={getCategoryName("container")}
              iconComponent={getCategoryIcon("container")}
              items={containerSDKs}
              selectedItemId={selectedItem}
              collapsible={true}
              isOpen={openSections.container}
              onToggle={() => toggleSection("container")}
              getSDKIconColor={getSDKIconColor}
              isServiceToggleDisabled={isServiceToggleDisabled}
              onSDKClick={(sdk) => handleSDKClick(sdk)}
              onServiceToggle={(sdk, next) =>
                setServiceRunning(sdk, next)}
            />
          {/if}

          {#if aiSDKs.length > 0}
            <Separator />
            <SDKCategorySection
              variant="collapsed"
              title="AI SDKs"
              iconComponent={getCategoryIcon("ai")}
              items={aiSDKs}
              selectedItemId={selectedItem}
              collapsible={true}
              isOpen={openSections.ai}
              onToggle={() => toggleSection("ai")}
              getSDKIconColor={getSDKIconColor}
              isServiceToggleDisabled={isServiceToggleDisabled}
              onSDKClick={(sdk) => handleSDKClick(sdk)}
              onServiceToggle={(sdk, next) =>
                setServiceRunning(sdk, next)}
            />
          {/if}
        {/if}
      </div>
    </ScrollArea>
  {:else}
    <!-- Header -->
    <div class="divider-edge-b divider-edge-full flex-shrink-0 bg-background p-3">
      <h2 class="text-base font-semibold">SDK Manager</h2>
      <p class="text-xs text-muted-foreground">
        Manage your development environment
      </p>
    </div>

    <!-- Sidebar Content -->
    <ScrollArea class="min-h-0 flex-1 overflow-hidden">
      <div class="space-y-4 p-3">
        <!-- Navigation Section -->
        <div class="space-y-2">
          <h3 class="text-xs font-medium text-muted-foreground">Navigation</h3>
          <div class="space-y-0.5">
            {#each resolvedNavigationItems as item}
              <Button
                variant="ghost"
                class="flex h-auto w-full cursor-pointer items-center gap-2 rounded-md p-1.5 text-left {isNavigationItemActive(item.id)
                  ? 'bg-muted'
                  : ''}"
                onclick={() => goto(item.id)}
              >
                <div class="flex h-4 w-4 items-center justify-center">
                  {#if item.icon === "lucide:layout-dashboard"}
                    <Settings class="h-4 w-4" />
                  {:else if item.icon === "lucide:settings"}
                    <Settings class="h-4 w-4" />
                  {:else if item.icon === "lucide:download"}
                    <Download class="h-4 w-4" />
                  {:else if item.icon === "lucide:package"}
                    <Package class="h-4 w-4" />
                  {/if}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="truncate text-xs font-medium">{item.name}</div>
                  <div class="truncate text-xs text-muted-foreground">
                    {item.description}
                  </div>
                </div>
              </Button>
            {/each}
          </div>
        </div>

        <Separator />

        {#if managerSDKs.length > 0}
          <SDKCategorySection
            variant="expanded"
            title="SDK Managers"
            iconComponent={Settings}
            items={managerSDKs}
            selectedItemId={selectedItem}
            collapsible={true}
            isOpen={openSections.manager}
            onToggle={() => toggleSection("manager")}
            getSDKIconColor={getSDKIconColor}
            isServiceToggleDisabled={isServiceToggleDisabled}
            onSDKClick={(sdk) => handleSDKClick(sdk)}
            onServiceToggle={(sdk, next) => setServiceRunning(sdk, next)}
          />
        {/if}

        {#if languageSDKs.length > 0}
          <SDKCategorySection
            variant="expanded"
            title={getCategoryName("language")}
            iconComponent={getCategoryIcon("language")}
            items={languageSDKs}
            selectedItemId={selectedItem}
            collapsible={true}
            isOpen={openSections.language}
            onToggle={() => toggleSection("language")}
            getSDKIconColor={getSDKIconColor}
            isServiceToggleDisabled={isServiceToggleDisabled}
            onSDKClick={(sdk) => handleSDKClick(sdk)}
            onServiceToggle={(sdk, next) =>
              setServiceRunning(sdk, next)}
          />
        {/if}

        <Separator />

        {#if databaseSDKs.length > 0}
          <SDKCategorySection
            variant="expanded"
            title={getCategoryName("database")}
            iconComponent={getCategoryIcon("database")}
            items={databaseSDKs}
            selectedItemId={selectedItem}
            collapsible={true}
            isOpen={openSections.database}
            onToggle={() => toggleSection("database")}
            getSDKIconColor={getSDKIconColor}
            isServiceToggleDisabled={isServiceToggleDisabled}
            onSDKClick={(sdk) => handleSDKClick(sdk)}
            onServiceToggle={(sdk, next) =>
              setServiceRunning(sdk, next)}
          />
        {/if}

        <Separator />

        {#if webServerSDKs.length > 0}
          <SDKCategorySection
            variant="expanded"
            title={getCategoryName("web")}
            iconComponent={getCategoryIcon("web")}
            items={webServerSDKs}
            selectedItemId={selectedItem}
            collapsible={true}
            isOpen={openSections.web}
            onToggle={() => toggleSection("web")}
            getSDKIconColor={getSDKIconColor}
            isServiceToggleDisabled={isServiceToggleDisabled}
            onSDKClick={(sdk) => handleSDKClick(sdk)}
            onServiceToggle={(sdk, next) =>
              setServiceRunning(sdk, next)}
          />
        {/if}

        <Separator />

        {#if containerSDKs.length > 0}
          <SDKCategorySection
            variant="expanded"
            title={getCategoryName("container")}
            iconComponent={getCategoryIcon("container")}
            items={containerSDKs}
            selectedItemId={selectedItem}
            collapsible={true}
            isOpen={openSections.container}
            onToggle={() => toggleSection("container")}
            getSDKIconColor={getSDKIconColor}
            isServiceToggleDisabled={isServiceToggleDisabled}
            onSDKClick={(sdk) => handleSDKClick(sdk)}
            onServiceToggle={(sdk, next) =>
              setServiceRunning(sdk, next)}
          />
        {/if}

        {#if aiSDKs.length > 0}
          <SDKCategorySection
            variant="expanded"
            title="AI SDKs"
            iconComponent={getCategoryIcon("ai")}
            items={aiSDKs}
            selectedItemId={selectedItem}
            collapsible={true}
            isOpen={openSections.ai}
            onToggle={() => toggleSection("ai")}
            getSDKIconColor={getSDKIconColor}
            isServiceToggleDisabled={isServiceToggleDisabled}
            onSDKClick={(sdk) => handleSDKClick(sdk)}
            onServiceToggle={(sdk, next) =>
              setServiceRunning(sdk, next)}
          />
          <Separator />
        {/if}

        <!-- No Data Fallback -->
        {#if sidebarLoading}
          <div class="py-4 text-center text-muted-foreground">
            <div class="mb-2 text-xl">⏳</div>
            <h3 class="mb-1 text-sm font-medium">Loading SDKs...</h3>
          </div>
        {:else if
          managerSDKs.length === 0 &&
            languageSDKs.length === 0 &&
            databaseSDKs.length === 0 &&
            webServerSDKs.length === 0 &&
            containerSDKs.length === 0 &&
            aiSDKs.length === 0}
          <div class="py-4 text-center text-muted-foreground">
            <div class="mb-2 text-xl">🔍</div>
            <h3 class="mb-1 text-sm font-medium">No SDKs Detected</h3>
            <p class="text-xs">
              No SDK managers or SDKs were found on your system. Install SDK
              managers like NVM, Pyenv, or SDKMAN to get started.
            </p>
          </div>
        {/if}
      </div>
    </ScrollArea>

    <!-- Footer -->
    <div class="divider-edge-t divider-edge-full p-3">
      <div class="space-y-1 text-center text-xs text-muted-foreground">
        <div>
          {allSDKs.filter((sdk) => sdk.installed).length} of {allSDKs
            .length} SDKs
        </div>
      </div>
    </div>
  {/if}
</div>
