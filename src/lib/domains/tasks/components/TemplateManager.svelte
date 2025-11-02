<script lang="ts">
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { Textarea } from '@/lib/components/ui/textarea';
	import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/lib/components/ui/dialog';
	import Select from '@/lib/components/ui/select.svelte';
	import { taskActions, taskTemplates } from '../stores/taskStore';
	import type { TaskTemplate, TaskStatus, TaskPriority } from '../types';
	import { TASK_TYPE_OPTIONS, TASK_STATUS_OPTIONS, TASK_PRIORITY_OPTIONS } from '../types';
	import Icon from '@iconify/svelte';

	interface Props {
		onApplyTemplate?: (template: TaskTemplate) => void;
	}

	let { onApplyTemplate }: Props = $props();

	let showCreateDialog = $state(false);
	let showEditDialog = $state(false);
	let editingTemplate: TaskTemplate | null = $state(null);

	// Template form state
	let templateName = $state('');
	let templateDescription = $state('');
	let templateStatus: TaskStatus = $state('pending');
	let templatePriority: TaskPriority = $state('medium');
	let templateType = $state('');
	let templateTags = $state<string[]>([]);
	let templateEstimatedTime = $state(0);
	let templateAssignee = $state('');

	// Tag management
	let newTag = $state('');

	function addTag() {
		if (newTag.trim() && !templateTags.includes(newTag.trim())) {
			templateTags = [...templateTags, newTag.trim()];
			newTag = '';
		}
	}

	function removeTag(tagToRemove: string) {
		templateTags = templateTags.filter(tag => tag !== tagToRemove);
	}

	function handleTagKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			addTag();
		}
	}

	function applyTemplate(template: TaskTemplate) {
		onApplyTemplate?.(template);
	}

	function createTemplate() {
		if (!templateName.trim()) return;

		const newTemplate: TaskTemplate = {
			id: crypto.randomUUID(),
			name: templateName.trim(),
			description: templateDescription.trim() || undefined,
			defaultStatus: templateStatus,
			defaultPriority: templatePriority,
			defaultType: templateType || undefined,
			defaultTags: templateTags.length > 0 ? templateTags : undefined,
			defaultEstimatedTime: templateEstimatedTime || undefined,
			createdAt: new Date(),
			updatedAt: new Date()
		};

		taskActions.createTemplate(newTemplate);
		resetForm();
		showCreateDialog = false;
	}

	function editTemplate(template: TaskTemplate) {
		editingTemplate = template;
		templateName = template.name;
		templateDescription = template.description || '';
		templateStatus = template.defaultStatus;
		templatePriority = template.defaultPriority;
		templateType = template.defaultType || '';
		templateTags = template.defaultTags || [];
		templateEstimatedTime = template.defaultEstimatedTime || 0;
		templateAssignee = '';
		showEditDialog = true;
	}

	function updateTemplate() {
		if (!editingTemplate) return;

		const updatedTemplate: TaskTemplate = {
			...editingTemplate,
			name: templateName.trim(),
			description: templateDescription.trim() || undefined,
			defaultStatus: templateStatus,
			defaultPriority: templatePriority,
			defaultType: templateType || undefined,
			defaultTags: templateTags.length > 0 ? templateTags : undefined,
			defaultEstimatedTime: templateEstimatedTime || undefined,
			updatedAt: new Date()
		};

		taskActions.updateTemplate(updatedTemplate);
		resetForm();
		showEditDialog = false;
		editingTemplate = null;
	}

	function deleteTemplate(templateId: string) {
		taskActions.deleteTemplate(templateId);
	}

	function resetForm() {
		templateName = '';
		templateDescription = '';
		templateStatus = 'pending';
		templatePriority = 'medium';
		templateType = '';
		templateTags = [];
		templateEstimatedTime = 0;
		templateAssignee = '';
		newTag = '';
	}
</script>

<Card class="w-full">
	<CardHeader>
		<div class="flex items-center justify-between">
			<CardTitle class="text-lg">Task Templates</CardTitle>
			<Dialog bind:open={showCreateDialog}>
				<DialogTrigger>
					<Button variant="outline" size="sm">
						<Icon icon="mdi:plus" class="w-4 h-4 mr-1" />
						New Template
					</Button>
				</DialogTrigger>
				<DialogContent class="max-w-2xl">
					<DialogHeader>
						<DialogTitle>Create Task Template</DialogTitle>
						<DialogDescription>
							Create a reusable template for common task types.
						</DialogDescription>
					</DialogHeader>
					<div class="space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label for="templateName">Template Name</Label>
								<Input
									id="templateName"
									bind:value={templateName}
									placeholder="e.g., Daily Standup"
								/>
							</div>
							<div class="space-y-2">
								<Label for="templateType">Default Type</Label>
								<Select
									options={TASK_TYPE_OPTIONS}
									defaultValue={templateType}
									placeholder="Select type..."
									onSelect={(value) => templateType = value}
								/>
							</div>
						</div>

						<div class="space-y-2">
							<Label for="templateDescription">Description</Label>
							<Textarea
								id="templateDescription"
								bind:value={templateDescription}
								placeholder="Optional description..."
								rows={2}
							/>
						</div>

						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label for="templateStatus">Default Status</Label>
								<Select
									options={TASK_STATUS_OPTIONS}
									defaultValue={templateStatus}
									onSelect={(value) => templateStatus = value as TaskStatus}
								/>
							</div>
							<div class="space-y-2">
								<Label for="templatePriority">Default Priority</Label>
								<Select 
									options={TASK_PRIORITY_OPTIONS}
									defaultValue={templatePriority}
									onSelect={(value) => templatePriority = value as TaskPriority}
								/>
							</div>
						</div>

						<div class="space-y-2">
							<Label>Default Tags</Label>
							<div class="flex gap-2">
								<Input
									placeholder="Add a tag..."
									bind:value={newTag}
									onkeydown={handleTagKeydown}
								/>
								<Button type="button" variant="outline" onclick={addTag}>
									<Icon icon="mdi:plus" class="w-4 h-4" />
								</Button>
							</div>
							{#if templateTags.length > 0}
								<div class="flex flex-wrap gap-2">
									{#each templateTags as tag}
										<Badge variant="secondary" class="flex items-center gap-1">
											{tag}
											<Button
												type="button"
												variant="ghost"
												size="sm"
												onclick={() => removeTag(tag)}
												class="h-4 w-4 p-0 hover:bg-destructive hover:text-destructive-foreground"
											>
												<Icon icon="mdi:close" class="w-3 h-3" />
											</Button>
										</Badge>
									{/each}
								</div>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="templateEstimatedTime">Default Estimated Time (minutes)</Label>
							<Input
								id="templateEstimatedTime"
								type="number"
								min="0"
								bind:value={templateEstimatedTime}
								placeholder="e.g., 30"
							/>
						</div>
					</div>
					<DialogFooter>
						<Button variant="outline" onclick={() => showCreateDialog = false}>
							Cancel
						</Button>
						<Button onclick={createTemplate} disabled={!templateName.trim()}>
							Create Template
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</div>
	</CardHeader>
	<CardContent class="p-4 pt-0">
		{#if $taskTemplates.length === 0}
			<div class="text-center py-8 text-muted-foreground">
				<Icon icon="mdi:template" class="w-12 h-12 mx-auto mb-2" />
				<div class="text-sm">No templates yet</div>
				<div class="text-xs">Create your first template to get started</div>
			</div>
		{:else}
			<div class="space-y-2">
				{#each $taskTemplates as template}
					<div class="flex items-center justify-between p-3 rounded-lg border hover:bg-muted/50 transition-colors">
						<div class="flex items-center gap-3">
							<Button
								variant="ghost"
								class="justify-start flex-1 h-auto p-0"
								onclick={() => applyTemplate(template)}
							>
								<div class="flex items-center gap-2">
									<Icon icon="mdi:template" class="w-4 h-4" />
									<div class="text-left">
										<div class="font-medium">{template.name}</div>
										{#if template.description}
											<div class="text-xs text-muted-foreground">{template.description}</div>
										{/if}
										<div class="flex items-center gap-2 mt-1">
											<Badge variant="outline" class="text-xs">{template.defaultStatus}</Badge>
											<Badge variant="outline" class="text-xs">{template.defaultPriority}</Badge>
											{#if template.defaultType}
												<Badge variant="outline" class="text-xs">{template.defaultType}</Badge>
											{/if}
											{#if template.defaultEstimatedTime}
												<Badge variant="outline" class="text-xs">{template.defaultEstimatedTime}m</Badge>
											{/if}
										</div>
									</div>
								</div>
							</Button>
						</div>
						
						<div class="flex items-center gap-1">
							<Button
								variant="ghost"
								size="sm"
								onclick={() => editTemplate(template)}
								title="Edit template"
							>
								<Icon icon="mdi:pencil" class="w-4 h-4" />
							</Button>
							<Button
								variant="ghost"
								size="sm"
								onclick={() => deleteTemplate(template.id)}
								title="Delete template"
								class="text-destructive hover:text-destructive"
							>
								<Icon icon="mdi:delete" class="w-4 h-4" />
							</Button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</CardContent>
</Card>
