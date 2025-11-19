<!--
	Pipeline Management Page
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import PipelineBuilder from '@/lib/domains/projects/pipelines/components/PipelineBuilder.svelte';
	import ExecutionMonitor from '@/lib/domains/projects/pipelines/components/ExecutionMonitor.svelte';
	import type { Pipeline, PipelineExecution } from '@/lib/domains/projects/pipelines';
	import { pipelineService, pipelineStore, executionService } from '@/lib/domains/projects/pipelines';

	let projectId = $derived($page.params.id);
	let pipelines = $state<Pipeline[]>([]);
	let selectedPipeline: Pipeline | null = $state(null);
	let showBuilder = $state(false);
	let currentExecution: PipelineExecution | null = $state(null);
	let loading = $state(false);

	onMount(async () => {
		if (projectId) {
			await loadPipelines();
		}
	});

	async function loadPipelines() {
		if (!projectId) return;
		loading = true;
		try {
			pipelines = await pipelineService.getPipelines(projectId);
		} catch (error) {
			console.error('Failed to load pipelines', error);
		} finally {
			loading = false;
		}
	}

	async function handleCreatePipeline() {
		selectedPipeline = null;
		showBuilder = true;
	}

	async function handleEditPipeline(pipeline: Pipeline) {
		selectedPipeline = pipeline;
		showBuilder = true;
	}

	async function handleDeletePipeline(pipelineId: string) {
		if (confirm('Are you sure you want to delete this pipeline?')) {
			try {
				await pipelineService.deletePipeline(pipelineId);
				await loadPipelines();
			} catch (error) {
				console.error('Failed to delete pipeline', error);
			}
		}
	}

	async function handleExecutePipeline(pipelineId: string) {
		try {
			const execution = await executionService.executePipeline({ 
				pipelineId
			});
			currentExecution = execution;
		} catch (error) {
			console.error('Failed to execute pipeline', error);
		}
	}

	function handleBuilderClose() {
		showBuilder = false;
		selectedPipeline = null;
		loadPipelines();
	}
</script>

<div class="container mx-auto p-6 space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-bold">Pipelines</h1>
		<div class="flex gap-2">
			<Button variant="outline" onclick={() => goto(`/projects/${projectId}/pipelines/new`)}>
				Create from Template
			</Button>
			<Button onclick={handleCreatePipeline}>Create Pipeline</Button>
		</div>
	</div>

	{#if showBuilder}
		<PipelineBuilder
			pipeline={selectedPipeline || undefined}
			{projectId}
			onSave={handleBuilderClose}
			onCancel={handleBuilderClose}
		/>
	{:else if currentExecution}
		<ExecutionMonitor executionId={currentExecution.id} onClose={() => (currentExecution = null)} />
	{:else}
		{#if loading}
			<p class="text-center text-muted-foreground py-8">Loading pipelines...</p>
		{:else if pipelines.length === 0}
			<Card>
				<CardContent class="py-12 text-center">
					<p class="text-muted-foreground mb-4">No pipelines yet</p>
					<Button onclick={handleCreatePipeline}>Create Your First Pipeline</Button>
				</CardContent>
			</Card>
		{:else}
			<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
				{#each pipelines as pipeline (pipeline.id)}
					<Card>
						<CardHeader>
							<CardTitle>{pipeline.name}</CardTitle>
							{#if pipeline.description}
								<p class="text-sm text-muted-foreground">{pipeline.description}</p>
							{/if}
						</CardHeader>
						<CardContent class="space-y-2">
							<p class="text-sm text-muted-foreground">
								{pipeline.steps.length} step{pipeline.steps.length !== 1 ? 's' : ''}
							</p>
							<div class="flex gap-2">
								<Button
									size="sm"
									onclick={() => handleExecutePipeline(pipeline.id)}
									disabled={!pipeline.enabled}
								>
									Run
								</Button>
								<Button
									size="sm"
									variant="outline"
									onclick={() => handleEditPipeline(pipeline)}
								>
									Edit
								</Button>
								<Button
									size="sm"
									variant="destructive"
									onclick={() => handleDeletePipeline(pipeline.id)}
								>
									Delete
								</Button>
							</div>
						</CardContent>
					</Card>
				{/each}
			</div>
		{/if}
	{/if}
</div>

