<script lang="ts">
  import { onMount } from 'svelte';
  import { automationStore } from '$lib/domains/automation/stores/automationStore';
  import WorkflowTrigger from '$lib/domains/automation/components/WorkflowTrigger.svelte';
  import WorkflowStatus from '$lib/domains/automation/components/WorkflowStatus.svelte';
  import type { WorkflowResult } from '$lib/domains/automation/types';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import Select from '$lib/components/ui/select.svelte';
  import { projectService } from '$lib/domains/projects/services/projectService';
  import type { Project } from '$lib/domains/projects/types';
  import { PageHeader, PageLoading } from '$lib/components/shell';
  import { setBreadcrumbs } from '$lib/domains/shared/stores/breadcrumbStore';

  setBreadcrumbs([
    { label: 'Automation', href: '/automation' },
    { label: 'Workflows', href: '/automation/workflows' },
  ]);

  let projects = $state<Project[]>([]);
  let loading = $state(true);
  let selectedProjectId = $state('');
  let executionIds = $state<string[]>([]);

  onMount(async () => {
    automationStore.checkHealth();
    try {
      projects = await projectService.loadProjects();
    } catch (error) {
      console.error('Failed to load projects:', error);
    } finally {
      loading = false;
    }
  });

  const selectedProject = $derived(projects.find((p) => p.id === selectedProjectId) ?? null);
  const projectOptions = $derived(projects.map((p) => ({ value: p.id, label: p.name })));

  function onWorkflowComplete(result: WorkflowResult) {
    if (result.execution_id) executionIds = [result.execution_id, ...executionIds];
  }
</script>

<svelte:head>
  <title>Workflows - Automation - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader
    title="n8n Workflows"
    description="Trigger n8n workflows for a project and watch their execution. Requires a running n8n server on localhost:5678 (pnpm n8n:start)."
  >
    {#snippet actions()}
      <Badge variant={$automationStore.isN8nHealthy ? 'default' : 'secondary'}>
        {$automationStore.isN8nHealthy ? 'n8n connected' : 'n8n offline'}
      </Badge>
    {/snippet}
  </PageHeader>

  {#if loading}
    <PageLoading message="Loading projects…" />
  {:else}
    <Card>
      <CardHeader>
        <CardTitle class="text-lg">Project</CardTitle>
        <CardDescription>Pick the project to run workflows against</CardDescription>
      </CardHeader>
      <CardContent>
        <Select
          bind:value={selectedProjectId}
          options={projectOptions}
          placeholder="Select a project…"
          class="max-w-md"
        />
      </CardContent>
    </Card>

    {#if selectedProject}
      <Card>
        <CardContent class="pt-6">
          <WorkflowTrigger project={selectedProject} {onWorkflowComplete} />
        </CardContent>
      </Card>
    {/if}

    {#if executionIds.length > 0}
      <div class="space-y-2">
        <h3 class="text-lg font-semibold">Executions</h3>
        {#each executionIds as executionId (executionId)}
          <WorkflowStatus {executionId} />
        {/each}
      </div>
    {/if}
  {/if}
</div>
