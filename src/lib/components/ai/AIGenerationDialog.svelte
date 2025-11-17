<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { goto } from '$app/navigation';
	import Icon from '@iconify/svelte';

	export interface AIErrorInfo {
		message: string;
		type: 'configuration' | 'provider_unavailable' | 'network' | 'model_not_found' | 'unknown';
		actionable: boolean;
		settingsPath?: string;
	}

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
		generateFn: (input: TInput, history?: Array<{ role: 'user' | 'assistant'; content: string }>, developerNote?: string, instruction?: string) => Promise<TOutput>;
		/** Function to serialize result for display in regenerate dialog */
		serializeResult?: (result: TOutput) => string;
		/** Optional provider type to use (defaults to Ollama) */
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

	const defaultParseError = (err: unknown): AIErrorInfo => {
		const errorMessage =
			err instanceof Error
				? err.message
				: typeof err === 'string'
				? err
				: 'An error occurred during AI generation';

		// Check for configuration issues
		if (
			errorMessage.includes('No default provider') ||
			errorMessage.includes('No default provider set') ||
			errorMessage.includes('Configuration incomplete') ||
			errorMessage.includes('provider configuration') ||
			errorMessage.includes('Missing fields') ||
			errorMessage.includes('not configured') ||
			errorMessage.includes('Provider not found') ||
			errorMessage.includes('Provider not registered')
		) {
			return {
				message:
					'AI provider is not configured. Please set up an AI provider (Ollama, Gemini, OpenAI, or Anthropic) in Settings.',
				type: 'configuration',
				actionable: true,
				settingsPath: '/settings/learning',
			};
		}

		// Check for model not found errors
		if (
			errorMessage.includes('model') &&
			(errorMessage.includes('not found') ||
				errorMessage.includes('404') ||
				errorMessage.includes('does not exist') ||
				errorMessage.includes('not installed'))
		) {
			// Extract model name if possible
			let modelMatch = errorMessage.match(/model\s+['"]([^'"]+)['"]/);
			if (!modelMatch) {
				modelMatch = errorMessage.match(/model\s+['"]([^'"]+?)['"]/);
			}
			if (!modelMatch) {
				modelMatch = errorMessage.match(/"error":\s*"model\s+['"]([^'"]+?)['"]/);
			}
			if (!modelMatch) {
				modelMatch = errorMessage.match(/\{"error":\s*"model\s+['"]([^'"]+?)['"]/);
			}

			const modelName = modelMatch ? modelMatch[1] : 'the specified model';

			return {
				message: `Model "${modelName}" is not installed in Ollama. Please install it using: ollama pull ${modelName}`,
				type: 'model_not_found',
				actionable: true,
				settingsPath: '/settings/learning',
			};
		}

		// Check for provider unavailable
		if (
			errorMessage.includes('Provider not available') ||
			errorMessage.includes('service is not running') ||
			errorMessage.includes('failed to connect')
		) {
			return {
				message:
					'AI provider is not available. Please check that your AI service is running and properly configured.',
				type: 'provider_unavailable',
				actionable: true,
				settingsPath: '/settings/learning',
			};
		}

		// Check for network issues
		if (
			errorMessage.includes('Network error') ||
			errorMessage.includes('timeout') ||
			errorMessage.includes('connection')
		) {
			return {
				message: 'Network error connecting to AI provider. Please check your connection and try again.',
				type: 'network',
				actionable: false,
			};
		}

		// Default unknown error
		return {
			message: errorMessage,
			type: 'unknown',
			actionable: false,
		};
	};

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

		// Basic validation
		if (!inputText.trim()) {
			error = 'Please enter input text';
			return;
		}

		if (inputText.length > maxLength) {
			error = `Input text is too long (max ${maxLength} characters)`;
			return;
		}

		// Custom validation
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
			// Check if error has errorInfo property (from custom error)
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

			// Build conversation history for regeneration
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
		goto('/settings/learning');
		onOpenChange(false);
	}

	function getResultPreview(): string {
		if (!previousResult) return '';
		if (config.serializeResult) {
			return config.serializeResult(previousResult as any);
		}
		return JSON.stringify(previousResult, null, 2);
	}

	// Reset input when dialog closes
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

<Dialog.Root bind:open={open} onOpenChange={onOpenChange}>
	<Dialog.Content class="max-w-3xl max-h-[90vh] overflow-y-auto">
		<Dialog.Header>
			<Dialog.Title>{config.title}</Dialog.Title>
			<Dialog.Description>{config.description}</Dialog.Description>
		</Dialog.Header>

		<div class="space-y-4 py-4">
			<!-- Input Field -->
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

			<!-- Developer Note (Optional) -->
			{#if config.showDeveloperNote}
				<div class="space-y-2">
					<Label for="developer-note">
						<Icon icon="lucide:code" class="h-4 w-4 inline mr-1" />
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
						This will be included in the task description to guide developers on implementation approach.
					</p>
				</div>
			{/if}

			<!-- Instruction (Optional) -->
			{#if config.showInstruction}
				<div class="space-y-2">
					<Label for="instruction">
						<Icon icon="lucide:wand-2" class="h-4 w-4 inline mr-1" />
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

			<!-- Info Box -->
			{#if config.infoContent}
				<div class="rounded-lg border bg-muted/50 p-4">
					<div class="flex gap-2">
						<Icon icon="lucide:info" class="h-5 w-5 text-muted-foreground shrink-0 mt-0.5" />
						<div class="text-sm text-muted-foreground space-y-1">
							{#if typeof config.infoContent === 'string'}
								<p>{config.infoContent}</p>
							{:else}
								<p class="font-medium">{config.infoContent.title}</p>
								<ul class="list-disc list-inside space-y-0.5 ml-2">
									{#each config.infoContent.items as item}
										<li>{item}</li>
									{/each}
								</ul>
							{/if}
						</div>
					</div>
				</div>
			{/if}

			<!-- Configuration Warning -->
			{#if error && error.includes('not configured')}
				<div
					class="rounded-lg border border-amber-200 bg-amber-50 dark:border-amber-800 dark:bg-amber-900/20 p-4"
				>
					<div class="flex gap-2">
						<Icon
							icon="lucide:alert-triangle"
							class="h-5 w-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5"
						/>
						<div class="text-sm space-y-2 flex-1">
							<p class="font-medium text-amber-900 dark:text-amber-100">
								AI Provider Not Configured
							</p>
							<p class="text-amber-800 dark:text-amber-200">
								To use AI features, you need to configure an AI provider. You can use Ollama (local),
								Gemini, OpenAI, or Anthropic.
							</p>
							<Button
								variant="outline"
								size="sm"
								onclick={handleOpenSettings}
								class="mt-2 border-amber-300 text-amber-900 hover:bg-amber-100 dark:border-amber-700 dark:text-amber-100 dark:hover:bg-amber-900/40"
							>
								<Icon icon="lucide:settings" class="h-4 w-4 mr-2" />
								Configure AI Provider
							</Button>
						</div>
					</div>
				</div>
			{/if}

			<!-- Model Not Found Warning -->
			{#if error && (error.includes('not installed') || (error.includes('model') && error.includes('not found')))}
				<div
					class="rounded-lg border border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-900/20 p-4"
				>
					<div class="flex gap-2">
						<Icon
							icon="lucide:package"
							class="h-5 w-5 text-blue-600 dark:text-blue-400 shrink-0 mt-0.5"
						/>
						<div class="text-sm space-y-2 flex-1">
							<p class="font-medium text-blue-900 dark:text-blue-100">Model Not Installed</p>
							<p class="text-blue-800 dark:text-blue-200">{error}</p>
							<p class="text-blue-700 dark:text-blue-300 text-xs">
								You can install the model in the terminal or use the "Load Models" button in Settings to
								see available models.
							</p>
							<Button
								variant="outline"
								size="sm"
								onclick={handleOpenSettings}
								class="mt-2 border-blue-300 text-blue-900 hover:bg-blue-100 dark:border-blue-700 dark:text-blue-100 dark:hover:bg-blue-900/40"
							>
								<Icon icon="lucide:settings" class="h-4 w-4 mr-2" />
								Open Settings
							</Button>
						</div>
					</div>
				</div>
			{/if}
		</div>

		<Dialog.Footer>
			{#if previousResult}
				<Button
					variant="outline"
					onclick={() => (showRegenerateDialog = true)}
					disabled={isGenerating}
				>
					<Icon icon="lucide:refresh-cw" class="h-4 w-4 mr-2" />
					Regenerate
				</Button>
			{/if}
			<Button variant="outline" onclick={handleCancel} disabled={isGenerating}>
				Cancel
			</Button>
			<Button onclick={handleGenerate} disabled={isGenerating || !inputText.trim()} loading={isGenerating}>
				<Icon icon="lucide:sparkles" class="h-4 w-4 mr-2" />
				Generate
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Regenerate Dialog -->
{#if showRegenerateDialog && previousResult}
	<Dialog.Root bind:open={showRegenerateDialog}>
		<Dialog.Content class="max-w-2xl max-h-[90vh] overflow-y-auto">
			<Dialog.Header>
				<Dialog.Title>Regenerate with Instructions</Dialog.Title>
				<Dialog.Description>
					Provide instructions on how to modify the previous generation. The AI will use the previous result
					and your instructions to create an improved version.
				</Dialog.Description>
			</Dialog.Header>

			<div class="space-y-4 py-4">
				<!-- Previous Result Preview -->
				<div class="space-y-2">
					<Label>Previous Result (Reference)</Label>
					<div class="rounded-lg border bg-muted/50 p-4 max-h-[300px] overflow-y-auto">
						<pre class="text-xs whitespace-pre-wrap font-mono">{getResultPreview()}</pre>
					</div>
				</div>

				<!-- Instructions -->
				<div class="space-y-2">
					<Label for="regenerate-instructions">Instructions for Regeneration</Label>
					<Textarea
						id="regenerate-instructions"
						bind:value={regenerateInstructions}
						placeholder="e.g., Make the titles more concise, add 2 more subtasks, change priority to high..."
						rows={6}
						class={error ? 'border-destructive' : ''}
						disabled={isRegenerating}
					/>
					<p class="text-xs text-muted-foreground">
						Describe what changes you'd like to make to the previous generation.
					</p>
					{#if error}
						<p class="text-sm text-destructive">{error}</p>
					{/if}
				</div>
			</div>

			<Dialog.Footer>
				<Button
					variant="outline"
					onclick={() => {
						showRegenerateDialog = false;
						regenerateInstructions = '';
						error = null;
					}}
					disabled={isRegenerating}
				>
					Cancel
				</Button>
				<Button
					onclick={handleRegenerate}
					disabled={isRegenerating || !regenerateInstructions.trim()}
					loading={isRegenerating}
				>
					<Icon icon="lucide:refresh-cw" class="h-4 w-4 mr-2" />
					Regenerate
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}

