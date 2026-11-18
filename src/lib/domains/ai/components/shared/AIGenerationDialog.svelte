<script module lang="ts">
  import type { AIErrorInfo as ErrorInfo } from './aiGenerationErrors';
  export type AIErrorInfo = ErrorInfo;
</script>

<script lang="ts">
  import FaCode from '~icons/fa6-solid/code';
  import FaWandMagicSparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import FaRotate from '~icons/fa6-solid/rotate';

  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import { toastActions } from '$lib/utils/toast';
  import { goto } from '$app/navigation';
  import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';
  import { defaultParseError } from './aiGenerationErrors';
  import AIGenerationNotices from './AIGenerationNotices.svelte';
  import AIRegenerateDialog from './AIRegenerateDialog.svelte';

  export interface AIGenerationConfig<TInput = string, TOutput = unknown> {
    /** Title for the dialog */
    title: string;
    /** Description for the dialog */
    description: string;
    /** Label for the input field */
    inputLabel: string;
    /** Placeholder for the input field */
    inputPlaceholder: string;
    /** Number of rows for textarea (default: 12) */
    inputRows?: number;
    /** Maximum input length (default: 10000) */
    maxLength?: number;
    /** Function to generate AI response */
    generateFn: (
      input: TInput,
      history?: Array<{ role: 'user' | 'assistant'; content: string }>,
      developerNote?: string,
      instruction?: string
    ) => Promise<TOutput>;
    /** Function to serialize result for display in regenerate dialog */
    serializeResult?: (result: TOutput) => string;
    /** Optional provider type to use (defaults to AgentPlatform) */
    providerType?: string;
    /** Optional info box content */
    infoContent?: string | { title: string; items: string[] };
    /** Show developer note input field */
    showDeveloperNote?: boolean;
    /** Show instruction input field */
    showInstruction?: boolean;
    /** Custom validation function */
    validateInput?: (input: TInput) => string | null;
    /** Custom error parser */
    parseError?: (error: unknown) => AIErrorInfo;
  }

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSuccess: (result: unknown, originalInput: string) => void;
    config: AIGenerationConfig;
    /** Previous result for regeneration (optional) */
    previousResult?: unknown;
  }

  let { open, onOpenChange, onSuccess, config, previousResult }: Props = $props();

  let inputText = $state('');
  let developerNote = $state('');
  let instruction = $state('');
  let isGenerating = $state(false);
  let error = $state<string | null>(null);
  let showRegenerateDialog = $state(false);
  let regenerateInstructions = $state('');
  let isRegenerating = $state(false);

  function showErrorToast(errorInfo: AIErrorInfo) {
    if (errorInfo.actionable && errorInfo.settingsPath) {
      toastActions.error('AI Generation Failed', errorInfo.message, {
        action: {
          label: 'Open Settings',
          onClick: () => {
            goto(errorInfo.settingsPath!);
            onOpenChange(false);
          },
        },
      });
    } else {
      toastActions.error('AI Generation Failed', errorInfo.message);
    }
  }

  async function handleGenerate() {
    const maxLength = config.maxLength ?? 10000;
    const validate = config.validateInput;

    if (!inputText.trim()) {
      error = 'Please enter input text';
      return;
    }

    if (inputText.length > maxLength) {
      error = `Input text is too long (max ${maxLength} characters)`;
      return;
    }

    if (validate) {
      const validationError = validate(inputText as any);
      if (validationError) {
        error = validationError;
        return;
      }
    }

    try {
      isGenerating = true;
      error = null;

      const result = await config.generateFn(
        inputText as any,
        undefined,
        developerNote.trim() || undefined,
        instruction.trim() || undefined
      );

      toastActions.success('AI generation completed successfully!');
      const textToPass = inputText;
      onSuccess(result, textToPass);
      onOpenChange(false);
      inputText = '';
    } catch (err) {
      const parseError = config.parseError || defaultParseError;
      const errorInfo: AIErrorInfo =
        err instanceof Error && 'errorInfo' in err
          ? (err as Error & { errorInfo: AIErrorInfo }).errorInfo
          : parseError(err);

      error = errorInfo.message;
      showErrorToast(errorInfo);
      console.error('AI generation error:', err);
    } finally {
      isGenerating = false;
    }
  }

  async function handleRegenerate() {
    if (!previousResult) {
      return;
    }

    if (!regenerateInstructions.trim()) {
      error = 'Please provide instructions for regeneration';
      return;
    }

    try {
      isRegenerating = true;
      error = null;

      const history = [
        {
          role: 'assistant' as const,
          content: config.serializeResult
            ? config.serializeResult(previousResult as any)
            : JSON.stringify(previousResult, null, 2),
        },
        {
          role: 'user' as const,
          content: `Please regenerate the above with the following changes: ${regenerateInstructions.trim()}`,
        },
      ];

      const result = await config.generateFn(
        inputText as any,
        history,
        developerNote.trim() || undefined,
        instruction.trim() || undefined
      );

      toastActions.success('Regeneration completed successfully!');
      const textToPass = inputText;
      showRegenerateDialog = false;
      regenerateInstructions = '';
      onSuccess(result, textToPass);
      onOpenChange(false);
    } catch (err) {
      const parseError = config.parseError || defaultParseError;
      const errorInfo: AIErrorInfo =
        err instanceof Error && 'errorInfo' in err
          ? (err as Error & { errorInfo: AIErrorInfo }).errorInfo
          : parseError(err);

      error = errorInfo.message;
      showErrorToast(errorInfo);
      console.error('AI regeneration error:', err);
    } finally {
      isRegenerating = false;
    }
  }

  function handleCancel() {
    inputText = '';
    developerNote = '';
    instruction = '';
    error = null;
    onOpenChange(false);
  }

  function handleOpenSettings() {
    goto(AI_PROVIDER_SETTINGS_PATH);
    onOpenChange(false);
  }

  function getResultPreview(): string {
    if (!previousResult) return '';
    if (config.serializeResult) {
      return config.serializeResult(previousResult as any);
    }
    return JSON.stringify(previousResult, null, 2);
  }

  $effect(() => {
    if (!open) {
      inputText = '';
      developerNote = '';
      instruction = '';
      error = null;
      isGenerating = false;
      showRegenerateDialog = false;
      regenerateInstructions = '';
    }
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
          bind:value={inputText}
          placeholder={config.inputPlaceholder}
          rows={config.inputRows ?? 12}
          class={error ? 'border-destructive' : ''}
          disabled={isGenerating}
        />
        {#if error}
          <p class="text-sm text-destructive">{error}</p>
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
            bind:value={developerNote}
            placeholder="Add developer-friendly instructions about the required work, technical considerations, or implementation details..."
            rows={4}
            disabled={isGenerating}
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
            bind:value={instruction}
            placeholder="e.g., Focus on API endpoints only, break into smaller subtasks, prioritize security features..."
            rows={3}
            disabled={isGenerating}
          />
          <p class="text-xs text-muted-foreground">
            Specify how you want the tasks to be generated (format, focus areas, structure, etc.).
          </p>
        </div>
      {/if}

      <AIGenerationNotices
        infoContent={config.infoContent}
        {error}
        onOpenSettings={handleOpenSettings}
      />
    </div>

    <Dialog.Footer>
      {#if previousResult}
        <Button
          variant="outline"
          onclick={() => (showRegenerateDialog = true)}
          disabled={isGenerating}
        >
          <FaRotate class="mr-2 h-4 w-4" />
          Regenerate
        </Button>
      {/if}
      <Button variant="outline" onclick={handleCancel} disabled={isGenerating}>Cancel</Button>
      <Button onclick={handleGenerate} disabled={isGenerating || !inputText.trim()}>
        <FaWandMagicSparkles class="mr-2 h-4 w-4" />
        Generate
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

{#if previousResult}
  <AIRegenerateDialog
    bind:open={showRegenerateDialog}
    preview={getResultPreview()}
    bind:instructions={regenerateInstructions}
    {error}
    {isRegenerating}
    onCancel={() => {
      showRegenerateDialog = false;
      regenerateInstructions = '';
      error = null;
    }}
    onRegenerate={handleRegenerate}
  />
{/if}
