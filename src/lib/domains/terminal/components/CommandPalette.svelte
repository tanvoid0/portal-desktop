<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { commandPaletteStore } from '../stores/commandPaletteStore';
  import { commandHistoryStore } from '../stores/commandHistoryStore';
  import { Button } from '@/lib/components/ui/button';
  import { Input } from '@/lib/components/ui/input';
  import { Badge } from '@/lib/components/ui/badge';
  import { 
    Search, 
    X, 
    Play, 
    Square, 
    Trash2, 
    Copy, 
    Clock, 
    Terminal,
    Zap
  } from 'lucide-svelte';

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
      if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
        event.preventDefault();
        commandPaletteStore.open();
      }
    };

    document.addEventListener('keydown', handleKeydown);
    
    // Cleanup
    unsubscribe = () => {
      document.removeEventListener('keydown', handleKeydown);
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
        id: 'kill-process',
        label: 'Kill Current Process',
        description: 'Terminate the currently running process',
        icon: 'Square',
        action: () => {
          onKillProcess();
          commandPaletteStore.close();
        },
        keywords: ['kill', 'stop', 'terminate', 'process']
      },
      {
        id: 'clear-terminal',
        label: 'Clear Terminal',
        description: 'Clear the terminal screen',
        icon: 'Trash2',
        action: () => {
          onClearTerminal();
          commandPaletteStore.close();
        },
        keywords: ['clear', 'terminal', 'screen']
      },
      {
        id: 'rerun-last',
        label: 'Rerun Last Command',
        description: 'Execute the most recent command again',
        icon: 'Play',
        action: () => {
          const history = commandHistoryStore.getTabHistory(tabId);
          if (history.length > 0) {
            onRerunCommand(history[0].command);
          }
          commandPaletteStore.close();
        },
        keywords: ['rerun', 'repeat', 'last', 'command']
      }
    ];

    // Add recent commands as actions
    const recentCommands = commandHistoryStore.getTabHistory(tabId).slice(0, 5);
    recentCommands.forEach((entry, index) => {
      actions.push({
        id: `rerun-${entry.id}`,
        label: `Rerun: ${entry.command}`,
        description: `Execute: ${entry.command}`,
        icon: 'Clock',
        action: () => {
          onRerunCommand(entry.command);
          commandPaletteStore.close();
        },
        keywords: ['rerun', 'repeat', entry.command.toLowerCase()]
      });
    });

    commandPaletteStore.setActions(actions);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!isOpen) return;

    switch (event.key) {
      case 'Escape':
        event.preventDefault();
        commandPaletteStore.close();
        break;
      case 'ArrowDown':
        event.preventDefault();
        commandPaletteStore.selectNext();
        break;
      case 'ArrowUp':
        event.preventDefault();
        commandPaletteStore.selectPrevious();
        break;
      case 'Enter':
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
{#if isOpen}
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-start justify-center z-50 pt-20"
    onclick={() => commandPaletteStore.close()}
    onkeydown={handleKeydown}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div 
      class="bg-gray-900 rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[60vh] flex flex-col"
      role="dialog"
      aria-labelledby="command-palette-title"
    >
      <!-- Header -->
      <div class="flex items-center p-4 border-b border-gray-700">
        <Search class="w-5 h-5 text-gray-400 mr-3" />
        <Input
          bind:ref={searchInput}
          bind:value={query}
          placeholder="Search commands and actions..."
          class="flex-1 bg-transparent border-0 text-gray-200 placeholder-gray-400 focus:ring-0"
          oninput={handleQueryChange}
          onkeydown={handleKeydown}
        />
        <Button
          variant="ghost"
          size="sm"
          onclick={() => commandPaletteStore.close()}
          class="ml-2"
        >
          <X class="w-4 h-4" />
        </Button>
      </div>

      <!-- Actions List -->
      <div class="flex-1 overflow-y-auto">
        {#if filteredActions.length > 0}
          {#each filteredActions as action, index (action.id)}
            <button
              class="w-full p-3 text-left hover:bg-gray-800 transition-colors flex items-center space-x-3 {selectedIndex === index ? 'bg-gray-800' : ''}"
              onclick={() => handleActionClick(action)}
              onkeydown={(e) => e.key === 'Enter' && handleActionClick(action)}
            >
              <div class="flex-shrink-0">
                {#if action.icon === 'Square'}
                  <Square class="w-5 h-5 text-red-400" />
                {:else if action.icon === 'Trash2'}
                  <Trash2 class="w-5 h-5 text-orange-400" />
                {:else if action.icon === 'Play'}
                  <Play class="w-5 h-5 text-green-400" />
                {:else if action.icon === 'Clock'}
                  <Clock class="w-5 h-5 text-blue-400" />
                {:else}
                  <Terminal class="w-5 h-5 text-gray-400" />
                {/if}
              </div>
              
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium text-gray-200 truncate">
                  {action.label}
                </div>
                <div class="text-xs text-gray-400 truncate">
                  {action.description}
                </div>
              </div>
              
              <div class="flex-shrink-0 flex items-center space-x-2">
                {#if action.id.startsWith('rerun-')}
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={(e) => {
                      e.stopPropagation();
                      copyToClipboard(action.label.replace('Rerun: ', ''));
                    }}
                    class="h-6 w-6 p-0"
                  >
                    <Copy class="w-3 h-3" />
                  </Button>
                {/if}
                {#if selectedIndex === index}
                  <Badge variant="outline" class="text-xs">
                    Enter
                  </Badge>
                {/if}
              </div>
            </button>
          {/each}
        {:else}
          <div class="p-6 text-center text-gray-500">
            <Zap class="w-8 h-8 mx-auto mb-2 text-gray-600" />
            <div class="text-sm">No actions found</div>
            <div class="text-xs mt-1">Try a different search term</div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="p-3 border-t border-gray-700 text-xs text-gray-400">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <span>↑↓ Navigate</span>
            <span>Enter Execute</span>
            <span>Esc Close</span>
          </div>
          <div>
            Cmd+K to open
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
 
</style>
