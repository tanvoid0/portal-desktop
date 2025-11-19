<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import Icon from '@iconify/svelte';
	import { aiTaskService, type GeneratedTaskStructure, type ConversationMessage, type TaskContext } from '../services/aiTaskService';
	import { taskActions } from '../stores/taskStore';
	import { toastActions } from '@/lib/domains/shared/stores/toastStore';
	import { tauriTaskService } from '../services/tauriTaskService';
	import type { Task, CreateTaskRequest, UpdateTaskRequest, TaskPriority, TaskStatus } from '../types';
	import TaskPreviewCard from './TaskPreviewCard.svelte';
	import TaskPreviewList from './TaskPreviewList.svelte';
	import AIChatPanel from '$lib/domains/ai/components/chat/AIChatPanel.svelte';
	import type { ChatMessage } from '$lib/domains/ai/types';
	import InstructionTemplateManager from './InstructionTemplateManager.svelte';
	import LoadingSpinner from '@/lib/components/ui/loading-spinner.svelte';

	interface Props {
		taskId?: string; // For updating existing task
	}

	let { taskId }: Props = $props();

	let storyText = $state('');
	let customInstruction = $state('');
	let selectedTemplateId = $state<string | null>(null);
	let isGenerating = $state(false);
	let generatedData = $state<GeneratedTaskStructure | null>(null);
	let chatMessages = $state<ChatMessage[]>([]);
	let taskContext = $state<TaskContext | undefined>(undefined);
	let existingTask = $state<Task | null>(null);
	let isLoadingTask = $state(false);
	let createdMainTaskId = $state<string | null>(null);
	let originalTaskState = $state<{
		status?: TaskStatus;
		priority?: TaskPriority;
		tags?: string[];
		type?: string;
		estimatedTime?: number;
	} | null>(null);
	let existingSubtasks = $state<Map<number, Task>>(new Map()); // Map of order/index to existing subtask

	// Load existing task if taskId is provided
	onMount(async () => {
		if (taskId) {
			isLoadingTask = true;
			try {
				existingTask = await tauriTaskService.getTask(taskId);
				if (existingTask) {
					// Pre-fill story text with task description
					storyText = existingTask.description || '';
					
					// Preload existing task as preview
					await preloadExistingTaskAsPreview();
				}
				// Build context for updating
				taskContext = await buildTaskContext();
			} catch (error) {
				toastActions.error('Failed to load task', error instanceof Error ? error.message : 'Unknown error');
			} finally {
				isLoadingTask = false;
			}
		}
	});

	async function preloadExistingTaskAsPreview() {
		if (!existingTask || !taskId) return;

		try {
			// Get all tasks to find subtasks
			const allTasks = await tauriTaskService.getTasks();
			const subtasks = allTasks.filter((t) => t.parentId === taskId);

			// Store existing subtasks mapped by their order/index
			existingSubtasks = new Map();
			subtasks.forEach((st, index) => {
				existingSubtasks.set(index, st);
			});

			// Convert existing task to GeneratedTaskStructure format
			generatedData = {
				main_task: {
					title: existingTask.title,
					description: existingTask.description || '',
					priority: existingTask.priority,
					type_: existingTask.type || '',
					estimated_time: existingTask.estimatedTime || null,
					tags: existingTask.tags || [],
				},
				subtasks: subtasks.map((st, index) => ({
					title: st.title,
					description: st.description || '',
					estimated_time: st.estimatedTime || null,
					dependencies: [],
					order: index + 1,
				})),
				suggested_project: null,
				suggested_labels: existingTask.tags || [],
				confidence: 1.0,
				model_used: 'existing-task',
			};

			// Store original state for preservation
			originalTaskState = {
				status: existingTask.status,
				priority: existingTask.priority,
				tags: existingTask.tags || [],
				type: existingTask.type,
				estimatedTime: existingTask.estimatedTime,
			};

			chatMessages = [
				{
					role: 'assistant',
					content: `I've loaded your existing task "${existingTask.title}" with ${subtasks.length} subtask(s). You can refine it using AI or edit manually.`,
					timestamp: new Date(),
				},
			];
		} catch (error) {
			console.error('Failed to preload task as preview:', error);
		}
	}

	async function buildTaskContext(): Promise<TaskContext | undefined> {
		if (!taskId) return undefined;

		const context: TaskContext = {};
		try {
			const task = await tauriTaskService.getTask(taskId);
			if (task) {
				context.parentTask = {
					id: task.id,
					title: task.title,
					description: task.description,
					priority: task.priority,
					type: task.type,
					tags: task.tags,
				};

				// Get existing children
				const allTasks = await tauriTaskService.getTasks();
				const children = allTasks
					.filter((t) => t.parentId === taskId)
					.map((t) => ({
						id: t.id,
						title: t.title,
						description: t.description,
						status: t.status,
					}));

				if (children.length > 0) {
					context.existingChildren = children;
				}
			}
		} catch (error) {
			console.error('Failed to build task context:', error);
		}

		return Object.keys(context).length > 0 ? context : undefined;
	}

	async function handleGenerate() {
		if (!storyText.trim()) {
			toastActions.error('Please enter story text');
			return;
		}

		isGenerating = true;
		try {
			const result = await aiTaskService.generateTasksFromStory({
				story_text: storyText,
				provider_type: 'Ollama',
				instruction: customInstruction.trim() || undefined,
				context: taskContext,
			});

			// If updating existing task, preserve original state in generated data
			if (taskId && originalTaskState) {
				// Merge AI result with original state preservation
				generatedData = {
					...result,
					main_task: {
						...result.main_task,
						// Preserve original priority if not explicitly changed
						priority: result.main_task.priority || originalTaskState.priority || 'medium',
						// Preserve original type if not explicitly changed
						type_: result.main_task.type_ || originalTaskState.type || '',
						// Preserve original estimated time if not explicitly changed
						estimated_time: result.main_task.estimated_time || originalTaskState.estimatedTime || null,
					},
					// Preserve original tags if AI didn't suggest new ones
					suggested_labels: result.suggested_labels.length > 0 
						? result.suggested_labels 
						: originalTaskState.tags || [],
				};
			} else {
				generatedData = result;
			}
			
			chatMessages = [
				{
					role: 'assistant',
					content: `I've ${taskId ? 'refined' : 'generated'} a task structure with ${result.subtasks.length} subtask(s). You can review and edit the preview, or chat with me to make adjustments.`,
					timestamp: new Date(),
				},
			];
		} catch (error) {
			toastActions.error(
				'Failed to generate tasks',
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		} finally {
			isGenerating = false;
		}
	}

	async function handleChatMessage(message: string, history: ChatMessage[]) {
		if (!generatedData) {
			toastActions.error('Please generate tasks first');
			return;
		}

		// Convert chat messages to conversation format
		// Include the original generation result in the history
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
			const result = await aiTaskService.generateTasksFromStory({
				story_text: storyText,
				provider_type: 'Ollama',
				history: conversationHistory,
				instruction: customInstruction.trim() || undefined,
				context: taskContext,
			});

			// If updating existing task, preserve original state
			if (taskId && originalTaskState) {
				generatedData = {
					...result,
					main_task: {
						...result.main_task,
						priority: result.main_task.priority || originalTaskState.priority || 'medium',
						type_: result.main_task.type_ || originalTaskState.type || '',
						estimated_time: result.main_task.estimated_time || originalTaskState.estimatedTime || null,
					},
					suggested_labels: result.suggested_labels.length > 0 
						? result.suggested_labels 
						: originalTaskState.tags || [],
				};
			} else {
				generatedData = result;
			}
			
			chatMessages = [
				...chatMessages,
				{
					role: 'assistant',
					content: `I've updated the task structure based on your feedback. The preview has been refreshed.`,
					timestamp: new Date(),
				},
			];
		} catch (error) {
			toastActions.error(
				'Failed to update tasks',
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		}
	}

	async function handleAddTask(task: any): Promise<void> {
		try {
			// If updating existing task, use update instead of create
			if (taskId && existingTask) {
				const updateData: UpdateTaskRequest = {
					title: task.title,
					description: task.description,
					// Preserve original status and other fields unless explicitly changed
					status: (originalTaskState?.status || existingTask.status) as TaskStatus,
					priority: (task.priority as TaskPriority) || originalTaskState?.priority || existingTask.priority,
					type: task.type_ || originalTaskState?.type || existingTask.type,
					estimatedTime: task.estimated_time || originalTaskState?.estimatedTime || existingTask.estimatedTime,
					tags: (generatedData && generatedData.suggested_labels.length > 0)
						? generatedData.suggested_labels 
						: originalTaskState?.tags || existingTask.tags,
				};

				await taskActions.updateTask(taskId, updateData);
				toastActions.success(`Task "${task.title}" updated successfully`);
				createdMainTaskId = taskId;
			} else {
				// Creating new task
				const taskData: CreateTaskRequest = {
					title: task.title,
					description: task.description,
					status: 'pending',
					priority: task.priority as TaskPriority,
					type: task.type_ || undefined,
					estimatedTime: task.estimated_time || undefined,
					tags: generatedData?.suggested_labels || undefined,
				};

				const createdTask = await taskActions.createTask(taskData);
				toastActions.success(`Task "${createdTask.title}" created successfully`);
				createdMainTaskId = createdTask.id;
			}
		} catch (error) {
			toastActions.error(
				`Failed to ${taskId ? 'update' : 'create'} task`,
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
			throw error;
		}
	}

	async function handleAddSubtask(index: number, subtask: any) {
		if (!generatedData) return;

		try {
			// If main task hasn't been created yet, create it first
			let parentTaskId = taskId || createdMainTaskId;
			if (!parentTaskId && generatedData.main_task) {
				await handleAddTask(generatedData.main_task);
				parentTaskId = createdMainTaskId;
			}

			if (!parentTaskId) {
				toastActions.error('Please create the main task first');
				return;
			}

			const subtaskData: CreateTaskRequest = {
				title: subtask.title,
				description: subtask.description || undefined,
				status: 'pending',
				priority: 'medium',
				parentId: parentTaskId,
				estimatedTime: subtask.estimated_time || undefined,
			};

			await taskActions.createTask(subtaskData);
			toastActions.success(`Subtask "${subtask.title}" created successfully`);
		} catch (error) {
			toastActions.error(
				'Failed to create subtask',
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		}
	}

	async function handleAddAll() {
		if (!generatedData) return;

		try {
			// Update or create main task first
			await handleAddTask(generatedData.main_task);
			const mainTaskId = createdMainTaskId || taskId;
			
			if (!mainTaskId) {
				toastActions.error('Failed to create main task');
				return;
			}

			let updatedCount = 0;
			let createdCount = 0;

			// Update existing subtasks or create new ones
			for (let index = 0; index < generatedData.subtasks.length; index++) {
				const subtask = generatedData.subtasks[index];
				if (!subtask.title.trim()) continue;

				const existingSubtask = existingSubtasks.get(index);
				
				if (existingSubtask && taskId) {
					// Update existing subtask
					const updateData: UpdateTaskRequest = {
						title: subtask.title,
						description: subtask.description || undefined,
						status: existingSubtask.status, // Preserve status
						priority: existingSubtask.priority, // Preserve priority
						estimatedTime: subtask.estimated_time || existingSubtask.estimatedTime,
					};
					await taskActions.updateTask(existingSubtask.id, updateData);
					updatedCount++;
				} else {
					// Create new subtask
					const subtaskData: CreateTaskRequest = {
						title: subtask.title,
						description: subtask.description || undefined,
						status: 'pending',
						priority: 'medium',
						parentId: mainTaskId,
						estimatedTime: subtask.estimated_time || undefined,
					};
					await taskActions.createTask(subtaskData);
					createdCount++;
				}
			}

			const action = taskId ? 'Updated' : 'Created';
			const message = taskId 
				? `Updated task with ${updatedCount} updated and ${createdCount} new subtask(s)`
				: `Created task with ${generatedData.subtasks.length} subtask(s)`;
			
			toastActions.success(message);

			// Navigate to the task
			goto(`/tasks/${mainTaskId}`);
		} catch (error) {
			toastActions.error(
				`Failed to ${taskId ? 'update' : 'create'} tasks`,
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		}
	}

	function handleTemplateSelect(templateId: string, instruction: string) {
		selectedTemplateId = templateId;
		customInstruction = instruction;
	}

	function handleUpdateMainTask(task: any) {
		if (generatedData) {
			generatedData = {
				...generatedData,
				main_task: task,
			};
		}
	}

	function handleUpdateSubtask(index: number, subtask: any) {
		if (generatedData) {
			generatedData = {
				...generatedData,
				subtasks: generatedData.subtasks.map((s, i) => (i === index ? subtask : s)),
			};
		}
	}
</script>

<div class="min-h-screen bg-background">
	<div class="container mx-auto p-6">
		<!-- Header -->
		<div class="mb-6">
			<div class="flex items-center justify-between mb-4">
				<div>
					<h1 class="text-2xl font-bold">
						{taskId ? 'Update Task with AI' : 'Generate Tasks with AI'}
					</h1>
					<p class="text-sm text-muted-foreground">
						{taskId
							? 'Use AI to generate subtasks or update the current task'
							: 'Describe your work and let AI create structured tasks for you'}
					</p>
				</div>
				<Button variant="ghost" onclick={() => goto(taskId ? `/tasks/${taskId}` : '/tasks')}>
					<Icon icon="lucide:arrow-left" class="h-4 w-4 mr-2" />
					Back
				</Button>
			</div>
		</div>

		{#if isLoadingTask}
			<div class="flex items-center justify-center py-12">
				<LoadingSpinner size="lg" text="Loading task..." />
			</div>
		{:else}
			<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
				<!-- Left Column: Input and Preview -->
				<div class="lg:col-span-2 space-y-6">
					<!-- Story Input -->
					<Card>
						<CardHeader>
							<CardTitle>Story / Description</CardTitle>
							<CardDescription>
								Paste your story, issue description, or requirements here
							</CardDescription>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="story-text">Story Text</Label>
								<Textarea
									id="story-text"
									bind:value={storyText}
									placeholder="Paste your story, issue description, or requirements here..."
									rows={8}
									disabled={isGenerating}
								/>
							</div>

							<!-- Instruction Template Manager -->
							<InstructionTemplateManager
								bind:selectedTemplateId={selectedTemplateId}
								bind:customInstruction={customInstruction}
								onTemplateSelect={handleTemplateSelect}
							/>

							<Button
								onclick={handleGenerate}
								disabled={!storyText.trim() || isGenerating}
								
								class="w-full"
							>
								<Icon icon="lucide:sparkles" class="h-4 w-4 mr-2" />
								Generate Tasks
							</Button>
						</CardContent>
					</Card>

					<!-- Preview -->
					{#if generatedData}
						<div class="space-y-6">
							<div class="flex items-center justify-between">
								<div>
									<h2 class="text-xl font-bold">Preview</h2>
									<p class="text-sm text-muted-foreground">
										Review and edit the generated tasks before creating them
									</p>
								</div>
								<Badge variant="outline" class="text-xs">
									Confidence: {Math.round(generatedData.confidence * 100)}% | Model:{' '}
									{generatedData.model_used}
								</Badge>
							</div>

							<TaskPreviewCard
								task={generatedData.main_task}
								bind:suggestedLabels={generatedData.suggested_labels}
								onUpdate={handleUpdateMainTask}
								onAdd={handleAddTask}
								onAddAll={handleAddAll}
								hasSubtasks={generatedData.subtasks.length > 0}
								showAddButton={true}
								originalStatus={originalTaskState?.status}
								isUpdateMode={!!taskId}
							/>

							<TaskPreviewList
								bind:subtasks={generatedData.subtasks}
								onUpdate={handleUpdateSubtask}
								onAdd={handleAddSubtask}
								showAddButtons={true}
							/>
						</div>
					{/if}
				</div>

				<!-- Right Column: Chat Panel -->
				<div class="lg:col-span-1">
					<AIChatPanel
						bind:messages={chatMessages}
						onSendMessageWithHistory={handleChatMessage}
						bind:isLoading={isGenerating}
						placeholder="Ask me to adjust the tasks, add more subtasks, change priorities..."
						title="Refine Tasks"
						class="h-[calc(100vh-12rem)]"
					/>
				</div>
			</div>
		{/if}
	</div>
</div>

