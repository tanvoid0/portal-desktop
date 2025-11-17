<!--
	Global Blocks Management Page
	Manage reusable pipeline blocks/steps/scripts
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { Textarea } from '@/lib/components/ui/textarea';
	import { Badge } from '@/lib/components/ui/badge';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/lib/components/ui/tabs';
	import * as Dialog from '@/lib/components/ui/dialog';
	import { Label } from '@/lib/components/ui/label';
	import Select from '@/lib/components/ui/select.svelte';
	import { Plus, Search, Upload, Download, Edit, Trash2, Package, Code, Rocket, Wrench, Settings, FileText } from '@lucide/svelte';
	import type { Block, CreateBlockRequest } from '@/lib/domains/projects/pipelines';
	import { blockLibraryService, blockLibraryStore, blocks as blocksStore } from '@/lib/domains/projects/pipelines';
	import { pipelineTemplateService } from '@/lib/domains/projects/pipelines/services/pipelineTemplateService';
	import type { PipelineTemplate } from '@/lib/domains/projects/pipelines';
	import { toast } from 'svelte-sonner';

	let blocks = $state<Block[]>([]);
	let loading = $state(false);
	let searchQuery = $state('');
	let selectedCategory = $state<Block['category'] | null>(null);
	let showCreateDialog = $state(false);
	let showEditDialog = $state(false);
	let showImportDialog = $state(false);
	let editingBlock: Block | null = $state(null);

	// Template state
	let templates = $state<PipelineTemplate[]>([]);
	let templateSearchQuery = $state('');
	let activeTab = $state<'blocks' | 'templates'>('blocks');
	let showTemplateImportDialog = $state(false);
	let showTemplateExportDialog = $state(false);
	let selectedTemplateForExport: PipelineTemplate | null = $state(null);

	// Form state
	let formData = $state<CreateBlockRequest>({
		name: '',
		description: '',
		category: 'utility',
		parameters: [],
		command: '',
		executionType: 'command',
		defaultConfig: {},
		tags: [],
	});

	onMount(async () => {
		await loadBlocks();
		loadTemplates();
	});

	async function loadBlocks() {
		loading = true;
		try {
			await blockLibraryStore.loadBlocks();
			// Subscribe to blocks store
			const unsubscribe = blocksStore.subscribe((b) => {
				blocks = b;
			});
			// Keep subscription active
			return () => unsubscribe();
		} catch (error) {
			console.error('Failed to load blocks', error);
			toast.error('Failed to load blocks');
		} finally {
			loading = false;
		}
	}

	// Subscribe to blocks store
	$effect(() => {
		const unsubscribe = blocksStore.subscribe((b) => {
			blocks = b;
		});
		return unsubscribe;
	});

	function handleCreateBlock() {
		editingBlock = null;
		formData = {
			name: '',
			description: '',
			category: 'utility',
			parameters: [],
			command: '',
			executionType: 'command',
			defaultConfig: {},
			tags: [],
		};
		showCreateDialog = true;
	}

	function handleEditBlock(block: Block) {
		editingBlock = block;
		formData = {
			name: block.name,
			description: block.description,
			category: block.category,
			parameters: block.parameters,
			command: block.command,
			executionType: block.executionType,
			defaultConfig: block.defaultConfig,
			tags: block.tags || [],
		};
		showEditDialog = true;
	}

	async function handleSaveBlock() {
		try {
			if (editingBlock) {
				await blockLibraryService.updateBlock(editingBlock.id, formData);
				toast.success('Block updated successfully');
			} else {
				await blockLibraryService.createBlock(formData);
				toast.success('Block created successfully');
			}
			showCreateDialog = false;
			showEditDialog = false;
			await loadBlocks();
		} catch (error) {
			console.error('Failed to save block', error);
			toast.error('Failed to save block');
		}
	}

	async function handleDeleteBlock(blockId: string) {
		if (!confirm('Are you sure you want to delete this block?')) return;
		try {
			await blockLibraryService.deleteBlock(blockId);
			toast.success('Block deleted successfully');
			await loadBlocks();
		} catch (error) {
			console.error('Failed to delete block', error);
			toast.error('Failed to delete block');
		}
	}

	function handleImportBlock() {
		// Trigger the hidden file input
		const fileInput = document.getElementById('block-import-input') as HTMLInputElement;
		if (fileInput) {
			fileInput.click();
		}
	}

	async function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		try {
			const content = await file.text();
			let blockData: CreateBlockRequest;

			if (file.name.endsWith('.json')) {
				blockData = JSON.parse(content);
			} else {
				// For YAML, we'd need a YAML parser
				toast.error('YAML import not yet supported. Please use JSON format.');
				return;
			}

			await blockLibraryService.createBlock(blockData);
			toast.success('Block imported successfully');
			showImportDialog = false;
			await loadBlocks();
		} catch (error) {
			console.error('Failed to import block', error);
			toast.error('Failed to import block');
		} finally {
			// Reset the input
			if (target) {
				target.value = '';
			}
		}
	}

	async function handleExportBlock(block: Block) {
		try {
			const dataStr = JSON.stringify(block, null, 2);
			const dataBlob = new Blob([dataStr], { type: 'application/json' });
			const url = URL.createObjectURL(dataBlob);
			const link = document.createElement('a');
			link.href = url;
			link.download = `${block.name.toLowerCase().replace(/\s+/g, '-')}.json`;
			link.click();
			URL.revokeObjectURL(url);
			toast.success('Block exported successfully');
		} catch (error) {
			console.error('Failed to export block', error);
			toast.error('Failed to export block');
		}
	}

	const filteredBlocks = $derived(() => {
		let filtered: Block[] = blocks;

		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			filtered = filtered.filter(
				(block: Block) =>
					block.name.toLowerCase().includes(query) ||
					block.description.toLowerCase().includes(query) ||
					(block.tags || []).some((tag: string) => tag.toLowerCase().includes(query))
			);
		}

		if (selectedCategory) {
			filtered = filtered.filter((block: Block) => block.category === selectedCategory);
		}

		return filtered;
	});

	function getCategoryIcon(category: Block['category']) {
		switch (category) {
			case 'build':
				return Package;
			case 'test':
				return Wrench;
			case 'deploy':
				return Rocket;
			case 'utility':
				return Settings;
			default:
				return Code;
		}
	}

	function getCategoryColor(category: Block['category']) {
		switch (category) {
			case 'build':
				return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
			case 'test':
				return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
			case 'deploy':
				return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
			case 'utility':
				return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
			default:
				return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
		}
	}

	// Template functions
	function loadTemplates() {
		templates = pipelineTemplateService.getAllTemplates();
	}

	function handleExportTemplate(template: PipelineTemplate) {
		try {
			const json = pipelineTemplateService.exportTemplate(template.key);
			const dataBlob = new Blob([json], { type: 'application/json' });
			const url = URL.createObjectURL(dataBlob);
			const link = document.createElement('a');
			link.href = url;
			link.download = `${template.key}.json`;
			link.click();
			URL.revokeObjectURL(url);
			toast.success('Template exported successfully');
		} catch (error) {
			console.error('Failed to export template', error);
			toast.error('Failed to export template');
		}
	}

	async function handleImportTemplate() {
		const fileInput = document.getElementById('template-import-input') as HTMLInputElement;
		if (fileInput) {
			fileInput.click();
		}
	}

	async function handleTemplateFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		try {
			const content = await file.text();
			const template = pipelineTemplateService.importTemplate(content);
			toast.success('Template imported successfully');
			showTemplateImportDialog = false;
			loadTemplates();
		} catch (error) {
			console.error('Failed to import template', error);
			toast.error(error instanceof Error ? error.message : 'Failed to import template');
		} finally {
			if (target) {
				target.value = '';
			}
		}
	}

	async function handleDeleteTemplate(template: PipelineTemplate) {
		if (!confirm(`Are you sure you want to delete template "${template.name}"?`)) return;
		
		try {
			pipelineTemplateService.deleteTemplate(template.key);
			toast.success('Template deleted successfully');
			loadTemplates();
		} catch (error) {
			console.error('Failed to delete template', error);
			toast.error(error instanceof Error ? error.message : 'Failed to delete template');
		}
	}

	const filteredTemplates = $derived(() => {
		if (!templateSearchQuery) return templates;
		const query = templateSearchQuery.toLowerCase();
		return templates.filter(
			(t) =>
				t.name.toLowerCase().includes(query) ||
				t.description.toLowerCase().includes(query) ||
				t.key.toLowerCase().includes(query) ||
				(t.framework && t.framework.toLowerCase().includes(query))
		);
	});
</script>

<svelte:head>
	<title>Blocks Management - Portal Desktop</title>
</svelte:head>

	<div class="container mx-auto p-6 space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Pipeline Blocks & Templates</h1>
			<p class="text-muted-foreground mt-1">
				Manage reusable blocks, steps, scripts, and pipeline templates
			</p>
		</div>
	</div>

	<!-- Tabs -->
	<Tabs bind:value={activeTab} class="w-full">
		<TabsList>
			<TabsTrigger value="blocks">Blocks</TabsTrigger>
			<TabsTrigger value="templates">Templates</TabsTrigger>
		</TabsList>

		<!-- Blocks Tab -->
		<TabsContent value="blocks" class="space-y-6">
			<div class="flex items-center justify-between">
				<div class="flex gap-2">
					<Button variant="outline" onclick={() => (showImportDialog = true)}>
						<Upload class="h-4 w-4 mr-2" />
						Import Block
					</Button>
					<Button onclick={handleCreateBlock}>
						<Plus class="h-4 w-4 mr-2" />
						Create Block
					</Button>
				</div>
			</div>

			<!-- Search and Filters -->
			<div class="flex gap-4">
				<div class="flex-1">
					<Input
						bind:value={searchQuery}
						placeholder="Search blocks..."
						class="w-full"
					>
						<Search class="h-4 w-4 absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground" />
					</Input>
				</div>
				<div class="flex gap-2">
					<Button
						variant={selectedCategory === null ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = null)}
					>
						All
					</Button>
					<Button
						variant={selectedCategory === 'build' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = 'build')}
					>
						Build
					</Button>
					<Button
						variant={selectedCategory === 'test' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = 'test')}
					>
						Test
					</Button>
					<Button
						variant={selectedCategory === 'deploy' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = 'deploy')}
					>
						Deploy
					</Button>
					<Button
						variant={selectedCategory === 'utility' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = 'utility')}
					>
						Utility
					</Button>
				</div>
			</div>

			<!-- Blocks Grid -->
			{#if loading}
				<div class="flex items-center justify-center py-12">
					<p class="text-muted-foreground">Loading blocks...</p>
				</div>
			{:else if filteredBlocks().length === 0}
				<Card>
					<CardContent class="py-12 text-center">
						<p class="text-muted-foreground mb-4">No blocks found</p>
						<Button onclick={handleCreateBlock}>Create Your First Block</Button>
					</CardContent>
				</Card>
			{:else}
				<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
					{#each filteredBlocks() as block (block.id)}
						{@const CategoryIcon = getCategoryIcon(block.category)}
						<Card>
							<CardHeader>
								<div class="flex items-start justify-between">
									<div class="flex-1">
										<CardTitle class="text-lg">{block.name}</CardTitle>
										<CardDescription class="mt-1">{block.description}</CardDescription>
									</div>
									<Badge class={getCategoryColor(block.category)}>
										<CategoryIcon class="h-3 w-3 mr-1" />
										{block.category}
									</Badge>
								</div>
							</CardHeader>
							<CardContent class="space-y-4">
								<div class="space-y-2">
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<span>Type:</span>
										<Badge variant="outline">{block.executionType}</Badge>
									</div>
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<span>Version:</span>
										<span class="font-medium">{block.version}</span>
									</div>
									{#if block.parameters.length > 0}
										<div class="flex items-center gap-2 text-sm text-muted-foreground">
											<span>Parameters:</span>
											<span class="font-medium">{block.parameters.length}</span>
										</div>
									{/if}
								</div>

								{#if block.tags && block.tags.length > 0}
									<div class="flex flex-wrap gap-1">
										{#each block.tags.slice(0, 3) as tag}
											<Badge variant="outline" class="text-xs">{tag}</Badge>
										{/each}
										{#if block.tags.length > 3}
											<Badge variant="outline" class="text-xs">+{block.tags.length - 3}</Badge>
										{/if}
									</div>
								{/if}

								<div class="flex gap-2 pt-2 border-t">
									<Button
										size="sm"
										variant="outline"
										onclick={() => handleEditBlock(block)}
										class="flex-1"
									>
										<Edit class="h-3 w-3 mr-1" />
										Edit
									</Button>
									<Button
										size="sm"
										variant="outline"
										onclick={() => handleExportBlock(block)}
									>
										<Download class="h-3 w-3" />
									</Button>
									<Button
										size="sm"
										variant="destructive"
										onclick={() => handleDeleteBlock(block.id)}
									>
										<Trash2 class="h-3 w-3" />
									</Button>
								</div>
							</CardContent>
						</Card>
					{/each}
				</div>
			{/if}
		</TabsContent>

		<!-- Templates Tab -->
		<TabsContent value="templates" class="space-y-6">
			<div class="flex items-center justify-between">
				<div class="flex gap-2">
					<Button variant="outline" onclick={handleImportTemplate}>
						<Upload class="h-4 w-4 mr-2" />
						Import Template
					</Button>
				</div>
			</div>

			<!-- Template Search -->
			<div class="flex-1">
				<Input
					bind:value={templateSearchQuery}
					placeholder="Search templates..."
					class="w-full"
				>
					<Search class="h-4 w-4 absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground" />
				</Input>
			</div>

			<!-- Templates Grid -->
			{#if filteredTemplates().length === 0}
				<Card>
					<CardContent class="py-12 text-center">
						<p class="text-muted-foreground mb-4">No templates found</p>
					</CardContent>
				</Card>
			{:else}
				<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
					{#each filteredTemplates() as template (template.key)}
						<Card>
							<CardHeader>
								<div class="flex items-start justify-between">
									<div class="flex-1">
										<CardTitle class="text-lg">{template.name}</CardTitle>
										<CardDescription class="mt-1">{template.description}</CardDescription>
									</div>
									{#if !template.id}
										<Badge variant="outline" class="text-xs">Built-in</Badge>
									{/if}
								</div>
							</CardHeader>
							<CardContent class="space-y-4">
								<div class="space-y-2">
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<span>Framework:</span>
										<Badge variant="outline">{template.framework || 'N/A'}</Badge>
									</div>
									{#if template.category}
										<div class="flex items-center gap-2 text-sm text-muted-foreground">
											<span>Category:</span>
											<Badge variant="outline">{template.category}</Badge>
										</div>
									{/if}
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<span>Steps:</span>
										<span class="font-medium">{template.steps.length}</span>
									</div>
									{#if template.variables && Array.isArray(template.variables) && template.variables.length > 0}
										<div class="flex items-center gap-2 text-sm text-muted-foreground">
											<span>Variables:</span>
											<span class="font-medium">{template.variables.length}</span>
										</div>
									{/if}
								</div>

								{#if template.tags && Array.isArray(template.tags) && template.tags.length > 0}
									<div class="flex flex-wrap gap-1">
										{#each template.tags.slice(0, 3) as tag}
											<Badge variant="outline" class="text-xs">{tag}</Badge>
										{/each}
										{#if template.tags.length > 3}
											<Badge variant="outline" class="text-xs">+{template.tags.length - 3}</Badge>
										{/if}
									</div>
								{:else if template.framework}
									<Badge variant="outline" class="text-xs">{template.framework}</Badge>
								{/if}

								<div class="flex gap-2 pt-2 border-t">
									<Button
										size="sm"
										variant="outline"
										onclick={() => handleExportTemplate(template)}
										class="flex-1"
									>
										<Download class="h-3 w-3 mr-1" />
										Export
									</Button>
									{#if template.id}
										<Button
											size="sm"
											variant="destructive"
											onclick={() => handleDeleteTemplate(template)}
										>
											<Trash2 class="h-3 w-3" />
										</Button>
									{/if}
								</div>
							</CardContent>
						</Card>
					{/each}
				</div>
			{/if}
		</TabsContent>
	</Tabs>
</div>

<!-- Create/Edit Block Dialog -->
<Dialog.Root bind:open={showCreateDialog}>
	<Dialog.Content class="max-w-2xl max-h-[90vh] overflow-y-auto">
		<Dialog.Header>
			<Dialog.Title>{editingBlock ? 'Edit Block' : 'Create New Block'}</Dialog.Title>
			<Dialog.Description>
				Define a reusable block that can be used in pipeline steps
			</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div>
					<Label for="name">Name *</Label>
					<Input id="name" bind:value={formData.name} placeholder="e.g., Install Dependencies" />
				</div>
				<div>
					<Label for="category">Category *</Label>
					<Select
						options={[
							{ value: 'build', label: 'Build' },
							{ value: 'test', label: 'Test' },
							{ value: 'deploy', label: 'Deploy' },
							{ value: 'utility', label: 'Utility' },
							{ value: 'custom', label: 'Custom' },
						]}
						bind:value={formData.category}
						placeholder="Select category"
					/>
				</div>
			</div>
			<div>
				<Label for="description">Description *</Label>
				<Textarea
					id="description"
					bind:value={formData.description}
					placeholder="Describe what this block does..."
					rows={3}
				/>
			</div>
			<div class="grid grid-cols-2 gap-4">
				<div>
					<Label for="executionType">Execution Type *</Label>
					<Select
						options={[
							{ value: 'command', label: 'Command' },
							{ value: 'script', label: 'Script' },
							{ value: 'docker', label: 'Docker' },
						]}
						bind:value={formData.executionType}
						placeholder="Select execution type"
					/>
				</div>
				<div>
					<Label for="command">Command *</Label>
					<Input
						id="command"
						bind:value={formData.command}
						placeholder="e.g., npm install"
					/>
				</div>
			</div>
			<div>
				<Label for="tags">Tags (comma-separated)</Label>
				<Input
					id="tags"
					value={(formData.tags || []).join(', ')}
					oninput={(e) => {
						formData.tags = (e.target as HTMLInputElement).value
							.split(',')
							.map((t) => t.trim())
							.filter((t) => t.length > 0);
					}}
					placeholder="e.g., node, npm, install"
				/>
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showCreateDialog = false)}>Cancel</Button>
			<Button onclick={handleSaveBlock} disabled={!formData.name || !formData.description || !formData.command}>
				{editingBlock ? 'Update' : 'Create'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={showEditDialog}>
	<Dialog.Content class="max-w-2xl max-h-[90vh] overflow-y-auto">
		<Dialog.Header>
			<Dialog.Title>Edit Block</Dialog.Title>
			<Dialog.Description>Update block details</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div>
					<Label for="edit-name">Name *</Label>
					<Input id="edit-name" bind:value={formData.name} />
				</div>
				<div>
					<Label for="edit-category">Category *</Label>
					<Select
						options={[
							{ value: 'build', label: 'Build' },
							{ value: 'test', label: 'Test' },
							{ value: 'deploy', label: 'Deploy' },
							{ value: 'utility', label: 'Utility' },
							{ value: 'custom', label: 'Custom' },
						]}
						bind:value={formData.category}
						placeholder="Select category"
					/>
				</div>
			</div>
			<div>
				<Label for="edit-description">Description *</Label>
				<Textarea id="edit-description" bind:value={formData.description} rows={3} />
			</div>
			<div class="grid grid-cols-2 gap-4">
				<div>
					<Label for="edit-executionType">Execution Type *</Label>
					<Select
						options={[
							{ value: 'command', label: 'Command' },
							{ value: 'script', label: 'Script' },
							{ value: 'docker', label: 'Docker' },
						]}
						bind:value={formData.executionType}
						placeholder="Select execution type"
					/>
				</div>
				<div>
					<Label for="edit-command">Command *</Label>
					<Input id="edit-command" bind:value={formData.command} />
				</div>
			</div>
			<div>
				<Label for="edit-tags">Tags (comma-separated)</Label>
				<Input
					id="edit-tags"
					value={(formData.tags || []).join(', ')}
					oninput={(e) => {
						formData.tags = (e.target as HTMLInputElement).value
							.split(',')
							.map((t) => t.trim())
							.filter((t) => t.length > 0);
					}}
				/>
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showEditDialog = false)}>Cancel</Button>
			<Button onclick={handleSaveBlock} disabled={!formData.name || !formData.description || !formData.command}>
				Update
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Import Dialog -->
<Dialog.Root bind:open={showImportDialog}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Import Block</Dialog.Title>
			<Dialog.Description>Import a block from a JSON file</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4">
			<p class="text-sm text-muted-foreground">
				Select a JSON file containing block definition. The file should match the CreateBlockRequest format.
			</p>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showImportDialog = false)}>Cancel</Button>
			<Button onclick={handleImportBlock}>
				<Upload class="h-4 w-4 mr-2" />
				Select File
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

