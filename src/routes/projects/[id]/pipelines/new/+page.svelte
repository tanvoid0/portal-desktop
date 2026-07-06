<!--
	Pipeline Builder Page - Dedicated page for building pipelines from templates
-->
<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import {
    ArrowLeft,
    Sparkles,
    Code,
    Package,
    Rocket,
    Wrench,
  } from "@lucide/svelte";
  import PipelineBuilder from "$lib/domains/projects/pipelines/components/PipelineBuilder.svelte";
  import {
    pipelineTemplateService,
    type PipelineTemplate,
  } from "$lib/domains/projects/pipelines/services/pipelineTemplateService";
  import { pipelineService } from "$lib/domains/projects/pipelines";
  import { createProjectQuery } from "$lib/domains/projects/queries/projectQueries";
  import { projectIconRegistry } from "$lib/domains/projects/utils/iconRegistry";
  import { getProjectFramework } from "$lib/domains/projects/utils/display";
  import {
    PageHeader,
    PageLoading,
    PageEmpty,
  } from "$lib/components/shell";
  import type { Pipeline } from "$lib/domains/projects/pipelines/types";
  import { toast } from "$lib/utils/toast";

  const projectId = $derived($page.params.id);
  const projectQuery = createProjectQuery(() => projectId);

  let selectedTemplate: PipelineTemplate | null = $state(null);
  let showBuilder = $state(false);
  let generatedPipeline: Omit<
    Pipeline,
    "id" | "created_at" | "updated_at"
  > | null = $state(null);
  let activeTab = $state<"templates" | "builder">("templates");

  const project = $derived(projectQuery.data ?? null);
  const loading = $derived(projectQuery.isPending);

  let registryReady = $state(false);

  $effect(() => {
    void projectIconRegistry.ensureLoaded().then(() => {
      registryReady = true;
    });
  });

  const projectFramework = $derived.by(() => {
    if (!project || !registryReady) return "";
    const frameworks = projectIconRegistry.resolveFrameworks(project);
    if (frameworks.length > 0) return frameworks[0].name;
    return getProjectFramework(project) ?? "";
  });

  function handleSelectTemplate(template: PipelineTemplate) {
    selectedTemplate = template;
    if (project) {
      try {
        generatedPipeline =
          pipelineTemplateService.generatePipelineFromTemplate(
            template.key,
            projectId,
            project.name,
          );
        showBuilder = true;
        activeTab = "builder";
      } catch (error) {
        console.error("Failed to generate pipeline", error);
        toast.error("Failed to generate pipeline from template");
      }
    }
  }

  function handleStartFromScratch() {
    generatedPipeline = null;
    selectedTemplate = null;
    showBuilder = true;
    activeTab = "builder";
  }

  function handleBuilderClose() {
    showBuilder = false;
    generatedPipeline = null;
    selectedTemplate = null;
    activeTab = "templates";
  }

  async function handleSavePipeline(pipeline: Pipeline) {
    try {
      await pipelineService.createPipeline(pipeline);
      toast.success("Pipeline created successfully!");
      goto(`/projects/${projectId}/pipelines`);
    } catch (error) {
      console.error("Failed to save pipeline", error);
      toast.error("Failed to save pipeline");
    }
  }

  const recommendedTemplates = $derived(
    projectFramework
      ? pipelineTemplateService.getRecommendedTemplates(projectFramework)
      : [],
  );
  const allTemplates = $derived(pipelineTemplateService.getAllTemplates());

  function getCategoryIcon(category: string | undefined) {
    if (!category) return Code;
    switch (category) {
      case "build":
        return Package;
      case "test":
        return Wrench;
      case "deploy":
        return Rocket;
      case "ci-cd":
        return Code;
      default:
        return Sparkles;
    }
  }

  function getCategoryColor(category: string | undefined) {
    if (!category)
      return "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200";
    switch (category) {
      case "build":
        return "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200";
      case "test":
        return "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200";
      case "deploy":
        return "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200";
      case "ci-cd":
        return "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200";
      default:
        return "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200";
    }
  }
</script>

<svelte:head>
  <title>Create Pipeline - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto space-y-6 p-6">
  <Button
    variant="ghost"
    size="sm"
    onclick={() => goto(`/projects/${projectId}?tab=pipelines`)}
    class="w-fit gap-2"
  >
    <ArrowLeft class="h-4 w-4" />
    Back to Pipelines
  </Button>

  <PageHeader
    title="Create Pipeline"
    description={project ? `For project: ${project.name}` : undefined}
  >
    {#snippet actions()}
      <Button variant="outline" onclick={handleStartFromScratch} class="gap-2">
        <Code class="h-4 w-4" />
        Start from Scratch
      </Button>
    {/snippet}
  </PageHeader>

  {#if loading}
    <PageLoading message="Loading project..." />
  {:else if showBuilder}
    <Tabs bind:value={activeTab} class="w-full">
      <TabsList>
        <TabsTrigger value="templates">Templates</TabsTrigger>
        <TabsTrigger value="builder">Pipeline Builder</TabsTrigger>
      </TabsList>

      <TabsContent value="templates" class="mt-6">
        <div class="space-y-6">
          {#if projectFramework && recommendedTemplates.length > 0}
            <div>
              <h2 class="mb-4 flex items-center gap-2 text-lg font-semibold">
                <Sparkles class="h-5 w-5" />
                Recommended for {projectFramework}
              </h2>
              <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {#each recommendedTemplates.filter((t) => t.framework === projectFramework) as template}
                  {@const CategoryIcon = getCategoryIcon(template.category)}
                  <Card
                    class="cursor-pointer transition-colors hover:border-primary {selectedTemplate?.key ===
                    template.key
                      ? 'border-primary'
                      : ''}"
                    onclick={() => handleSelectTemplate(template)}
                  >
                    <CardHeader>
                      <div class="flex items-start justify-between">
                        <CardTitle class="text-lg">{template.name}</CardTitle>
                        <Badge class={getCategoryColor(template.category)}>
                          <CategoryIcon class="mr-1 h-3 w-3" />
                          {template.category}
                        </Badge>
                      </div>
                      <CardDescription>{template.description}</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div class="space-y-2">
                        <p class="text-sm text-muted-foreground">
                          {template.steps.length} step{template.steps.length !==
                          1
                            ? "s"
                            : ""}
                        </p>
                        {#if template.variables && template.variables.length > 0}
                          <p class="text-xs text-muted-foreground">
                            {template.variables.length} variable{template
                              .variables.length !== 1
                              ? "s"
                              : ""}
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
            <h2 class="mb-4 text-lg font-semibold">All Templates</h2>
            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {#each allTemplates as template}
                {@const CategoryIcon = getCategoryIcon(template.category)}
                <Card
                  class="cursor-pointer transition-colors hover:border-primary {selectedTemplate?.key ===
                  template.key
                    ? 'border-primary'
                    : ''}"
                  onclick={() => handleSelectTemplate(template)}
                >
                  <CardHeader>
                    <div class="flex items-start justify-between">
                      <CardTitle class="text-lg">{template.name}</CardTitle>
                      <Badge class={getCategoryColor(template.category)}>
                        <CategoryIcon class="mr-1 h-3 w-3" />
                        {template.category}
                      </Badge>
                    </div>
                    <CardDescription>{template.description}</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div class="space-y-2">
                      <div class="flex items-center gap-2">
                        <Badge variant="outline" class="text-xs"
                          >{template.framework}</Badge
                        >
                      </div>
                      <p class="text-sm text-muted-foreground">
                        {template.steps.length} step{template.steps.length !== 1
                          ? "s"
                          : ""}
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
            {projectId}
            onSave={handleSavePipeline}
            onCancel={handleBuilderClose}
          />
        {:else}
          <PipelineBuilder
            {projectId}
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
        {#if projectFramework && recommendedTemplates.length > 0}
          <div class="space-y-4">
            <div class="mb-4 flex items-center gap-2">
              <Sparkles class="h-5 w-5" />
              <h2 class="text-lg font-semibold">
                Recommended for {projectFramework}
              </h2>
            </div>
            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {#each recommendedTemplates.filter((t) => t.framework === projectFramework) as template}
                {@const CategoryIcon = getCategoryIcon(template.category)}
                <Card
                  class="cursor-pointer transition-colors hover:border-primary"
                  onclick={() => handleSelectTemplate(template)}
                >
                  <CardHeader>
                    <div class="flex items-start justify-between">
                      <CardTitle class="text-lg">{template.name}</CardTitle>
                      <Badge class={getCategoryColor(template.category)}>
                        <CategoryIcon class="mr-1 h-3 w-3" />
                        {template.category}
                      </Badge>
                    </div>
                    <CardDescription>{template.description}</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div class="space-y-2">
                      <p class="text-sm text-muted-foreground">
                        {template.steps.length} step{template.steps.length !== 1
                          ? "s"
                          : ""}
                      </p>
                      {#if template.variables && template.variables.length > 0}
                        <p class="text-xs text-muted-foreground">
                          {template.variables.length} variable{template
                            .variables.length !== 1
                            ? "s"
                            : ""}
                        </p>
                      {/if}
                    </div>
                  </CardContent>
                </Card>
              {/each}
            </div>
          </div>
        {:else}
          <PageEmpty
            title="No recommended templates"
            description="No framework detected for this project. Browse all templates or start from scratch."
            icon={Sparkles}
          />
        {/if}
      </TabsContent>

      <TabsContent value="all" class="mt-6">
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {#each allTemplates as template}
            {@const CategoryIcon = getCategoryIcon(template.category)}
            <Card
              class="cursor-pointer transition-colors hover:border-primary"
              onclick={() => handleSelectTemplate(template)}
            >
              <CardHeader>
                <div class="flex items-start justify-between">
                  <CardTitle class="text-lg">{template.name}</CardTitle>
                  <Badge class={getCategoryColor(template.category)}>
                    <CategoryIcon class="mr-1 h-3 w-3" />
                    {template.category}
                  </Badge>
                </div>
                <CardDescription>{template.description}</CardDescription>
              </CardHeader>
              <CardContent>
                <div class="space-y-2">
                  <div class="flex items-center gap-2">
                    <Badge variant="outline" class="text-xs"
                      >{template.framework}</Badge
                    >
                  </div>
                  <p class="text-sm text-muted-foreground">
                    {template.steps.length} step{template.steps.length !== 1
                      ? "s"
                      : ""}
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
