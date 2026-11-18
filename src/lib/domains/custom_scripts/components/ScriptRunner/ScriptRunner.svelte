<!--
	Script Runner Component
	Runs custom scripts with parameter inputs and terminal integration
-->

<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { onMount, onDestroy } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import X from '~icons/fa6-solid/xmark';
  import Play from '~icons/fa6-solid/play';
  import Square from '~icons/fa6-solid/square';
  import {
    CustomScriptService,
    type CustomScript,
  } from '$lib/domains/custom_scripts/services/customScriptService';
  import {
    TerminalService,
    type TerminalProcess,
    type TerminalOutput,
  } from '$lib/domains/terminal';
  import { useRunningScripts } from '$lib/domains/custom_scripts/hooks/useRunningScripts';
  import ScriptParameterForm from './ScriptParameterForm.svelte';
  import ScriptCommandPreview from './ScriptCommandPreview.svelte';
  import ScriptOutputCard from './ScriptOutputCard.svelte';

  interface Props {
    script: CustomScript;
    runningInstanceId?: string; // ID of this running instance (if viewing existing)
    onClose: () => void;
    onRun?: () => void;
    onStopped?: () => void;
  }

  let {
    script,
    runningInstanceId: providedInstanceId,
    onClose,
    onRun,
    onStopped,
  }: Props = $props();

  const runningScripts = useRunningScripts();
  let runningInstanceId = $state<string | null>(providedInstanceId || null);

  // Parse parameters
  const parameters = $derived(CustomScriptService.parseParameters(script.parameters_json));

  // Parameter values
  let parameterValues = $state<Record<string, string>>({});
  let booleanValues = $state<Record<string, boolean>>({});
  let terminalProcess = $state<TerminalProcess | null>(null);
  let terminalOutput = $state<string>('');
  let isRunning = $state(false);
  let error = $state<string | null>(null);

  // If viewing an existing running instance, load its state and subscribe to output
  $effect(() => {
    if (providedInstanceId) {
      const instance = runningScripts.getById(providedInstanceId);
      if (instance) {
        terminalOutput = instance.output;
        isRunning = true;
        runningInstanceId = providedInstanceId;

        // Try to get the process
        TerminalService.getProcess(instance.processId)
          .then((process) => {
            if (process) {
              terminalProcess = process;
            }
          })
          .catch(console.error);

        // Subscribe to output updates if not already subscribed
        if (!instance.outputUnsubscribe) {
          TerminalService.subscribeToOutput(instance.processId, (output: TerminalOutput) => {
            terminalOutput += output.content;
            runningScripts.appendOutput(instance.id, output.content);
          })
            .then((unsub) => {
              outputUnsubscribe = unsub;
              const updated = runningScripts.getById(instance.id);
              if (updated) {
                updated.outputUnsubscribe = unsub;
              }
            })
            .catch(console.error);
        }
      }
    }
  });

  // Initialize default values
  $effect(() => {
    const defaults: Record<string, string> = {};
    const boolDefaults: Record<string, boolean> = {};
    for (const param of parameters) {
      if (param.parameter_type === 'boolean') {
        boolDefaults[param.name] = param.default_value === 'true';
      } else {
        // Initialize all non-boolean parameters with default value or empty string
        defaults[param.name] = param.default_value || '';
      }
    }
    parameterValues = defaults;
    booleanValues = boolDefaults;
  });

  // Terminal output handler
  let outputUnsubscribe: (() => void) | null = null;

  // Sync output from running scripts manager when viewing existing instance
  $effect(() => {
    if (runningInstanceId && providedInstanceId) {
      const instance = runningScripts.getById(runningInstanceId);
      if (instance) {
        terminalOutput = instance.output;
      }
    }
  });

  // Boolean params are tracked separately and merged in as 'true'/'false'.
  const command = $derived.by(() => {
    const allValues = { ...parameterValues };
    for (const param of parameters) {
      if (param.parameter_type === 'boolean') {
        allValues[param.name] = booleanValues[param.name] ? 'true' : 'false';
      }
    }
    return CustomScriptService.buildCommand(
      script.command,
      parameters,
      allValues,
      script.requires_sudo
    );
  });

  async function handleRun() {
    // Validate required parameters
    for (const param of parameters) {
      if (param.required) {
        if (param.parameter_type === 'boolean') {
          // Boolean params are always valid
          continue;
        }
        if (!parameterValues[param.name]?.trim()) {
          error = `Parameter "${param.label}" is required`;
          return;
        }
      }
    }

    error = null;
    isRunning = true;
    terminalOutput = '';

    try {
      // Record script run
      await CustomScriptService.recordScriptRun(script.id);

      // Create terminal process
      const tabId = `script-${script.id}-${Date.now()}`;
      terminalProcess = await TerminalService.createProcess(tabId, {
        shell: 'bash',
        working_directory: '/home/tan',
      });

      // Register this running instance first (before subscribing to output)
      runningInstanceId = runningScripts.add(script, terminalProcess.id, tabId, handleStop);

      // Subscribe to output
      outputUnsubscribe = await TerminalService.subscribeToOutput(
        terminalProcess.id,
        (output: TerminalOutput) => {
          terminalOutput += output.content;
          // Update output in running scripts manager
          if (runningInstanceId) {
            runningScripts.appendOutput(runningInstanceId, output.content);
          }
        }
      );

      // Update the instance with output unsubscribe
      const instance = runningScripts.getById(runningInstanceId);
      if (instance) {
        instance.outputUnsubscribe = outputUnsubscribe;
      }

      // Start tracking the command
      TerminalService.startCommandTracking(command, tabId);

      // Execute the command (sudo/admin is already included in the command if requires_sudo is true)
      await TerminalService.sendInput(terminalProcess.id, `${command}\n`);

      onRun?.();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to run script';
      console.error('Failed to run script:', err);
      isRunning = false;
      if (runningInstanceId) {
        runningScripts.remove(runningInstanceId);
        runningInstanceId = null;
      }
    }
  }

  async function handleStop() {
    if (terminalProcess) {
      try {
        await TerminalService.killProcess(terminalProcess.id);
      } catch (err) {
        console.error('Failed to kill process:', err);
      }
    }
    if (outputUnsubscribe) {
      outputUnsubscribe();
      outputUnsubscribe = null;
    }
    isRunning = false;
    terminalProcess = null;

    // Remove from running scripts manager
    if (runningInstanceId) {
      runningScripts.remove(runningInstanceId);
      runningInstanceId = null;
    }

    onStopped?.();
  }

  // Handle process exit
  $effect(() => {
    if (terminalProcess) {
      // Listen for exit events
      const checkExit = setInterval(async () => {
        if (terminalProcess) {
          const process = await TerminalService.getProcess(terminalProcess.id);
          if (process && process.status === 'completed') {
            clearInterval(checkExit);
            isRunning = false;
            if (outputUnsubscribe) {
              outputUnsubscribe();
              outputUnsubscribe = null;
            }

            // Remove from running scripts manager
            if (runningInstanceId) {
              runningScripts.remove(runningInstanceId);
              runningInstanceId = null;
            }

            onStopped?.();
          }
        }
      }, 1000);

      return () => clearInterval(checkExit);
    }
  });

  onDestroy(() => {
    // Don't remove running instances from the manager when closing the dialog
    // The instances should persist even when the dialog is closed
    // They will only be removed when the script actually stops (via handleStop or process completion)

    // Only clean up local subscription if we created one
    // But don't remove from manager - the script is still running!
    if (outputUnsubscribe && providedInstanceId) {
      // This is viewing an existing instance - just clean up local subscription
      // Don't remove from manager or kill process
      outputUnsubscribe();
    }
    // If it's a new run (no providedInstanceId), the outputUnsubscribe is already
    // stored in the instance, so we don't need to clean it up here
    // The instance will persist in the manager even after this component is destroyed
  });

  function sendInput(input: string) {
    if (terminalProcess && isRunning) {
      TerminalService.sendInput(terminalProcess.id, input).catch(console.error);
    }
  }
</script>

<Dialog.Root
  open={true}
  onOpenChange={(isOpen) => {
    if (!isOpen) onClose();
  }}
>
  <Dialog.Content class="flex max-h-[90vh] max-w-6xl flex-col overflow-hidden p-0">
    <div class="divider-edge-b divider-edge-full flex items-center justify-between p-6">
      <div>
        <Dialog.Title class="flex items-center gap-2 text-2xl">
          {#if script.icon}
            <span>{script.icon}</span>
          {/if}
          {script.name}
        </Dialog.Title>
        {#if script.description}
          <p class="mt-1 text-sm text-muted-foreground">{script.description}</p>
        {/if}
      </div>
      <Button variant="ghost" size="sm" onclick={onClose}>
        <X class="h-4 w-4" />
      </Button>
    </div>

    <div class="flex-1 space-y-6 overflow-y-auto p-6">
      {#if error}
        <div class="rounded-md border border-red-200 bg-red-50 p-4">
          <p class="text-sm text-red-600">{error}</p>
        </div>
      {/if}

      {#if parameters.length > 0}
        <ScriptParameterForm
          {parameters}
          bind:values={parameterValues}
          bind:booleans={booleanValues}
        />
      {/if}

      <ScriptCommandPreview {command} requiresSudo={script.requires_sudo} />

      {#if isRunning || terminalOutput}
        <ScriptOutputCard output={terminalOutput} {isRunning} onSendInput={sendInput} />
      {/if}
    </div>

    <div class="divider-edge-t divider-edge-full flex justify-end gap-2 p-6">
      {#if isRunning}
        <Button variant="destructive" onclick={handleStop}>
          <Square class="mr-2 h-4 w-4" />
          Stop
        </Button>
      {:else}
        <Button variant="outline" onclick={onClose}>Close</Button>
        <Button onclick={handleRun}>
          <Play class="mr-2 h-4 w-4" />
          Run Script
        </Button>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
