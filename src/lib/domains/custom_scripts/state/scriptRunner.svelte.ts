import { onDestroy } from 'svelte';
import {
  CustomScriptService,
  type CustomScript,
} from '$lib/domains/custom_scripts/services/customScriptService';
import { TerminalService, type TerminalProcess, type TerminalOutput } from '$lib/domains/terminal';
import { useRunningScripts } from '$lib/domains/custom_scripts/hooks/useRunningScripts';

export interface ScriptRunnerProps {
  script: CustomScript;
  runningInstanceId?: string; // ID of this running instance (if viewing existing)
  onRun?: () => void;
  onStopped?: () => void;
}

/**
 * Run/stop/output state for ScriptRunner. Construct during component init: it
 * registers $effects and onDestroy. Pass a getter so props stay reactive.
 */
export class ScriptRunnerState {
  private runningScripts = useRunningScripts();
  private outputUnsubscribe: (() => void) | null = null;

  runningInstanceId = $state<string | null>(null);
  parameterValues = $state<Record<string, string>>({});
  booleanValues = $state<Record<string, boolean>>({});
  terminalProcess = $state<TerminalProcess | null>(null);
  terminalOutput = $state<string>('');
  isRunning = $state(false);
  error = $state<string | null>(null);

  parameters = $derived.by(() =>
    CustomScriptService.parseParameters(this.props().script.parameters_json)
  );

  // Boolean params are tracked separately and merged in as 'true'/'false'.
  command = $derived.by(() => {
    const { script } = this.props();
    const allValues = { ...this.parameterValues };
    for (const param of this.parameters) {
      if (param.parameter_type === 'boolean') {
        allValues[param.name] = this.booleanValues[param.name] ? 'true' : 'false';
      }
    }
    return CustomScriptService.buildCommand(
      script.command,
      this.parameters,
      allValues,
      script.requires_sudo
    );
  });

  constructor(private props: () => ScriptRunnerProps) {
    this.runningInstanceId = props().runningInstanceId || null;

    // If viewing an existing running instance, load its state and subscribe to output
    $effect(() => {
      const providedInstanceId = this.props().runningInstanceId;
      if (!providedInstanceId) return;
      const instance = this.runningScripts.getById(providedInstanceId);
      if (!instance) return;
      this.terminalOutput = instance.output;
      this.isRunning = true;
      this.runningInstanceId = providedInstanceId;

      // Try to get the process
      TerminalService.getProcess(instance.processId)
        .then((process) => {
          if (process) {
            this.terminalProcess = process;
          }
        })
        .catch(console.error);

      // Subscribe to output updates if not already subscribed
      if (!instance.outputUnsubscribe) {
        TerminalService.subscribeToOutput(instance.processId, (output: TerminalOutput) => {
          this.terminalOutput += output.content;
          this.runningScripts.appendOutput(instance.id, output.content);
        })
          .then((unsub) => {
            this.outputUnsubscribe = unsub;
            const updated = this.runningScripts.getById(instance.id);
            if (updated) {
              updated.outputUnsubscribe = unsub;
            }
          })
          .catch(console.error);
      }
    });

    // Initialize default values
    $effect(() => {
      const defaults: Record<string, string> = {};
      const boolDefaults: Record<string, boolean> = {};
      for (const param of this.parameters) {
        if (param.parameter_type === 'boolean') {
          boolDefaults[param.name] = param.default_value === 'true';
        } else {
          // Initialize all non-boolean parameters with default value or empty string
          defaults[param.name] = param.default_value || '';
        }
      }
      this.parameterValues = defaults;
      this.booleanValues = boolDefaults;
    });

    // Sync output from running scripts manager when viewing existing instance
    $effect(() => {
      if (this.runningInstanceId && this.props().runningInstanceId) {
        const instance = this.runningScripts.getById(this.runningInstanceId);
        if (instance) {
          this.terminalOutput = instance.output;
        }
      }
    });

    // Handle process exit
    $effect(() => {
      if (!this.terminalProcess) return;
      const checkExit = setInterval(async () => {
        if (this.terminalProcess) {
          const process = await TerminalService.getProcess(this.terminalProcess.id);
          if (process && process.status === 'completed') {
            clearInterval(checkExit);
            this.isRunning = false;
            this.releaseOutput();
            this.removeInstance();
            this.props().onStopped?.();
          }
        }
      }, 1000);

      return () => clearInterval(checkExit);
    });

    // Closing the dialog must not remove running instances from the manager or
    // kill the process — they persist until the script stops (handleStop or
    // process completion). Only drop the local subscription when viewing an
    // existing instance; a new run's unsubscribe lives on the manager's instance.
    onDestroy(() => {
      if (this.outputUnsubscribe && this.props().runningInstanceId) {
        this.outputUnsubscribe();
      }
    });
  }

  private releaseOutput() {
    if (this.outputUnsubscribe) {
      this.outputUnsubscribe();
      this.outputUnsubscribe = null;
    }
  }

  private removeInstance() {
    if (this.runningInstanceId) {
      this.runningScripts.remove(this.runningInstanceId);
      this.runningInstanceId = null;
    }
  }

  handleRun = async () => {
    const { script, onRun } = this.props();
    // Validate required parameters (boolean params are always valid)
    for (const param of this.parameters) {
      if (param.required && param.parameter_type !== 'boolean') {
        if (!this.parameterValues[param.name]?.trim()) {
          this.error = `Parameter "${param.label}" is required`;
          return;
        }
      }
    }

    this.error = null;
    this.isRunning = true;
    this.terminalOutput = '';

    try {
      await CustomScriptService.recordScriptRun(script.id);

      const tabId = `script-${script.id}-${Date.now()}`;
      const process = await TerminalService.createProcess(tabId, {
        shell: 'bash',
        working_directory: '/home/tan',
      });
      this.terminalProcess = process;

      // Register this running instance first (before subscribing to output)
      const instanceId = this.runningScripts.add(script, process.id, tabId, this.handleStop);
      this.runningInstanceId = instanceId;

      this.outputUnsubscribe = await TerminalService.subscribeToOutput(
        process.id,
        (output: TerminalOutput) => {
          this.terminalOutput += output.content;
          if (this.runningInstanceId) {
            this.runningScripts.appendOutput(this.runningInstanceId, output.content);
          }
        }
      );

      const instance = this.runningScripts.getById(instanceId);
      if (instance) {
        instance.outputUnsubscribe = this.outputUnsubscribe;
      }

      TerminalService.startCommandTracking(this.command, tabId);

      // sudo/admin is already included in the command if requires_sudo is true
      await TerminalService.sendInput(process.id, `${this.command}\n`);

      onRun?.();
    } catch (err) {
      this.error = err instanceof Error ? err.message : 'Failed to run script';
      console.error('Failed to run script:', err);
      this.isRunning = false;
      this.removeInstance();
    }
  };

  handleStop = async () => {
    if (this.terminalProcess) {
      try {
        await TerminalService.killProcess(this.terminalProcess.id);
      } catch (err) {
        console.error('Failed to kill process:', err);
      }
    }
    this.releaseOutput();
    this.isRunning = false;
    this.terminalProcess = null;
    this.removeInstance();
    this.props().onStopped?.();
  };

  sendInput = (input: string) => {
    if (this.terminalProcess && this.isRunning) {
      TerminalService.sendInput(this.terminalProcess.id, input).catch(console.error);
    }
  };
}
