<script lang="ts">
  import { commandHistoryStore, type CommandHistoryEntry } from '../stores/commandHistoryStore';
  import { formatCommandOutput } from '../utils/textUtils';
  import { formatDuration, getDurationColor, getDurationBadgeVariant, calculateAverageDuration } from '../utils/durationUtils';
  import { Badge } from '@/lib/components/ui/badge';
  import { Button } from '@/lib/components/ui/button';
  import { onMount } from 'svelte';

  interface Props {
    tabId: string;
  }

  let {
    tabId
  }: Props = $props();

  let selectedEntry = $state<CommandHistoryEntry | null>(null);
  let showModal = $state(false);

  // Get reactive history for the current tab
  const tabHistoryStore = $derived(commandHistoryStore.getTabHistoryReactive(tabId));
  const averageDuration = $derived(calculateAverageDuration(tabHistoryStore));

  function formatTimestamp(date: Date): string {
    return date.toLocaleTimeString();
  }

  function truncateText(text: string, maxLength: number = 100): string {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + '...';
  }

  function getStatusIcon(entry: CommandHistoryEntry): string {
    if (entry.intercepted) return '🎯';
    if (entry.exitCode === 0) return '✅';
    if (entry.exitCode && entry.exitCode !== 0) return '❌';
    return '⏳';
  }

  function getStatusColor(entry: CommandHistoryEntry): string {
    if (entry.intercepted) return 'text-blue-400';
    if (entry.exitCode === 0) return 'text-green-400';
    if (entry.exitCode && entry.exitCode !== 0) return 'text-red-400';
    return 'text-yellow-400';
  }

  function showEntryDetails(entry: CommandHistoryEntry) {
    selectedEntry = entry;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    selectedEntry = null;
  }

  function clearHistory() {
    commandHistoryStore.clearHistory(tabId);
  }
</script>

<div class="command-history h-full flex flex-col">
  <!-- Header -->
  <div class="flex items-center justify-between p-3 border-b border-gray-700">
    <div class="flex items-center space-x-2">
      <h3 class="text-sm font-medium text-gray-300">Command History</h3>
      {#if averageDuration > 0}
        <Badge variant="outline" class="text-xs">
          Avg: {formatDuration(averageDuration)}
        </Badge>
      {/if}
    </div>
    <Button
      variant="ghost"
      size="sm"
      onclick={clearHistory}
      class="text-xs text-gray-400 hover:text-gray-200"
      title="Clear history"
    >
      Clear
    </Button>
  </div>

  <!-- History List -->
  <div class="flex-1 overflow-y-auto">
    {#each $tabHistoryStore as entry (entry.id)}
      <Button
        variant="ghost"
        class="w-full p-3 border-b border-gray-800 hover:bg-gray-800 cursor-pointer transition-colors text-left justify-start h-auto"
        onclick={() => showEntryDetails(entry)}
        onkeydown={(e) => e.key === 'Enter' && showEntryDetails(entry)}
        type="button"
      >
        <div class="flex items-start justify-between mb-2">
          <div class="flex items-center space-x-2">
            <span class="text-xs {getStatusColor(entry)}">
              {getStatusIcon(entry)}
            </span>
            <code class="text-sm text-gray-200 font-mono bg-gray-900 px-2 py-1 rounded">
              {entry.command}
            </code>
            {#if entry.duration}
              <Badge 
                variant={getDurationBadgeVariant(entry.duration)} 
                class="text-xs {getDurationColor(entry.duration)}"
              >
                {formatDuration(entry.duration)}
              </Badge>
            {/if}
          </div>
          <span class="text-xs text-gray-500">
            {formatTimestamp(entry.timestamp)}
          </span>
        </div>
        
        <div class="text-xs text-gray-400">
          {formatCommandOutput(entry.output, 80)}
        </div>
      </Button>
    {:else}
      <div class="p-6 text-center text-gray-500">
        <div class="text-4xl mb-2">📝</div>
        <div class="text-sm">No commands executed yet</div>
        <div class="text-xs mt-1">Commands will appear here as you use the terminal</div>
      </div>
    {/each}
  </div>
</div>

<!-- Modal -->
{#if showModal && selectedEntry}
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" 
    onclick={closeModal}
    onkeydown={(e) => e.key === 'Escape' && closeModal()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div 
      class="bg-gray-900 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[80vh] flex flex-col" 
      role="document"
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-700">
        <h2 class="text-lg font-medium text-gray-200">Command Details</h2>
        <Button
          variant="ghost"
          size="sm"
          onclick={closeModal}
          class="text-gray-400 hover:text-gray-200 text-xl h-auto p-1"
        >
          ×
        </Button>
      </div>

      <!-- Modal Content -->
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <!-- Command -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-2">Command</h3>
          <code class="block bg-gray-800 text-green-400 p-3 rounded font-mono text-sm">
            {selectedEntry.command}
          </code>
        </div>

        <!-- Metadata -->
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-gray-400">Timestamp:</span>
            <span class="text-gray-200 ml-2">{selectedEntry.timestamp.toLocaleString()}</span>
          </div>
          <div>
            <span class="text-gray-400">Status:</span>
            <span class="ml-2 {getStatusColor(selectedEntry)}">
              {getStatusIcon(selectedEntry)} 
              {selectedEntry.intercepted ? 'Intercepted' : 
               selectedEntry.exitCode === 0 ? 'Success' : 
               selectedEntry.exitCode ? `Failed (${selectedEntry.exitCode})` : 'Running'}
            </span>
          </div>
          {#if selectedEntry.duration}
            <div>
              <span class="text-gray-400">Duration:</span>
              <span class="ml-2 {getDurationColor(selectedEntry.duration)} font-mono">
                {formatDuration(selectedEntry.duration)}
              </span>
            </div>
          {/if}
        </div>

        <!-- Output -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-2">Output</h3>
          <pre class="bg-gray-800 text-gray-200 p-3 rounded text-xs overflow-x-auto whitespace-pre-wrap">{selectedEntry.output || '(No output)'}</pre>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="flex justify-end p-4 border-t border-gray-700">
        <Button
          variant="secondary"
          onclick={closeModal}
          class="px-4 py-2"
        >
          Close
        </Button>
      </div>
    </div>
  </div>
{/if}

<style>
  .command-history {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  }
</style>
