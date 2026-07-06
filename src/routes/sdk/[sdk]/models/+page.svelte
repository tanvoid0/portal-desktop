<!--
	SDK Models Page
	Generic model management UI for AI runtimes that expose model operations.
-->

<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";

  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
    CardDescription,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Alert, AlertDescription } from "$lib/components/ui/alert";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";

  import { AlertCircle, ArrowLeft, Play, Square } from "@lucide/svelte";

  import {
    sdkConfigService,
    type ProcessedSDKConfig,
  } from "$lib/domains/sdk/services/sdkConfigService";

  import { invokeClient } from "$lib/utils/invokeClient";
  import ModelList from "$lib/components/ModelList.svelte";
  import ModelTreeList from "$lib/components/ModelTreeList.svelte";
  import { PageLoading, PageError } from "$lib/components/shell";

  // SDK ID from URL
  const sdkId = $page.params.sdk as string;

  // Config
  let sdkConfig = $state<ProcessedSDKConfig | null>(null);

  // Service status
  let serviceLoading = $state(true);
  let serviceError = $state<string | null>(null);
  let serviceInfo = $state<{
    running: boolean;
    pid?: number;
    port?: number;
    status: string;
  } | null>(null);

  // Models
  let modelTab = $state<"local" | "library">("local");
  let modelsLoading = $state(false);
  let modelsError = $state<string | null>(null);
  let models = $state<any[]>([]);

  let availableModelsLoading = $state(false);
  let availableModelsError = $state<string | null>(null);
  let availableModels = $state<Record<string, any[]>>({});

  // Install progress (UI-side simulation)
  let installingModel = $state<string | null>(null);
  let installationProgress = $state(0);
  let installationStatus = $state("");

  onMount(async () => {
    await load();
  });

  async function load() {
    serviceLoading = true;
    serviceError = null;
    try {
      sdkConfig = await sdkConfigService.getSDKConfig(sdkId);

      // If not a model-capable SDK, show an error banner.
      if (
        !sdkConfig ||
        !sdkConfig.category_features?.modelManagement ||
        !sdkConfig.tabs.some((t) => t.id === "models")
      ) {
        serviceError = "Model management is not available for this SDK.";
        return;
      }

      await loadServiceStatus();
      await loadModelsIfAllowed();
    } catch (err) {
      serviceError =
        err instanceof Error ? err.message : "Failed to load SDK models";
    } finally {
      serviceLoading = false;
    }
  }

  async function loadServiceStatus() {
    const status = await invokeClient.post<{
      running: boolean;
      pid?: number;
      port?: number;
      status: string;
    }>("get_service_status", { sdkType: sdkId });

    serviceInfo = status;
  }

  async function loadModelsIfAllowed() {
    modelsLoading = true;
    modelsError = null;
    try {
      if (!serviceInfo?.running) {
        modelsError =
          "Service is not running. Start the service to view and manage models.";
        return;
      }

      const result = await invokeClient.post<any[]>("get_runtime_models", {
        sdkType: sdkId,
      });
      models = result || [];
    } catch (err) {
      modelsError =
        err instanceof Error ? err.message : "Failed to load models";
    } finally {
      modelsLoading = false;
    }
  }

  async function loadAvailableModels() {
    availableModelsLoading = true;
    availableModelsError = null;
    try {
      const result = await invokeClient.post<Record<string, any[]>>(
        "get_runtime_available_models",
        { sdkType: sdkId },
      );
      availableModels = result || {};
    } catch (err) {
      availableModelsError =
        err instanceof Error ? err.message : "Failed to load available models";
    } finally {
      availableModelsLoading = false;
    }
  }

  async function startService() {
    if (!sdkId) return;
    try {
      await invokeClient.post<string>("start_service", { sdkType: sdkId });
      // Give backend a moment to transition
      await new Promise((r) => setTimeout(r, 500));
      await loadServiceStatus();
      await loadModelsIfAllowed();
    } catch (err) {
      serviceError =
        err instanceof Error ? err.message : "Failed to start service";
    }
  }

  function cancelInstallation() {
    installingModel = null;
    installationProgress = 0;
    installationStatus = "";
  }

  async function installModel(modelName: string) {
    if (!serviceInfo?.running) {
      // Model install requires the runtime to be up for this current implementation.
      modelsError = "Start the service before installing models.";
      return;
    }

    installingModel = modelName;
    installationProgress = 10;
    installationStatus = `Starting download of ${modelName}...`;

    try {
      toastHard("Download started", modelName);

      // Kick off install (backend returns quickly)
      await invokeClient.post<string>("install_runtime_model", {
        sdkType: sdkId,
        modelName,
      });

      // UI-side progress simulation (until we add real progress events)
      installationProgress = 50;
      installationStatus = `Downloading ${modelName}...`;
      await new Promise((r) => setTimeout(r, 2000));
      installationProgress = 100;
      installationStatus = "Installation complete (refreshing models...)";

      await loadModelsIfAllowed();
      toastHard("Model installed", modelName);
    } catch (err) {
      modelsError =
        err instanceof Error ? err.message : "Failed to install model";
    } finally {
      setTimeout(() => {
        cancelInstallation();
      }, 1500);
    }
  }

  async function removeModel(modelName: string) {
    if (!serviceInfo?.running) {
      modelsError = "Start the service before removing models.";
      return;
    }

    try {
      await invokeClient.post<string>("remove_runtime_model", {
        sdkType: sdkId,
        modelName,
      });
      await loadModelsIfAllowed();
    } catch (err) {
      modelsError =
        err instanceof Error ? err.message : "Failed to remove model";
    }
  }

  function toastHard(_title: string, _detail: string) {
    // Placeholder: avoid tight coupling to the toast system here.
    // The UI remains functional without toasts.
  }
</script>

<div class="container mx-auto space-y-6 p-6">
  {#if serviceLoading}
    <PageLoading message="Loading models..." />
  {:else if serviceError}
    <PageError
      title="Failed to load models"
      message={serviceError}
      onRetry={load}
    />
  {:else if sdkConfig}
    <div class="space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between gap-4">
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
            <h1 class="text-3xl font-bold">Models</h1>
            <p class="text-muted-foreground">{sdkConfig.display_name}</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          {#if serviceInfo?.running}
            <Badge variant="default" class="bg-green-500 text-white">
              Running
            </Badge>
          {:else}
            <Badge variant="outline" class="text-muted-foreground">
              Stopped
            </Badge>
          {/if}
        </div>
      </div>

      <!-- Service start prompt -->
      {#if serviceInfo && !serviceInfo.running}
        <Card>
          <CardContent class="pt-6">
            <Alert>
              <AlertCircle class="h-4 w-4" />
              <AlertDescription>
                Start the service to view installed models and manage model
                libraries.
              </AlertDescription>
            </Alert>
            <div class="mt-4">
              <Button onclick={startService}>
                <Play class="mr-2 h-4 w-4" />
                Start Service
              </Button>
            </div>
          </CardContent>
        </Card>
      {/if}

      <Tabs bind:value={modelTab} class="w-full">
        <TabsList>
          <TabsTrigger value="local">Installed</TabsTrigger>
          <TabsTrigger value="library" onclick={loadAvailableModels}>
            Library
          </TabsTrigger>
        </TabsList>

        <TabsContent value="local" class="mt-6">
          <Card>
            <CardHeader>
              <CardTitle>Installed Models</CardTitle>
              <CardDescription
                >Manage models available inside the runtime.</CardDescription
              >
            </CardHeader>
            <CardContent>
              <ModelList
                {models}
                isInstalled={true}
                loading={modelsLoading}
                error={modelsError}
                {installingModel}
                {installationProgress}
                {installationStatus}
                onInstall={installModel}
                onRemove={removeModel}
                onRetry={loadModelsIfAllowed}
                onBrowseAvailable={() => {
                  modelTab = "library";
                  loadAvailableModels();
                }}
              />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="library" class="mt-6">
          <Card>
            <CardHeader>
              <CardTitle>Model Library</CardTitle>
              <CardDescription>Browse and install models.</CardDescription>
            </CardHeader>
            <CardContent>
              <ModelTreeList
                models={availableModels}
                loading={availableModelsLoading}
                error={availableModelsError}
                {installingModel}
                {installationProgress}
                {installationStatus}
                onInstall={installModel}
                onRetry={loadAvailableModels}
                onCancel={cancelInstallation}
              />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  {/if}
</div>
