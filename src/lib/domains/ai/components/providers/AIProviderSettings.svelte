<!--
	AI Provider Settings — Agent Platform
-->

<script lang="ts">
  import { onMount } from "svelte";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Switch } from "$lib/components/ui/switch";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Alert,
    AlertDescription,
    AlertTitle,
  } from "$lib/components/ui/alert";
  import { Separator } from "$lib/components/ui/separator";
  import { toastActions } from "$lib/utils/toast";
  import { aiProviderService } from "../../services/aiProviderService.js";
  import {
    formatModelMetadata,
    flattenCatalogModels,
  } from "../../utils/catalog.js";
  import type {
    ProviderConfig,
    ProviderType,
    ConfigurationStatus,
    PlatformCatalog,
    CatalogProvider,
    CatalogModel,
  } from "../../types/index.js";
  import {
    Brain,
    CheckCircle2,
    RefreshCw,
    AlertTriangle,
    Server,
    Sparkles,
    Loader2,
    Key,
  } from "@lucide/svelte";

  const providerType: ProviderType = "AgentPlatform";

  let providerConfig = $state<ProviderConfig | null>(null);
  let defaultProvider = $state<ProviderType | null>(null);
  let configStatus = $state<ConfigurationStatus | null>(null);
  let isLoading = $state(false);
  let testingProvider = $state(false);
  let catalog = $state<PlatformCatalog | null>(null);
  let catalogProviders = $state<CatalogProvider[]>([]);
  let catalogModels = $state<CatalogModel[]>([]);
  let loadingCatalog = $state(false);
  let catalogLive = $state(false);

  onMount(loadProvider);

  async function loadProvider() {
    isLoading = true;
    try {
      const [config, defaultProviderType, status] = await Promise.all([
        aiProviderService.getProviderConfig(providerType).catch(() => null),
        aiProviderService.getDefaultProvider(),
        aiProviderService.getConfigStatus(providerType).catch(() => null),
      ]);

      providerConfig = config;
      defaultProvider = defaultProviderType;
      configStatus = status;
    } catch (error) {
      console.error("Failed to load Agent Platform configuration:", error);
      toastActions.error("Failed to load configuration", error);
    } finally {
      isLoading = false;
    }
  }

  async function updateConfig(updates: Partial<ProviderConfig>) {
    if (!providerConfig) return;

    const updated: ProviderConfig = {
      ...providerConfig,
      ...updates,
    };

    try {
      await aiProviderService.saveProviderConfig(updated);
      providerConfig = updated;

      const status = await aiProviderService.getConfigStatus(providerType);
      configStatus = status;

      if (updated.enabled && status.is_configured && !defaultProvider) {
        await aiProviderService.setDefaultProvider(providerType);
        defaultProvider = providerType;
      }

      toastActions.success("Configuration saved");
    } catch (error) {
      console.error("Failed to save config:", error);
      toastActions.error("Failed to save configuration", error);
    }
  }

  async function setAsDefault() {
    try {
      await aiProviderService.setDefaultProvider(providerType);
      defaultProvider = providerType;
      toastActions.success("Agent Platform set as default provider");
    } catch (error) {
      toastActions.error("Failed to set default provider", error);
    }
  }

  async function testConnection() {
    if (!providerConfig) return;

    testingProvider = true;
    try {
      const configToSave = { ...providerConfig, enabled: true };
      await aiProviderService.saveProviderConfig(configToSave);
      providerConfig = configToSave;

      if (!defaultProvider) {
        await aiProviderService.setDefaultProvider(providerType);
        defaultProvider = providerType;
      }

      await new Promise((resolve) => setTimeout(resolve, 100));
      await aiProviderService.testProvider(providerType);
      toastActions.success("Connection successful");

      configStatus = await aiProviderService.getConfigStatus(providerType);
    } catch (error) {
      toastActions.error("Connection test failed", error);
    } finally {
      testingProvider = false;
    }
  }

  async function loadModels(live = false) {
    if (!providerConfig) return;

    loadingCatalog = true;
    catalogLive = live;
    try {
      await aiProviderService.saveProviderConfig({
        ...providerConfig,
        enabled: true,
      });

      catalog = live
        ? await aiProviderService.getCatalogLive()
        : await aiProviderService.getCatalogAliases();

      catalogProviders = catalog.providers;
      catalogModels = flattenCatalogModels(catalog.providers);

      if (catalogModels.length === 0) {
        toastActions.info(
          "No models found",
          "Check that agent-platform is running and has models configured in /config",
        );
      } else {
        const liveNote = live ? " (live upstream)" : " (aliases)";
        toastActions.success(
          `Loaded ${catalogModels.length} model(s) from ${catalogProviders.length} provider(s)${liveNote}`,
        );

        if (
          !providerConfig.model &&
          catalog.resolved_defaults?.model
        ) {
          await updateConfig({ model: catalog.resolved_defaults.model });
        }
      }
    } catch (error) {
      toastActions.error("Failed to load catalog", error);
    } finally {
      loadingCatalog = false;
    }
  }
</script>

<div class="space-y-6">
  <div>
    <h2 class="flex items-center gap-2 text-2xl font-bold">
      <Brain class="h-6 w-6" />
      AI Provider Configuration
    </h2>
    <p class="mt-1 text-muted-foreground">
      Connect to agent-platform for all AI features. Backend providers (Ollama,
      Gemini, LM Studio, …) and model aliases are configured on the platform at
      <code class="rounded bg-muted px-1">/config</code> — Portal only needs the
      platform URL, token, and a default model alias.
    </p>
  </div>

  {#if defaultProvider === providerType}
    <Alert>
      <CheckCircle2 class="h-4 w-4" />
      <AlertTitle>Default provider</AlertTitle>
      <AlertDescription>
        Agent Platform is the default for all AI features in this app.
      </AlertDescription>
    </Alert>
  {/if}

  {#if isLoading || !providerConfig}
    <Card>
      <CardContent class="py-8">
        <div class="text-center text-muted-foreground">Loading…</div>
      </CardContent>
    </Card>
  {:else}
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Server class="h-5 w-5" />
          Agent Platform
          {#if configStatus?.is_configured}
            <Badge class="ml-2 bg-green-500 hover:bg-green-600">Ready</Badge>
          {/if}
        </CardTitle>
        <CardDescription>
          OpenAI-compatible proxy at <code>http://127.0.0.1:18410</code> by
          default. Use <strong>Load catalog</strong> to fetch providers and
          models from <code>/v1/catalog</code>.
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-6">
        {#if configStatus && !configStatus.is_configured}
          <Alert variant="destructive">
            <AlertTriangle class="h-4 w-4" />
            <AlertTitle>Configuration incomplete</AlertTitle>
            <AlertDescription>
              Missing: {configStatus.missing_fields.join(", ")}
            </AlertDescription>
          </Alert>
        {/if}

        {#if configStatus?.warnings.length}
          <Alert>
            <AlertTriangle class="h-4 w-4" />
            <AlertTitle>Notes</AlertTitle>
            <AlertDescription>
              <ul class="list-inside list-disc">
                {#each configStatus.warnings as warning}
                  <li>{warning}</li>
                {/each}
              </ul>
            </AlertDescription>
          </Alert>
        {/if}

        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <Label>Enable Agent Platform</Label>
            <p class="text-sm text-muted-foreground">
              Route AI requests through the platform proxy
            </p>
          </div>
          <Switch
            checked={providerConfig.enabled}
            onCheckedChange={(checked: boolean) =>
              updateConfig({ enabled: checked })}
          />
        </div>

        <Separator />

        <div class="space-y-2">
          <Label for="base-url">
            <Server class="mr-1 inline h-4 w-4" />
            Base URL
          </Label>
          <Input
            id="base-url"
            placeholder="http://127.0.0.1:18410"
            value={providerConfig.base_url || ""}
            oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
              updateConfig({ base_url: e.currentTarget.value || null })}
          />
        </div>

        <div class="space-y-2">
          <Label for="api-key">
            <Key class="mr-1 inline h-4 w-4" />
            API token
          </Label>
          <Input
            id="api-key"
            type="text"
            autocomplete="off"
            spellcheck={false}
            class="font-mono text-sm"
            placeholder="agp_… workspace token from agent-platform /config"
            value={providerConfig.api_key || ""}
            oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
              updateConfig({ api_key: e.currentTarget.value || null })}
          />
          <p class="text-xs text-muted-foreground">
            Shown in plain text for verification. Paste a scoped
            <code class="rounded bg-muted px-1">agp_</code> token from
            agent-platform — not the platform master key.
          </p>
        </div>

        <div class="space-y-2">
          <Label for="model">
            <Sparkles class="mr-1 inline h-4 w-4" />
            Default model alias
          </Label>
          <div class="flex gap-2">
            <Input
              id="model"
              placeholder="e.g. llama3, fast"
              value={providerConfig.model}
              oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
                updateConfig({ model: e.currentTarget.value })}
              class="flex-1"
            />
            <Button
              variant="outline"
              size="sm"
              onclick={() => loadModels(false)}
              disabled={loadingCatalog}
            >
              {#if loadingCatalog && !catalogLive}
                <Loader2 class="mr-2 h-4 w-4 animate-spin" />
              {:else}
                <RefreshCw class="mr-2 h-4 w-4" />
              {/if}
              Load catalog
            </Button>
            <Button
              variant="outline"
              size="sm"
              onclick={() => loadModels(true)}
              disabled={loadingCatalog}
              title="Fetch live models from upstream providers"
            >
              {#if loadingCatalog && catalogLive}
                <Loader2 class="mr-2 h-4 w-4 animate-spin" />
              {:else}
                <Sparkles class="mr-2 h-4 w-4" />
              {/if}
              Live
            </Button>
          </div>

          {#if catalog}
            <p class="text-xs text-muted-foreground">
              Platform defaults:
              <code class="rounded bg-muted px-1"
                >{catalog.resolved_defaults.provider}</code
              >
              /
              <code class="rounded bg-muted px-1"
                >{catalog.resolved_defaults.model}</code
              >
            </p>
          {/if}

          {#if catalogProviders.length > 0}
            <div class="space-y-4 rounded-md border p-3">
              {#each catalogProviders as provider (provider.id)}
                <div class="space-y-2">
                  <div class="flex flex-wrap items-center gap-2">
                    <span class="text-sm font-medium">{provider.label}</span>
                    <Badge variant="outline" class="text-xs">{provider.id}</Badge>
                    {#if provider.configured}
                      <Badge class="bg-green-500 text-xs hover:bg-green-600"
                        >configured</Badge
                      >
                    {:else}
                      <Badge variant="secondary" class="text-xs"
                        >not configured</Badge
                      >
                    {/if}
                    {#if provider.reachable}
                      <Badge variant="secondary" class="text-xs">reachable</Badge>
                    {/if}
                    {#if provider.default_model}
                      <span class="text-xs text-muted-foreground">
                        default: {provider.default_model}
                      </span>
                    {/if}
                  </div>
                  {#if provider.models.length > 0}
                    <div class="flex flex-wrap gap-2">
                      {#each provider.models as model (model.id)}
                        <Button
                          variant="outline"
                          size="sm"
                          onclick={() => updateConfig({ model: model.id })}
                          class={providerConfig.model === model.id
                            ? "bg-primary text-primary-foreground"
                            : ""}
                          title={formatModelMetadata(model)}
                        >
                          {model.id}
                          {#if model.source === "alias"}
                            <Badge
                              variant="secondary"
                              class="ml-1 text-[10px]">alias</Badge
                            >
                          {/if}
                        </Button>
                      {/each}
                    </div>
                  {:else}
                    <p class="text-xs text-muted-foreground">
                      No models listed for this provider.
                    </p>
                  {/if}
                </div>
              {/each}
            </div>
          {:else if catalogModels.length > 0}
            <div class="mt-2 flex flex-wrap gap-2">
              {#each catalogModels as model (model.id)}
                <Button
                  variant="outline"
                  size="sm"
                  onclick={() => updateConfig({ model: model.id })}
                  class={providerConfig.model === model.id
                    ? "bg-primary text-primary-foreground"
                    : ""}
                  title={formatModelMetadata(model)}
                >
                  {model.id}
                </Button>
              {/each}
            </div>
          {/if}
          <p class="text-xs text-muted-foreground">
            Model alias from the platform's <code class="rounded bg-muted px-1"
              >config.yaml</code
            >, or a raw backend model id.
          </p>
        </div>

        <Separator />

        <div class="flex items-center justify-end gap-2">
          <Button
            variant="outline"
            onclick={testConnection}
            disabled={testingProvider}
          >
            {#if testingProvider}
              <Loader2 class="mr-2 h-4 w-4 animate-spin" />
            {:else}
              <RefreshCw class="mr-2 h-4 w-4" />
            {/if}
            Test connection
          </Button>
          {#if defaultProvider !== providerType && configStatus?.is_configured}
            <Button variant="default" onclick={setAsDefault}>
              Set as default
            </Button>
          {/if}
        </div>
      </CardContent>
    </Card>
  {/if}
</div>
