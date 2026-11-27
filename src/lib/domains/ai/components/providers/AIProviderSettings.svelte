<!--
	AI Provider Settings — Agent Platform
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import { Switch } from '$lib/components/ui/switch';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { Separator } from '$lib/components/ui/separator';
  import { AIProviderSettingsState } from '../../state/aiProviderSettings.svelte.js';
  import PlatformHealthBar from './PlatformHealthBar.svelte';
  import { aiPlatform } from '../../state/aiPlatform.svelte.js';
  import PlatformPairForm from './PlatformPairForm.svelte';
  import CatalogModelPicker from './CatalogModelPicker.svelte';
  import Brain from '~icons/fa6-solid/brain';
  import CheckCircle2 from '~icons/fa6-solid/circle-check';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import AlertTriangle from '~icons/fa6-solid/triangle-exclamation';
  import Server from '~icons/fa6-solid/server';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Key from '~icons/fa6-solid/key';
  import Eye from '~icons/fa6-solid/eye';
  import EyeOff from '~icons/fa6-solid/eye-slash';

  const st = new AIProviderSettingsState();
  const providerType = st.providerType;

  // Port of the local base URL; null when the base URL points elsewhere.
  const localPort = $derived.by(() => {
    const base = st.providerConfig?.base_url?.trim();
    if (!base) return 18410;
    const m = /^http:\/\/(?:127\.0\.0\.1|localhost):(\d+)\/?$/.exec(base);
    const port = Number(m?.[1]);
    return port > 0 && port <= 65535 ? port : null;
  });

  async function setPort(e: Event & { currentTarget: HTMLInputElement }) {
    const port = Number(e.currentTarget.value);
    if (!Number.isInteger(port) || port < 1024 || port > 65535) {
      e.currentTarget.value = String(localPort ?? '');
      return;
    }
    await st.updateConfig({ base_url: `http://127.0.0.1:${port}` });
    await aiPlatform.start();
  }

  onMount(st.init);
</script>

<div class="space-y-6">
  <div>
    <h2 class="flex items-center gap-2 text-2xl font-bold">
      <Brain class="h-6 w-6" />
      AI Provider Configuration
    </h2>
    <p class="mt-1 text-muted-foreground">
      Connect to agent-platform for all AI features. Backend providers (Ollama, Gemini, LM Studio,
      …) and model aliases are configured on the platform at
      <code class="rounded bg-muted px-1">/config</code> — Portal only needs the platform URL, token,
      and a default model alias.
    </p>
  </div>

  {#if st.defaultProvider === providerType}
    <Alert>
      <CheckCircle2 class="h-4 w-4" />
      <AlertTitle>Default provider</AlertTitle>
      <AlertDescription>
        Agent Platform is the default for all AI features in this app.
      </AlertDescription>
    </Alert>
  {/if}

  {#if st.isLoading || !st.providerConfig}
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
          {#if st.connection === 'unauthorized'}
            <Badge variant="destructive" class="ml-2">Token rejected</Badge>
          {:else if st.connection === 'unreachable'}
            <Badge variant="destructive" class="ml-2">Unreachable</Badge>
          {:else if st.connection === 'ok'}
            <Badge class="ml-2 bg-green-500 hover:bg-green-600">Ready</Badge>
          {/if}
        </CardTitle>
        <PlatformHealthBar />
        <CardDescription>
          OpenAI-compatible proxy at <code>http://127.0.0.1:18410</code> by default. Use
          <strong>Load catalog</strong>
          to fetch providers and models from <code>/v1/catalog</code>.
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-6">
        {#if st.configStatus && !st.configStatus.is_configured}
          <Alert variant="destructive">
            <AlertTriangle class="h-4 w-4" />
            <AlertTitle>Configuration incomplete</AlertTitle>
            <AlertDescription>
              Missing: {st.configStatus.missing_fields.join(', ')}
            </AlertDescription>
          </Alert>
        {/if}

        {#if st.configStatus?.warnings.length}
          <Alert>
            <AlertTriangle class="h-4 w-4" />
            <AlertTitle>Notes</AlertTitle>
            <AlertDescription>
              <ul class="list-inside list-disc">
                {#each st.configStatus.warnings as warning}
                  <li>{warning}</li>
                {/each}
              </ul>
            </AlertDescription>
          </Alert>
        {/if}

        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <Label for="agent-platform-enabled">Enable Agent Platform</Label>
            <p class="text-sm text-muted-foreground">
              Route AI requests through the platform proxy
            </p>
          </div>
          <Switch
            id="agent-platform-enabled"
            checked={st.providerConfig.enabled}
            onCheckedChange={(checked: boolean) => st.updateConfig({ enabled: checked })}
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
            value={st.providerConfig.base_url || ''}
            oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
              st.updateConfig({ base_url: e.currentTarget.value || null })}
          />
        </div>

        <div class="space-y-2">
          <Label for="platform-port">Local port</Label>
          <Input
            id="platform-port"
            inputmode="numeric"
            maxlength={5}
            class="w-32"
            disabled={localPort === null}
            value={localPort ?? ''}
            onchange={setPort}
          />
          <p class="text-xs text-muted-foreground">
            {localPort === null
              ? 'The base URL is not on this machine, so Portal does not start a server for it.'
              : 'Portal starts its bundled server on this port, or attaches to yours. Change it if another program holds the port.'}
          </p>
        </div>

        <div class="space-y-2">
          <Label for="api-key">
            <Key class="mr-1 inline h-4 w-4" />
            API token
          </Label>
          <div class="flex gap-2">
            <Input
              id="api-key"
              type={st.showApiKey ? 'text' : 'password'}
              autocomplete="off"
              spellcheck={false}
              class="font-mono text-sm"
              placeholder="agp_… workspace token from agent-platform /config"
              value={st.providerConfig.api_key || ''}
              oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
                st.updateConfig({ api_key: e.currentTarget.value || null })}
            />
            <Button
              variant="outline"
              size="icon"
              type="button"
              aria-label={st.showApiKey ? 'Hide API token' : 'Show API token'}
              onclick={() => (st.showApiKey = !st.showApiKey)}
            >
              {#if st.showApiKey}<EyeOff class="h-4 w-4" />{:else}<Eye class="h-4 w-4" />{/if}
            </Button>
          </div>
          <div class="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              type="button"
              onclick={() => (st.pairOpen = !st.pairOpen)}
            >
              Pair with agent-platform
            </Button>
          </div>
          {#if st.pairOpen}
            <PlatformPairForm pairing={st.pairing} onPair={st.pair} />
          {/if}
          <p class="text-xs text-muted-foreground">
            Hidden by default; use the eye button to reveal. Paste a scoped
            <code class="rounded bg-muted px-1">agp_</code> token from agent-platform — not the platform
            master key.
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
              value={st.providerConfig.model}
              oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
                st.updateConfig({ model: e.currentTarget.value })}
              class="flex-1"
            />
            <Button
              variant="outline"
              size="sm"
              onclick={() => st.loadModels(false)}
              disabled={st.loadingCatalog}
            >
              {#if st.loadingCatalog && !st.catalogLive}
                <Loader2 class="mr-2 h-4 w-4 animate-spin" />
              {:else}
                <RefreshCw class="mr-2 h-4 w-4" />
              {/if}
              Load catalog
            </Button>
            <Button
              variant="outline"
              size="sm"
              onclick={() => st.loadModels(true)}
              disabled={st.loadingCatalog}
              title="Fetch live models from upstream providers"
            >
              {#if st.loadingCatalog && st.catalogLive}
                <Loader2 class="mr-2 h-4 w-4 animate-spin" />
              {:else}
                <Sparkles class="mr-2 h-4 w-4" />
              {/if}
              Live
            </Button>
          </div>

          <CatalogModelPicker
            catalog={st.catalog}
            providers={st.catalogProviders}
            models={st.catalogModels}
            selectedModel={st.providerConfig.model}
            onSelect={(id) => st.updateConfig({ model: id })}
          />
          <p class="text-xs text-muted-foreground">
            Model alias from the platform's <code class="rounded bg-muted px-1">config.yaml</code>,
            or a raw backend model id.
          </p>
        </div>

        <Separator />

        <div class="flex items-center justify-end gap-2">
          <Button variant="outline" onclick={st.testConnection} disabled={st.testingProvider}>
            {#if st.testingProvider}
              <Loader2 class="mr-2 h-4 w-4 animate-spin" />
            {:else}
              <RefreshCw class="mr-2 h-4 w-4" />
            {/if}
            Test connection
          </Button>
          {#if st.defaultProvider !== providerType && st.configStatus?.is_configured}
            <Button variant="default" onclick={st.setAsDefault}>Set as default</Button>
          {/if}
        </div>
      </CardContent>
    </Card>
  {/if}
</div>
