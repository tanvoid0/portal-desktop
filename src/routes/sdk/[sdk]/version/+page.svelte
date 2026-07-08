<!--
	SDK Version Page
	Version management for a specific SDK
-->

<script lang="ts">
  import { page } from "$app/stores";
  import { invokeClient } from "$lib/utils/invokeClient";
  import {
    sdkConfigService,
    type ProcessedSDKConfig,
  } from "$lib/domains/sdk/services/sdkConfigService";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Alert, AlertDescription } from "$lib/components/ui/alert";
  import { AlertCircle, RefreshCw, ArrowLeft } from "@lucide/svelte";
  import VersionList from "$lib/domains/sdk/components/VersionList.svelte";
  import { goto } from "$app/navigation";
  import { confirmAction } from "$lib/utils/confirm";
  import { PageLoading, PageError } from "$lib/components/shell";

  interface VersionInfo {
    version: string;
    installed: boolean;
    active: boolean;
    downloading: boolean;
    progress?: number;
    error?: string;
    lts?: boolean;
    releaseDate?: string;
    description?: string;
  }

  // Get SDK ID from URL
  let sdkId = $derived($page.params.sdk);

  // State
  let loading = $state(true);
  let error = $state<string | null>(null);
  let sdkConfig = $state<ProcessedSDKConfig | null>(null);
  let versions = $state<VersionInfo[]>([]);
  let loadingVersions = $state(false);
  let installingVersion = $state<string | null>(null);
  let switchingVersion = $state<string | null>(null);
  let uninstallingVersion = $state<string | null>(null);

  // Initialize data
  $effect(() => {
    loadData();
  });

  async function loadData() {
    loading = true;
    error = null;

    try {
      if (!sdkId) {
        error = "SDK ID is required";
        return;
      }

      const config = await sdkConfigService.getSDKConfig(sdkId);
      if (!config) {
        error = `SDK '${sdkId}' not found`;
        return;
      }

      sdkConfig = config;

      // Load versions from SDK manager
      await loadVersions();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to load SDK configuration";
      console.error("Failed to load SDK config:", err);
    } finally {
      loading = false;
    }
  }

  async function loadVersions() {
    if (!sdkConfig) return;

    loadingVersions = true;
    error = null;

    try {
      // Find the first installed SDK manager for this SDK
      const manager = sdkConfig.sdk_managers.find((m) => m.installed);

      if (!manager) {
        // No manager installed, show empty list
        versions = [];
        return;
      }

      // Load installed and available versions
      const [
        installedVersionsResult,
        availableVersionsResult,
        currentVersionResult,
      ] = await Promise.all([
        invokeClient
          .post<
            string[]
          >("get_manager_installed_versions", { managerName: manager.id })
          .catch(() => [] as string[]),
        invokeClient
          .post<
            string[]
          >("get_manager_available_versions", { managerName: manager.id })
          .catch(() => [] as string[]),
        invokeClient
          .post<
            string | null
          >("get_manager_current_version", { managerName: manager.id })
          .catch(() => null as string | null),
      ]);

      const installedVersions: string[] = installedVersionsResult || [];
      const availableVersions: string[] = availableVersionsResult || [];
      const currentVersion: string | null = currentVersionResult || null;

      // Combine installed and available versions
      const allVersions = new Set<string>();
      installedVersions.forEach((v) => allVersions.add(v));
      availableVersions.forEach((v) => allVersions.add(v));

      // Convert to VersionInfo format
      versions = Array.from(allVersions).map((version) => ({
        version,
        installed: installedVersions.includes(version),
        active: currentVersion === version,
        downloading: installingVersion === version,
        error: undefined,
        lts:
          version.toLowerCase().includes("lts") ||
          version.match(/^\d+\.\d+\.\d+$/)?.[0] !== undefined,
        description: undefined,
        releaseDate: undefined,
      }));

      // Sort versions (newest first, handle semantic versioning)
      versions.sort((a, b) => {
        // Try to parse as semantic versions
        const aParts = a.version.replace(/^v/, "").split(".").map(Number);
        const bParts = b.version.replace(/^v/, "").split(".").map(Number);

        // If both are valid semantic versions, compare numerically
        if (aParts.every((p) => !isNaN(p)) && bParts.every((p) => !isNaN(p))) {
          for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
            const aPart = aParts[i] || 0;
            const bPart = bParts[i] || 0;
            if (aPart !== bPart) {
              return bPart - aPart; // Descending order
            }
          }
        }

        // Fallback to string comparison
        return b.version.localeCompare(a.version);
      });
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load versions";
      console.error("Failed to load versions:", err);
    } finally {
      loadingVersions = false;
    }
  }

  async function installVersion(version: VersionInfo) {
    if (!sdkConfig) return;

    const manager = sdkConfig.sdk_managers.find((m) => m.installed);
    if (!manager) {
      error = "No SDK manager installed for this SDK";
      return;
    }

    installingVersion = version.version;
    version.downloading = true;
    version.error = undefined;

    try {
      await invokeClient.post<string>("install_version_via_manager", {
        managerName: manager.id,
        version: version.version,
      });

      // Reload versions to update status
      await loadVersions();
    } catch (err) {
      version.error =
        err instanceof Error ? err.message : "Failed to install version";
      console.error("Failed to install version:", err);
    } finally {
      installingVersion = null;
      version.downloading = false;
    }
  }

  async function switchVersion(version: VersionInfo) {
    if (!sdkConfig) return;

    const manager = sdkConfig.sdk_managers.find((m) => m.installed);
    if (!manager) {
      error = "No SDK manager installed for this SDK";
      return;
    }

    switchingVersion = version.version;

    try {
      await invokeClient.post<string>("switch_version_via_manager", {
        managerName: manager.id,
        version: version.version,
      });

      // Reload versions to update active status
      await loadVersions();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to switch version";
      console.error("Failed to switch version:", err);
    } finally {
      switchingVersion = null;
    }
  }

  async function uninstallVersion(version: VersionInfo) {
    if (!sdkConfig) return;

    const manager = sdkConfig.sdk_managers.find((m) => m.installed);
    if (!manager) {
      error = "No SDK manager installed for this SDK";
      return;
    }

    const confirmed = await confirmAction(
      `Are you sure you want to uninstall version ${version.version}?`,
      "Uninstall version",
    );
    if (!confirmed) return;

    uninstallingVersion = version.version;

    try {
      await invokeClient.post<string>("uninstall_version_via_manager", {
        managerName: manager.id,
        version: version.version,
      });

      // Reload versions to update status
      await loadVersions();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to uninstall version";
      console.error("Failed to uninstall version:", err);
    } finally {
      uninstallingVersion = null;
    }
  }
</script>

<div class="space-y-6">
  {#if loading}
    <PageLoading message="Loading SDK versions..." />
  {:else if error}
    <PageError
      title="Failed to load SDK versions"
      message={error}
      onRetry={loadData}
    />
  {:else if sdkConfig}
    <div class="space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <Button
            variant="ghost"
            size="sm"
            onclick={() => goto(`/sdk/${sdkId}`)}
          >
            <ArrowLeft class="mr-2 h-4 w-4" />
            Back
          </Button>
          <div>
            <h1 class="text-3xl font-bold">Version Management</h1>
            <p class="text-muted-foreground">
              {sdkConfig.display_name} Versions
            </p>
          </div>
        </div>
      </div>

      <!-- Version Content -->
      {#if sdkConfig.sdk_managers.length === 0 || !sdkConfig.sdk_managers.some((m) => m.installed)}
        <Card>
          <CardContent class="pt-6">
            <Alert>
              <AlertCircle class="h-4 w-4" />
              <AlertDescription>
                No SDK manager is installed for this SDK. Install a version
                manager (e.g., {sdkConfig.sdk_managers
                  .map((m) => m.display_name)
                  .join(", ")}) to manage versions.
              </AlertDescription>
            </Alert>
          </CardContent>
        </Card>
      {:else}
        <Card>
          <CardHeader>
            <div class="flex items-center justify-between">
              <CardTitle>Versions</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onclick={loadVersions}
                disabled={loadingVersions}
              >
                <RefreshCw
                  class={`mr-2 h-4 w-4 ${loadingVersions ? "animate-spin" : ""}`}
                />
                Refresh
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <VersionList
              {versions}
              onInstall={installVersion}
              onUninstall={uninstallVersion}
              onSetActive={switchVersion}
              loading={loadingVersions}
            />
          </CardContent>
        </Card>
      {/if}
    </div>
  {/if}
</div>
