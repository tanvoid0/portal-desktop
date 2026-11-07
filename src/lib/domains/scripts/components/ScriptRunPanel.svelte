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
  import { Badge } from '$lib/components/ui/badge';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Switch } from '$lib/components/ui/switch';
  import Select from '$lib/components/ui/select.svelte';
  import Copy from '~icons/fa6-solid/copy';
  import Play from '~icons/fa6-solid/play';
  import Square from '~icons/fa6-solid/square';
  import Terminal from '~icons/fa6-solid/terminal';
  import type { Block } from '$lib/domains/projects/pipelines';
  import { automation } from '$lib/domains/automation';
  import { scriptExecutionService, type ScriptExecutionInfo } from '../services';
  import EmbeddedTerminal from './EmbeddedTerminal.svelte';
  import { getStatusIcon, getStatusColor, formatDuration } from '../utils/executionStatus';
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

  function handleCopyOutput() {
    if (!currentExecution?.output) return;
    navigator.clipboard.writeText(currentExecution.output);
    toast.success('Output copied to clipboard');
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
      <div class="space-y-3">
        <Label class="text-sm font-medium">Parameters</Label>
        {#each script.parameters as param}
          <div class="grid grid-cols-3 items-start gap-4">
            <div class="flex items-center gap-2">
              <code class="font-mono text-sm">{param.name}</code>
              {#if param.required}
                <span class="text-red-500">*</span>
              {/if}
            </div>
            <div class="col-span-2">
              {#if param.type === 'select' && param.options}
                <Select
                  options={param.options.map((o) => ({
                    value: o,
                    label: o,
                  }))}
                  bind:value={parameterValues[param.name]}
                  placeholder={param.description || `Select ${param.name}`}
                />
              {:else if param.type === 'boolean'}
                <div class="flex items-center gap-2">
                  <Checkbox
                    aria-label={param.name}
                    checked={parameterValues[param.name] === 'true'}
                    onCheckedChange={(checked) => {
                      parameterValues[param.name] = checked === true ? 'true' : 'false';
                    }}
                    id={`param-${param.name}`}
                  />
                  <Label for={`param-${param.name}`} class="text-sm text-muted-foreground">
                    {param.description}
                  </Label>
                </div>
              {:else}
                <Input
                  bind:value={parameterValues[param.name]}
                  placeholder={param.description || param.name}
                />
              {/if}
            </div>
          </div>
        {/each}
      </div>
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
      {@const StatusIcon = getStatusIcon(currentExecution.status)}
      <div class="divider-edge-t divider-edge-full space-y-3 pt-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Terminal class="h-4 w-4" />
            <span class="font-medium">Output</span>
            {#if currentExecution.output}
              <Button variant="ghost" size="sm" onclick={handleCopyOutput} class="h-6 px-2">
                <Copy class="mr-1 h-3 w-3" />
                Copy
              </Button>
            {/if}
          </div>
          <div class="flex items-center gap-2">
            <StatusIcon class="h-4 w-4 {getStatusColor(currentExecution.status)}" />
            <Badge
              variant={currentExecution.status === 'success'
                ? 'default'
                : currentExecution.status === 'failed'
                  ? 'destructive'
                  : 'secondary'}
            >
              {currentExecution.status}
            </Badge>
            {#if currentExecution.finishedAt}
              <span class="text-xs text-muted-foreground">
                {formatDuration(currentExecution.startedAt, currentExecution.finishedAt)}
              </span>
            {/if}
          </div>
        </div>
        <div
          class="max-h-80 overflow-auto rounded-lg bg-black p-4 font-mono text-sm text-green-400"
        >
          {#if currentExecution.output}
            <pre class="whitespace-pre-wrap">{currentExecution.output}</pre>
          {:else if currentExecution.status === 'running'}
            <span class="animate-pulse">Waiting for output...</span>
          {:else}
            <span class="text-gray-500">No output</span>
          {/if}
        </div>
        {#if currentExecution.error}
          <div class="rounded-lg border border-red-500/30 bg-red-500/10 p-4">
            <p class="text-sm font-medium text-red-500">Error</p>
            <p class="mt-1 text-sm text-red-400">
              {currentExecution.error}
            </p>
          </div>
        {/if}
        {#if currentExecution.exitCode !== null}
          <p class="text-sm text-muted-foreground">
            Exit code: <code class="rounded bg-muted px-1">{currentExecution.exitCode}</code>
          </p>
        {/if}
      </div>
    {/if}
  </CardContent>
</Card>
