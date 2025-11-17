<!--
	Pipeline Builder Page - Dedicated page for building pipelines from templates
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/lib/components/ui/tabs';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { ArrowLeft, Sparkles, Code, Package, Rocket, Wrench } from '@lucide/svelte';
	import PipelineBuilder from '@/lib/domains/projects/pipelines/components/PipelineBuilder.svelte';
	import { pipelineTemplateService, type PipelineTemplate } from '@/lib/domains/projects/pipelines/services/pipelineTemplateService';
	import { pipelineService } from '@/lib/domains/projects/pipelines';
	import { projectService } from '@/lib/domains/projects/services/projectService';
	import type { Project, Pipeline } from '@/lib/domains/projects';
	import { toast } from 'svelte-sonner';

	const projectId = $derived($page.params.id);
	
	let project = $state<Project | null>(null);
	let selectedTemplate: PipelineTemplate | null = $state(null);
	let showBuilder = $state(false);
	let generatedPipeline: Omit<Pipeline, 'id' | 'created_at' | 'updated_at'> | null = $state(null);
	let activeTab = $state<'templates' | 'builder'>('templates');
	let loading = $state(false);

	onMount(async () => {
		if (projectId) {
			await loadProject();
		}
	});

	async function loadProject() {
		if (!projectId) return;
		try {
			loading = true;
			project = await projectService.getProject(projectId);
		} catch (error) {
			console.error('Failed to load project', error);
			toast.error('Failed to load project');
		} finally {
			loading = false;
		}
	}

	function handleSelectTemplate(template: PipelineTemplate) {
		selectedTemplate = template;
		if (project) {
			try {
				generatedPipeline = pipelineTemplateService.generatePipelineFromTemplate(
					template.key,
					projectId,
					project.name
				);
				showBuilder = true;
				activeTab = 'builder';
			} catch (error) {
				console.error('Failed to generate pipeline', error);
				toast.error('Failed to generate pipeline from template');
			}
		}
	}

	function handleStartFromScratch() {
		generatedPipeline = null;
		selectedTemplate = null;
		showBuilder = true;
		activeTab = 'builder';
	}

	function handleBuilderClose() {
		showBuilder = false;
		generatedPipeline = null;
		selectedTemplate = null;
		activeTab = 'templates';
	}

	async function handleSavePipeline(pipeline: Pipeline) {
		try {
			await pipelineService.createPipeline(pipeline);
			toast.success('Pipeline created successfully!');
			goto(`/projects/${projectId}/pipelines`);
		} catch (error) {
			console.error('Failed to save pipeline', error);
			toast.error('Failed to save pipeline');
		}
	}

	const recommendedTemplates = $derived(
		project ? pipelineTemplateService.getRecommendedTemplates(project.framework) : []
	);
	const allTemplates = $derived(pipelineTemplateService.getAllTemplates());

	function getCategoryIcon(category: string) {
		switch (category) {
			case 'build':
				return Package;
			case 'test':
				return Wrench;
			case 'deploy':
				return Rocket;
			case 'ci-cd':
				return Code;
			default:
				return Sparkles;
		}
	}

	function getCategoryColor(category: string) {
		switch (category) {
			case 'build':
				return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
			case 'test':
				return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
			case 'deploy':
				return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
			case 'ci-cd':
				return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
			default:
				return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
		}
	}
</script>

<svelte:head>
	<title>Create Pipeline - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto p-6 space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<Button variant="ghost" size="sm" onclick={() => goto(`/projects/${projectId}/pipelines`)}>
				<ArrowLeft class="h-4 w-4 mr-2" />
				Back to Pipelines
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Create New Pipeline</h1>
				{#if project}
					<p class="text-sm text-muted-foreground">For project: {project.name}</p>
				{/if}
			</div>
		</div>
		<Button variant="outline" onclick={handleStartFromScratch}>
			<Code class="h-4 w-4 mr-2" />
			Start from Scratch
		</Button>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<p class="text-muted-foreground">Loading project...</p>
		</div>
	{:else if showBuilder}
		<Tabs bind:value={activeTab} class="w-full">
			<TabsList>
				<TabsTrigger value="templates">Templates</TabsTrigger>
				<TabsTrigger value="builder">Pipeline Builder</TabsTrigger>
			</TabsList>
			
			<TabsContent value="templates" class="mt-6">
				<div class="space-y-6">
					{#if project?.framework && recommendedTemplates.length > 0}
						<div>
							<h2 class="text-lg font-semibold mb-4 flex items-center gap-2">
								<Sparkles class="h-5 w-5" />
								Recommended for {project.framework}
							</h2>
							<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
								{#each recommendedTemplates.filter((t) => t.framework === project.framework) as template}
									{@const CategoryIcon = getCategoryIcon(template.category)}
								<Card
									class="cursor-pointer hover:border-primary transition-colors {selectedTemplate?.key === template.key ? 'border-primary' : ''}"
									onclick={() => handleSelectTemplate(template)}
								>
										<CardHeader>
											<div class="flex items-start justify-between">
												<CardTitle class="text-lg">{template.name}</CardTitle>
												<Badge class={getCategoryColor(template.category)}>
													<CategoryIcon class="h-3 w-3 mr-1" />
													{template.category}
												</Badge>
											</div>
											<CardDescription>{template.description}</CardDescription>
										</CardHeader>
										<CardContent>
											<div class="space-y-2">
												<p class="text-sm text-muted-foreground">
													{template.steps.length} step{template.steps.length !== 1 ? 's' : ''}
												</p>
												{#if template.variables && template.variables.length > 0}
													<p class="text-xs text-muted-foreground">
														{template.variables.length} variable{template.variables.length !== 1 ? 's' : ''}
													</p>
												{/if}
											</div>
										</CardContent>
									</Card>
								{/each}
							</div>
						</div>
					{/if}

					<div>
						<h2 class="text-lg font-semibold mb-4">All Templates</h2>
						<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
							{#each allTemplates as template}
								{@const CategoryIcon = getCategoryIcon(template.category)}
								<Card
									class="cursor-pointer hover:border-primary transition-colors {selectedTemplate?.key === template.key ? 'border-primary' : ''}"
									onclick={() => handleSelectTemplate(template)}
								>
									<CardHeader>
										<div class="flex items-start justify-between">
											<CardTitle class="text-lg">{template.name}</CardTitle>
											<Badge class={getCategoryColor(template.category)}>
												<CategoryIcon class="h-3 w-3 mr-1" />
												{template.category}
											</Badge>
										</div>
										<CardDescription>{template.description}</CardDescription>
									</CardHeader>
									<CardContent>
										<div class="space-y-2">
											<div class="flex items-center gap-2">
												<Badge variant="outline" class="text-xs">{template.framework}</Badge>
											</div>
											<p class="text-sm text-muted-foreground">
												{template.steps.length} step{template.steps.length !== 1 ? 's' : ''}
											</p>
										</div>
									</CardContent>
								</Card>
							{/each}
						</div>
					</div>
				</div>
			</TabsContent>
			
			<TabsContent value="builder" class="mt-6">
				{#if generatedPipeline}
					<PipelineBuilder
						pipeline={generatedPipeline as any}
						projectId={projectId}
						onSave={handleSavePipeline}
						onCancel={handleBuilderClose}
					/>
				{:else}
					<PipelineBuilder
						projectId={projectId}
						onSave={handleSavePipeline}
						onCancel={handleBuilderClose}
					/>
				{/if}
			</TabsContent>
		</Tabs>
	{:else}
		<!-- Template Selection -->
		<Tabs value="templates" class="w-full">
			<TabsList>
				<TabsTrigger value="recommended">Recommended</TabsTrigger>
				<TabsTrigger value="all">All Templates</TabsTrigger>
			</TabsList>

			<TabsContent value="recommended" class="mt-6">
				{#if project?.framework && recommendedTemplates.length > 0}
					<div class="space-y-4">
						<div class="flex items-center gap-2 mb-4">
							<Sparkles class="h-5 w-5" />
							<h2 class="text-lg font-semibold">Recommended for {project.framework}</h2>
						</div>
						<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
							{#each recommendedTemplates.filter((t) => t.framework === project.framework) as template}
								{@const CategoryIcon = getCategoryIcon(template.category)}
								<Card
									class="cursor-pointer hover:border-primary transition-colors"
									onclick={() => handleSelectTemplate(template)}
								>
									<CardHeader>
										<div class="flex items-start justify-between">
											<CardTitle class="text-lg">{template.name}</CardTitle>
											<Badge class={getCategoryColor(template.category)}>
												<CategoryIcon class="h-3 w-3 mr-1" />
												{template.category}
											</Badge>
										</div>
										<CardDescription>{template.description}</CardDescription>
									</CardHeader>
									<CardContent>
										<div class="space-y-2">
											<p class="text-sm text-muted-foreground">
												{template.steps.length} step{template.steps.length !== 1 ? 's' : ''}
											</p>
											{#if template.variables && template.variables.length > 0}
												<p class="text-xs text-muted-foreground">
													{template.variables.length} variable{template.variables.length !== 1 ? 's' : ''}
												</p>
											{/if}
										</div>
									</CardContent>
								</Card>
							{/each}
						</div>
					</div>
				{:else}
					<Card>
						<CardContent class="py-12 text-center">
							<p class="text-muted-foreground mb-4">No framework detected. Browse all templates below.</p>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>

			<TabsContent value="all" class="mt-6">
				<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
					{#each allTemplates as template}
						{@const CategoryIcon = getCategoryIcon(template.category)}
						<Card
							class="cursor-pointer hover:border-primary transition-colors"
							onclick={() => handleSelectTemplate(template)}
						>
							<CardHeader>
								<div class="flex items-start justify-between">
									<CardTitle class="text-lg">{template.name}</CardTitle>
									<Badge class={getCategoryColor(template.category)}>
										<CategoryIcon class="h-3 w-3 mr-1" />
										{template.category}
									</Badge>
								</div>
								<CardDescription>{template.description}</CardDescription>
							</CardHeader>
							<CardContent>
								<div class="space-y-2">
									<div class="flex items-center gap-2">
										<Badge variant="outline" class="text-xs">{template.framework}</Badge>
									</div>
									<p class="text-sm text-muted-foreground">
										{template.steps.length} step{template.steps.length !== 1 ? 's' : ''}
									</p>
								</div>
							</CardContent>
						</Card>
					{/each}
				</div>
			</TabsContent>
		</Tabs>
	{/if}
</div>

