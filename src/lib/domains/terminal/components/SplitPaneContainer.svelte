<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import ResizablePane from './ResizablePane.svelte';
  import { Button } from '@/lib/components/ui/button';
  import { Maximize2, Minimize2, Split, X } from '@lucide/svelte';
  
  export let direction: 'horizontal' | 'vertical' = 'horizontal';
  export let panes: Array<{
    id: string;
    component: any;
    props: any;
    size: number;
  }> = [];
  
  let container: HTMLDivElement;
  let isMaximized = false;
  let maximizedPaneId: string | null = null;
  
  function handleResize(event: { size: number }) {
    const { size } = event;
    // Update pane sizes based on resize
    panes = panes.map((pane, index) => {
      if (index === 0) {
        return { ...pane, size };
      } else {
        return { ...pane, size: 100 - size };
      }
    });
  }
  
  function splitPane(paneId: string, splitDirection: 'horizontal' | 'vertical') {
    const paneIndex = panes.findIndex(p => p.id === paneId);
    if (paneIndex === -1) return;
    
    const newPane = {
      id: `pane-${Date.now()}`,
      component: panes[paneIndex].component,
      props: { ...panes[paneIndex].props },
      size: 50
    };
    
    // Insert new pane after current pane
    panes = [
      ...panes.slice(0, paneIndex + 1),
      newPane,
      ...panes.slice(paneIndex + 1)
    ];
  }
  
  function closePane(paneId: string) {
    if (panes.length <= 1) return;
    
    panes = panes.filter(p => p.id !== paneId);
    
    // Redistribute sizes
    const totalSize = panes.reduce((sum, p) => sum + p.size, 0);
    panes = panes.map(p => ({
      ...p,
      size: (p.size / totalSize) * 100
    }));
  }
  
  function maximizePane(paneId: string) {
    isMaximized = true;
    maximizedPaneId = paneId;
  }
  
  function minimizePane() {
    isMaximized = false;
    maximizedPaneId = null;
  }
  
  function toggleMaximize(paneId: string) {
    if (isMaximized && maximizedPaneId === paneId) {
      minimizePane();
    } else {
      maximizePane(paneId);
    }
  }
</script>

<div 
  bind:this={container}
  class="split-pane-container h-full w-full flex {direction === 'horizontal' ? 'flex-row' : 'flex-col'}"
>
  {#if isMaximized && maximizedPaneId}
    <!-- Maximized view -->
    <div class="flex-1 relative">
      <div class="absolute top-2 right-2 z-10">
        <Button
          variant="ghost"
          size="sm"
          onclick={minimizePane}
          class="h-8 w-8 p-0"
        >
          <Minimize2 class="w-4 h-4" />
        </Button>
      </div>
      
      {#each panes as pane (pane.id)}
        {#if pane.id === maximizedPaneId}
          {@const Component = pane.component}
          <Component {...pane.props} />
        {/if}
      {/each}
    </div>
  {:else}
    <!-- Split view -->
    {#each panes as pane, index (pane.id)}
      <ResizablePane
        direction={direction}
        initialSize={pane.size}
        minSize={10}
        maxSize={90}
        onResize={handleResize}
      >
        <div class="relative h-full w-full">
          <!-- Pane Controls -->
          <div class="absolute top-2 right-2 z-10 flex items-center space-x-1 opacity-0 hover:opacity-100 transition-opacity">
            <Button
              variant="ghost"
              size="sm"
              onclick={() => splitPane(pane.id, direction === 'horizontal' ? 'vertical' : 'horizontal')}
              class="h-6 w-6 p-0"
              title="Split pane"
            >
              <Split class="w-3 h-3" />
            </Button>
            
            <Button
              variant="ghost"
              size="sm"
              onclick={() => toggleMaximize(pane.id)}
              class="h-6 w-6 p-0"
              title="Maximize pane"
            >
              <Maximize2 class="w-3 h-3" />
            </Button>
            
            {#if panes.length > 1}
              <Button
                variant="ghost"
                size="sm"
                onclick={() => closePane(pane.id)}
                class="h-6 w-6 p-0 hover:bg-red-500/20 hover:text-red-400"
                title="Close pane"
              >
                <X class="w-3 h-3" />
              </Button>
            {/if}
          </div>
          
          <!-- Pane Content -->
          {@const Component = pane.component}
          <Component {...pane.props} />
        </div>
      </ResizablePane>
    {/each}
  {/if}
</div>

<style>
  .split-pane-container {
    position: relative;
  }
</style>
