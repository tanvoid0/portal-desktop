<!--
	SDK Overview Page - FlyEnv-style dashboard
	Main SDK management interface with service and version management
-->

<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { sdkService } from "$lib/domains/sdk/services/sdkService";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Progress } from "$lib/components/ui/progress";
  import { Alert, AlertDescription } from "$lib/components/ui/alert";
  import {
    Database,
    Code,
    Globe,
    Container,
    Package,
    Download,
    Settings,
    Play,
    Square,
    AlertCircle,
    CheckCircle,
    Activity,
    TrendingUp,
    Clock,
    Star,
    RefreshCw,
    Plus,
    Zap,
  } from "@lucide/svelte";
  import ServiceCard from "$lib/domains/sdk/components/ServiceCard.svelte";
  import VersionList from "$lib/domains/sdk/components/VersionList.svelte";
  import Devicon from "$lib/components/ui/devicon.svelte";
  import {
    sdkManagers as sdkManagersStore,
    installedSDKs as installedSDKsStore,
    sdkActions,
  } from "$lib/domains/sdk/stores/sdkStore";
  import { goto } from "$app/navigation";
  import type { SDKManagerInfo } from "$lib/domains/sdk/types";

  // State
  let loading = $state(false);
  let error = $state<string | null>(null);
  let services = $state<any[]>([]);
  let versions = $state<any[]>([]);
  let availableVersions = $state<any[]>([]);
  let activeTab = $state("overview");

  // Service state
  let serviceRunning = $state(false);
  let serviceStarting = $state(false);

  // Real data from Tauri commands
  let installedSDKs = $state<any[]>([]);
  let availableSDKs = $state<any[]>([]);
  let recentActivity = $state<any[]>([]);

  // SDK Managers data
  let sdkManagers = $state<any[]>([]);
  let managerInstallationStatus = $state<Record<string, boolean>>({});

  // Get real SDK managers from store
  let realSDKManagers = $derived($sdkManagersStore);
  let realInstalledSDKs = $derived($installedSDKsStore);

  // Also use the directly loaded managers data as fallback
  let allManagers = $derived.by(() => {
    // Combine store data with directly loaded data
    const storeManagers = realSDKManagers || [];
    const loadedManagers = installedSDKs || [];

    // If store has data, use it; otherwise use loaded data
    if (storeManagers.length > 0) {
      return storeManagers;
    }

    // Convert loaded data to SDKManagerInfo format
    return loadedManagers.map((m: any) => ({
      name: m.name || m.type || m.id || "",
      type: m.type || m.name || m.id || "",
      display_name: m.display_name || m.name || m.type || "",
      category: m.category || "language",
      installed:
        m.installed === true || m.installed === "true" || m.installed === true,
      version: m.version || null,
      description: m.description || "",
      sdk_type: m.sdk_type || m.type || m.name || "",
    }));
  });

  // Handle URL parameters
  $effect(() => {
    const urlParams = new URLSearchParams($page.url.search);
    const tab = urlParams.get("tab");
    if (tab && ["overview", "managers", "services", "versions"].includes(tab)) {
      activeTab = tab;
    }
  });

  // Initialize data
  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    loading = true;
    error = null;

    try {
      // Load SDK managers and available SDKs
      const [managersData, availableData] = await Promise.all([
        sdkService.detectManagers(),
        sdkService.getAllAvailableSDKs(),
      ]);

      // Process managers data
      installedSDKs = Array.isArray(managersData) ? managersData : [];

      // Process available SDKs
      availableSDKs = Array.isArray(availableData) ? availableData : [];

      // Populate stores with detected managers (convert to SDKManagerInfo format)
      if (Array.isArray(managersData)) {
        const managers: SDKManagerInfo[] = managersData.map((m: any) => ({
          name: m.name || m.type || m.id || "",
          type: m.type || m.name || m.id || "",
          display_name: m.display_name || m.name || m.type || "",
          category: m.category || "language",
          installed:
            m.installed === true ||
            m.installed === "true" ||
            m.installed === true,
          version: m.version || null,
          description: m.description || "",
          sdk_type: m.sdk_type || m.type || m.name || "",
        }));
        sdkActions.setManagers(managers);
      }

      // Initialize SDK Managers list
      sdkManagers = [
        {
          id: "nvm",
          name: "Node Version Manager (NVM)",
          description: "Manage multiple Node.js versions",
          icon: "nodejs",
          category: "JavaScript",
          website: "https://github.com/nvm-sh/nvm",
          installCommand:
            "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash",
          features: [
            "Node.js version management",
            "Automatic switching",
            "Global/local versions",
          ],
        },
        {
          id: "rustup",
          name: "Rustup",
          description: "Rust toolchain installer and version manager",
          icon: "rust",
          category: "Systems Programming",
          website: "https://rustup.rs/",
          installCommand:
            'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh',
          features: [
            "Rust toolchain management",
            "Cross-compilation",
            "Component management",
          ],
        },
        {
          id: "pyenv",
          name: "Pyenv",
          description: "Python version management",
          icon: "python",
          category: "Python",
          website: "https://github.com/pyenv/pyenv",
          installCommand: "curl https://pyenv.run | bash",
          features: [
            "Python version management",
            "Virtual environments",
            "Automatic switching",
          ],
        },
        {
          id: "sdkman",
          name: "SDKMAN!",
          description: "SDK Manager for Java, Kotlin, Scala, and more",
          icon: "java",
          category: "JVM",
          website: "https://sdkman.io/",
          installCommand: 'curl -s "https://get.sdkman.io" | bash',
          features: [
            "Multi-language support",
            "Java ecosystem",
            "Easy installation",
          ],
        },
        {
          id: "rbenv",
          name: "rbenv",
          description: "Ruby version management",
          icon: "ruby",
          category: "Ruby",
          website: "https://github.com/rbenv/rbenv",
          installCommand:
            "curl -fsSL https://github.com/rbenv/rbenv-installer/raw/HEAD/bin/rbenv-installer | bash",
          features: [
            "Ruby version management",
            "Gem management",
            "Project-specific versions",
          ],
        },
        {
          id: "phpenv",
          name: "phpenv",
          description: "PHP version management",
          icon: "php",
          category: "PHP",
          website: "https://github.com/phpenv/phpenv",
          installCommand:
            "curl -fsSL https://github.com/phpenv/phpenv-installer/raw/HEAD/bin/phpenv-installer | bash",
          features: [
            "PHP version management",
            "Extension management",
            "Multiple PHP versions",
          ],
        },
      ];

      // Check installation status for each manager
      for (const manager of sdkManagers) {
        managerInstallationStatus[manager.id] =
          await sdkService.checkManagerInstalled(manager.id);
      }

      // Load service status for each SDK type
      const servicePromises = availableSDKs.map(async (sdk: any) => {
        try {
          // Ensure we have a valid sdkType
          const sdkType = sdk.id || sdk.name || sdk.type;
          if (!sdkType) {
            console.warn("Skipping SDK without valid id/name/type:", sdk);
            return {
              ...sdk,
              services: [],
            };
          }

          const serviceStatus = await sdkService.getServiceStatus(sdkType);
          return {
            ...sdk,
            services: serviceStatus,
          };
        } catch (err) {
          console.warn("Failed to load service status for SDK:", sdk, err);
          return {
            ...sdk,
            services: [],
          };
        }
      });

      const sdkWithServices = await Promise.all(servicePromises);

      // Flatten services from all SDKs
      services = sdkWithServices.flatMap((sdk) => {
        // Ensure services is an array
        const sdkServices = Array.isArray(sdk.services) ? sdk.services : [];
        return sdkServices.map((service: any) => ({
          ...service,
          sdkType: sdk.id,
          sdkName: sdk.name,
        }));
      });

      // Load versions for each SDK
      const versionPromises = availableSDKs.map(async (sdk: any) => {
        try {
          const versions = await sdkService.fetchAvailableVersions(sdk.id);
          return {
            ...sdk,
            versions,
          };
        } catch {
          return {
            ...sdk,
            versions: [],
          };
        }
      });

      const sdkWithVersions = await Promise.all(versionPromises);

      // Flatten versions from all SDKs
      versions = sdkWithVersions.flatMap((sdk) =>
        sdk.versions.map((version: any) => ({
          ...version,
          sdkType: sdk.id,
          sdkName: sdk.name,
        })),
      );
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load data";
      console.error("Failed to load SDK data:", err);
    } finally {
      loading = false;
    }
  }

  // Service management
  async function toggleService(service: any) {
    try {
      if (service.status === "running") {
        // Stop service
        service.status = "stopping";
        await sdkService.stopService(service.sdkType, service.pid);
        service.status = "stopped";
        service.pid = null;
      } else {
        // Start service
        service.status = "starting";
        const config = {
          port: service.port,
          host: "localhost",
          data_dir: null,
          config_file: null,
          environment: {},
        };
        const pid = await sdkService.startService(
          service.sdkType,
          service.version,
          config,
        );
        service.status = "running";
        service.pid = pid;
      }
    } catch (err) {
      service.status = "error";
      error = err instanceof Error ? err.message : "Failed to toggle service";
    }
  }

  async function changeServiceVersion(service: any, version: string) {
    try {
      // Stop service if running
      if (service.status === "running") {
        await sdkService.stopService(service.sdkType, service.pid);
      }

      // Update version
      service.version = version;

      // Restart service if it was running
      if (service.status === "running") {
        const config = {
          port: service.port,
          host: "localhost",
          data_dir: null,
          config_file: null,
          environment: {},
        };
        const pid = await sdkService.startService(
          service.sdkType,
          service.version,
          config,
        );
        service.pid = pid;
      }
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to change service version";
    }
  }

  async function configureService(service: any) {
    // FUTURE: Open configuration dialog for service settings
    console.log("Configure service:", service.id);
  }

  async function viewServiceLogs(service: any) {
    // FUTURE: Open log viewer with service logs
    console.log("View logs for:", service.id);
  }

  async function openServiceUrl(service: any) {
    if (service.port) {
      window.open(`http://localhost:${service.port}`, "_blank");
    }
  }

  // Version management
  async function installVersion(version: any) {
    version.downloading = true;
    version.progress = 0;

    try {
      await sdkService.downloadAndInstallVersion(
        version.sdkType,
        version.version,
        false,
      );

      version.downloading = false;
      version.installed = true;

      // Reload data to get updated status
      await loadData();
    } catch (err) {
      version.downloading = false;
      version.error =
        err instanceof Error ? err.message : "Installation failed";
    }
  }

  async function uninstallVersion(version: any) {
    try {
      await sdkService.uninstallVersion({
        type: version.sdkType,
        version: version.version,
        manager: version.manager ?? version.sdkType,
      });
      version.installed = false;

      // Reload data to get updated status
      await loadData();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to uninstall version";
    }
  }

  async function setActiveVersion(version: any) {
    try {
      await sdkService.switchVersion({
        type: version.sdkType,
        version: version.version,
      });

      // Update all versions
      versions.forEach((v) => (v.active = false));
      version.active = true;

      // Reload data to get updated status
      await loadData();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to set active version";
    }
  }

  // SDK Manager installation
  async function installSDKManager(manager: any) {
    manager.installing = true;
    manager.installProgress = 0;

    try {
      // Execute installation command
      await sdkService.runShellCommand(manager.installCommand, "/tmp");

      manager.installing = false;
      managerInstallationStatus[manager.id] = true;

      // Reload data to get updated status
      await loadData();
    } catch (err) {
      manager.installing = false;
      manager.installError =
        err instanceof Error ? err.message : "Installation failed";
    }
  }

  async function uninstallSDKManager(manager: any) {
    try {
      await sdkService.uninstallManager(manager.id);
      managerInstallationStatus[manager.id] = false;

      // Reload data to get updated status
      await loadData();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to uninstall manager";
    }
  }

  // Statistics
  let totalServices = $derived(services.length);
  let runningServices = $derived(
    services.filter((s) => s.status === "running").length,
  );
  let totalVersions = $derived(versions.length);
  let installedVersions = $derived(versions.filter((v) => v.installed).length);
  let activeVersions = $derived(versions.filter((v) => v.active).length);
  let totalManagers = $derived(sdkManagers.length);
  let installedManagers = $derived(
    Object.values(managerInstallationStatus).filter(Boolean).length,
  );

  // Helper functions to organize SDKs by category (similar to sidebar)
  function getSDKIcon(sdkType: string): string {
    const iconMap: Record<string, string> = {
      java: "devicon-java-plain",
      node: "devicon-nodejs-plain",
      nodejs: "devicon-nodejs-plain",
      python: "devicon-python-plain",
      rust: "devicon-rust-plain",
      go: "devicon-go-plain",
      php: "devicon-php-plain",
      ruby: "devicon-ruby-plain",
      bun: "devicon-bun-plain",
      deno: "devicon-deno-plain",
      mysql: "devicon-mysql-plain",
      postgresql: "devicon-postgresql-plain",
      mongodb: "devicon-mongodb-plain",
      mariadb: "devicon-mariadb-plain",
      nginx: "devicon-nginx-original",
      apache: "devicon-apache-plain",
      docker: "devicon-docker-plain",
      kubernetes: "devicon-kubernetes-plain",
      nvm: "devicon-nodejs-plain",
      pyenv: "devicon-python-plain",
      rustup: "devicon-rust-plain",
      sdkman: "devicon-sdkman-plain",
      rbenv: "devicon-ruby-plain",
      phpenv: "devicon-php-plain",
    };
    return iconMap[sdkType.toLowerCase()] || "devicon-devicon-plain";
  }

  function getSDKRoute(sdkId: string, category: string): string {
    const normalizedId = sdkId.toLowerCase().trim();

    // SDK Managers go to /sdk/manager/[name] (singular, matches route structure)
    if (category === "manager") {
      return `/sdk/manager/${normalizedId}`;
    }

    // Use the new dynamic route for all SDKs (language, database, ai, server, container, etc.)
    // The dynamic route /sdk/[sdk]/+page.svelte handles all SDKs
    return `/sdk/${normalizedId}`;
  }

  // Organize SDKs by category
  let quickNavSDKs = $derived(() => {
    const categories: Record<string, any[]> = {
      manager: [],
      language: [],
      database: [],
      web: [],
      container: [],
    };

    allManagers.forEach((manager: SDKManagerInfo) => {
      const category = manager.category || "";
      const sdkItem = {
        id: manager.type || manager.name,
        name: manager.type || manager.name,
        displayName: manager.display_name || manager.name,
        icon: getSDKIcon(manager.type || manager.name),
        category: category || "language",
        installed: manager.installed === true || manager.installed === "true",
        version: manager.version,
        description: manager.description,
      };

      if (category === "manager") {
        categories.manager.push(sdkItem);
      } else if (category === "language") {
        categories.language.push(sdkItem);
      } else if (category === "database") {
        categories.database.push(sdkItem);
      } else if (category === "web") {
        categories.web.push(sdkItem);
      } else if (category === "container") {
        categories.container.push(sdkItem);
      } else {
        // Default to language if category is not specified
        categories.language.push(sdkItem);
      }
    });

    return categories;
  });

  function handleSDKClick(sdk: any) {
    const route = getSDKRoute(sdk.id, sdk.category);
    goto(route);
  }

  // Check if a service is running for an SDK
  function isServiceRunning(sdkId: string): boolean {
    return services.some(
      (s) =>
        (s.sdkType === sdkId || s.sdkName === sdkId) && s.status === "running",
    );
  }
</script>

<div class="p-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold">SDK Management</h1>
      <p class="text-muted-foreground">
        Manage your development environment and SDK versions
      </p>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="outline" onclick={loadData} disabled={loading}>
        <RefreshCw class="mr-2 h-4 w-4" />
        Refresh
      </Button>
      <Button>
        <Settings class="mr-2 h-4 w-4" />
        Settings
      </Button>
    </div>
  </div>

  <!-- Error Alert -->
  {#if error}
    <Alert variant="destructive">
      <AlertCircle class="h-4 w-4" />
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {/if}

  <!-- Statistics Cards -->
  <div class="grid grid-cols-1 gap-4 md:grid-cols-5">
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Managers</CardTitle>
        <Package class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">
          {installedManagers}/{totalManagers}
        </div>
        <p class="text-xs text-muted-foreground">
          {installedManagers} installed
        </p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Services</CardTitle>
        <Database class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">{runningServices}/{totalServices}</div>
        <p class="text-xs text-muted-foreground">
          {runningServices} running
        </p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Versions</CardTitle>
        <Code class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">
          {installedVersions}/{totalVersions}
        </div>
        <p class="text-xs text-muted-foreground">
          {installedVersions} installed
        </p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Active</CardTitle>
        <CheckCircle class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">{activeVersions}</div>
        <p class="text-xs text-muted-foreground">Active versions</p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Status</CardTitle>
        <Globe class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">
          {runningServices > 0 ? "Online" : "Offline"}
        </div>
        <p class="text-xs text-muted-foreground">Environment status</p>
      </CardContent>
    </Card>
  </div>

  <!-- Main Content Tabs -->
  <Tabs bind:value={activeTab} class="w-full">
    <TabsList class="grid w-full grid-cols-4">
      <TabsTrigger value="overview">Overview</TabsTrigger>
      <TabsTrigger value="managers">SDK Managers</TabsTrigger>
      <TabsTrigger value="services">Services</TabsTrigger>
      <TabsTrigger value="versions">Versions</TabsTrigger>
    </TabsList>

    <TabsContent value="overview" class="space-y-6">
      <!-- Loading State -->
      {#if loading}
        <Card>
          <CardContent class="py-8">
            <div class="flex flex-col items-center justify-center space-y-4">
              <RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
              <p class="text-sm text-muted-foreground">Loading SDK data...</p>
            </div>
          </CardContent>
        </Card>
      {/if}

      {#if !loading}
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Settings class="h-5 w-5 text-muted-foreground" />
              SDK Tools
            </CardTitle>
            <p class="mt-1 text-sm text-muted-foreground">
              Jump to common SDK sections.
            </p>
          </CardHeader>
          <CardContent>
            <div class="grid grid-cols-1 gap-2 md:grid-cols-3">
              <Button variant="outline" onclick={() => (activeTab = "managers")}>
                <Settings class="mr-2 h-4 w-4" />
                SDK Managers
              </Button>
              <Button
                variant="outline"
                onclick={() => goto("/sdk/software-installer")}
              >
                <Package class="mr-2 h-4 w-4" />
                Software Installer
              </Button>
              <Button
                variant="outline"
                onclick={() => goto("/sdk/installations")}
              >
                <Download class="mr-2 h-4 w-4" />
                Installations
              </Button>
            </div>
          </CardContent>
        </Card>
      {/if}

      <!-- Quick Navigation Section -->
      {#if !loading}
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Activity class="h-5 w-5" />
              Quick Navigation
            </CardTitle>
            <p class="mt-1 text-sm text-muted-foreground">
              Quick access to all SDKs and managers. Green indicates
              installed/active, gray indicates not installed.
            </p>
          </CardHeader>
          <CardContent>
            <div class="space-y-6">
              <!-- SDK Managers -->
              {#if quickNavSDKs().manager.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Package class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">SDK Managers</h3>
                  </div>
                  <div
                    class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4"
                  >
                    {#each quickNavSDKs().manager as sdk}
                      <button
                        onclick={() => handleSDKClick(sdk)}
                        class="relative rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium"
                            >{sdk.displayName}</span
                          >
                        </div>
                        <div class="flex items-center justify-between">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground"
                              >{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >Not installed</span
                            >
                          {/if}
                          {#if sdk.installed}
                            <CheckCircle
                              class="h-3 w-3 flex-shrink-0 text-green-600"
                            />
                          {:else}
                            <AlertCircle
                              class="h-3 w-3 flex-shrink-0 text-muted-foreground"
                            />
                          {/if}
                        </div>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Language & Runtime -->
              {#if quickNavSDKs().language.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Code class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">Language & Runtime</h3>
                  </div>
                  <div
                    class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4"
                  >
                    {#each quickNavSDKs().language as sdk}
                      <button
                        onclick={() => handleSDKClick(sdk)}
                        class="relative rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium"
                            >{sdk.displayName}</span
                          >
                        </div>
                        <div class="flex items-center justify-between">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground"
                              >{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >Not installed</span
                            >
                          {/if}
                          <div class="flex items-center gap-1">
                            {#if isServiceRunning(sdk.id)}
                              <Badge
                                variant="default"
                                class="bg-blue-500 px-1 py-0 text-xs"
                                >Running</Badge
                              >
                            {/if}
                            {#if sdk.installed}
                              <CheckCircle
                                class="h-3 w-3 flex-shrink-0 text-green-600"
                              />
                            {:else}
                              <AlertCircle
                                class="h-3 w-3 flex-shrink-0 text-muted-foreground"
                              />
                            {/if}
                          </div>
                        </div>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Database Server -->
              {#if quickNavSDKs().database.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Database class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">Database Server</h3>
                  </div>
                  <div
                    class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4"
                  >
                    {#each quickNavSDKs().database as sdk}
                      <button
                        onclick={() => handleSDKClick(sdk)}
                        class="relative rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium"
                            >{sdk.displayName}</span
                          >
                        </div>
                        <div class="flex items-center justify-between">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground"
                              >{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >Not installed</span
                            >
                          {/if}
                          <div class="flex items-center gap-1">
                            {#if isServiceRunning(sdk.id)}
                              <Badge
                                variant="default"
                                class="bg-blue-500 px-1 py-0 text-xs"
                                >Running</Badge
                              >
                            {/if}
                            {#if sdk.installed}
                              <CheckCircle
                                class="h-3 w-3 flex-shrink-0 text-green-600"
                              />
                            {:else}
                              <AlertCircle
                                class="h-3 w-3 flex-shrink-0 text-muted-foreground"
                              />
                            {/if}
                          </div>
                        </div>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Web Server -->
              {#if quickNavSDKs().web.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Globe class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">Web Server</h3>
                  </div>
                  <div
                    class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4"
                  >
                    {#each quickNavSDKs().web as sdk}
                      <button
                        onclick={() => handleSDKClick(sdk)}
                        class="relative rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium"
                            >{sdk.displayName}</span
                          >
                        </div>
                        <div class="flex items-center justify-between">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground"
                              >{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >Not installed</span
                            >
                          {/if}
                          <div class="flex items-center gap-1">
                            {#if isServiceRunning(sdk.id)}
                              <Badge
                                variant="default"
                                class="bg-blue-500 px-1 py-0 text-xs"
                                >Running</Badge
                              >
                            {/if}
                            {#if sdk.installed}
                              <CheckCircle
                                class="h-3 w-3 flex-shrink-0 text-green-600"
                              />
                            {:else}
                              <AlertCircle
                                class="h-3 w-3 flex-shrink-0 text-muted-foreground"
                              />
                            {/if}
                          </div>
                        </div>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Container Platform -->
              {#if quickNavSDKs().container.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Container class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">Container Platform</h3>
                  </div>
                  <div
                    class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4"
                  >
                    {#each quickNavSDKs().container as sdk}
                      <button
                        onclick={() => handleSDKClick(sdk)}
                        class="relative rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium"
                            >{sdk.displayName}</span
                          >
                        </div>
                        <div class="flex items-center justify-between">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground"
                              >{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >Not installed</span
                            >
                          {/if}
                          <div class="flex items-center gap-1">
                            {#if isServiceRunning(sdk.id)}
                              <Badge
                                variant="default"
                                class="bg-blue-500 px-1 py-0 text-xs"
                                >Running</Badge
                              >
                            {/if}
                            {#if sdk.installed}
                              <CheckCircle
                                class="h-3 w-3 flex-shrink-0 text-green-600"
                              />
                            {:else}
                              <AlertCircle
                                class="h-3 w-3 flex-shrink-0 text-muted-foreground"
                              />
                            {/if}
                          </div>
                        </div>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Empty State -->
              {#if quickNavSDKs().manager.length === 0 && quickNavSDKs().language.length === 0 && quickNavSDKs().database.length === 0 && quickNavSDKs().web.length === 0 && quickNavSDKs().container.length === 0}
                <div class="py-8 text-center text-muted-foreground">
                  <AlertCircle class="mx-auto mb-2 h-12 w-12 opacity-50" />
                  <p class="mb-2 text-sm">No SDKs detected.</p>
                  <p class="text-xs">
                    Click "Refresh" to detect installed SDKs, or install SDK
                    managers to get started.
                  </p>
                </div>
              {/if}
            </div>
          </CardContent>
        </Card>
      {/if}

      <!-- Quick Start Section -->
      {#if installedSDKs.length === 0 && !loading}
        <Card class="border-dashed">
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Zap class="h-5 w-5 text-primary" />
              Quick Start
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="space-y-4">
              <p class="text-muted-foreground">
                Get started with SDK management. Install your first SDK or start
                a service.
              </p>
              <div class="flex gap-2">
                <Button onclick={() => (activeTab = "versions")}>
                  <Download class="mr-2 h-4 w-4" />
                  Install SDK
                </Button>
                <Button
                  variant="outline"
                  onclick={() => (activeTab = "services")}
                >
                  <Play class="mr-2 h-4 w-4" />
                  Start Service
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      {/if}

      <!-- System Status Card -->
      {#if !loading}
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Activity class="h-5 w-5" />
              System Status
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="grid gap-4 md:grid-cols-3">
              <div class="flex items-center gap-3">
                <div class="h-3 w-3 rounded-full bg-green-500"></div>
                <div>
                  <div class="font-medium">System Health</div>
                  <div class="text-sm text-muted-foreground">
                    All systems operational
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <div class="h-3 w-3 rounded-full bg-blue-500"></div>
                <div>
                  <div class="font-medium">Services</div>
                  <div class="text-sm text-muted-foreground">
                    {services.filter((s) => s.status === "running").length} running
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <div class="h-3 w-3 rounded-full bg-purple-500"></div>
                <div>
                  <div class="font-medium">SDK Managers</div>
                  <div class="text-sm text-muted-foreground">
                    {sdkManagers.filter((m) => managerInstallationStatus[m.id])
                      .length} installed
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Quick Actions -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Plus class="h-5 w-5" />
              Quick Actions
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="grid grid-cols-2 gap-4 md:grid-cols-4">
              <Button
                variant="outline"
                class="flex h-20 flex-col gap-2"
                onclick={() => (activeTab = "services")}
              >
                <Play class="h-6 w-6" />
                <span>Start All</span>
              </Button>
              <Button
                variant="outline"
                class="flex h-20 flex-col gap-2"
                onclick={() => (activeTab = "services")}
              >
                <Square class="h-6 w-6" />
                <span>Stop All</span>
              </Button>
              <Button
                variant="outline"
                class="flex h-20 flex-col gap-2"
                onclick={() => (activeTab = "versions")}
              >
                <Download class="h-6 w-6" />
                <span>Install Latest</span>
              </Button>
              <Button
                variant="outline"
                class="flex h-20 flex-col gap-2"
                onclick={() => (activeTab = "versions")}
              >
                <Settings class="h-6 w-6" />
                <span>Configure</span>
              </Button>
            </div>
          </CardContent>
        </Card>

        <!-- Recommended Versions -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Star class="h-5 w-5" />
              Recommended Versions
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="grid gap-3 md:grid-cols-2">
              <div
                class="flex items-center justify-between rounded-lg border p-3"
              >
                <div>
                  <div class="font-medium">Node.js LTS</div>
                  <div class="text-sm text-muted-foreground">
                    18.17.0 - Long Term Support
                  </div>
                </div>
                <Button size="sm" variant="outline">
                  <Download class="mr-2 h-4 w-4" />
                  Install
                </Button>
              </div>
              <div
                class="flex items-center justify-between rounded-lg border p-3"
              >
                <div>
                  <div class="font-medium">Python 3.11</div>
                  <div class="text-sm text-muted-foreground">
                    Latest stable release
                  </div>
                </div>
                <Button size="sm" variant="outline">
                  <Download class="mr-2 h-4 w-4" />
                  Install
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Recent Activity -->
        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="space-y-4">
              {#if recentActivity.length > 0}
                {#each recentActivity as activity}
                  <div class="flex items-center gap-3">
                    {#if activity.type === "version_activated"}
                      <CheckCircle class="h-4 w-4 text-green-600" />
                    {:else if activity.type === "service_started"}
                      <Play class="h-4 w-4 text-blue-600" />
                    {:else if activity.type === "version_installed"}
                      <Download class="h-4 w-4 text-purple-600" />
                    {:else}
                      <AlertCircle class="h-4 w-4 text-gray-600" />
                    {/if}
                    <span class="text-sm">{activity.message}</span>
                    <Badge variant="outline" class="text-xs"
                      >{activity.timestamp}</Badge
                    >
                  </div>
                {/each}
              {:else}
                <div class="py-4 text-center text-muted-foreground">
                  No recent activity
                </div>
              {/if}
            </div>
          </CardContent>
        </Card>
      {/if}
    </TabsContent>

    <TabsContent value="managers" class="space-y-6">
      <!-- SDK Managers Grid -->
      <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {#each sdkManagers as manager}
          <Card class="relative">
            <CardHeader>
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div
                    class="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 font-bold text-white"
                  >
                    {manager.icon === "nodejs"
                      ? "N"
                      : manager.icon === "rust"
                        ? "R"
                        : manager.icon === "python"
                          ? "P"
                          : manager.icon === "java"
                            ? "J"
                            : manager.icon === "ruby"
                              ? "R"
                              : "P"}
                  </div>
                  <div>
                    <CardTitle class="text-lg">{manager.name}</CardTitle>
                    <Badge variant="outline" class="text-xs"
                      >{manager.category}</Badge
                    >
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  {#if managerInstallationStatus[manager.id]}
                    <Badge
                      variant="default"
                      class="bg-green-100 text-green-800"
                    >
                      <CheckCircle class="mr-1 h-3 w-3" />
                      Installed
                    </Badge>
                  {:else}
                    <Badge variant="outline" class="text-gray-500">
                      Not Installed
                    </Badge>
                  {/if}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <p class="mb-4 text-sm text-muted-foreground">
                {manager.description}
              </p>

              <!-- Features -->
              <div class="mb-4">
                <h4 class="mb-2 text-sm font-medium">Features:</h4>
                <div class="flex flex-wrap gap-1">
                  {#each manager.features as feature}
                    <Badge variant="secondary" class="text-xs">{feature}</Badge>
                  {/each}
                </div>
              </div>

              <!-- Actions -->
              <div class="flex items-center gap-2">
                {#if managerInstallationStatus[manager.id]}
                  <Button
                    variant="outline"
                    size="sm"
                    onclick={() => uninstallSDKManager(manager)}
                  >
                    <Square class="mr-1 h-4 w-4" />
                    Uninstall
                  </Button>
                {:else}
                  <Button
                    size="sm"
                    onclick={() => installSDKManager(manager)}
                    disabled={manager.installing}
                  >
                    {#if manager.installing}
                      <Progress
                        value={manager.installProgress}
                        class="mr-1 h-4 w-4"
                      />
                      Installing...
                    {:else}
                      <Download class="mr-1 h-4 w-4" />
                      Install
                    {/if}
                  </Button>
                {/if}

                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => window.open(manager.website, "_blank")}
                >
                  <Globe class="mr-1 h-4 w-4" />
                  Website
                </Button>
              </div>

              <!-- Installation Error -->
              {#if manager.installError}
                <Alert variant="destructive" class="mt-3">
                  <AlertCircle class="h-4 w-4" />
                  <AlertDescription class="text-xs">
                    {manager.installError}
                  </AlertDescription>
                </Alert>
              {/if}
            </CardContent>
          </Card>
        {/each}
      </div>

      <!-- Installation Guide -->
      <Card>
        <CardHeader>
          <CardTitle>Installation Guide</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="space-y-4">
            <p class="text-sm text-muted-foreground">
              SDK Managers help you install and manage multiple versions of
              development tools. Click "Install" on any manager above to get
              started.
            </p>

            <div class="grid gap-4 md:grid-cols-2">
              <div>
                <h4 class="mb-2 font-medium">Why use SDK Managers?</h4>
                <ul class="space-y-1 text-sm text-muted-foreground">
                  <li>• Switch between tool versions easily</li>
                  <li>• Test compatibility across versions</li>
                  <li>• Maintain project-specific environments</li>
                  <li>• Avoid system-wide conflicts</li>
                </ul>
              </div>

              <div>
                <h4 class="mb-2 font-medium">Getting Started</h4>
                <ul class="space-y-1 text-sm text-muted-foreground">
                  <li>• Install managers for your preferred languages</li>
                  <li>• Use the Services tab to run development servers</li>
                  <li>• Manage versions in the Versions tab</li>
                  <li>• Configure environment variables as needed</li>
                </ul>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </TabsContent>

    <TabsContent value="services" class="space-y-6">
      <div class="grid gap-6">
        {#each services as service}
          <ServiceCard
            {service}
            availableVersions={service.availableVersions}
            onToggle={toggleService}
            onVersionChange={changeServiceVersion}
            onConfigure={configureService}
            onViewLogs={viewServiceLogs}
            onOpenUrl={openServiceUrl}
          />
        {/each}
      </div>
    </TabsContent>

    <TabsContent value="versions" class="space-y-6">
      <VersionList
        {versions}
        onInstall={installVersion}
        onUninstall={uninstallVersion}
        onSetActive={setActiveVersion}
        {loading}
      />
    </TabsContent>
  </Tabs>
</div>
