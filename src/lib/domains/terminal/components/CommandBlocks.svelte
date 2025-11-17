<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import CommandBlock from './CommandBlock.svelte';
  import { Button } from '@/lib/components/ui/button';
  import { Badge } from '@/lib/components/ui/badge';
  import { Trash2, Maximize2, Minimize2 } from '@lucide/svelte';
  import type { ShellIntegrationEvent, CommandStartEvent, CommandOutputEvent, CommandEndEvent, CommandDetectedEvent, TerminalOutputEvent } from '../types';
  
  interface Props {
    processId: string;
  }

  let {
    processId
  }: Props = $props();
  
  let commandBlocks = $state<Array<{
    id: string;
    command: string;
    output: string;
    exitCode?: number;
    duration?: number;
    workingDirectory?: string;
    timestamp: string;
    isExpanded?: boolean;
  }>>([]);
  
  let isExpanded = $state(true);
  let unsubscribe = $state<(() => void) | null>(null);

  onMount(async () => {
    console.log('CommandBlocks mounted with processId:', processId);
    
    // Listen for shell integration events
    const shellIntegrationUnsubscribe = await listen('shell-integration-event', (event) => {
      console.log('Shell integration event:', event.payload);
      handleShellIntegrationEvent(event.payload as ShellIntegrationEvent);
    });
    
    // Also listen for terminal output to detect commands
    const terminalOutputUnsubscribe = await listen('terminal-output', (event) => {
      // console.log('Terminal output event received:', event.payload);
      // If processId is empty, listen to all events for debugging
      if (!processId || (event.payload as any).process_id === processId) {
        // console.log('Processing terminal output (processId:', processId, ')');
        handleTerminalOutput(event.payload as any);
      } else {
        // console.log('Process ID mismatch:', (event.payload as any).process_id, 'vs', processId);
      }
    });
    
    // Store both unsubscribe functions
    unsubscribe = () => {
      shellIntegrationUnsubscribe();
      terminalOutputUnsubscribe();
    };
    
    // Add a test command block for demonstration
    const testBlock = {
      id: 'test-cmd-1',
      command: 'ping google.com',
      output: 'PING google.com (142.250.191.14) 56(84) bytes of data.\n64 bytes from 142.250.191.14: icmp_seq=1 ttl=57 time=15.9 ms\n64 bytes from 142.250.191.14: icmp_seq=2 ttl=57 time=16.1 ms',
      exitCode: 0,
      duration: 2000,
      workingDirectory: '/home/tan',
      timestamp: new Date().toISOString(),
      isExpanded: true
    };
    commandBlocks = [testBlock];
    // console.log('Added test command block:', testBlock);
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });

  function handleShellIntegrationEvent(event: ShellIntegrationEvent) {
    console.log('Shell integration event received:', event);
    switch (event.type) {
      case 'CommandStart':
        handleCommandStart(event.payload as CommandStartEvent);
        break;
      case 'CommandOutput':
        handleCommandOutput(event.payload as CommandOutputEvent);
        break;
      case 'CommandEnd':
        handleCommandEnd(event.payload as CommandEndEvent);
        break;
      case 'CommandDetected':
        handleCommandDetected(event.payload as CommandDetectedEvent);
        break;
    }
  }

  function handleTerminalOutput(output: TerminalOutputEvent) {
    // console.log('Terminal output received for process:', output.process_id, 'content:', output.content);
    
    const content = output.content;
    
    // Look for new command patterns (prompt followed by command)
    if (content.includes('$ ') || content.includes('# ') || content.includes('> ')) {
      const lines = content.split('\n');
      for (const line of lines) {
        // Match prompt patterns: $ command, # command, > command
        const commandMatch = line.trim().match(/^[$#>]\s+(.+)/);
        if (commandMatch && commandMatch[1] && commandMatch[1].trim().length > 0) {
          const command = commandMatch[1].trim();
          // console.log('Found potential command:', command);
          // Allow echo commands for now to test
          if (command.length > 0) {
            // console.log('Detected command:', command);
            handleCommandDetected({ command });
          }
        }
      }
    }
    
    // Look for command completion patterns
    if (content.includes('$ ') || content.includes('# ') || content.includes('> ')) {
      // If we see a new prompt, the previous command likely completed
      if (commandBlocks.length > 0) {
        const currentBlock = commandBlocks[0];
        if (currentBlock.exitCode === undefined && currentBlock.command) {
          // Mark as completed with exit code 0 (success)
          currentBlock.exitCode = 0;
          currentBlock.duration = Date.now() - new Date(currentBlock.timestamp).getTime();
          commandBlocks = [...commandBlocks]; // Trigger reactivity
        }
      }
    }
    
    // Add output to current block if we have one
    if (commandBlocks.length > 0) {
      const currentBlock = commandBlocks[0];
      if (currentBlock.exitCode === undefined) { // Still running
        currentBlock.output += content;
        commandBlocks = [...commandBlocks]; // Trigger reactivity
      }
    }
  }

  function handleCommandStart(event: CommandStartEvent) {
    const newBlock = {
      id: event.id || `cmd-${Date.now()}`,
      command: event.command || '',
      output: '',
      exitCode: undefined,
      duration: undefined,
      workingDirectory: event.workingDirectory,
      timestamp: event.timestamp instanceof Date ? event.timestamp.toISOString() : event.timestamp,
      isExpanded: true
    };
    
    commandBlocks = [newBlock, ...commandBlocks];
  }

  function handleCommandOutput(event: CommandOutputEvent) {
    if (commandBlocks.length > 0) {
      const currentBlock = commandBlocks[0];
      if (currentBlock.id === event.process_id || currentBlock.id === 'current') {
        currentBlock.output += event.content;
        commandBlocks = [...commandBlocks]; // Trigger reactivity
      }
    }
  }

  function handleCommandEnd(event: CommandEndEvent) {
    if (commandBlocks.length > 0) {
      const currentBlock = commandBlocks[0];
      if (currentBlock.id === event.id || currentBlock.id === 'current') {
        currentBlock.exitCode = event.exitCode;
        currentBlock.duration = event.duration;
        commandBlocks = [...commandBlocks]; // Trigger reactivity
      }
    }
  }

  function handleCommandDetected(event: any) {
    console.log('Command detected:', event.command);
    
    // Create a new command block
    const newBlock = {
      id: `cmd-${Date.now()}`,
      command: event.command,
      output: '',
      exitCode: undefined,
      duration: undefined,
      workingDirectory: undefined,
      timestamp: new Date().toISOString(),
      isExpanded: true
    };
    
    // Add to the beginning of the list
    commandBlocks = [newBlock, ...commandBlocks];
  }

  function clearBlocks() {
    commandBlocks = [];
  }

  function addTestCommand() {
    const testBlock = {
      id: `test-cmd-${Date.now()}`,
      command: 'echo hello world',
      output: 'hello world',
      exitCode: 0,
      duration: 100,
      workingDirectory: '/home/tan',
      timestamp: new Date().toISOString(),
      isExpanded: true
    };
    commandBlocks = [testBlock, ...commandBlocks];
    console.log('Added test command:', testBlock);
  }

  function toggleExpanded() {
    isExpanded = !isExpanded;
  }

  function toggleBlockExpansion(blockId: string) {
    commandBlocks = commandBlocks.map(block => 
      block.id === blockId 
        ? { ...block, isExpanded: !block.isExpanded }
        : block
    );
  }

  // Get summary stats
  const totalBlocks = $derived(commandBlocks.length);
  const successfulBlocks = $derived(commandBlocks.filter(b => b.exitCode === 0).length);
  const failedBlocks = $derived(commandBlocks.filter(b => b.exitCode !== undefined && b.exitCode !== 0).length);
  const runningBlocks = $derived(commandBlocks.filter(b => b.exitCode === undefined).length);
</script>

<div class="command-blocks-container">
  <!-- Header -->
  <div class="flex items-center justify-between p-3 border-b border-border bg-muted/30">
    <div class="flex items-center gap-3">
      <h3 class="text-sm font-semibold">Command Blocks</h3>
      <div class="flex items-center gap-2">
        <Badge variant="outline" class="text-xs">
          {totalBlocks} total
        </Badge>
        {#if successfulBlocks > 0}
          <Badge variant="default" class="text-xs bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
            {successfulBlocks} success
          </Badge>
        {/if}
        {#if failedBlocks > 0}
          <Badge variant="destructive" class="text-xs">
            {failedBlocks} failed
          </Badge>
        {/if}
        {#if runningBlocks > 0}
          <Badge variant="secondary" class="text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
            {runningBlocks} running
          </Badge>
        {/if}
      </div>
    </div>
    
      <div class="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          onclick={addTestCommand}
          class="h-8 px-2 text-xs"
        >
          Test
        </Button>
        
        <Button
          variant="ghost"
          size="sm"
          onclick={toggleExpanded}
          class="h-8 w-8 p-0"
        >
          {#if isExpanded}
            <Minimize2 class="w-4 h-4" />
          {:else}
            <Maximize2 class="w-4 h-4" />
          {/if}
        </Button>
        
        <Button
          variant="ghost"
          size="sm"
          onclick={clearBlocks}
          class="h-8 w-8 p-0 text-destructive hover:text-destructive"
        >
          <Trash2 class="w-4 h-4" />
        </Button>
      </div>
  </div>

  <!-- Command Blocks List -->
  {#if isExpanded}
    <div class="p-3 space-y-3 max-h-96 overflow-y-auto">
      {#if commandBlocks.length === 0}
        <div class="text-center py-8 text-muted-foreground">
          <p class="text-sm">No command blocks yet</p>
          <p class="text-xs mt-1">Execute commands to see structured output</p>
        </div>
      {:else}
        {#each commandBlocks as block (block.id)}
          <CommandBlock 
            {block} 
            onclick={() => toggleBlockExpansion(block.id)}
          />
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .command-blocks-container {
    background-color: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 0.5rem;
  }
</style>
