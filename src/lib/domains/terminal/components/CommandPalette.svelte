<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { onMount, onDestroy } from "svelte";
  import { commandPaletteStore } from "../stores/commandPaletteStore";
  import { commandHistoryStore } from "../stores/commandHistoryStore";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Search,
    X,
    Play,
    Square,
    Trash2,
    Copy,
    Clock,
    Terminal,
    Zap,
  } from "@lucide/svelte";

  export let tabId: string;
  export let onKillProcess: () => void;
  export let onClearTerminal: () => void;
  export let onRerunCommand: (command: string) => void;

  let searchInput: HTMLInputElement;
  let unsubscribe: (() => void) | null = null;

  // Get reactive state
  $: isOpen = $commandPaletteStore.isOpen;
  $: query = $commandPaletteStore.query;
  $: selectedIndex = $commandPaletteStore.selectedIndex;
  $: filteredActions = $commandPaletteStore.filteredActions;

  onMount(() => {
    // Set up keyboard shortcuts
    const handleKeydown = (event: KeyboardEvent) => {
      // Cmd/Ctrl + K to open palette
      if ((event.metaKey || event.ctrlKey) && event.key === "k") {
        event.preventDefault();
        commandPaletteStore.open();
      }
    };

    document.addEventListener("keydown", handleKeydown);

    // Cleanup
    unsubscribe = () => {
      document.removeEventListener("keydown", handleKeydown);
    };

    // Initialize actions
    initializeActions();
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });

  function initializeActions() {
    const actions = [
      {
        id: "kill-process",
        label: "Kill Current Process",
        description: "Terminate the currently running process",
        icon: "Square",
        action: () => {
          onKillProcess();
          commandPaletteStore.close();
        },
        keywords: ["kill", "stop", "terminate", "process"],
      },
      {
        id: "clear-terminal",
        label: "Clear Terminal",
        description: "Clear the terminal screen",
        icon: "Trash2",
        action: () => {
          onClearTerminal();
          commandPaletteStore.close();
        },
        keywords: ["clear", "terminal", "screen"],
      },
      {
        id: "rerun-last",
        label: "Rerun Last Command",
        description: "Execute the most recent command again",
        icon: "Play",
        action: () => {
          const history = commandHistoryStore.getTabHistory(tabId);
          if (history.length > 0) {
            onRerunCommand(history[0].command);
          }
          commandPaletteStore.close();
        },
        keywords: ["rerun", "repeat", "last", "command"],
      },
    ];

    // Add recent commands as actions
    const recentCommands = commandHistoryStore.getTabHistory(tabId).slice(0, 5);
    recentCommands.forEach((entry, index) => {
      actions.push({
        id: `rerun-${entry.id}`,
        label: `Rerun: ${entry.command}`,
        description: `Execute: ${entry.command}`,
        icon: "Clock",
        action: () => {
          onRerunCommand(entry.command);
          commandPaletteStore.close();
        },
        keywords: ["rerun", "repeat", entry.command.toLowerCase()],
      });
    });

    commandPaletteStore.setActions(actions);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!isOpen) return;

    switch (event.key) {
      case "Escape":
        event.preventDefault();
        commandPaletteStore.close();
        break;
      case "ArrowDown":
        event.preventDefault();
        commandPaletteStore.selectNext();
        break;
      case "ArrowUp":
        event.preventDefault();
        commandPaletteStore.selectPrevious();
        break;
      case "Enter":
        event.preventDefault();
        commandPaletteStore.executeSelected();
        break;
    }
  }

  function handleQueryChange(event: Event) {
    const target = event.target as HTMLInputElement;
    commandPaletteStore.setQuery(target.value);
  }

  function handleActionClick(action: any) {
    action.action();
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
    commandPaletteStore.close();
  }
</script>

<!-- Command Palette Modal -->
<Dialog.Root
  open={isOpen}
  onOpenChange={(open) => {
    if (!open) commandPaletteStore.close();
  }}
>
  <Dialog.Content
    class="mx-4 flex max-h-[60vh] max-w-2xl flex-col bg-gray-900 p-0 text-gray-200"
    onkeydown={handleKeydown}
  >
    <div class="flex items-center border-b border-gray-700 p-4">
        <Search class="mr-3 h-5 w-5 text-gray-400" />
        <Input
          bind:ref={searchInput}
          bind:value={query}
          placeholder="Search commands and actions..."
          class="flex-1 border-0 bg-transparent text-gray-200 placeholder-gray-400 focus:ring-0"
          oninput={handleQueryChange}
          onkeydown={handleKeydown}
        />
        <Button
          variant="ghost"
          size="sm"
          onclick={() => commandPaletteStore.close()}
          class="ml-2"
        >
          <X class="h-4 w-4" />
        </Button>
      </div>

      <!-- Actions List -->
      <div class="flex-1 overflow-y-auto">
        {#if filteredActions.length > 0}
          {#each filteredActions as action, index (action.id)}
            <Button
              variant="ghost"
              class="flex h-auto w-full items-center space-x-3 p-3 text-left {selectedIndex ===
              index
                ? 'bg-gray-800'
                : ''}"
              onclick={() => handleActionClick(action)}
              onkeydown={(e) => e.key === "Enter" && handleActionClick(action)}
            >
              <div class="flex-shrink-0">
                {#if action.icon === "Square"}
                  <Square class="h-5 w-5 text-red-400" />
                {:else if action.icon === "Trash2"}
                  <Trash2 class="h-5 w-5 text-orange-400" />
                {:else if action.icon === "Play"}
                  <Play class="h-5 w-5 text-green-400" />
                {:else if action.icon === "Clock"}
                  <Clock class="h-5 w-5 text-blue-400" />
                {:else}
                  <Terminal class="h-5 w-5 text-gray-400" />
                {/if}
              </div>

              <div class="min-w-0 flex-1">
                <div class="truncate text-sm font-medium text-gray-200">
                  {action.label}
                </div>
                <div class="truncate text-xs text-gray-400">
                  {action.description}
                </div>
              </div>

              <div class="flex flex-shrink-0 items-center space-x-2">
                {#if action.id.startsWith("rerun-")}
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={(e) => {
                      e.stopPropagation();
                      copyToClipboard(action.label.replace("Rerun: ", ""));
                    }}
                    class="h-6 w-6 p-0"
                  >
                    <Copy class="h-3 w-3" />
                  </Button>
                {/if}
                {#if selectedIndex === index}
                  <Badge variant="outline" class="text-xs">Enter</Badge>
                {/if}
              </div>
            </Button>
          {/each}
        {:else}
          <div class="p-6 text-center text-gray-500">
            <Zap class="mx-auto mb-2 h-8 w-8 text-gray-600" />
            <div class="text-sm">No actions found</div>
            <div class="mt-1 text-xs">Try a different search term</div>
          </div>
        {/if}
      </div>

      <div class="border-t border-gray-700 p-3 text-xs text-gray-400">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <span>↑↓ Navigate</span>
            <span>Enter Execute</span>
            <span>Esc Close</span>
          </div>
          <div>Cmd+K to open</div>
        </div>
      </div>
  </Dialog.Content>
</Dialog.Root>

<style>
</style>
