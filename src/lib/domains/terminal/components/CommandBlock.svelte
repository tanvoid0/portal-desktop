<script lang="ts">
  import { Badge } from '@/lib/components/ui/badge';
  import { Button } from '@/lib/components/ui/button';
  import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/lib/components/ui/collapsible';
  import { ChevronDown, ChevronRight, Clock, Terminal, CheckCircle, XCircle, AlertCircle } from 'lucide-svelte';
  
  export let block: {
    id: string;
    command: string;
    output: string;
    exitCode?: number;
    duration?: number;
    workingDirectory?: string;
    timestamp: string;
    isExpanded?: boolean;
  };
  export let onclick: (() => void) | undefined = undefined;

  let isExpanded = block.isExpanded ?? false;
  let isRunning = block.exitCode === undefined;

  // Format duration
  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  }

  // Get status icon and color
  function getStatusInfo() {
    if (isRunning) {
      return { icon: Terminal, color: 'text-blue-500', bgColor: 'bg-blue-50 dark:bg-blue-950' };
    }
    
    if (block.exitCode === 0) {
      return { icon: CheckCircle, color: 'text-green-500', bgColor: 'bg-green-50 dark:bg-green-950' };
    }
    
    if (block.exitCode === undefined) {
      return { icon: AlertCircle, color: 'text-yellow-500', bgColor: 'bg-yellow-50 dark:bg-yellow-950' };
    }
    
    return { icon: XCircle, color: 'text-red-500', bgColor: 'bg-red-50 dark:bg-red-950' };
  }

  const statusInfo = getStatusInfo();
  const StatusIcon = statusInfo.icon;
</script>

<div class="command-block border border-border rounded-lg bg-card" onclick={() => onclick && onclick()}>
  <!-- Command Header -->
  <div class="flex items-center justify-between p-3 border-b border-border">
    <div class="flex items-center gap-3 flex-1 min-w-0">
      <!-- Status Icon -->
      <div class="flex-shrink-0">
        <StatusIcon class="w-4 h-4 {statusInfo.color}" />
      </div>
      
      <!-- Command -->
      <div class="flex-1 min-w-0">
        <code class="text-sm font-mono text-foreground break-all">
          {block.command}
        </code>
      </div>
      
      <!-- Metadata -->
      <div class="flex items-center gap-2 flex-shrink-0">
        {#if block.duration !== undefined}
          <Badge variant="outline" class="text-xs">
            <Clock class="w-3 h-3 mr-1" />
            {formatDuration(block.duration)}
          </Badge>
        {/if}
        
        {#if block.exitCode !== undefined}
          <Badge 
            variant={block.exitCode === 0 ? 'default' : 'destructive'}
            class="text-xs"
          >
            Exit {block.exitCode}
          </Badge>
        {/if}
      </div>
    </div>
    
    <!-- Expand/Collapse Button -->
    <Collapsible bind:open={isExpanded}>
      <CollapsibleTrigger>
        <Button
          variant="ghost"
          size="sm"
          class="ml-2 h-8 w-8 p-0"
        >
          {#if isExpanded}
            <ChevronDown class="w-4 h-4" />
          {:else}
            <ChevronRight class="w-4 h-4" />
          {/if}
        </Button>
      </CollapsibleTrigger>
    </Collapsible>
  </div>

  <!-- Output Content -->
  <Collapsible bind:open={isExpanded}>
    <CollapsibleContent>
      <div class="p-3 bg-muted/30">
        {#if block.output}
          <pre class="text-sm font-mono text-foreground whitespace-pre-wrap break-words overflow-x-auto">{block.output}</pre>
        {:else}
          <p class="text-sm text-muted-foreground italic">No output</p>
        {/if}
      </div>
    </CollapsibleContent>
  </Collapsible>
</div>

<style>
  .command-block {
    transition: all 0.2s ease-in-out;
  }
  
  .command-block:hover {
    box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  }
</style>
