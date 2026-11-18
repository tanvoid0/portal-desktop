<!--
	Run Script panel — working directory, interactive toggle, parameter values,
	the embedded terminal and the non-interactive execution output.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import Play from '~icons/fa6-solid/play';
  import Square from '~icons/fa6-solid/square';
  import Terminal from '~icons/fa6-solid/terminal';
  import type { Block } from '$lib/domains/projects/pipelines';
  import { automation } from '$lib/domains/automation';
  import { scriptExecutionService, type ScriptExecutionInfo } from '../services';
  import EmbeddedTerminal from './EmbeddedTerminal.svelte';
  import ScriptParamFields from './ScriptParamFields.svelte';
  import ScriptExecutionOutput from './ScriptExecutionOutput.svelte';
  import { toast } from '$lib/utils/toast';

  interface Props {
    script: Block;
    /** Bound so the history list can push a past execution into the output view. */
    currentExecution?: ScriptExecutionInfo | null;
    /** Fired when a run finishes, so the caller can refresh history. */
    onExecuted?: () => void;
  }

  let { script, currentExecution = $bindable(null), onExecuted }: Props = $props();

  let executing = $state(false);
  let interactiveMode = $state(true); // Default to interactive mode
  let showInteractiveTerminal = $state(false);
  let resolvedCommand = $state('');
  let workingDirectory = $state('');
  let interactiveStartedAt: string | undefined;
  let unsubscribe: (() => void) | null = null;

  // Parameter values for execution, seeded from defaults
  let parameterValues = $state<Record<string, string>>(seedParameterValues(script));

  function seedParameterValues(block: Block): Record<string, string> {
    const values: Record<string, string> = {};
    block.parameters.forEach((param) => {
      if (param.defaultValue !== undefined && param.defaultValue !== '') {
        values[param.name] = String(param.defaultValue);
      } else if (block.defaultConfig?.[param.name] !== undefined) {
        values[param.name] = String(block.defaultConfig[param.name]);
      } else {
        values[param.name] = '';
      }
    });
    return values;
  }

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });

  async function handleRun() {
    // Validate required parameters
    const missingParams = script.parameters
      .filter(
        (p) => p.required && (!parameterValues[p.name] || parameterValues[p.name].trim() === '')
      )
      .map((p) => p.name);

    if (missingParams.length > 0) {
      toast.error(`Missing required parameters: ${missingParams.join(', ')}`);
      return;
    }

    // Resolve command with parameters via shared block resolver
    try {
      const cwd = workingDirectory.trim() || '.';
      resolvedCommand = await automation.in(cwd).command(script.id, parameterValues);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to resolve command');
      return;
    }

    if (interactiveMode) {
      // Interactive mode - use embedded terminal
      interactiveStartedAt = new Date().toISOString();
      showInteractiveTerminal = true;
      executing = true;
      toast.success('Starting interactive terminal...');
      return;
    }

    // Non-interactive mode - use existing script execution service
    executing = true;
    try {
      const executionId = await scriptExecutionService.executeScript({
        blockId: script.id,
        command: script.command,
        parameters: parameterValues,
        workingDirectory: workingDirectory || undefined,
      });

      toast.success('Script execution started');

      // Subscribe to execution updates
      unsubscribe = scriptExecutionService.subscribeToExecution(
        executionId,
        (execution) => {
          currentExecution = execution;

          // Show completion toast
          if (execution.status === 'success') {
            toast.success('Script completed successfully');
            executing = false;
            onExecuted?.();
          } else if (execution.status === 'failed') {
            toast.error(`Script failed: ${execution.error || 'Unknown error'}`);
            executing = false;
            onExecuted?.();
          } else if (execution.status === 'cancelled') {
            toast.info('Script execution cancelled');
            executing = false;
            onExecuted?.();
          }
        },
        500
      );

      // Get initial execution state
      currentExecution = await scriptExecutionService.getExecution(executionId);
    } catch (error) {
      console.error('Failed to execute script', error);
      toast.error('Failed to start script execution');
      executing = false;
    }
  }

  async function handleCancel() {
    if (!currentExecution) return;

    try {
      await scriptExecutionService.cancelExecution(currentExecution.id);
      toast.info('Cancelling execution...');
    } catch (error) {
      console.error('Failed to cancel execution', error);
      toast.error('Failed to cancel execution');
    }
  }

  async function handleInteractiveComplete(exitCode: number | null) {
    executing = false;
    if (exitCode === 0) {
      toast.success('Script completed successfully');
    } else if (exitCode !== null) {
      toast.error(`Script exited with code ${exitCode}`);
    }
    try {
      await scriptExecutionService.recordInteractive({
        blockId: script.id,
        command: resolvedCommand,
        workingDirectory: workingDirectory || undefined,
        exitCode,
        startedAt: interactiveStartedAt,
      });
    } catch (e) {
      console.error('Failed to record interactive run', e);
    }
    onExecuted?.();
  }

  function handleInteractiveStart(processId: string) {
    console.log('Interactive terminal started with process:', processId);
  }

  function closeInteractiveTerminal() {
    showInteractiveTerminal = false;
    executing = false;
  }
</script>

<Card class="border-primary/50">
  <CardHeader>
    <div class="flex items-center justify-between">
      <div>
        <CardTitle class="flex items-center gap-2">
          <Play class="h-5 w-5" />
          Run Script
        </CardTitle>
        <CardDescription>Configure parameters and execute</CardDescription>
      </div>
      {#if executing}
        <Button variant="destructive" onclick={handleCancel}>
          <Square class="mr-2 h-4 w-4" />
          Stop
        </Button>
      {:else}
        <Button onclick={handleRun}>
          <Play class="mr-2 h-4 w-4" />
          Run
        </Button>
      {/if}
    </div>
  </CardHeader>
  <CardContent class="space-y-4">
    <!-- Working Directory -->
    <div>
      <Label for="workingDir">Working Directory (optional)</Label>
      <Input
        id="workingDir"
        bind:value={workingDirectory}
        placeholder="Leave empty to use current directory"
        class="mt-1"
      />
    </div>

    <!-- Interactive Mode Toggle -->
    <div class="flex items-center justify-between rounded-lg bg-muted/50 p-3">
      <div>
        <Label for="interactive-mode" class="text-sm font-medium">Interactive Mode</Label>
        <p class="mt-0.5 text-xs text-muted-foreground">
          Enable for scripts that require user input (passwords, confirmations, etc.)
        </p>
      </div>
      <Switch id="interactive-mode" bind:checked={interactiveMode} />
    </div>

    <!-- Parameters -->
    {#if script.parameters.length > 0}
      <ScriptParamFields parameters={script.parameters} bind:values={parameterValues} />
    {/if}

    <!-- Interactive Terminal -->
    {#if showInteractiveTerminal && resolvedCommand}
      <div class="divider-edge-t divider-edge-full space-y-3 pt-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Terminal class="h-4 w-4" />
            <span class="font-medium">Interactive Terminal</span>
          </div>
          <Button variant="outline" size="sm" onclick={closeInteractiveTerminal}>
            Close Terminal
          </Button>
        </div>
        <EmbeddedTerminal
          command={resolvedCommand}
          workingDirectory={workingDirectory || undefined}
          onComplete={handleInteractiveComplete}
          onStart={handleInteractiveStart}
        />
      </div>
    {/if}

    <!-- Execution Output (non-interactive mode) -->
    {#if currentExecution && !showInteractiveTerminal}
      <ScriptExecutionOutput execution={currentExecution} />
    {/if}
  </CardContent>
</Card>
