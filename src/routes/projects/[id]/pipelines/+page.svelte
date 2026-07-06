<!--
	Pipeline Management Page
-->
<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { confirmAction } from "$lib/utils/confirm";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { ArrowLeft, Sparkles, Code } from "@lucide/svelte";
  import PipelineBuilder from "$lib/domains/projects/pipelines/components/PipelineBuilder.svelte";
  import ExecutionMonitor from "$lib/domains/projects/pipelines/components/ExecutionMonitor.svelte";
  import type {
    Pipeline,
    PipelineExecution,
    StarterPackStatus,
  } from "$lib/domains/projects/pipelines";
  import {
    pipelineService,
    executionService,
    frameworkStarterService,
  } from "$lib/domains/projects/pipelines";
  import { createProjectQuery } from "$lib/domains/projects";
  import { projectIconRegistry } from "$lib/domains/projects/utils/iconRegistry";
  import {
    PageHeader,
    PageLoading,
    PageEmpty,
  } from "$lib/components/shell";
  import { toast } from "$lib/utils/toast";

  let projectId = $derived($page.params.id);
  const projectQuery = createProjectQuery(() => projectId ?? null);
  const project = $derived(projectQuery.data ?? null);

  let pipelines = $state<Pipeline[]>([]);
  let selectedPipeline: Pipeline | null = $state(null);
  let showBuilder = $state(false);
  let currentExecution: PipelineExecution | null = $state(null);
  let loading = $state(false);
  let provisioningPipelines = $state(false);
  let starterPackStatus = $state<StarterPackStatus | null>(null);
  let autoProvisionAttempted = $state(false);
  let pipelinesInitialized = $state(false);

  $effect(() => {
    projectId;
    pipelinesInitialized = false;
    autoProvisionAttempted = false;
  });

  $effect(() => {
    if (!projectId || !project || pipelinesInitialized) return;
    pipelinesInitialized = true;
    void (async () => {
      await projectIconRegistry.ensureLoaded();
      await loadPipelines();
      await tryAutoProvision();
    })();
  });

  async function refreshStarterStatus() {
    if (!project) {
      starterPackStatus = null;
      return;
    }
    await projectIconRegistry.ensureLoaded();
    starterPackStatus = frameworkStarterService.getStarterPackForProject(
      project,
      pipelines,
    );
  }

  async function loadPipelines() {
    if (!projectId) return;
    loading = true;
    try {
      pipelines = await pipelineService.getPipelines(projectId);
      await refreshStarterStatus();
    } catch (error) {
      console.error("Failed to load pipelines", error);
    } finally {
      loading = false;
    }
  }

  async function tryAutoProvision() {
    if (!projectId || !project || autoProvisionAttempted) return;

    await projectIconRegistry.ensureLoaded();
    const status = frameworkStarterService.getStarterPackForProject(
      project,
      pipelines,
    );
    starterPackStatus = status;

    if (pipelines.length > 0 || !status || status.missingKeys.length === 0) {
      autoProvisionAttempted = true;
      return;
    }

    autoProvisionAttempted = true;
    provisioningPipelines = true;
    try {
      const created =
        await frameworkStarterService.autoProvisionStarterPipelines(
          projectId,
          project,
          pipelines,
        );
      await loadPipelines();
      if (created.length > 0) {
        toast.success(
          `Set up ${created.length} ${status.pack.displayName} pipeline(s)`,
        );
      }
    } catch (error) {
      console.error("Failed to auto-provision pipelines", error);
      toast.error("Failed to set up default pipelines");
    } finally {
      provisioningPipelines = false;
    }
  }

  async function handleSetupDefaultPipelines() {
    if (!projectId || !project) return;
    provisioningPipelines = true;
    try {
      const created =
        await frameworkStarterService.autoProvisionStarterPipelines(
          projectId,
          project,
          pipelines,
        );
      await loadPipelines();
      if (created.length > 0) {
        toast.success(`Created ${created.length} default pipeline(s)`);
      } else {
        toast.info("Default pipelines are already set up");
      }
    } catch (error) {
      console.error("Failed to provision pipelines", error);
      toast.error("Failed to set up default pipelines");
    } finally {
      provisioningPipelines = false;
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
    const confirmed = await confirmAction(
      "Are you sure you want to delete this pipeline?",
      "Delete pipeline",
    );
    if (!confirmed) return;

    try {
      await pipelineService.deletePipeline(pipelineId);
      await loadPipelines();
    } catch (error) {
      console.error("Failed to delete pipeline", error);
    }
  }

  async function handleExecutePipeline(pipelineId: string) {
    try {
      const execution = await executionService.executePipeline({
        pipelineId,
      });
      currentExecution = execution;
    } catch (error) {
      console.error("Failed to execute pipeline", error);
    }
  }

  function handleBuilderClose() {
    showBuilder = false;
    selectedPipeline = null;
    loadPipelines();
  }

  function categoryBadgeVariant(
    category: string | undefined,
  ): "default" | "secondary" | "outline" {
    switch (category) {
      case "install":
        return "secondary";
      case "dev":
        return "default";
      case "build":
        return "outline";
      default:
        return "outline";
    }
  }

  function runButtonLabel(pipeline: Pipeline): string {
    return pipeline.category === "dev" ? "Start" : "Run";
  }

  function handleBack() {
    goto(`/projects/${projectId}?tab=pipelines`);
  }
</script>

<svelte:head>
  <title
    >Pipelines{project ? ` - ${project.name}` : ""} - Portal Desktop</title
  >
</svelte:head>

<div class="container mx-auto space-y-6 p-6">
  <Button variant="ghost" size="sm" onclick={handleBack} class="w-fit gap-2">
    <ArrowLeft class="h-4 w-4" />
    Back to Project
  </Button>

  <PageHeader
    title="Pipelines"
    description={project
      ? `Manage automation pipelines for ${project.name}`
      : "Manage project pipelines"}
  >
    {#snippet actions()}
      {#if starterPackStatus && starterPackStatus.missingKeys.length > 0}
        <Button
          variant="default"
          disabled={provisioningPipelines}
          onclick={handleSetupDefaultPipelines}
          class="gap-2"
        >
          <Sparkles class="h-4 w-4" />
          {provisioningPipelines
            ? "Setting up..."
            : `Setup ${starterPackStatus.pack.displayName} pipelines`}
        </Button>
      {/if}
      <Button
        variant="outline"
        onclick={() => goto(`/projects/${projectId}/pipelines/new`)}
      >
        Create from Template
      </Button>
      <Button onclick={handleCreatePipeline}>Create Pipeline</Button>
    {/snippet}
  </PageHeader>

  {#if showBuilder}
    <PipelineBuilder
      pipeline={selectedPipeline || undefined}
      {projectId}
      onSave={handleBuilderClose}
      onCancel={handleBuilderClose}
    />
  {:else if currentExecution}
    <ExecutionMonitor
      executionId={currentExecution.id}
      onClose={() => (currentExecution = null)}
    />
  {:else if loading || provisioningPipelines}
    <PageLoading
      message={provisioningPipelines
        ? `Setting up ${starterPackStatus?.pack.displayName ?? "default"} pipelines…`
        : "Loading pipelines..."}
    />
  {:else if pipelines.length === 0}
    <PageEmpty
      title="No pipelines yet"
      description={starterPackStatus
        ? `Set up Install, Dev, and Build pipelines for ${starterPackStatus.pack.displayName}`
        : "Create a pipeline to automate install, dev, build, and deploy workflows for this project."}
      icon={Code}
      actionLabel={starterPackStatus
        ? "Setup default pipelines"
        : "Create pipeline"}
      onAction={starterPackStatus
        ? handleSetupDefaultPipelines
        : handleCreatePipeline}
    />
  {:else}
    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {#each pipelines as pipeline (pipeline.id)}
        <Card>
          <CardHeader>
            <div class="flex items-start justify-between gap-2">
              <CardTitle class="text-base">{pipeline.name}</CardTitle>
              {#if pipeline.category}
                <Badge variant={categoryBadgeVariant(pipeline.category)}>
                  {pipeline.category}
                </Badge>
              {/if}
            </div>
            {#if pipeline.description}
              <CardDescription>{pipeline.description}</CardDescription>
            {/if}
          </CardHeader>
          <CardContent class="space-y-3">
            <p class="text-sm text-muted-foreground">
              {pipeline.steps.length} step{pipeline.steps.length !== 1 ? "s" : ""}
            </p>
            <div class="flex flex-wrap gap-2">
              <Button
                size="sm"
                onclick={() => handleExecutePipeline(pipeline.id)}
                disabled={!pipeline.enabled}
              >
                {runButtonLabel(pipeline)}
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
</div>
