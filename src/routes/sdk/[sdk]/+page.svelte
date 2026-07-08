<!--
	SDK Overview Page (Default)
	Shows overview information for a specific SDK
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
  import { Badge } from "$lib/components/ui/badge";
  import {
    CheckCircle,
    XCircle,
    Download,
    ExternalLink,
  } from "@lucide/svelte";
  import Devicon from "$lib/components/ui/devicon.svelte";
  import { goto } from "$app/navigation";
  import { PageLoading, PageError } from "$lib/components/shell";

  // Get SDK ID from URL
  let sdkId = $derived($page.params.sdk);

  // State
  let loading = $state(true);
  let error = $state<string | null>(null);
  let sdkConfig = $state<ProcessedSDKConfig | null>(null);

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
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to load SDK configuration";
      console.error("Failed to load SDK config:", err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="space-y-6">
  {#if loading}
    <PageLoading message="Loading SDK..." />
  {:else if error}
    <PageError title="Failed to load SDK" message={error} onRetry={loadData} />
  {:else if sdkConfig}
    {@const isInstalled =
      sdkConfig.sdk_installed ||
      sdkConfig.sdk_managers.some((m) => m.installed)}
    {@const rawVersion =
      sdkConfig.sdk_version ||
      sdkConfig.sdk_managers.find((m) => m.installed)?.version}
    {@const displayVersion = rawVersion
      ? rawVersion.trim().replace(/^v/, "")
      : null}
    {@const port = sdkConfig.service_port}
    {@const isRunning = sdkConfig.service_running}
    {@const isServiceStatusKnown =
      sdkConfig.service_running !== null &&
      sdkConfig.service_running !== undefined}
    <div class="space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <Devicon icon={sdkConfig.icon} class="h-12 w-12" />
          <div>
            <h1 class="text-3xl font-bold">{sdkConfig.display_name}</h1>
            <p class="text-muted-foreground">{sdkConfig.description}</p>
          </div>
        </div>
        <!-- Status Badge in top right -->
        <div class="flex items-center gap-2">
          {#if isInstalled}
            <Badge variant="default" class="bg-green-100 text-green-800">
              <CheckCircle class="mr-1 h-3 w-3" />
              Installed
              {#if displayVersion}
                <span class="ml-1">({displayVersion})</span>
              {/if}
            </Badge>
          {:else}
            <Badge variant="outline">
              <XCircle class="mr-1 h-3 w-3" />
              Not Installed
            </Badge>
          {/if}
          {#if port}
            <Badge variant="outline" class="text-xs">
              Port: {port}
            </Badge>
          {/if}
          {#if isServiceStatusKnown}
            {#if isRunning}
              <Badge variant="default" class="bg-green-500 text-white">
                Running
              </Badge>
            {:else if isInstalled}
              <Badge variant="outline" class="bg-yellow-100 text-yellow-800">
                Stopped
              </Badge>
            {/if}
          {/if}
        </div>
      </div>

      <!-- Navigation Links -->
      <div class="flex flex-wrap gap-2">
        {#if sdkConfig.tabs.some((t) => t.id === "service") && sdkConfig.service_config}
          <Button
            variant="outline"
            onclick={() => goto(`/sdk/${sdkId}/service`)}
          >
            Service
          </Button>
        {/if}
        {#if sdkConfig.tabs.some((t) => t.id === "models") && sdkConfig.category_features?.modelManagement}
          <Button
            variant="outline"
            onclick={() => goto(`/sdk/${sdkId}/models`)}
          >
            Models
          </Button>
        {/if}
        {#if sdkConfig.tabs.some((t) => t.id === "version")}
          <Button
            variant="outline"
            onclick={() => goto(`/sdk/${sdkId}/version`)}
          >
            Version
          </Button>
        {/if}
        {#if sdkConfig.tabs.some((t) => t.id === "package-manager")}
          <Button
            variant="outline"
            onclick={() => goto(`/sdk/${sdkId}/package-manager`)}
          >
            Package Manager
          </Button>
        {/if}
        {#if sdkConfig.tabs.some((t) => t.id === "projects")}
          <Button
            variant="outline"
            onclick={() => goto(`/sdk/${sdkId}/projects`)}
          >
            Projects
          </Button>
        {/if}
      </div>

      <!-- Overview Content -->
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <!-- Manager Information Card -->
        <Card>
          <CardHeader>
            <CardTitle>Manager Information</CardTitle>
          </CardHeader>
          <CardContent class="space-y-4">
            <div>
              <p class="text-sm font-medium text-muted-foreground">Category</p>
              <p class="mt-1 text-sm">{sdkConfig.category}</p>
            </div>
            {#if sdkConfig.sdk_managers.length > 0}
              {@const manager = sdkConfig.sdk_managers[0]}
              {#if manager.website}
                <div>
                  <p class="text-sm font-medium text-muted-foreground">
                    Website
                  </p>
                  <a
                    href={manager.website}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="mt-1 flex items-center gap-1 text-sm text-blue-600 hover:underline"
                  >
                    {manager.website}
                    <ExternalLink class="h-3 w-3" />
                  </a>
                </div>
              {/if}
              {#if manager.install_command}
                <div>
                  <p class="text-sm font-medium text-muted-foreground">
                    Installation Command
                  </p>
                  <code
                    class="mt-1 block rounded bg-muted p-2 font-mono text-xs"
                  >
                    {manager.install_command}
                  </code>
                </div>
              {/if}
            {/if}
          </CardContent>
        </Card>

        <!-- Features Card -->
        <Card>
          <CardHeader>
            <CardTitle>Features</CardTitle>
          </CardHeader>
          <CardContent>
            {#if sdkConfig.category_features}
              <div class="flex flex-wrap gap-2">
                {#each Object.entries(sdkConfig.category_features) as [key, value]}
                  {#if value === true}
                    <Badge variant="secondary" class="text-xs">
                      {key.replace(/([A-Z])/g, " $1").trim()}
                    </Badge>
                  {/if}
                {/each}
                <!-- Add common features based on category -->
                {#if sdkConfig.category === "language"}
                  <Badge variant="secondary" class="text-xs"
                    >Programming Language</Badge
                  >
                  {#if sdkConfig.package_managers.length > 0}
                    <Badge variant="secondary" class="text-xs"
                      >Package Management</Badge
                    >
                  {/if}
                {:else if sdkConfig.category === "database"}
                  <Badge variant="secondary" class="text-xs"
                    >Database Server</Badge
                  >
                  <Badge variant="secondary" class="text-xs">Data Storage</Badge
                  >
                {:else if sdkConfig.category === "ai"}
                  <Badge variant="secondary" class="text-xs">AI Runtime</Badge>
                  <Badge variant="secondary" class="text-xs"
                    >Model Management</Badge
                  >
                {:else if sdkConfig.category === "server"}
                  <Badge variant="secondary" class="text-xs">Web Server</Badge>
                  <Badge variant="secondary" class="text-xs">HTTP Server</Badge>
                {:else if sdkConfig.category === "container"}
                  <Badge variant="secondary" class="text-xs"
                    >Container Platform</Badge
                  >
                  <Badge variant="secondary" class="text-xs"
                    >Container Management</Badge
                  >
                {/if}
              </div>
            {:else}
              <p class="text-sm text-muted-foreground">No features listed</p>
            {/if}
          </CardContent>
        </Card>
      </div>

      <!-- Actions Card -->
      <Card>
        <CardHeader>
          <CardTitle>Actions</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex gap-4">
            {#if sdkConfig.sdk_managers.length > 0}
              {@const manager = sdkConfig.sdk_managers[0]}
              {#if !manager.installed && manager.install_command}
                <Button
                  onclick={() => {
                    // TODO: Implement install manager
                  }}
                >
                  <Download class="mr-2 h-4 w-4" />
                  Install Manager
                </Button>
              {/if}
              {#if manager.website}
                <Button
                  variant="outline"
                  onclick={() => {
                    if (manager.website) {
                      window.open(manager.website, "_blank");
                    }
                  }}
                >
                  <ExternalLink class="mr-2 h-4 w-4" />
                  Visit Website
                </Button>
              {/if}
            {/if}
          </div>
        </CardContent>
      </Card>
    </div>
  {/if}
</div>
