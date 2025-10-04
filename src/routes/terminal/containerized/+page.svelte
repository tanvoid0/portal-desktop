<script lang="ts">
  import { ContainerizedTerminal } from '$lib/domains/terminal';
  import { onMount } from 'svelte';

  let settings = {
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
    // Any initialization logic
  });
</script>

<svelte:head>
  <title>Containerized Terminal - Portal Desktop</title>
</svelte:head>

<div class="terminal-page h-screen w-full bg-gray-900 flex flex-col">
  <!-- Header -->
  <div class="terminal-header bg-gray-800 border-b border-gray-700 px-4 py-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-4">
        <h1 class="text-lg font-semibold text-gray-100">Containerized Terminal</h1>
        <div class="flex items-center space-x-2">
          <div class="w-2 h-2 bg-green-500 rounded-full"></div>
          <span class="text-sm text-gray-400">Connected</span>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <span class="text-xs text-gray-500">Commands with colored backgrounds</span>
      </div>
    </div>
  </div>

  <!-- Terminal Container -->
  <div class="flex-1 min-h-0">
    <ContainerizedTerminal />
  </div>
</div>

<style>
  .terminal-page {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }
</style>
