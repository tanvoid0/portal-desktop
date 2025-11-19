<script lang="ts">
	import AIGenerationDialog, {
		type AIGenerationConfig,
		type AIErrorInfo
	} from '$lib/components/ai/AIGenerationDialog.svelte';
	import { aiTaskService, type GeneratedTaskStructure } from '../services/aiTaskService';
	import { parseError } from '../services/aiTaskService';

	import type { TaskContext } from '../services/aiTaskService';

	interface Props {
		open: boolean;
		onOpenChange: (open: boolean) => void;
		onGenerate: (result: GeneratedTaskStructure, originalText: string) => void;
		previousResult?: GeneratedTaskStructure;
		context?: TaskContext;
	}

	let { open, onOpenChange, onGenerate, previousResult, context }: Props = $props();

	// Configure the base AI dialog for story import
	// Use $derived to make config reactive to context changes
	const config = $derived.by(() => ({
		title: 'Import Story and Generate Tasks',
		description:
			'Paste your story, issue, or description text below. AI will analyze it and generate structured tasks and subtasks.',
		inputLabel: 'Story/Description Text',
		inputPlaceholder: 'Paste your story, issue description, or requirements here...',
		inputRows: 12,
		maxLength: 10000,
		providerType: 'Ollama',
		infoContent: {
			title: 'How it works:',
			items: [
				'AI analyzes the text and extracts key information',
				'Creates a main task with clear, developer-friendly description',
				'Breaks down work into logical subtasks',
				'Suggests relevant tags, priority, and project links',
				'Preserves original text as reference',
			],
		},
		generateFn: async (
			storyText: string,
			history?: Array<{ role: 'user' | 'assistant'; content: string }>,
			developerNote?: string,
			instruction?: string
		) => {
			return await aiTaskService.generateTasksFromStory({
				story_text: storyText,
				provider_type: 'Ollama', // Explicitly use Ollama since it's the only supported provider
				history: history,
				context: context,
				developer_note: developerNote,
				instruction: instruction,
			});
		},
		showDeveloperNote: true,
		showInstruction: true,
		serializeResult: (result: GeneratedTaskStructure) => {
			return JSON.stringify(result, null, 2);
		},
		parseError: (error: unknown): AIErrorInfo => {
			// Use the error parsing from aiTaskService
			if (error instanceof Error && 'errorInfo' in error) {
				return (error as Error & { errorInfo: AIErrorInfo }).errorInfo;
			}
			return parseError(error);
		},
	} as AIGenerationConfig<string, unknown>));

	function handleSuccess(result: unknown, originalText: string) {
		onGenerate(result as GeneratedTaskStructure, originalText);
	}
</script>

<AIGenerationDialog {open} {onOpenChange} {config} onSuccess={handleSuccess} previousResult={previousResult} />

