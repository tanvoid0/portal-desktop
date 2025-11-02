<script lang="ts">
  import { Badge } from '@/lib/components/ui/badge';
  import { Button } from '@/lib/components/ui/button';
  import { AlertTriangle, Info, AlertCircle, X } from 'lucide-svelte';

  export let errorCount: number = 0;
  export let warningCount: number = 0;
  export let infoCount: number = 0;
  export let errors: string[] = [];
  export let onClear: () => void = () => {};

  let isExpanded = false;

  function toggleExpanded() {
    isExpanded = !isExpanded;
  }

  function getSeverityColor(severity: string): string {
    switch (severity) {
      case 'error': return 'text-red-400';
      case 'warning': return 'text-yellow-400';
      case 'info': return 'text-blue-400';
      default: return 'text-gray-400';
    }
  }

  function getSeverityIcon(severity: string) {
    switch (severity) {
      case 'error': return AlertCircle;
      case 'warning': return AlertTriangle;
      case 'info': return Info;
      default: return Info;
    }
  }
</script>

<div class="error-summary border-b border-gray-700">
  <!-- Header -->
  <div class="flex items-center justify-between p-3 border-b border-gray-700 bg-gray-750">
    <div class="flex items-center space-x-2">
      <h2 class="text-sm font-medium text-gray-300">Error Summary</h2>
      {#if errorCount > 0 || warningCount > 0 || infoCount > 0}
        <div class="flex items-center space-x-1">
          {#if errorCount > 0}
            <Badge variant="destructive" class="text-xs">
              {errorCount} error{errorCount !== 1 ? 's' : ''}
            </Badge>
          {/if}
          {#if warningCount > 0}
            <Badge variant="secondary" class="text-xs bg-yellow-500/10 text-yellow-400 border-yellow-500/20">
              {warningCount} warning{warningCount !== 1 ? 's' : ''}
            </Badge>
          {/if}
          {#if infoCount > 0}
            <Badge variant="outline" class="text-xs bg-blue-500/10 text-blue-400 border-blue-500/20">
              {infoCount} info
            </Badge>
          {/if}
        </div>
      {/if}
    </div>
    
    <div class="flex items-center gap-2">
      {#if errorCount > 0 || warningCount > 0 || infoCount > 0}
        <Button
          variant="ghost"
          size="sm"
          onclick={toggleExpanded}
          class="h-6 w-6 p-0"
        >
          {#if isExpanded}
            <X class="w-4 h-4" />
          {:else}
            <AlertCircle class="w-4 h-4" />
          {/if}
        </Button>
        
        <Button
          variant="ghost"
          size="sm"
          onclick={onClear}
          class="h-6 w-6 p-0 text-gray-400 hover:text-gray-200"
        >
          <X class="w-4 h-4" />
        </Button>
      {/if}
    </div>
  </div>

  <!-- Error Details -->
  {#if isExpanded && (errorCount > 0 || warningCount > 0 || infoCount > 0)}
    <div class="p-3 space-y-2 max-h-48 overflow-y-auto">
      {#if errors.length > 0}
        <div class="space-y-1">
          <h3 class="text-xs font-medium text-gray-400 mb-2">Recent Errors:</h3>
          {#each errors as error, index (index)}
            <div class="text-xs text-red-400 bg-red-500/10 p-2 rounded border border-red-500/20">
              {error}
            </div>
          {/each}
        </div>
      {:else}
        <div class="text-xs text-gray-500 text-center py-2">
          No specific error details available
        </div>
      {/if}
    </div>
  {:else if errorCount === 0 && warningCount === 0 && infoCount === 0}
    <div class="p-6 text-center text-gray-500">
      <div class="text-2xl mb-2">✅</div>
      <div class="text-sm">No errors detected</div>
      <div class="text-xs mt-1">Terminal output looks clean</div>
    </div>
  {/if}
</div>

<style>
  .error-summary {
    background: rgba(55, 65, 81, 0.5);
  }
</style>
