<script lang="ts">
  import { onMount } from 'svelte';
  import { terminalStore, terminalActions } from '../stores/terminalStore';
  import TabContainer from './TabContainer.svelte';
  import Terminal from './Terminal.svelte';
  import { Button } from '@/lib/components/ui/button';
  import { logger } from '@/lib/domains/shared';
  import type { TerminalConfig } from '../types';

  const log = logger.createScoped('TerminalTabContainer');

  // Props
  export let settings: TerminalConfig = {
    theme: 'dark',
    fontSize: 14,
    fontFamily: 'Monaco, Consolas, "Courier New", monospace',
    cursorStyle: 'block',
    scrollbackLines: 1000,
    bellSound: false,
    autoClose: true,
    confirmClose: true,
    defaultShell: navigator.userAgent.includes('Windows') ? 'cmd.exe' : 'zsh',
    workingDirectory: navigator.userAgent.includes('Windows') ? 'C:\\' : '/home/tan'
  };

  // Reactive stores - only show global tabs (not project-specific)
  $: tabs = $terminalStore.tabs.filter(tab => !tab.resourceName && !tab.resourceId);
  $: activeTab = $terminalStore.activeTabId;

  function createNewTerminalTab(shellCommand?: string) {
    log.debug('Creating new terminal tab', { shellCommand });
    const tabNumber = tabs.length + 1;
    log.debug('Tab creation', { currentTabs: tabs.length, newTabNumber: tabNumber });
    
    // Use the provided shell command or fallback to default shell
    const actualShellCommand = shellCommand || settings.defaultShell;
    
    const tabId = terminalActions.createTab({
      title: `Terminal ${tabNumber}`,
      type: 'terminal',
      workingDirectory: settings.workingDirectory,
      shell: actualShellCommand,
      icon: '💻',
      closable: true,
      // No resourceName or resourceId - this is a global terminal
    });

    log.info('Created tab', { tabId, shell: actualShellCommand });

    // Create a process for the new tab
    const processId = terminalActions.createProcess({
      tabId,
      command: actualShellCommand,
      workingDirectory: settings.workingDirectory,
      environment: {},
      status: 'running'
    });

    log.info('Created process', { processId });
    return tabId;
  }

  // Wrapper function to handle new tab creation with shell command
  function handleNewTabWithProfile(shellCommand?: string) {
    createNewTerminalTab(shellCommand);
  }

  onMount(() => {
    // Create initial tab if no global tabs exist
    if (tabs.length === 0) {
      createNewTerminalTab();
    } else {
      // Ensure at least one global tab is active
      if (!$terminalStore.activeTabId || !tabs.some(tab => tab.id === $terminalStore.activeTabId)) {
        terminalActions.setActiveTab(tabs[0].id);
      }
    }
  });
</script>

<TabContainer onNewTab={handleNewTabWithProfile} className="terminal-tab-container">
  <!-- Render all terminal instances but only show the active one -->
  {#each tabs as tab (tab.id)}
    <div class="terminal-tab-content" class:active={tab.id === activeTab} style:display={tab.id === activeTab ? 'block' : 'none'}>
      <Terminal 
        tabId={tab.id} 
        settings={{
          ...settings,
          defaultShell: tab.shell || settings.defaultShell
        }}
      />
    </div>
  {/each}
  
  {#if tabs.length === 0}
    <div class="flex items-center justify-center h-full">
      <div class="text-center">
        <div class="text-gray-400 mb-2">No tabs available</div>
        <Button
          onclick={() => createNewTerminalTab()}
          class="px-4 py-2"
          type="button"
        >
          Create New Tab
        </Button>
      </div>
    </div>
  {/if}
</TabContainer>

<style>
  :global(.terminal-tab-container) {
    background: #1f2937;
  }
  
  .terminal-tab-content {
    height: 100%;
    width: 100%;
  }
  
  .terminal-tab-content.active {
    display: block !important;
  }
</style>
