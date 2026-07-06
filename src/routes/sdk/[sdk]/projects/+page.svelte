<!--
	SDK Projects Page
	Project management for a specific SDK
-->

<script lang="ts">
  import { page } from "$app/stores";
  import {
    sdkConfigService,
    type ProcessedSDKConfig,
  } from "$lib/domains/sdk/services/sdkConfigService";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
    CardDescription,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import {
    ArrowLeft,
    FolderOpen,
    ExternalLink,
  } from "@lucide/svelte";
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

  function navigateToProjects() {
    // Navigate to main projects page
    // The user can filter by framework/package manager there
    goto("/projects");
  }
</script>

<div class="container mx-auto space-y-6 p-6">
  {#if loading}
    <PageLoading message="Loading projects..." />
  {:else if error}
    <PageError title="Failed to load projects" message={error} onRetry={loadData} />
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
            <h1 class="text-3xl font-bold">Projects</h1>
            <p class="text-muted-foreground">
              {sdkConfig.display_name} Projects
            </p>
          </div>
        </div>
      </div>

      <!-- Projects Content -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <FolderOpen class="h-5 w-5" />
            Projects Using {sdkConfig.display_name}
          </CardTitle>
          <CardDescription>
            View and manage all projects that use {sdkConfig.display_name}
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <p class="text-muted-foreground">
            To view projects for {sdkConfig.display_name}, navigate to the main
            Projects page where you can filter by framework or package manager.
          </p>

          {#if sdkConfig.package_managers.length > 0}
            <div class="space-y-2">
              <p class="text-sm font-medium">Related Package Managers:</p>
              <div class="flex flex-wrap gap-2">
                {#each sdkConfig.package_managers as pm}
                  <Badge variant="secondary">{pm.display_name}</Badge>
                {/each}
              </div>
            </div>
          {/if}

          <div class="pt-4">
            <Button onclick={navigateToProjects} class="w-full sm:w-auto">
              <ExternalLink class="mr-2 h-4 w-4" />
              Go to Projects Page
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  {/if}
</div>
