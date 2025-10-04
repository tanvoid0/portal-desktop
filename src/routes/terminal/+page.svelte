<script lang="ts">
  import { onMount } from 'svelte';
  import { Terminal, terminalActions, terminalStore } from '$lib/domains/terminal';
  import type { TerminalSettings } from '$lib/domains/terminal';

  let tabId: string;
  let settings: TerminalSettings = {
    theme: 'dark',
    fontSize: 14,
    fontFamily: 'Monaco, Consolas, "Courier New", monospace',
    cursorStyle: 'block',
    scrollbackLines: 1000,
    bellSound: false,
    autoClose: true,
    confirmClose: true,
    defaultShell: 'bash',
    workingDirectory: '/'
  };

  onMount(() => {
    // Create a new terminal tab
    tabId = terminalActions.createTab('Terminal', '/', undefined);
  });
</script>

<div class="h-screen w-full bg-gray-900">
  {#if tabId}
    <Terminal {tabId} {settings} />
  {:else}
    <div class="flex items-center justify-center h-full">
      <div class="text-gray-400">Initializing terminal...</div>
    </div>
  {/if}
</div>
