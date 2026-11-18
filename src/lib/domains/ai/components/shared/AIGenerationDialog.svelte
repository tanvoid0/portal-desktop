<script module lang="ts">
  export type { AIErrorInfo } from './aiGenerationErrors';
  export type { AIGenerationConfig } from './aiGenerationState.svelte';
</script>

<script lang="ts">
  import FaCode from '~icons/fa6-solid/code';
  import FaWandMagicSparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import FaRotate from '~icons/fa6-solid/rotate';

  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import {
    AIGenerationDialogState,
    type AIGenerationConfig as Config,
  } from './aiGenerationState.svelte';
  import AIGenerationNotices from './AIGenerationNotices.svelte';
  import AIRegenerateDialog from './AIRegenerateDialog.svelte';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSuccess: (result: unknown, originalInput: string) => void;
    config: Config;
    /** Previous result for regeneration (optional) */
    previousResult?: unknown;
  }

  let { open, onOpenChange, onSuccess, config, previousResult }: Props = $props();

  const st = new AIGenerationDialogState(() => ({
    config,
    previousResult,
    onSuccess,
    onOpenChange,
  }));

  $effect(() => {
    if (!open) st.reset();
  });
</script>

<Dialog.Root bind:open {onOpenChange}>
  <Dialog.Content class="max-h-[90vh] max-w-3xl overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>{config.title}</Dialog.Title>
      <Dialog.Description>{config.description}</Dialog.Description>
    </Dialog.Header>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="ai-input">{config.inputLabel}</Label>
        <Textarea
          id="ai-input"
          bind:value={st.inputText}
          placeholder={config.inputPlaceholder}
          rows={config.inputRows ?? 12}
          class={st.error ? 'border-destructive' : ''}
          disabled={st.isGenerating}
        />
        {#if st.error}
          <p class="text-sm text-destructive">{st.error}</p>
        {/if}
      </div>

      {#if config.showDeveloperNote}
        <div class="space-y-2">
          <Label for="developer-note">
            <FaCode class="mr-1 inline h-4 w-4" />
            Developer Note (Optional)
          </Label>
          <Textarea
            id="developer-note"
            bind:value={st.developerNote}
            placeholder="Add developer-friendly instructions about the required work, technical considerations, or implementation details..."
            rows={4}
            disabled={st.isGenerating}
            class="font-mono text-sm"
          />
          <p class="text-xs text-muted-foreground">
            This will be included in the task description to guide developers on implementation
            approach.
          </p>
        </div>
      {/if}

      {#if config.showInstruction}
        <div class="space-y-2">
          <Label for="instruction">
            <FaWandMagicSparkles class="mr-1 inline h-4 w-4" />
            Generation Instructions (Optional)
          </Label>
          <Textarea
            id="instruction"
            bind:value={st.instruction}
            placeholder="e.g., Focus on API endpoints only, break into smaller subtasks, prioritize security features..."
            rows={3}
            disabled={st.isGenerating}
          />
          <p class="text-xs text-muted-foreground">
            Specify how you want the tasks to be generated (format, focus areas, structure, etc.).
          </p>
        </div>
      {/if}

      <AIGenerationNotices
        infoContent={config.infoContent}
        error={st.error}
        onOpenSettings={st.openSettings}
      />
    </div>

    <Dialog.Footer>
      {#if previousResult}
        <Button
          variant="outline"
          onclick={() => (st.showRegenerateDialog = true)}
          disabled={st.isGenerating}
        >
          <FaRotate class="mr-2 h-4 w-4" />
          Regenerate
        </Button>
      {/if}
      <Button variant="outline" onclick={st.cancel} disabled={st.isGenerating}>Cancel</Button>
      <Button onclick={st.generate} disabled={st.isGenerating || !st.inputText.trim()}>
        <FaWandMagicSparkles class="mr-2 h-4 w-4" />
        Generate
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

{#if previousResult}
  <AIRegenerateDialog
    bind:open={st.showRegenerateDialog}
    preview={st.resultPreview}
    bind:instructions={st.regenerateInstructions}
    error={st.error}
    isRegenerating={st.isRegenerating}
    onCancel={st.cancelRegenerate}
    onRegenerate={st.regenerate}
  />
{/if}
