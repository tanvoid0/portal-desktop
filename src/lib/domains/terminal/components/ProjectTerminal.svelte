<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { terminalStore, terminalActions } from '../stores/terminalStore';
  import TabContainer from './TabContainer.svelte';
  import Terminal from './Terminal.svelte';
  import { Button } from '@/lib/components/ui/button';
  import type { TerminalConfig } from '../types';

  // Props
  interface Props {
    projectId: string;
    projectName: string;
    projectPath: string;
    settings?: TerminalConfig;
  }

  let {
    projectId,
    projectName,
    projectPath,
    settings: providedSettings
  }: Props = $props();

  const settings = $derived(providedSettings ?? {
    theme: 'dark' as const,
    fontSize: 14,
    fontFamily: 'Monaco, Consolas, "Courier New", monospace',
    cursorStyle: 'block' as const,
    scrollbackLines: 1000,
    bellSound: false,
    autoClose: true,
    confirmClose: true,
    defaultShell: navigator.userAgent.includes('Windows') ? 'cmd.exe' : 'zsh',
    workingDirectory: projectPath
  });

  // Reactive stores
  const tabs = $derived($terminalStore.tabs.filter(tab => 
    tab.resourceName === 'project' && tab.resourceId === projectId
  ));
  const activeTab = $derived($terminalStore.activeTabId);
  const hasTabs = $derived(tabs.length > 0);

  // Create a new terminal tab for this project
  function createNewTerminalTab(shellCommand?: string) {
    console.log('Creating new project terminal tab for project:', projectId);
    const tabNumber = tabs.length + 1;
    
    // Use the provided shell command or fallback to default shell
    const actualShellCommand = shellCommand || settings.defaultShell;
    
    const tabId = terminalActions.createTab({
      title: `${projectName} Terminal ${tabNumber}`,
      type: 'terminal',
      workingDirectory: projectPath,
      shell: actualShellCommand,
      icon: '💻',
      closable: true,
      resourceName: 'project',
      resourceId: projectId
    });

    console.log('Created project terminal tab with ID:', tabId);

    // Create a process for the new tab
    const processId = terminalActions.createProcess({
      tabId,
      command: actualShellCommand,
      workingDirectory: projectPath,
      environment: {},
      status: 'running'
    });

    console.log('Created process with ID:', processId);
    return tabId;
  }

  // Wrapper function to handle new tab creation with shell command
  function handleNewTabWithProfile(shellCommand?: string) {
    createNewTerminalTab(shellCommand);
  }

  // Load existing project terminals on mount
  onMount(() => {
    console.log('ProjectTerminal mounted for project:', projectId);
    
    // Clean up stale data on mount
    terminalActions.cleanupStaleData();
    
    // If no tabs exist for this project, create an initial one
    if (tabs.length === 0) {
      createNewTerminalTab();
    } else {
      console.log(`Restored ${tabs.length} existing terminal tabs for project ${projectId}`);
      
      // Ensure at least one tab is active
      if (!$terminalStore.activeTabId || !tabs.some(tab => tab.id === $terminalStore.activeTabId)) {
        terminalActions.setActiveTab(tabs[0].id);
      }
      
      // Note: Processes will be reconnected when the Terminal component mounts
      // If the process is still alive, it will reconnect; otherwise, a new one will be created
    }
  });

  // Cleanup project terminals when component is destroyed
  onDestroy(() => {
    console.log('ProjectTerminal destroyed for project:', projectId);
    // Note: We don't automatically close tabs here as the user might want to keep them
    // The tabs will be cleaned up when the project is closed or the app is closed
  });
</script>

<div class="project-terminal-container h-full w-full flex flex-col">
  <!-- Project Terminal Header -->
  <div class="project-terminal-header bg-gray-800 border-b border-gray-700 px-4 py-2">
    <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <h2 class="text-lg font-semibold text-gray-100">
            {projectName} Terminal
          </h2>
          <div class="flex items-center space-x-2">
            <div class="w-2 h-2 bg-green-500 rounded-full"></div>
            <span class="text-sm text-gray-400">
              {tabs.length} tab{tabs.length !== 1 ? 's' : ''}
            </span>
            {#if tabs.length > 0}
              <span class="text-xs text-blue-400 bg-blue-900/30 px-2 py-1 rounded">
                Restored
              </span>
            {/if}
          </div>
        </div>
      
      <div class="flex items-center space-x-2 text-xs text-gray-400">
        <span>Project: {String(projectId).slice(0, 8)}...</span>
        <span>•</span>
        <span class="font-mono">{projectPath}</span>
      </div>
    </div>
  </div>

  <!-- Tabbed Terminal Content -->
  <div class="flex-1 min-h-0">
    {#if hasTabs}
      <TabContainer 
        onNewTab={handleNewTabWithProfile} 
        showNewTabButton={true}
        closable={true}
        className="project-terminal-tabs"
      >
        <!-- Render all project terminal instances but only show the active one -->
        {#each tabs as tab (tab.id)}
          <div 
            class="project-terminal-tab-content" 
            class:active={tab.id === activeTab} 
            style:display={tab.id === activeTab ? 'block' : 'none'}
          >
            <Terminal 
              tabId={tab.id} 
              settings={{
                ...settings,
                defaultShell: tab.shell || settings.defaultShell,
                workingDirectory: projectPath
              }}
            />
          </div>
        {/each}
      </TabContainer>
    {:else}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <div class="text-gray-400 mb-4">No terminal tabs for this project</div>
          <Button
            onclick={() => createNewTerminalTab()}
            class="px-4 py-2"
            type="button"
          >
            Create Terminal Tab
          </Button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  :global(.project-terminal-container) {
    background: #1f2937;
  }
  
  .project-terminal-tab-content {
    height: 100%;
    width: 100%;
  }
  
  .project-terminal-tab-content.active {
    display: block !important;
  }

  .project-terminal-header {
    border-bottom: 1px solid #374151;
  }
</style>
