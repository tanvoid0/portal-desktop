<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import Icon from '@iconify/svelte';
	import { aiDocumentService, type GeneratedDocumentStructure, type ConversationMessage, type DocumentContext } from '../services/aiDocumentService';
	import { documentActions } from '../stores/documentStore';
	import { toastActions } from '@/lib/domains/shared/stores/toastStore';
	import type { Document } from '../types';
	import AIChatPanel, { type ChatMessage } from '@/lib/components/ai/AIChatPanel.svelte';
	import LoadingSpinner from '@/lib/components/ui/loading-spinner.svelte';
	import { goto } from '$app/navigation';

	interface Props {
		taskId?: number; // For creating document linked to task
		taskTitle?: string;
		taskDescription?: string;
	}

	let { taskId, taskTitle, taskDescription }: Props = $props();

	let prompt = $state(taskDescription || '');
	let customInstruction = $state('');
	let isGenerating = $state(false);
	let generatedData = $state<GeneratedDocumentStructure | null>(null);
	let chatMessages = $state<ChatMessage[]>([]);
	let documentContext = $state<DocumentContext | undefined>(
		taskId && taskTitle
			? {
					linkedTask: {
						id: taskId,
						title: taskTitle,
						description: taskDescription,
					},
				}
			: undefined
	);

	async function handleGenerate() {
		if (!prompt.trim()) {
			toastActions.error('Please enter a prompt');
			return;
		}

		isGenerating = true;
		try {
			const result = await aiDocumentService.generateDocumentFromPrompt({
				prompt,
				providerType: 'Ollama',
				instruction: customInstruction.trim() || undefined,
				context: documentContext,
			});

			generatedData = result;
			chatMessages = [
				{
					role: 'assistant',
					content: `I've generated a document titled "${result.title}". You can review and edit it, or chat with me to make adjustments.`,
					timestamp: new Date(),
				},
			];
		} catch (error) {
			toastActions.error(
				'Failed to generate document',
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		} finally {
			isGenerating = false;
		}
	}

	async function handleChatMessage(message: string, history: ChatMessage[]) {
		if (!generatedData) {
			toastActions.error('Please generate a document first');
			return;
		}

		const conversationHistory: ConversationMessage[] = [
			{
				role: 'assistant',
				content: JSON.stringify(generatedData, null, 2),
			},
			...history
				.filter((m) => m.role === 'user' || (m.role === 'assistant' && !m.content.includes('generated')))
				.map((m) => ({
					role: m.role,
					content: m.content,
				})),
			{
				role: 'user',
				content: message,
			},
		];

		try {
			const result = await aiDocumentService.generateDocumentFromPrompt({
				prompt,
				providerType: 'Ollama',
				history: conversationHistory,
				instruction: customInstruction.trim() || undefined,
				context: documentContext,
			});

			generatedData = result;
			chatMessages = [
				...chatMessages,
				{
					role: 'assistant',
					content: `I've updated the document based on your feedback. The preview has been refreshed.`,
					timestamp: new Date(),
				},
			];
		} catch (error) {
			toastActions.error(
				'Failed to update document',
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		}
	}

	async function handleCreateDocument() {
		if (!generatedData) return;

		try {
			const doc = await documentActions.createDocument(
				generatedData.title,
				generatedData.content,
				generatedData.suggestedTags
			);
			toastActions.success('Document created successfully');
			goto(`/documents/${doc.id}`);
		} catch (error) {
			toastActions.error(
				'Failed to create document',
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		}
	}
</script>

<div class="min-h-screen bg-background">
	<div class="container mx-auto p-6">
		<div class="mb-6">
			<h1 class="text-2xl font-bold">
				{taskId ? 'Create Document for Task' : 'Generate Document with AI'}
			</h1>
			<p class="text-sm text-muted-foreground">
				{taskId
					? 'Use AI to generate a document linked to this task'
					: 'Describe what you want to document and let AI create it for you'}
			</p>
		</div>

		<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
			<!-- Left Column: Input and Preview -->
			<div class="lg:col-span-2 space-y-6">
				<!-- Prompt Input -->
				<Card>
					<CardHeader>
						<CardTitle>Prompt / Description</CardTitle>
						<CardDescription>Describe what you want to document</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="prompt">Prompt</Label>
							<Textarea
								id="prompt"
								bind:value={prompt}
								placeholder="Describe what you want to document..."
								rows={8}
								disabled={isGenerating}
							/>
						</div>

						<div class="space-y-2">
							<Label for="instruction">Custom Instructions (Optional)</Label>
							<Textarea
								id="instruction"
								bind:value={customInstruction}
								placeholder="Add specific instructions for the AI..."
								rows={3}
								disabled={isGenerating}
							/>
						</div>

						<Button
							onclick={handleGenerate}
							disabled={!prompt.trim() || isGenerating}
							loading={isGenerating}
							class="w-full"
						>
							<Icon icon="lucide:sparkles" class="h-4 w-4 mr-2" />
							Generate Document
						</Button>
					</CardContent>
				</Card>

				<!-- Preview -->
				{#if generatedData}
					<Card>
						<CardHeader>
							<div class="flex items-center justify-between">
								<div>
									<CardTitle>Preview</CardTitle>
									<CardDescription>Review the generated document</CardDescription>
								</div>
								<Badge variant="outline" class="text-xs">
									Confidence: {Math.round(generatedData.confidence * 100)}% | Model:{' '}
									{generatedData.modelUsed}
								</Badge>
							</div>
						</CardHeader>
						<CardContent class="space-y-4">
							<div>
								<h3 class="font-semibold mb-2">{generatedData.title}</h3>
								<div class="prose dark:prose-invert max-w-none">
									<pre class="whitespace-pre-wrap text-sm">{generatedData.content}</pre>
								</div>
							</div>
							{#if generatedData.suggestedTags.length > 0}
								<div class="flex flex-wrap gap-2">
									<span class="text-sm text-muted-foreground">Tags:</span>
									{#each generatedData.suggestedTags as tag}
										<Badge variant="secondary">{tag}</Badge>
									{/each}
								</div>
							{/if}
							<Button onclick={handleCreateDocument} class="w-full">
								<Icon icon="lucide:save" class="h-4 w-4 mr-2" />
								Create Document
							</Button>
						</CardContent>
					</Card>
				{/if}
			</div>

			<!-- Right Column: Chat Panel -->
			<div class="lg:col-span-1">
				<AIChatPanel
					bind:messages={chatMessages}
					onSendMessageWithHistory={handleChatMessage}
					bind:isLoading={isGenerating}
					placeholder="Ask me to adjust the document, change the tone, add sections..."
					title="Refine Document"
					class="h-[calc(100vh-12rem)]"
				/>
			</div>
		</div>
	</div>
</div>

