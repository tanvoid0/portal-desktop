<!--
	Software Installer Page
	Install and manage software via multiple package managers (winget, scoop, chocolatey, etc.)
-->

<script lang="ts">
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Input } from "$lib/components/ui/input";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from "$lib/components/ui/dialog";
  import {
    Search,
    RefreshCw,
    Download,
    Trash2,
    ArrowUp,
    Package as PackageIcon,
    CheckCircle,
    XCircle,
    Info,
    Loader2,
  } from "@lucide/svelte";
  import Select from "$lib/components/ui/select.svelte";
  import {
    getAvailablePackageManagers,
    searchPackages,
    listInstalledPackages,
    getPackageDetails,
    installPackage,
    upgradePackage,
    uninstallPackage,
    checkUpdates,
    getPackageManagerInfo,
    type Package,
    type InstalledPackage,
    type PackageDetails,
    type PackageUpdate,
    type PackageManagerInfo,
  } from "$lib/domains/sdk/services/packageManagerService";
  import { toast } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";
  import { PageLoading, PageError } from "$lib/components/shell";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { buildTabUrl, resolveUrlTab } from "$lib/utils/url-tabs";

  const INSTALLER_TABS = ["browse", "installed", "updates"] as const;
  type InstallerTab = (typeof INSTALLER_TABS)[number];

  const activeTab = $derived(
    resolveUrlTab($page.url.searchParams, INSTALLER_TABS, "browse"),
  );

  function setActiveTab(tab: InstallerTab) {
    goto(buildTabUrl($page.url.pathname, $page.url.searchParams, tab), {
      replaceState: true,
      noScroll: true,
    });
  }

  // State
  let loading = $state(true);
  let error = $state<string | null>(null);
  let availableManagers = $state<string[]>([]);
  let selectedManager = $state<string>("");
  let managerInfo = $state<PackageManagerInfo | null>(null);

  // Search state
  let searchQuery = $state("");
  let searchResults = $state<Package[]>([]);
  let searching = $state(false);

  // Installed packages state
  let installedPackages = $state<InstalledPackage[]>([]);
  let loadingInstalled = $state(false);

  // Updates state
  let availableUpdates = $state<PackageUpdate[]>([]);
  let loadingUpdates = $state(false);

  // Package details dialog
  let showDetailsDialog = $state(false);
  let selectedPackageDetails = $state<PackageDetails | null>(null);
  let loadingDetails = $state(false);

  // Operation states
  let installingPackage = $state<string | null>(null);
  let upgradingPackage = $state<string | null>(null);
  let uninstallingPackage = $state<string | null>(null);

  // Debounce timer for search
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  // Load available managers on mount
  $effect(() => {
    loadAvailableManagers();
  });

  // Load manager info when selection changes
  $effect(() => {
    if (selectedManager) {
      (async () => {
        await loadManagerInfo();
        // Load data for current tab when manager info is loaded
        if (managerInfo?.available) {
          if (activeTab === "installed") {
            await loadInstalledPackages();
          } else if (activeTab === "updates") {
            await loadUpdates();
          }
        }
      })();
    }
  });

  async function loadAvailableManagers() {
    loading = true;
    error = null;

    try {
      const managers = await getAvailablePackageManagers();
      console.log("Available package managers:", managers);
      availableManagers = managers;

      // Auto-select first available manager
      if (managers.length > 0 && !selectedManager) {
        selectedManager = managers[0];
        console.log("Auto-selected manager:", selectedManager);
      }
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to load package managers";
      console.error("Failed to load package managers:", err);
      toast.error("Failed to load package managers", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      loading = false;
    }
  }

  async function loadManagerInfo() {
    if (!selectedManager) return;

    try {
      console.log("Loading manager info for:", selectedManager);
      const info = await getPackageManagerInfo(selectedManager);
      console.log("Manager info loaded:", info);
      managerInfo = info;
    } catch (err) {
      console.error("Failed to load manager info:", err);
      toast.error("Failed to load manager info", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
    }
  }

  async function handleSearch() {
    if (!selectedManager || !searchQuery.trim()) {
      searchResults = [];
      return;
    }

    searching = true;
    try {
      const results = await searchPackages(selectedManager, searchQuery);
      searchResults = results;
    } catch (err) {
      toast.error("Search failed", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
      searchResults = [];
    } finally {
      searching = false;
    }
  }

  // Debounced search
  function onSearchInput() {
    if (searchTimer) {
      clearTimeout(searchTimer);
    }

    searchTimer = setTimeout(() => {
      handleSearch();
    }, 500);
  }

  async function loadInstalledPackages() {
    if (!selectedManager) return;

    loadingInstalled = true;
    try {
      console.log("Loading installed packages for:", selectedManager);
      const packages = await listInstalledPackages(selectedManager);
      console.log("Installed packages loaded:", packages.length);
      installedPackages = packages;
    } catch (err) {
      console.error("Failed to load installed packages:", err);
      toast.error("Failed to load installed packages", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
      installedPackages = [];
    } finally {
      loadingInstalled = false;
    }
  }

  async function loadUpdates() {
    if (!selectedManager) return;

    loadingUpdates = true;
    try {
      console.log("Loading updates for:", selectedManager);
      const updates = await checkUpdates(selectedManager);
      console.log("Updates loaded:", updates.length);
      availableUpdates = updates;
    } catch (err) {
      console.error("Failed to check updates:", err);
      toast.error("Failed to check updates", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
      availableUpdates = [];
    } finally {
      loadingUpdates = false;
    }
  }

  async function showPackageDetails(pkg: Package | InstalledPackage) {
    if (!selectedManager) return;

    loadingDetails = true;
    showDetailsDialog = true;

    try {
      const details = await getPackageDetails(selectedManager, pkg.id);
      selectedPackageDetails = details;
    } catch (err) {
      toast.error("Failed to load package details", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
      showDetailsDialog = false;
    } finally {
      loadingDetails = false;
    }
  }

  async function handleInstall(pkg: Package) {
    if (!selectedManager) return;

    installingPackage = pkg.id;
    try {
      await installPackage(selectedManager, pkg.id, pkg.version);
      toast.success("Package installed", {
        description: `${pkg.name} has been installed successfully`,
      });
      // Refresh installed packages if on that tab
      if (activeTab === "installed") {
        await loadInstalledPackages();
      }
    } catch (err) {
      toast.error("Installation failed", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      installingPackage = null;
    }
  }

  async function handleUpgrade(pkg: InstalledPackage | PackageUpdate) {
    if (!selectedManager) return;

    upgradingPackage = pkg.id;
    try {
      await upgradePackage(selectedManager, pkg.id);
      toast.success("Package upgraded", {
        description: `${pkg.name} has been upgraded successfully`,
      });
      // Refresh data
      await loadInstalledPackages();
      await loadUpdates();
    } catch (err) {
      toast.error("Upgrade failed", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      upgradingPackage = null;
    }
  }

  async function handleUninstall(pkg: InstalledPackage) {
    if (!selectedManager) return;

    const confirmed = await confirmAction(
      `Are you sure you want to uninstall ${pkg.name}?`,
      "Uninstall package",
    );
    if (!confirmed) return;

    uninstallingPackage = pkg.id;
    try {
      await uninstallPackage(selectedManager, pkg.id);
      toast.success("Package uninstalled", {
        description: `${pkg.name} has been uninstalled successfully`,
      });
      await loadInstalledPackages();
    } catch (err) {
      toast.error("Uninstall failed", {
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      uninstallingPackage = null;
    }
  }

  // Load data when tab changes
  $effect(() => {
    if (selectedManager && managerInfo?.available) {
      if (activeTab === "installed") {
        loadInstalledPackages();
      } else if (activeTab === "updates") {
        loadUpdates();
      }
    }
  });

  function getManagerDisplayName(name: string): string {
    const names: Record<string, string> = {
      winget: "Windows Package Manager",
      scoop: "Scoop",
      chocolatey: "Chocolatey",
      cargo: "Cargo",
      homebrew: "Homebrew",
      npm: "NPM",
      pip: "Pip",
    };
    return names[name] || name;
  }
</script>

<svelte:head>
  <title>Software Installer - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  {#if loading && availableManagers.length === 0}
    <PageLoading message="Loading package managers..." />
  {:else}
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div class="flex-1">
      <h1 class="text-3xl font-bold">Software Installer</h1>
      <p class="text-muted-foreground">
        Install and manage software via package managers
      </p>
    </div>
    <div class="flex items-center gap-2">
      <Button
        variant="outline"
        onclick={loadAvailableManagers}
        disabled={loading}
      >
        <RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    </div>
  </div>

  {#if error}
    <PageError
      title="Failed to load package managers"
      message={error}
      onRetry={loadAvailableManagers}
    />
  {/if}

  <!-- Manager Selection and Search -->
  <Card>
    <CardContent class="pt-6">
      <div class="flex items-center gap-4">
        <!-- Package Manager Selector -->
        <div class="flex-shrink-0" style="width: 250px;">
          <div class="mb-2 text-sm font-medium">Package Manager</div>
          <Select
            options={availableManagers.map((m) => ({
              value: m,
              label: getManagerDisplayName(m),
            }))}
            value={selectedManager}
            placeholder="Select a package manager"
            onSelect={(value) => {
              selectedManager = value;
              searchResults = [];
              searchQuery = "";
              installedPackages = [];
              availableUpdates = [];
              managerInfo = null;
            }}
            disabled={loading || availableManagers.length === 0}
          />
          {#if managerInfo}
            <div class="mt-2 flex flex-wrap items-center gap-2">
              {#if managerInfo.available}
                <Badge
                  variant="default"
                  class="bg-green-100 text-xs text-green-800"
                >
                  <CheckCircle class="mr-1 h-3 w-3" />
                  Available
                </Badge>
              {:else}
                <Badge variant="outline" class="text-xs text-gray-500">
                  <XCircle class="mr-1 h-3 w-3" />
                  Not Available
                </Badge>
              {/if}
              {#if managerInfo.requires_elevation}
                <Badge variant="secondary" class="text-xs">Requires Admin</Badge
                >
              {/if}
            </div>
          {/if}
        </div>

        <!-- Search Bar -->
        {#if selectedManager && managerInfo?.available}
          <div class="flex-1">
            <div class="mb-2 text-sm font-medium">Search Packages</div>
            <div class="flex items-center gap-2">
              <div class="relative flex-1">
                <Search
                  class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
                />
                <Input
                  type="text"
                  placeholder="Search packages..."
                  bind:value={searchQuery}
                  oninput={onSearchInput}
                  class="pl-10"
                  disabled={!managerInfo.supports_search}
                  aria-label="Search packages"
                />
              </div>
              {#if searching}
                <Loader2 class="h-5 w-5 animate-spin text-muted-foreground" />
              {/if}
            </div>
          </div>
        {/if}
      </div>
      {#if managerInfo}
        <div class="mt-3 text-xs text-muted-foreground">
          Version: {managerInfo.version} | Platform: {managerInfo.platform}
        </div>
      {/if}
    </CardContent>
  </Card>

  {#if !selectedManager || !managerInfo?.available}
    <Card>
      <CardContent class="pt-6">
        <div class="py-12 text-center">
          <PackageIcon class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
          <p class="text-muted-foreground">
            {availableManagers.length === 0
              ? "No package managers are available on your system."
              : "Please select an available package manager to get started."}
          </p>
        </div>
      </CardContent>
    </Card>
  {:else}
    <!-- Tabs -->
    <Tabs
      value={activeTab}
      onValueChange={(v) => setActiveTab(v as InstallerTab)}
    >
      <TabsList>
        <TabsTrigger value="browse">
          Browse ({searchResults.length})
        </TabsTrigger>
        <TabsTrigger value="installed">
          Installed ({installedPackages.length})
        </TabsTrigger>
        <TabsTrigger value="updates">
          Updates ({availableUpdates.length})
        </TabsTrigger>
      </TabsList>

      <!-- Browse Tab -->
      <TabsContent value="browse" class="space-y-4">
        {#if searching}
          <div class="flex items-center justify-center py-12">
            <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        {:else if searchResults.length === 0 && searchQuery}
          <div class="py-12 text-center">
            <p class="text-muted-foreground">No packages found</p>
          </div>
        {:else if searchResults.length === 0 && !searchQuery}
          <div class="py-12 text-center">
            <Search class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
            <p class="mb-2 text-muted-foreground">
              Enter a search query to find packages
            </p>
            <p class="text-sm text-muted-foreground">
              Try searching for popular packages like "chrome", "vscode", or
              "git"
            </p>
          </div>
        {:else}
          <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {#each searchResults as pkg}
              <Card>
                <CardHeader>
                  <div class="flex items-start justify-between">
                    <div class="flex-1">
                      <CardTitle class="text-lg">{pkg.name}</CardTitle>
                      {#if pkg.version}
                        <p class="mt-1 text-xs text-muted-foreground">
                          v{pkg.version}
                        </p>
                      {/if}
                    </div>
                    <Badge variant="secondary" class="text-xs">
                      {pkg.source}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent>
                  {#if pkg.description}
                    <p class="mb-4 line-clamp-2 text-sm text-muted-foreground">
                      {pkg.description}
                    </p>
                  {/if}
                  <div class="flex items-center gap-2">
                    <Button
                      variant="default"
                      size="sm"
                      onclick={() => handleInstall(pkg)}
                      disabled={installingPackage === pkg.id}
                      class="flex-1"
                    >
                      {#if installingPackage === pkg.id}
                        <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                        Installing...
                      {:else}
                        <Download class="mr-1 h-4 w-4" />
                        Install
                      {/if}
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={() => showPackageDetails(pkg)}
                    >
                      <Info class="h-4 w-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
            {/each}
          </div>
        {/if}
      </TabsContent>

      <!-- Installed Tab -->
      <TabsContent value="installed" class="space-y-4">
        <div class="flex items-center justify-between">
          <CardTitle>Installed Packages</CardTitle>
          <Button
            variant="outline"
            size="sm"
            onclick={loadInstalledPackages}
            disabled={loadingInstalled}
          >
            <RefreshCw
              class="mr-2 h-4 w-4 {loadingInstalled ? 'animate-spin' : ''}"
            />
            Refresh
          </Button>
        </div>
        {#if loadingInstalled}
          <div class="flex items-center justify-center py-12">
            <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        {:else if installedPackages.length === 0}
          <div class="py-12 text-center">
            <PackageIcon class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
            <p class="mb-2 text-muted-foreground">No packages installed</p>
            <p class="text-sm text-muted-foreground">
              Click Refresh to load installed packages
            </p>
          </div>
        {:else}
          <div class="space-y-2">
            {#each installedPackages as pkg}
              <Card>
                <CardContent class="pt-6">
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <div class="flex items-center gap-2">
                        <h3 class="font-semibold">{pkg.name}</h3>
                        <Badge variant="secondary" class="text-xs">
                          {pkg.source}
                        </Badge>
                      </div>
                      <p class="text-sm text-muted-foreground">
                        Version: {pkg.version}
                      </p>
                    </div>
                    <div class="flex items-center gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => showPackageDetails(pkg)}
                      >
                        <Info class="h-4 w-4" />
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => handleUpgrade(pkg)}
                        disabled={upgradingPackage === pkg.id}
                      >
                        {#if upgradingPackage === pkg.id}
                          <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                        {:else}
                          <ArrowUp class="mr-1 h-4 w-4" />
                        {/if}
                        Upgrade
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => handleUninstall(pkg)}
                        disabled={uninstallingPackage === pkg.id}
                      >
                        {#if uninstallingPackage === pkg.id}
                          <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                        {:else}
                          <Trash2 class="mr-1 h-4 w-4" />
                        {/if}
                        Uninstall
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            {/each}
          </div>
        {/if}
      </TabsContent>

      <!-- Updates Tab -->
      <TabsContent value="updates" class="space-y-4">
        <div class="flex items-center justify-between">
          <CardTitle>Available Updates</CardTitle>
          <Button
            variant="outline"
            size="sm"
            onclick={loadUpdates}
            disabled={loadingUpdates}
          >
            <RefreshCw
              class="mr-2 h-4 w-4 {loadingUpdates ? 'animate-spin' : ''}"
            />
            Check Updates
          </Button>
        </div>
        {#if loadingUpdates}
          <div class="flex items-center justify-center py-12">
            <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        {:else if availableUpdates.length === 0}
          <div class="py-12 text-center">
            <CheckCircle class="mx-auto mb-4 h-12 w-12 text-green-500" />
            <p class="mb-2 text-muted-foreground">
              All packages are up to date
            </p>
            <p class="text-sm text-muted-foreground">
              Click "Check Updates" to refresh
            </p>
          </div>
        {:else}
          <div class="space-y-2">
            {#each availableUpdates as update}
              <Card>
                <CardContent class="pt-6">
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <div class="flex items-center gap-2">
                        <h3 class="font-semibold">{update.name}</h3>
                        <Badge variant="secondary" class="text-xs">
                          {update.source}
                        </Badge>
                      </div>
                      <p class="text-sm text-muted-foreground">
                        {update.current_version} → {update.available_version}
                      </p>
                    </div>
                    <Button
                      variant="default"
                      size="sm"
                      onclick={() => handleUpgrade(update)}
                      disabled={upgradingPackage === update.id}
                    >
                      {#if upgradingPackage === update.id}
                        <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                        Upgrading...
                      {:else}
                        <ArrowUp class="mr-1 h-4 w-4" />
                        Upgrade
                      {/if}
                    </Button>
                  </div>
                </CardContent>
              </Card>
            {/each}
          </div>
        {/if}
      </TabsContent>
    </Tabs>
  {/if}
  {/if}
</div>

<!-- Package Details Dialog -->
<Dialog bind:open={showDetailsDialog}>
  <DialogContent class="max-h-[80vh] max-w-2xl overflow-y-auto">
    <DialogHeader>
      <DialogTitle>
        {selectedPackageDetails?.name || "Package Details"}
      </DialogTitle>
      <DialogDescription>
        Package information from {selectedPackageDetails?.source ||
          "package manager"}
      </DialogDescription>
    </DialogHeader>
    {#if loadingDetails}
      <div class="flex items-center justify-center py-8">
        <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    {:else if selectedPackageDetails}
      <div class="space-y-4">
        <div>
          <p class="text-sm font-medium">ID</p>
          <p class="font-mono text-sm text-muted-foreground">
            {selectedPackageDetails.id}
          </p>
        </div>
        {#if selectedPackageDetails.version}
          <div>
            <p class="text-sm font-medium">Version</p>
            <p class="text-sm text-muted-foreground">
              {selectedPackageDetails.version}
            </p>
          </div>
        {/if}
        {#if selectedPackageDetails.publisher}
          <div>
            <p class="text-sm font-medium">Publisher</p>
            <p class="text-sm text-muted-foreground">
              {selectedPackageDetails.publisher}
            </p>
          </div>
        {/if}
        {#if selectedPackageDetails.description}
          <div>
            <p class="text-sm font-medium">Description</p>
            <p class="text-sm text-muted-foreground">
              {selectedPackageDetails.description}
            </p>
          </div>
        {/if}
        {#if selectedPackageDetails.homepage}
          <div>
            <p class="text-sm font-medium">Homepage</p>
            <a
              href={selectedPackageDetails.homepage}
              target="_blank"
              class="text-sm text-blue-600 hover:underline"
            >
              {selectedPackageDetails.homepage}
            </a>
          </div>
        {/if}
        {#if selectedPackageDetails.license}
          <div>
            <p class="text-sm font-medium">License</p>
            <p class="text-sm text-muted-foreground">
              {selectedPackageDetails.license}
            </p>
          </div>
        {/if}
        {#if selectedPackageDetails.dependencies.length > 0}
          <div>
            <p class="text-sm font-medium">Dependencies</p>
            <div class="mt-2 flex flex-wrap gap-2">
              {#each selectedPackageDetails.dependencies as dep}
                <Badge variant="secondary" class="text-xs">{dep}</Badge>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </DialogContent>
</Dialog>
