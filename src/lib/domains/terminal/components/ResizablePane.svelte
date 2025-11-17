<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { Snippet } from 'svelte';
  
  // Get children snippet from props for Svelte 5
  let { 
    children,
    direction = 'horizontal',
    initialSize = 50,
    minSize = 10,
    maxSize = 90,
    resizable = true,
    onResize
  }: { 
    children: Snippet<[]>;
    direction?: 'horizontal' | 'vertical';
    initialSize?: number;
    minSize?: number;
    maxSize?: number;
    resizable?: boolean;
    onResize?: (event: { size: number }) => void;
  } = $props();
  
  let container: HTMLDivElement;
  let isResizing = $state(false);
  let startPosition = 0;
  let startSize = 0;
  let currentSize = initialSize;
  
  function handleMouseDown(event: MouseEvent) {
    if (!resizable) return;
    
    event.preventDefault();
    isResizing = true;
    startPosition = direction === 'horizontal' ? event.clientX : event.clientY;
    startSize = currentSize;
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize';
    document.body.style.userSelect = 'none';
  }
  
  function handleMouseMove(event: MouseEvent) {
    if (!isResizing || !container) return;
    
    const containerRect = container.getBoundingClientRect();
    const currentPosition = direction === 'horizontal' ? event.clientX : event.clientY;
    const containerSize = direction === 'horizontal' ? containerRect.width : containerRect.height;
    
    const delta = currentPosition - startPosition;
    const deltaPercentage = (delta / containerSize) * 100;
    
    const newSize = Math.max(minSize, Math.min(maxSize, startSize + deltaPercentage));
    currentSize = newSize;
    
    if (onResize) onResize({ size: newSize });
  }
  
  function handleMouseUp() {
    isResizing = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }
  
  function handleKeyDown(event: KeyboardEvent) {
    if (!resizable || !container) return;
    
    const step = 1; // 1% per keypress
    let newSize = currentSize;
    
    if (direction === 'horizontal') {
      if (event.key === 'ArrowLeft') {
        event.preventDefault();
        newSize = Math.max(minSize, currentSize - step);
      } else if (event.key === 'ArrowRight') {
        event.preventDefault();
        newSize = Math.min(maxSize, currentSize + step);
      }
    } else {
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        newSize = Math.max(minSize, currentSize - step);
      } else if (event.key === 'ArrowDown') {
        event.preventDefault();
        newSize = Math.min(maxSize, currentSize + step);
      }
    }
    
    if (newSize !== currentSize) {
      currentSize = newSize;
      if (onResize) onResize({ size: newSize });
    }
  }
  
  onDestroy(() => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  });
  
  let sizeStyle = $derived(direction === 'horizontal' 
    ? `width: ${currentSize}%` 
    : `height: ${currentSize}%`);
</script>

<div 
  bind:this={container}
  class="resizable-pane relative {direction === 'horizontal' ? 'flex' : 'flex flex-col'}"
  style={sizeStyle}
>
  {@render children()}
  
  {#if resizable}
    <div
      class="resize-handle absolute {direction === 'horizontal' ? 'right-0 top-0 w-1 h-full cursor-col-resize hover:bg-blue-400' : 'bottom-0 left-0 h-1 w-full cursor-row-resize hover:bg-blue-400'} {isResizing ? 'bg-blue-400' : 'bg-transparent'} transition-colors"
      onmousedown={handleMouseDown}
      onkeydown={handleKeyDown}
      role="separator"
      aria-orientation={direction}
      aria-label={direction === 'horizontal' ? 'Resize pane width' : 'Resize pane height'}
      tabindex="0"
    ></div>
  {/if}
</div>

<style>
  .resize-handle {
    transition: background-color 0.2s ease;
  }
  
  .resize-handle:focus {
    outline: 2px solid rgba(59, 130, 246, 0.5);
    outline-offset: -1px;
  }
</style>
