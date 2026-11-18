<!--
	Ollama Service Page
	Generic service page structure that can be reused for other AI services
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { ollamaApi } from '$lib/domains/sdk/api/ollamaApi';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { logger } from '$lib/domains/shared';
  import { toast } from '$lib/utils/toast';
  import { buildTabUrl, resolveUrlTab } from '$lib/utils/url-tabs';
  import { OllamaModels } from '$lib/domains/sdk/state/ollamaModels.svelte';
  import OllamaHeader from '$lib/domains/sdk/components/ollama/OllamaHeader.svelte';
  import OllamaServiceTab from '$lib/domains/sdk/components/ollama/OllamaServiceTab.svelte';
  import OllamaVersionsTab from '$lib/domains/sdk/components/ollama/OllamaVersionsTab.svelte';
  import OllamaInfoTab from '$lib/domains/sdk/components/ollama/OllamaInfoTab.svelte';
  import OllamaModelsTab from '$lib/domains/sdk/components/ollama/OllamaModelsTab.svelte';

  const OLLAMA_TABS = ['service', 'version', 'models', 'configuration', 'log'] as const;
  type OllamaTab = (typeof OLLAMA_TABS)[number];

  const activeTab = $derived(resolveUrlTab(page.url.searchParams, OLLAMA_TABS, 'service'));

  const log = logger.createScoped('OllamaService');
  let serviceInfo = $state<any>(null);
  let serviceLoading = $state(true);
  let serviceError = $state<string | null>(null);

  // Version management
  let versions = $state<any[]>([]);
  let versionsLoading = $state(false);
  let versionsError = $state<string | null>(null);

  const models = new OllamaModels(() => !!serviceInfo?.running);

  onMount(async () => {
    await loadServiceInfo();
  });

  $effect(() => {
    loadTabData(activeTab);
  });

  const errMsg = (error: unknown) => (error instanceof Error ? error.message : String(error));

  async function loadServiceInfo() {
    try {
      serviceLoading = true;
      serviceError = null;

      const info = (await ollamaApi.getServiceStatus()) as any;
      serviceInfo = info;

      log.info('Ollama service info loaded', info);
    } catch (error) {
      serviceError = errMsg(error);
      log.error('Failed to load Ollama service info', { error: serviceError });
    } finally {
      serviceLoading = false;
    }
  }

  function loadTabData(tab: OllamaTab) {
    if (tab === 'version') {
      if (versions.length === 0) void loadVersions();
    } else if (tab === 'models') {
      if (models.tab === 'local') {
        if (serviceInfo?.running) void models.loadLocal();
      } else {
        void models.loadLibrary();
      }
    }
  }

  async function loadVersions() {
    try {
      versionsLoading = true;
      versionsError = null;

      // Get real Ollama versions from GitHub API
      const versionsData = (await ollamaApi.getVersions()) as any[];
      versions = versionsData;

      log.info('Ollama versions loaded', versionsData);
    } catch (error) {
      const errorMessage = errMsg(error);
      versionsError = errorMessage;
      log.error('Failed to load Ollama versions', error);

      // Show user-friendly error for GitHub API issues
      if (errorMessage.includes('403') || errorMessage.includes('Forbidden')) {
        toast.warning(
          'GitHub API Rate Limited',
          'Unable to fetch versions due to GitHub API rate limiting. Please try again later.'
        );
      } else {
        toast.error('Failed to Load Versions', errorMessage);
      }
    } finally {
      versionsLoading = false;
    }
  }

  function installOllamaVersion(version: string) {
    log.info('Installing Ollama version:', version);
    // For now, show a message that manual installation is needed
    // In a real implementation, this would download and install Ollama
    toast.info(
      'Manual Installation Required',
      `To install Ollama version ${version}, please download it from GitHub.`,
      {
        action: {
          label: 'Download Version',
          onClick: () =>
            window.open(`https://github.com/ollama/ollama/releases/tag/v${version}`, '_blank'),
        },
      }
    );
    return loadVersions(); // Refresh versions list
  }

  const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

  /** Toast for a failed start/stop/restart; returns true if it was the "not installed" case. */
  function notifyNotInstalled(errorMessage: string) {
    if (!errorMessage.includes('not installed') && !errorMessage.includes('program not found')) {
      return false;
    }
    toast.error(
      'Ollama Not Installed',
      'Please install Ollama from https://ollama.com/download first.',
      {
        action: {
          label: 'Download Ollama',
          onClick: () => window.open('https://ollama.com/download', '_blank'),
        },
      }
    );
    return true;
  }

  /**
   * Start or stop the service. The API can error even though the state changed, so
   * after a failure re-read the status and treat the desired state as success.
   */
  async function setServiceRunning(running: boolean) {
    const [verb, past, call] = running
      ? (['Start', 'started', () => ollamaApi.startService()] as const)
      : (['Stop', 'stopped', () => ollamaApi.stopService()] as const);
    try {
      serviceError = null; // Clear any previous errors
      const result = await call();
      await wait(500);
      await loadServiceInfo();
      log.info(`Ollama service ${past} successfully`, result);
      toast.success(
        `Service ${past[0].toUpperCase()}${past.slice(1)}`,
        `Ollama service ${running ? 'is now running' : 'has been stopped'}`
      );
    } catch (error) {
      const errorMessage = errMsg(error);
      await wait(500);
      await loadServiceInfo();

      if (!!serviceInfo?.running === running) {
        log.info(`Ollama service ${past} (verified after error)`, errorMessage);
        serviceError = null;
        toast.success(
          `Service ${past[0].toUpperCase()}${past.slice(1)}`,
          `Ollama service ${running ? 'is now running' : 'has been stopped'}`
        );
        return;
      }

      serviceError = errorMessage;
      log.error(`Failed to ${verb.toLowerCase()} Ollama service`, { error: errorMessage });
      if (!notifyNotInstalled(errorMessage)) toast.error(`Failed to ${verb} Service`, errorMessage);
    }
  }

  async function restartService() {
    try {
      await ollamaApi.stopService();
      await wait(2000);
      await ollamaApi.startService();
      await loadServiceInfo();
      log.info('Ollama service restarted successfully');
      toast.success('Service Restarted', 'Ollama service has been restarted');
    } catch (error) {
      const errorMessage = errMsg(error);
      serviceError = errorMessage;
      log.error('Failed to restart Ollama service', { error: errorMessage });
      notifyNotInstalled(errorMessage);
    }
  }

  async function checkUpdates() {
    try {
      log.info('Checking for Ollama updates...');
      const result = await ollamaApi.checkUpdates();
      log.info('Update check result:', result);
      toast.success('Update Check Complete', result ? String(result) : 'No updates available');
    } catch (error) {
      log.error('Failed to check for updates', { error: errMsg(error) });
      toast.error('Update Check Failed', errMsg(error));
    }
  }

  async function updateOllama() {
    try {
      log.info('Updating Ollama...');
      const result = await ollamaApi.update();
      log.info('Update result:', result);
      toast.success('Ollama Updated', String(result));
      await loadServiceInfo();
    } catch (error) {
      log.error('Failed to update Ollama', { error: errMsg(error) });
      toast.error('Update Failed', errMsg(error));
    }
  }

  function setActiveTab(tab: OllamaTab) {
    goto(buildTabUrl(page.url.pathname, page.url.searchParams, tab), {
      replaceState: true,
      reset: false,
    });
  }
</script>

<OllamaHeader {serviceInfo} />

<div class="flex-1 p-6">
  <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as OllamaTab)} class="w-full">
    <TabsList>
      <TabsTrigger value="service">Service</TabsTrigger>
      <TabsTrigger value="version">Version</TabsTrigger>
      <TabsTrigger value="models">Models</TabsTrigger>
      <TabsTrigger value="configuration">Configuration</TabsTrigger>
      <TabsTrigger value="log">Log</TabsTrigger>
    </TabsList>

    <TabsContent value="service" class="mt-6">
      <OllamaServiceTab
        {serviceInfo}
        loading={serviceLoading}
        error={serviceError}
        onretry={loadServiceInfo}
        onstart={() => setServiceRunning(true)}
        onstop={() => setServiceRunning(false)}
        onrestart={restartService}
        onviewlogs={() => setActiveTab('log')}
        oncheckupdates={checkUpdates}
        onupdate={updateOllama}
      />
    </TabsContent>

    <TabsContent value="version" class="mt-6">
      <OllamaVersionsTab
        {versions}
        loading={versionsLoading}
        error={versionsError}
        onload={loadVersions}
        oninstall={installOllamaVersion}
      />
    </TabsContent>

    <TabsContent value="models" class="mt-6">
      <OllamaModelsTab
        state={models}
        serviceRunning={!!serviceInfo?.running}
        onstart={() => setServiceRunning(true)}
      />
    </TabsContent>

    <TabsContent value="configuration" class="mt-6">
      <OllamaInfoTab kind="configuration" />
    </TabsContent>

    <TabsContent value="log" class="mt-6">
      <OllamaInfoTab kind="log" />
    </TabsContent>
  </Tabs>
</div>
