<script lang="ts">
  import { onMount } from 'svelte';
  import { Terminal, ContainerizedTerminal, terminalActions, terminalStore } from '$lib/domains/terminal';
  import type { TerminalSettings } from '$lib/domains/terminal';

  let tabId: string;
  let showContainerized = false;
  let settings: TerminalSettings = {
    theme: 'dark',
    fontSize: 14,
    fontFamily: 'Monaco, Consolas, "Courier New", monospace',
    cursorStyle: 'block',
    scrollbackLines: 1000,
    bellSound: false,
    autoClose: true,
    confirmClose: true,
    defaultShell: navigator.userAgent.includes('Windows') ? 'cmd.exe' : 'bash',
    workingDirectory: navigator.userAgent.includes('Windows') ? 'C:\\' : '/'
  };

  onMount(() => {
    // Create a new terminal tab
    tabId = terminalActions.createTab('Terminal', '/', undefined);
  });

  function toggleView() {
    showContainerized = !showContainerized;
  }
</script>

<div class="h-screen w-full bg-gray-900 flex flex-col">
  <!-- Header with Toggle -->
  <div class="terminal-header bg-gray-800 border-b border-gray-700 px-4 py-2">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-4">
        <h1 class="text-lg font-semibold text-gray-100">Terminal</h1>
        <div class="flex items-center space-x-2">
          <div class="w-2 h-2 bg-green-500 rounded-full"></div>
          <span class="text-sm text-gray-400">Connected</span>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <button
          on:click={toggleView}
          class="px-3 py-1 text-sm rounded transition-colors {showContainerized ? 'bg-blue-600 text-white' : 'bg-gray-600 text-gray-300 hover:bg-gray-500'}"
          type="button"
        >
          {showContainerized ? 'Standard View' : 'Containerized View'}
        </button>
      </div>
    </div>
  </div>

  <!-- Terminal Content -->
  <div class="flex-1 min-h-0">
    {#if showContainerized}
      <ContainerizedTerminal />
    {:else if tabId}
      <Terminal {tabId} {settings} />
    {:else}
      <div class="flex items-center justify-center h-full">
        <div class="text-gray-400">Initializing terminal...</div>
      </div>
    {/if}
  </div>
</div>
