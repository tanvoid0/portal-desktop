import { goto } from '$app/navigation';
import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';
import { toastActions } from '$lib/utils/toast';
import { defaultParseError, type AIErrorInfo } from './aiGenerationErrors';

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

interface DialogProps {
  config: AIGenerationConfig;
  previousResult?: unknown;
  onSuccess: (result: unknown, originalInput: string) => void;
  onOpenChange: (open: boolean) => void;
}

/** Form state and generate/regenerate handlers for AIGenerationDialog. */
export class AIGenerationDialogState {
  inputText = $state('');
  developerNote = $state('');
  instruction = $state('');
  isGenerating = $state(false);
  error = $state<string | null>(null);
  showRegenerateDialog = $state(false);
  regenerateInstructions = $state('');
  isRegenerating = $state(false);

  /** Props are read lazily so the class always sees the current values. */
  constructor(private props: () => DialogProps) {}

  private showErrorToast(errorInfo: AIErrorInfo) {
    if (errorInfo.actionable && errorInfo.settingsPath) {
      toastActions.error('AI Generation Failed', errorInfo.message, {
        action: {
          label: 'Open Settings',
          onClick: () => {
            goto(errorInfo.settingsPath!);
            this.props().onOpenChange(false);
          },
        },
      });
    } else {
      toastActions.error('AI Generation Failed', errorInfo.message);
    }
  }

  private fail(err: unknown, label: string) {
    const parseError = this.props().config.parseError || defaultParseError;
    const errorInfo: AIErrorInfo =
      err instanceof Error && 'errorInfo' in err
        ? (err as Error & { errorInfo: AIErrorInfo }).errorInfo
        : parseError(err);

    this.error = errorInfo.message;
    this.showErrorToast(errorInfo);
    console.error(label, err);
  }

  private serialize(result: unknown): string {
    const { config } = this.props();
    return config.serializeResult
      ? config.serializeResult(result)
      : JSON.stringify(result, null, 2);
  }

  get resultPreview(): string {
    const { previousResult } = this.props();
    return previousResult ? this.serialize(previousResult) : '';
  }

  generate = async () => {
    const { config, onSuccess, onOpenChange } = this.props();
    const maxLength = config.maxLength ?? 10000;
    const validate = config.validateInput;

    if (!this.inputText.trim()) {
      this.error = 'Please enter input text';
      return;
    }

    if (this.inputText.length > maxLength) {
      this.error = `Input text is too long (max ${maxLength} characters)`;
      return;
    }

    if (validate) {
      const validationError = validate(this.inputText);
      if (validationError) {
        this.error = validationError;
        return;
      }
    }

    try {
      this.isGenerating = true;
      this.error = null;

      const result = await config.generateFn(
        this.inputText,
        undefined,
        this.developerNote.trim() || undefined,
        this.instruction.trim() || undefined
      );

      toastActions.success('AI generation completed successfully!');
      const textToPass = this.inputText;
      onSuccess(result, textToPass);
      onOpenChange(false);
      this.inputText = '';
    } catch (err) {
      this.fail(err, 'AI generation error:');
    } finally {
      this.isGenerating = false;
    }
  };

  regenerate = async () => {
    const { config, previousResult, onSuccess, onOpenChange } = this.props();
    if (!previousResult) {
      return;
    }

    if (!this.regenerateInstructions.trim()) {
      this.error = 'Please provide instructions for regeneration';
      return;
    }

    try {
      this.isRegenerating = true;
      this.error = null;

      const history = [
        { role: 'assistant' as const, content: this.serialize(previousResult) },
        {
          role: 'user' as const,
          content: `Please regenerate the above with the following changes: ${this.regenerateInstructions.trim()}`,
        },
      ];

      const result = await config.generateFn(
        this.inputText,
        history,
        this.developerNote.trim() || undefined,
        this.instruction.trim() || undefined
      );

      toastActions.success('Regeneration completed successfully!');
      const textToPass = this.inputText;
      this.showRegenerateDialog = false;
      this.regenerateInstructions = '';
      onSuccess(result, textToPass);
      onOpenChange(false);
    } catch (err) {
      this.fail(err, 'AI regeneration error:');
    } finally {
      this.isRegenerating = false;
    }
  };

  cancel = () => {
    this.inputText = '';
    this.developerNote = '';
    this.instruction = '';
    this.error = null;
    this.props().onOpenChange(false);
  };

  cancelRegenerate = () => {
    this.showRegenerateDialog = false;
    this.regenerateInstructions = '';
    this.error = null;
  };

  openSettings = () => {
    goto(AI_PROVIDER_SETTINGS_PATH);
    this.props().onOpenChange(false);
  };

  /** Dialog closed: drop everything the user typed. */
  reset() {
    this.inputText = '';
    this.developerNote = '';
    this.instruction = '';
    this.error = null;
    this.isGenerating = false;
    this.showRegenerateDialog = false;
    this.regenerateInstructions = '';
  }
}
