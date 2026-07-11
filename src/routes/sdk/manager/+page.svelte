<!--
	SDK Managers Page
	Shows all SDK managers and their installation status
-->

<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import {
    CheckCircle,
    XCircle,
    Download,
    ExternalLink,
    RefreshCw,
    ArrowRight,
    Loader2,
  } from "@lucide/svelte";
  import Devicon from "$lib/components/ui/devicon.svelte";
  import { PageLoading, PageError, PageEmpty } from "$lib/components/shell";
  import { toast } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";
  import {
    listSDKManagers,
    installSDKManager,
    uninstallSDKManager,
    type SDKManagerInfo,
  } from "$lib/domains/sdk/services/sdkManagerService";

  // State
  let loading = $state(true);
  let error = $state<string | null>(null);
  let managers = $state<SDKManagerInfo[]>([]);

  // Load managers on mount
  $effect(() => {
    loadManagers();
  });

  async function loadManagers() {
    loading = true;
    error = null;

    try {
      managers = await listSDKManagers();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to load SDK managers";
      console.error("Failed to load SDK managers:", err);
    } finally {
      loading = false;
    }
  }

  let installingManager = $state<string | null>(null);
  let uninstallingManager = $state<string | null>(null);
  let installError = $state<Record<string, string>>({});

  async function installManager(manager: SDKManagerInfo) {
    if (!manager.install_available) {
      installError[manager.id] =
        manager.install_unavailable_reason ||
        "Automatic installation is not available for this manager.";
      toast.error("Install unavailable", {
        description: installError[manager.id],
      });
      return;
    }

    installingManager = manager.id;
    installError[manager.id] = "";

    try {
      const result = await installSDKManager(manager.id);
      toast.success("Manager installed", {
        description: result,
      });
      await loadManagers();
    } catch (err) {
      installError[manager.id] =
        err instanceof Error ? err.message : "Installation failed";
      toast.error("Installation failed", {
        description: installError[manager.id],
      });
    } finally {
      installingManager = null;
    }
  }

  async function uninstallManager(manager: SDKManagerInfo) {
    if (!manager.uninstall_available) {
      installError[manager.id] =
        manager.uninstall_unavailable_reason ||
        "Automatic uninstall is not available for this manager.";
      toast.error("Uninstall unavailable", {
        description: installError[manager.id],
      });
      return;
    }

    const confirmed = await confirmAction(
      `Are you sure you want to uninstall ${manager.display_name}?`,
      "Uninstall SDK manager",
    );
    if (!confirmed) return;

    uninstallingManager = manager.id;
    installError[manager.id] = "";

    try {
      const result = await uninstallSDKManager(manager.id);
      toast.success("Manager uninstalled", {
        description: result,
      });
      await loadManagers();
    } catch (err) {
      installError[manager.id] =
        err instanceof Error ? err.message : "Failed to uninstall manager";
      toast.error("Uninstall failed", {
        description: installError[manager.id],
      });
    } finally {
      uninstallingManager = null;
    }
  }

  function getManagerIcon(managerId: string): string {
    const iconMap: Record<string, string> = {
      nvm: "devicon-nodejs-plain",
      pyenv: "devicon-python-plain",
      rustup: "devicon-rust-plain",
      sdkman: "devicon-sdkman-plain",
      goenv: "devicon-go-plain",
      rbenv: "devicon-ruby-plain",
      phpenv: "devicon-php-plain",
      fnm: "devicon-nodejs-plain",
    };
    return iconMap[managerId.toLowerCase()] || "devicon-devicon-plain";
  }

  function navigateToManagerDetails(managerId: string) {
    goto(`/sdk/manager/${managerId}`);
  }
</script>

<svelte:head>
  <title>SDK Managers - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center gap-4">
    <div class="flex-1">
      <h1 class="text-3xl font-bold">SDK Managers</h1>
      <p class="text-muted-foreground">
        Install and manage SDK version managers for different programming
        languages
      </p>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="outline" onclick={loadManagers} disabled={loading}>
        <RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    </div>
  </div>

  <!-- Content -->
  {#if loading}
    <PageLoading message="Loading SDK managers..." />
  {:else if error}
    <PageError title="Failed to load SDK managers" message={error} onRetry={loadManagers} />
  {:else if managers.length === 0}
    <PageEmpty
      title="No SDK managers found"
      description="SDK managers will appear here once detected on your system."
      actionLabel="Refresh"
      onAction={loadManagers}
    />
  {:else}
    <!-- Managers Grid -->
    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      {#each managers as manager}
        <Card class="relative">
          <CardHeader>
            <div class="flex items-center justify-between">
              <Button
                type="button"
                variant="ghost"
                class="h-auto flex-1 justify-start gap-3 px-0 text-left hover:opacity-80"
                onclick={() => navigateToManagerDetails(manager.id)}
                onkeydown={(e) =>
                  e.key === "Enter" && navigateToManagerDetails(manager.id)}
              >
                <Devicon icon={getManagerIcon(manager.id)} class="h-10 w-10" />
                <div>
                  <CardTitle class="text-lg">{manager.display_name}</CardTitle>
                  {#if manager.version}
                    <p class="mt-1 text-xs text-muted-foreground">
                      {manager.version}
                    </p>
                  {/if}
                </div>
              </Button>
              <div class="flex items-center gap-2">
                {#if manager.installed}
                  <Badge variant="default" class="bg-green-100 text-green-800">
                    <CheckCircle class="mr-1 h-3 w-3" />
                    Installed
                  </Badge>
                {:else}
                  <Badge variant="outline" class="text-gray-500">
                    <XCircle class="mr-1 h-3 w-3" />
                    Not Installed
                  </Badge>
                {/if}
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <!-- Features -->
            <div class="mb-4">
              <div class="flex flex-wrap gap-1">
                {#if manager.supports_installation}
                  <Badge variant="secondary" class="text-xs">Installation</Badge
                  >
                {/if}
                {#if manager.supports_version_switching}
                  <Badge variant="secondary" class="text-xs"
                    >Version Switching</Badge
                  >
                {/if}
              </div>
            </div>

            <!-- Actions -->
            <div class="flex flex-wrap items-center gap-2">
              <Button
                variant="default"
                size="sm"
                onclick={() => navigateToManagerDetails(manager.id)}
                class="flex-1"
              >
                View Details
                <ArrowRight class="ml-1 h-4 w-4" />
              </Button>

              {#if manager.installed}
                <Button
                  variant="outline"
                  size="sm"
                  onclick={() => uninstallManager(manager)}
                  disabled={
                    uninstallingManager === manager.id ||
                    installingManager === manager.id ||
                    !manager.uninstall_available
                  }
                >
                  {#if uninstallingManager === manager.id}
                    <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                    Uninstalling...
                  {:else}
                    <XCircle class="mr-1 h-4 w-4" />
                    Uninstall
                  {/if}
                </Button>
              {:else if manager.supports_installation && manager.install_command}
                <Button
                  variant="outline"
                  size="sm"
                  onclick={() => installManager(manager)}
                  disabled={
                    installingManager === manager.id || !manager.install_available
                  }
                >
                  {#if installingManager === manager.id}
                    <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                    Installing...
                  {:else}
                    <Download class="mr-1 h-4 w-4" />
                    Install
                  {/if}
                </Button>
              {/if}

              {#if manager.website}
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => window.open(manager.website!, "_blank")}
                >
                  <ExternalLink class="mr-1 h-4 w-4" />
                  Website
                </Button>
              {/if}
            </div>

            <!-- Installation Command -->
            {#if manager.install_command && !manager.installed}
              <div class="mt-4">
                <p class="mb-1 text-xs font-medium text-muted-foreground">
                  Installation Command:
                </p>
                <code
                  class="block break-all rounded bg-muted p-2 font-mono text-xs"
                >
                  {manager.install_command}
                </code>
              </div>
            {/if}

            {#if !manager.installed && manager.install_unavailable_reason}
              <div class="mt-3 rounded-md border border-amber-200 bg-amber-50 p-3">
                <p class="text-xs text-amber-700">
                  {manager.install_unavailable_reason}
                </p>
              </div>
            {/if}

            {#if manager.installed && manager.uninstall_unavailable_reason}
              <div class="mt-3 rounded-md border border-amber-200 bg-amber-50 p-3">
                <p class="text-xs text-amber-700">
                  {manager.uninstall_unavailable_reason}
                </p>
              </div>
            {/if}

            <!-- Installation Error -->
            {#if installError[manager.id]}
              <div class="mt-3 rounded-md border border-red-200 bg-red-50 p-3">
                <p class="text-xs text-red-600">
                  {installError[manager.id]}
                </p>
              </div>
            {/if}
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}
</div>
