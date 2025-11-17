<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '$lib/components/ui/collapsible/index.js';
	import Select from '$lib/components/ui/select.svelte';
	import Icon from '@iconify/svelte';
	import { toastActions } from '@/lib/domains/shared/stores/toastStore';

	export interface InstructionTemplate {
		id: string;
		name: string;
		instruction: string;
		isDefault?: boolean;
		createdAt: Date;
	}

	interface Props {
		selectedTemplateId?: string | null;
		customInstruction?: string;
		onTemplateSelect?: (templateId: string, instruction: string) => void;
		onInstructionChange?: (instruction: string) => void;
	}

	let {
		selectedTemplateId = $bindable<string | null>(null),
		customInstruction = $bindable(''),
		onTemplateSelect,
		onInstructionChange
	}: Props = $props();

	// Default templates
	const defaultTemplates: InstructionTemplate[] = [
		{
			id: 'default-1',
			name: 'Standard Breakdown',
			instruction: 'Break down into logical subtasks, prioritize developer-friendly descriptions, and estimate time for each subtask.',
			isDefault: true,
			createdAt: new Date(),
		},
		{
			id: 'default-2',
			name: 'Detailed Subtasks',
			instruction: 'Create detailed subtasks with clear acceptance criteria. Focus on breaking down complex work into smaller, actionable items.',
			isDefault: true,
			createdAt: new Date(),
		},
		{
			id: 'default-3',
			name: 'API Focus',
			instruction: 'Focus on API endpoints, request/response structures, and error handling. Break down by endpoint or resource.',
			isDefault: true,
			createdAt: new Date(),
		},
	];

	// Initialize templates - load from localStorage or use defaults
	function loadTemplatesFromStorage(): InstructionTemplate[] {
		try {
			const stored = localStorage.getItem('task-instruction-templates');
			if (stored) {
				const parsed = JSON.parse(stored);
				return parsed.map((t: any) => ({
					...t,
					createdAt: new Date(t.createdAt),
				}));
			}
		} catch (error) {
			console.error('Failed to load templates from localStorage:', error);
		}
		return [...defaultTemplates];
	}

	let templates = $state<InstructionTemplate[]>(loadTemplatesFromStorage());
	let showTemplateManager = $state(false);
	let showNewTemplateForm = $state(false);
	let newTemplateName = $state('');
	let newTemplateInstruction = $state('');
	let editingTemplateId = $state<string | null>(null);

	onMount(() => {
		// Save defaults if they don't exist in localStorage
		if (!localStorage.getItem('task-instruction-templates')) {
			saveTemplates();
		}
	});


	function saveTemplates() {
		try {
			localStorage.setItem('task-instruction-templates', JSON.stringify(templates));
		} catch (error) {
			console.error('Failed to save templates:', error);
		}
	}

	function handleTemplateSelect(templateId: string) {
		const template = templates.find((t) => t.id === templateId);
		if (template) {
			selectedTemplateId = templateId;
			customInstruction = template.instruction;
			onTemplateSelect?.(templateId, template.instruction);
		}
	}

	function handleCreateTemplate() {
		if (!newTemplateName.trim() || !newTemplateInstruction.trim()) {
			toastActions.error('Please fill in both name and instruction');
			return;
		}

		const newTemplate: InstructionTemplate = {
			id: `template-${Date.now()}`,
			name: newTemplateName.trim(),
			instruction: newTemplateInstruction.trim(),
			createdAt: new Date(),
		};

		templates = [...templates, newTemplate];
		saveTemplates();
		toastActions.success('Template created successfully');

		// Select the new template
		handleTemplateSelect(newTemplate.id);

		// Reset form
		newTemplateName = '';
		newTemplateInstruction = '';
		showNewTemplateForm = false;
	}

	function handleDeleteTemplate(templateId: string) {
		const template = templates.find((t) => t.id === templateId);
		if (template?.isDefault) {
			toastActions.error('Cannot delete default templates');
			return;
		}

		if (confirm('Are you sure you want to delete this template?')) {
			templates = templates.filter((t) => t.id !== templateId);
			saveTemplates();
			if (selectedTemplateId === templateId) {
				selectedTemplateId = null;
				customInstruction = '';
			}
			toastActions.success('Template deleted');
		}
	}

	function handleEditTemplate(templateId: string) {
		const template = templates.find((t) => t.id === templateId);
		if (template) {
			editingTemplateId = templateId;
			newTemplateName = template.name;
			newTemplateInstruction = template.instruction;
			showNewTemplateForm = true;
		}
	}

	function handleUpdateTemplate() {
		if (!editingTemplateId || !newTemplateName.trim() || !newTemplateInstruction.trim()) {
			return;
		}

		const template = templates.find((t) => t.id === editingTemplateId);
		if (template?.isDefault) {
			toastActions.error('Cannot edit default templates');
			return;
		}

		templates = templates.map((t) =>
			t.id === editingTemplateId
				? { ...t, name: newTemplateName.trim(), instruction: newTemplateInstruction.trim() }
				: t
		);
		saveTemplates();
		toastActions.success('Template updated');

		// Update selected instruction if this template is selected
		if (selectedTemplateId === editingTemplateId) {
			customInstruction = newTemplateInstruction.trim();
			onInstructionChange?.(customInstruction);
		}

		// Reset form
		editingTemplateId = null;
		newTemplateName = '';
		newTemplateInstruction = '';
		showNewTemplateForm = false;
	}

	function handleCancelForm() {
		editingTemplateId = null;
		newTemplateName = '';
		newTemplateInstruction = '';
		showNewTemplateForm = false;
	}

	const templateOptions = $derived(templates.map((t) => ({
		value: t.id,
		label: t.name + (t.isDefault ? ' (Default)' : ''),
	})));
	
	const selectOptions = $derived([{ value: '', label: 'None (Custom)' }, ...templateOptions]);
</script>

<Collapsible bind:open={showTemplateManager}>
	<div class="space-y-2">
		<CollapsibleTrigger>
			<Button variant="outline" class="w-full justify-between">
				<span class="flex items-center gap-2">
					<Icon icon="lucide:file-text" class="h-4 w-4" />
					Custom Instructions
				</span>
				<Icon icon="lucide:chevron-down" class="h-4 w-4" />
			</Button>
		</CollapsibleTrigger>
		<CollapsibleContent>
			<div class="space-y-4 pt-2">
				<!-- Template Selection -->
				<div class="space-y-2">
					<Label>Instruction Template</Label>
					<Select
						options={selectOptions}
						defaultValue={selectedTemplateId || ''}
						onSelect={(value) => {
							if (value) {
								handleTemplateSelect(value);
							} else {
								selectedTemplateId = null;
								customInstruction = '';
							}
						}}
					/>
				</div>

				<!-- Custom Instruction -->
				<div class="space-y-2">
					<Label for="custom-instruction">Custom Instruction</Label>
					<Textarea
						id="custom-instruction"
						bind:value={customInstruction}
						placeholder="e.g., Focus on API endpoints only, break into smaller subtasks, prioritize security features..."
						rows={3}
						oninput={() => {
							selectedTemplateId = null;
						}}
					/>
					<p class="text-xs text-muted-foreground">
						Specify how you want the tasks to be generated (format, focus areas, structure, etc.).
					</p>
				</div>

				<!-- Template Management -->
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label>Manage Templates</Label>
						<Button
							variant="ghost"
							size="sm"
							onclick={() => {
								showNewTemplateForm = !showNewTemplateForm;
								if (!showNewTemplateForm) {
									handleCancelForm();
								}
							}}
						>
							<Icon icon={showNewTemplateForm ? 'lucide:minus' : 'lucide:plus'} class="h-4 w-4 mr-1" />
							{showNewTemplateForm ? 'Cancel' : 'New Template'}
						</Button>
					</div>

					{#if showNewTemplateForm}
						<Card>
							<CardContent class="pt-4 space-y-3">
								<div class="space-y-2">
									<Label for="template-name">Template Name</Label>
									<Input
										id="template-name"
										bind:value={newTemplateName}
										placeholder="e.g., API Development"
									/>
								</div>
								<div class="space-y-2">
									<Label for="template-instruction">Instruction</Label>
									<Textarea
										id="template-instruction"
										bind:value={newTemplateInstruction}
										placeholder="Enter the instruction text..."
										rows={3}
									/>
								</div>
								<div class="flex gap-2">
									<Button
										size="sm"
										onclick={editingTemplateId ? handleUpdateTemplate : handleCreateTemplate}
									>
										{editingTemplateId ? 'Update' : 'Create'} Template
									</Button>
									{#if editingTemplateId}
										<Button size="sm" variant="outline" onclick={handleCancelForm}>
											Cancel
										</Button>
									{/if}
								</div>
							</CardContent>
						</Card>
					{/if}

					<!-- Template List -->
					<div class="space-y-2 max-h-48 overflow-y-auto">
						{#each templates as template}
							<div class="flex items-center justify-between p-2 border rounded-lg">
								<div class="flex-1 min-w-0">
									<div class="flex items-center gap-2">
										<span class="font-medium text-sm truncate">{template.name}</span>
										{#if template.isDefault}
											<Badge variant="secondary" class="text-xs">Default</Badge>
										{/if}
									</div>
									<p class="text-xs text-muted-foreground truncate">{template.instruction}</p>
								</div>
								<div class="flex items-center gap-1 ml-2">
									{#if !template.isDefault}
										<Button
											variant="ghost"
											size="sm"
											onclick={() => handleEditTemplate(template.id)}
											class="h-7 w-7 p-0"
										>
											<Icon icon="lucide:pencil" class="h-3 w-3" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => handleDeleteTemplate(template.id)}
											class="h-7 w-7 p-0 text-destructive"
										>
											<Icon icon="lucide:trash-2" class="h-3 w-3" />
										</Button>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				</div>
			</div>
		</CollapsibleContent>
	</div>
</Collapsible>

