<!--
	Service Config Editor - FlyEnv-style configuration editor
	Allows editing service configuration files with syntax highlighting
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
  } from '$lib/components/ui/dialog';
  import ServiceConfigFilesTab from './ServiceConfigFilesTab.svelte';
  import ServiceConfigEnvTab from './ServiceConfigEnvTab.svelte';
  import type { ConfigFile, EnvironmentVariable } from './serviceConfigTypes';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import Settings from '~icons/fa6-solid/gear';
  import AlertCircle from '~icons/fa6-solid/circle-exclamation';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import FileText from '~icons/fa6-solid/file-lines';
  import Code from '~icons/fa6-solid/code';

  interface Props {
    serviceId: string;
    serviceName: string;
    isOpen: boolean;
    onClose: () => void;
  }

  let { serviceId, serviceName, isOpen, onClose }: Props = $props();

  // State
  let configFiles = $state<ConfigFile[]>([]);
  let environmentVars = $state<EnvironmentVariable[]>([]);
  let activeTab = $state('files');
  let selectedFile = $state<ConfigFile | null>(null);
  let editedContent = $state('');
  let hasChanges = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);

  // Initialize
  onMount(async () => {
    if (isOpen) {
      await loadConfig();
    }
  });

  // Watch for dialog open/close
  $effect(() => {
    if (isOpen) {
      loadConfig();
    }
  });

  async function loadConfig() {
    loading = true;
    error = null;

    try {
      // Mock config files - in real implementation, this would load from the service
      configFiles = [
        {
          name: 'nginx.conf',
          path: '/etc/nginx/nginx.conf',
          content: `server {
    listen 80;
    server_name localhost;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}`,
          format: 'text',
          lastModified: new Date().toISOString(),
        },
        {
          name: 'config.json',
          path: '/path/to/service/config.json',
          content: `{
    "port": 3000,
    "host": "localhost",
    "database": {
        "host": "localhost",
        "port": 5432,
        "name": "myapp"
    },
    "logging": {
        "level": "info",
        "file": "/var/log/service.log"
    }
}`,
          format: 'json',
          lastModified: new Date().toISOString(),
        },
      ];

      // Mock environment variables
      environmentVars = [
        { name: 'NODE_ENV', value: 'development', scope: 'project' },
        { name: 'PORT', value: '3000', scope: 'session' },
        {
          name: 'DATABASE_URL',
          value: 'postgres://localhost:5432/myapp',
          scope: 'project',
        },
      ];
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load configuration';
      console.error('Failed to load service config:', err);
    } finally {
      loading = false;
    }
  }

  function selectFile(file: ConfigFile) {
    selectedFile = file;
    editedContent = file.content;
    hasChanges = false;
  }

  function handleContentChange() {
    hasChanges = editedContent !== selectedFile?.content;
  }

  async function saveConfig() {
    if (!selectedFile) return;

    loading = true;
    error = null;
    success = null;

    try {
      await invoke('update_service_config', {
        serviceId,
        config: {
          [selectedFile.name]: editedContent,
        },
      });

      // Update the file content
      selectedFile.content = editedContent;
      selectedFile.lastModified = new Date().toISOString();
      hasChanges = false;
      success = 'Configuration saved successfully';

      // Clear success message after 3 seconds
      setTimeout(() => {
        success = null;
      }, 3000);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to save configuration';
    } finally {
      loading = false;
    }
  }

  function resetChanges() {
    if (selectedFile) {
      editedContent = selectedFile.content;
      hasChanges = false;
    }
  }

  async function downloadConfig() {
    if (!selectedFile) return;

    try {
      const blob = new Blob([editedContent], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = selectedFile.name;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      error = 'Failed to download configuration file';
    }
  }
</script>

<Dialog bind:open={isOpen} onOpenChange={(open) => !open && onClose()}>
  <DialogContent class="flex h-[90vh] max-w-6xl flex-col">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Settings class="h-5 w-5" />
        Service Configuration: {serviceName}
      </DialogTitle>
    </DialogHeader>

    <!-- Error/Success Messages -->
    {#if error}
      <div class="rounded-md border border-red-200 bg-red-50 p-3">
        <div class="flex items-center gap-2">
          <AlertCircle class="h-4 w-4 text-red-600" />
          <span class="text-sm text-red-600">{error}</span>
        </div>
      </div>
    {/if}

    {#if success}
      <div class="rounded-md border border-green-200 bg-green-50 p-3">
        <div class="flex items-center gap-2">
          <CheckCircle class="h-4 w-4 text-green-600" />
          <span class="text-sm text-green-600">{success}</span>
        </div>
      </div>
    {/if}

    <!-- Main Content -->
    <Tabs bind:value={activeTab} class="flex flex-1 flex-col">
      <TabsList>
        <TabsTrigger value="files">
          <FileText class="mr-2 h-4 w-4" />
          Configuration Files
        </TabsTrigger>
        <TabsTrigger value="environment">
          <Code class="mr-2 h-4 w-4" />
          Environment Variables
        </TabsTrigger>
      </TabsList>

      <TabsContent value="files" class="flex flex-1 gap-4">
        <ServiceConfigFilesTab
          {configFiles}
          {selectedFile}
          bind:editedContent
          {hasChanges}
          {loading}
          onselect={selectFile}
          onchange={handleContentChange}
          onsave={saveConfig}
          onreset={resetChanges}
          ondownload={downloadConfig}
        />
      </TabsContent>

      <TabsContent value="environment" class="flex-1">
        <ServiceConfigEnvTab bind:environmentVars />
      </TabsContent>
    </Tabs>
  </DialogContent>
</Dialog>
