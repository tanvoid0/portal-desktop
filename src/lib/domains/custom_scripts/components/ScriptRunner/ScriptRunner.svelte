<!--
	Script Runner Component
	Runs custom scripts with parameter inputs and terminal integration
-->

<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import X from '~icons/fa6-solid/xmark';
  import Play from '~icons/fa6-solid/play';
  import Square from '~icons/fa6-solid/square';
  import type { CustomScript } from '$lib/domains/custom_scripts/services/customScriptService';
  import { ScriptRunnerState } from '$lib/domains/custom_scripts/state/scriptRunner.svelte';
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

  let { script, runningInstanceId, onClose, onRun, onStopped }: Props = $props();

  const r = new ScriptRunnerState(() => ({ script, runningInstanceId, onRun, onStopped }));
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
      {#if r.error}
        <div class="rounded-md border border-red-200 bg-red-50 p-4">
          <p class="text-sm text-red-600">{r.error}</p>
        </div>
      {/if}

      {#if r.parameters.length > 0}
        <ScriptParameterForm
          parameters={r.parameters}
          bind:values={r.parameterValues}
          bind:booleans={r.booleanValues}
        />
      {/if}

      <ScriptCommandPreview command={r.command} requiresSudo={script.requires_sudo} />

      {#if r.isRunning || r.terminalOutput}
        <ScriptOutputCard
          output={r.terminalOutput}
          isRunning={r.isRunning}
          onSendInput={r.sendInput}
        />
      {/if}
    </div>

    <div class="divider-edge-t divider-edge-full flex justify-end gap-2 p-6">
      {#if r.isRunning}
        <Button variant="destructive" onclick={r.handleStop}>
          <Square class="mr-2 h-4 w-4" />
          Stop
        </Button>
      {:else}
        <Button variant="outline" onclick={onClose}>Close</Button>
        <Button onclick={r.handleRun}>
          <Play class="mr-2 h-4 w-4" />
          Run Script
        </Button>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
