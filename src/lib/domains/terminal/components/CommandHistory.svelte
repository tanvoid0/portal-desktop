<script lang="ts">
  import {
    commandHistoryStore,
    type CommandHistoryEntry,
  } from "../stores/commandHistoryStore";
  import { formatCommandOutput } from "../utils/textUtils";
  import {
    formatDuration,
    getDurationColor,
    getDurationBadgeVariant,
    calculateAverageDuration,
  } from "../utils/durationUtils";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { onMount } from "svelte";

  interface Props {
    tabId: string;
  }

  let { tabId }: Props = $props();

  let selectedEntry = $state<CommandHistoryEntry | null>(null);
  let showModal = $state(false);

  // Get reactive history for the current tab
  const tabHistoryStore = $derived(
    commandHistoryStore.getTabHistoryReactive(tabId),
  );
  let tabHistory = $state<CommandHistoryEntry[]>([]);

  // Subscribe to store to get array
  $effect(() => {
    const unsubscribe = tabHistoryStore.subscribe((entries) => {
      tabHistory = entries;
    });
    return unsubscribe;
  });

  const averageDuration = $derived(calculateAverageDuration(tabHistory));

  function formatTimestamp(date: Date): string {
    return date.toLocaleTimeString();
  }

  function truncateText(text: string, maxLength: number = 100): string {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
  }

  function getStatusIcon(entry: CommandHistoryEntry): string {
    if (entry.intercepted) return "🎯";
    if (entry.exitCode === 0) return "✅";
    if (entry.exitCode && entry.exitCode !== 0) return "❌";
    return "⏳";
  }

  function getStatusColor(entry: CommandHistoryEntry): string {
    if (entry.intercepted) return "text-blue-400";
    if (entry.exitCode === 0) return "text-green-400";
    if (entry.exitCode && entry.exitCode !== 0) return "text-red-400";
    return "text-yellow-400";
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

<div class="command-history flex h-full flex-col">
  <!-- Header -->
  <div class="flex items-center justify-between border-b border-gray-700 p-3">
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
    {#each tabHistory as entry (entry.id)}
      <Button
        variant="ghost"
        class="h-auto w-full cursor-pointer justify-start border-b border-gray-800 p-3 text-left transition-colors hover:bg-gray-800"
        onclick={() => showEntryDetails(entry)}
        onkeydown={(e) => e.key === "Enter" && showEntryDetails(entry)}
        type="button"
      >
        <div class="mb-2 flex items-start justify-between">
          <div class="flex items-center space-x-2">
            <span class="text-xs {getStatusColor(entry)}">
              {getStatusIcon(entry)}
            </span>
            <code
              class="rounded bg-gray-900 px-2 py-1 font-mono text-sm text-gray-200"
            >
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
        <div class="text-xs mt-1">
          Commands will appear here as you use the terminal
        </div>
      </div>
    {/each}
  </div>
</div>

<Dialog.Root bind:open={showModal}>
  <Dialog.Content class="mx-4 flex max-h-[80vh] max-w-4xl flex-col bg-gray-900 text-gray-200">
    {#if selectedEntry}
      <Dialog.Header class="border-b border-gray-700">
        <Dialog.Title>Command Details</Dialog.Title>
      </Dialog.Header>

      <div class="flex-1 space-y-4 overflow-y-auto p-4">
        <!-- Command -->
        <div>
          <h3 class="mb-2 text-sm font-medium text-gray-300">Command</h3>
          <code
            class="block rounded bg-gray-800 p-3 font-mono text-sm text-green-400"
          >
            {selectedEntry.command}
          </code>
        </div>

        <!-- Metadata -->
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-gray-400">Timestamp:</span>
            <span class="ml-2 text-gray-200"
              >{selectedEntry.timestamp.toLocaleString()}</span
            >
          </div>
          <div>
            <span class="text-gray-400">Status:</span>
            <span class="ml-2 {getStatusColor(selectedEntry)}">
              {getStatusIcon(selectedEntry)}
              {selectedEntry.intercepted
                ? "Intercepted"
                : selectedEntry.exitCode === 0
                  ? "Success"
                  : selectedEntry.exitCode
                    ? `Failed (${selectedEntry.exitCode})`
                    : "Running"}
            </span>
          </div>
          {#if selectedEntry.duration}
            <div>
              <span class="text-gray-400">Duration:</span>
              <span
                class="ml-2 {getDurationColor(
                  selectedEntry.duration,
                )} font-mono"
              >
                {formatDuration(selectedEntry.duration)}
              </span>
            </div>
          {/if}
        </div>

        <!-- Output -->
        <div>
          <h3 class="mb-2 text-sm font-medium text-gray-300">Output</h3>
          <pre
            class="overflow-x-auto whitespace-pre-wrap rounded bg-gray-800 p-3 text-xs text-gray-200">{selectedEntry.output ||
              "(No output)"}</pre>
        </div>
      </div>

      <div class="flex justify-end border-t border-gray-700 p-4">
        <Button variant="secondary" onclick={closeModal}>Close</Button>
      </div>
    {/if}
  </Dialog.Content>
</Dialog.Root>

<style>
  .command-history {
    font-family:
      "Inter",
      -apple-system,
      BlinkMacSystemFont,
      sans-serif;
  }
</style>
