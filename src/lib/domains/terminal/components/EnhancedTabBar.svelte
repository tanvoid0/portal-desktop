<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { tabStore, type TerminalTab } from '../stores/tabStore';
  import { Button } from '@/lib/components/ui/button';
  import { Badge } from '@/lib/components/ui/badge';
  import { 
    Plus, 
    X, 
    Copy, 
    MoreHorizontal,
    Terminal,
    Folder,
    Activity
  } from '@lucide/svelte';

  interface Props {
    onNewTab: () => void;
    onCloseTab: (tabId: string) => void;
    onActivateTab: (tabId: string) => void;
    onDuplicateTab: (tabId: string) => void;
  }

  let {
    onNewTab,
    onCloseTab,
    onActivateTab,
    onDuplicateTab
  }: Props = $props();

  let draggedTab = $state<TerminalTab | null>(null);
  let dragOverIndex = $state<number | null>(null);

  // Get reactive state
  const tabs = $derived($tabStore.tabs);
  const activeTabId = $derived($tabStore.activeTabId);

  function handleDragStart(event: DragEvent, tab: TerminalTab) {
    draggedTab = tab;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', tab.id);
    }
  }

  function handleDragOver(event: DragEvent, index: number) {
    event.preventDefault();
    dragOverIndex = index;
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  function handleDrop(event: DragEvent, dropIndex: number) {
    event.preventDefault();
    
    if (draggedTab && dropIndex < tabs.length && draggedTab.id !== tabs[dropIndex]?.id) {
      const fromIndex = tabs.findIndex(tab => tab.id === draggedTab!.id);
      if (fromIndex !== -1) {
        tabStore.reorderTabs(fromIndex, dropIndex);
      }
    }
    
    draggedTab = null;
    dragOverIndex = null;
  }

  function handleTabClick(tabId: string) {
    onActivateTab(tabId);
    tabStore.activateTab(tabId);
  }

  function handleTabKeyDown(event: KeyboardEvent, tabId: string) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleTabClick(tabId);
    }
  }

  function handleCloseTab(event: MouseEvent, tabId: string) {
    event.stopPropagation();
    onCloseTab(tabId);
    tabStore.removeTab(tabId);
  }

  function handleDuplicateTab(event: MouseEvent, tabId: string) {
    event.stopPropagation();
    onDuplicateTab(tabId);
    tabStore.duplicateTab(tabId);
  }

  function formatLastActivity(lastActivity: string): string {
    const date = new Date(lastActivity);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    
    if (diffMins < 1) return 'now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
    return date.toLocaleDateString();
  }

  function getTabTitle(tab: TerminalTab): string {
    if (tab.title) return tab.title;
    const dir = tab.workingDirectory.split('/').pop() || 'home';
    return dir;
  }
</script>

<div class="enhanced-tab-bar flex items-center bg-gray-800 border-b border-gray-700 overflow-x-auto">
  <!-- Tabs -->
  <div class="flex items-center min-w-0 flex-1">
    {#each tabs as tab, index (tab.id)}
      <div
        class="tab-item flex items-center min-w-0 max-w-48 group relative {tab.isActive ? 'bg-gray-700' : 'hover:bg-gray-750'} {dragOverIndex === index ? 'border-t-2 border-blue-400' : ''}"
        draggable="true"
        ondragstart={(e) => handleDragStart(e, tab)}
        ondragover={(e) => handleDragOver(e, index)}
        ondragleave={handleDragLeave}
        ondrop={(e) => handleDrop(e, index)}
        onclick={() => handleTabClick(tab.id)}
        onkeydown={(e) => handleTabKeyDown(e, tab.id)}
        role="tab"
        aria-selected={tab.isActive}
        tabindex="0"
      >
        <!-- Tab Content -->
        <div class="flex items-center min-w-0 px-3 py-2">
          <!-- Tab Icon -->
          <div class="flex-shrink-0 mr-2">
            {#if tab.hasRunningProcess}
              <Activity class="w-4 h-4 text-green-400 animate-pulse" />
            {:else}
              <Terminal class="w-4 h-4 text-gray-400" />
            {/if}
          </div>
          
          <!-- Tab Title -->
          <div class="min-w-0 flex-1">
            <div class="text-sm font-medium text-gray-200 truncate">
              {getTabTitle(tab)}
            </div>
            <div class="text-xs text-gray-500 truncate">
              {tab.workingDirectory}
            </div>
          </div>
          
          <!-- Tab Status -->
          <div class="flex-shrink-0 ml-2 flex items-center space-x-1">
            {#if tab.hasRunningProcess}
              <Badge variant="secondary" class="text-xs bg-green-500/10 text-green-400 border-green-500/20">
                Running
              </Badge>
            {/if}
            
            <!-- Tab Actions -->
            <div class="opacity-0 group-hover:opacity-100 transition-opacity flex items-center space-x-1">
              <Button
                variant="ghost"
                size="sm"
                onclick={(e) => handleDuplicateTab(e, tab.id)}
                class="h-6 w-6 p-0 hover:bg-gray-600"
                title="Duplicate tab"
              >
                <Copy class="w-3 h-3" />
              </Button>
              
              <Button
                variant="ghost"
                size="sm"
                onclick={(e) => handleCloseTab(e, tab.id)}
                class="h-6 w-6 p-0 hover:bg-red-500/20 hover:text-red-400"
                title="Close tab"
              >
                <X class="w-3 h-3" />
              </Button>
            </div>
          </div>
        </div>
        
        <!-- Active Tab Indicator -->
        {#if tab.isActive}
          <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-400"></div>
        {/if}
      </div>
    {/each}
  </div>
  
  <!-- New Tab Button -->
  <div class="flex-shrink-0 p-2">
    <Button
      variant="ghost"
      size="sm"
      onclick={onNewTab}
      class="h-8 w-8 p-0 hover:bg-gray-700"
      title="New tab"
    >
      <Plus class="w-4 h-4" />
    </Button>
  </div>
</div>

<style>
  .enhanced-tab-bar {
    scrollbar-width: thin;
    scrollbar-color: rgba(156, 163, 175, 0.3) transparent;
  }
  
  .enhanced-tab-bar::-webkit-scrollbar {
    height: 4px;
  }
  
  .enhanced-tab-bar::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .enhanced-tab-bar::-webkit-scrollbar-thumb {
    background: rgba(156, 163, 175, 0.3);
    border-radius: 2px;
  }
  
  .tab-item {
    border-right: 1px solid rgba(55, 65, 81, 0.5);
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .tab-item:hover {
    background: rgba(55, 65, 81, 0.8);
  }
  
  .tab-item:focus {
    outline: 2px solid rgba(59, 130, 246, 0.5);
    outline-offset: -2px;
  }
</style>
