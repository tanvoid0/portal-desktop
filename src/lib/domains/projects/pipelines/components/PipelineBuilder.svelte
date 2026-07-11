<!--
	Pipeline Builder - Visual pipeline builder with drag-and-drop
-->
<script lang="ts">
  import Select from "$lib/components/ui/select.svelte";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Label } from "$lib/components/ui/label";
  import type { Pipeline, PipelineStep, ExecutionContext } from "../types";
  import { pipelineService } from "../services/pipelineService";
  import { blockLibraryService } from "../services/blockLibraryService";
  import { createStepFromBlock } from "$lib/domains/automation/utils/blockResolver";
  import { validateDependencies } from "../utils/dependencyResolver";
  import BlockLibrary from "./BlockLibrary.svelte";

  interface Props {
    pipeline?: Pipeline;
    projectId: string;
    onSave?: (pipeline: Pipeline) => void;
    onCancel?: () => void;
  }

  let { pipeline, projectId, onSave, onCancel }: Props = $props();

  let name = $state(pipeline?.name || "");
  let description = $state(pipeline?.description || "");
  let steps = $state<PipelineStep[]>(pipeline?.steps || []);
  let executionContext = $state<ExecutionContext>(
    pipeline?.executionContext || {
      type: "sdk",
      sdkType: "node",
      workingDirectory: ".",
    },
  );
  let enabled = $state(pipeline?.enabled ?? true);
  let showBlockLibrary = $state(false);
  let selectedStep: PipelineStep | null = $state(null);
  let validationErrors: string[] = $state([]);

  async function addStep(blockId: string) {
    const block = await blockLibraryService.getBlock(blockId);

    if (!block) {
      const step: PipelineStep = {
        id: crypto.randomUUID(),
        blockId,
        name: `Step ${steps.length + 1}`,
        config: {},
        dependsOn: [],
      };
      steps = [...steps, step];
      selectedStep = step;
      validatePipeline();
      return;
    }

    const step = createStepFromBlock(block, { name: block.name });
    steps = [...steps, step];
    selectedStep = step;
    validatePipeline();
  }

  function removeStep(stepId: string) {
    steps = steps.filter((s) => s.id !== stepId);
    // Remove dependencies on this step
    steps = steps.map((s) => ({
      ...s,
      dependsOn: s.dependsOn?.filter((id) => id !== stepId) || [],
    }));
    selectedStep = null;
    validatePipeline();
  }

  function updateStep(stepId: string, updates: Partial<PipelineStep>) {
    steps = steps.map((s) => (s.id === stepId ? { ...s, ...updates } : s));
    validatePipeline();
  }

  function validatePipeline() {
    const validation = validateDependencies(steps);
    validationErrors = validation.errors;
  }

  async function handleSave() {
    validatePipeline();
    if (validationErrors.length > 0) {
      return;
    }

    try {
      if (pipeline?.id) {
        await pipelineService.updatePipeline(pipeline.id, {
          name,
          description,
          steps,
          executionContext,
          enabled,
        });
      } else {
        const newPipeline = await pipelineService.createPipeline({
          name,
          description,
          projectId,
          steps,
          variables: pipeline?.variables || [],
          secrets: pipeline?.secrets || [],
          executionContext,
          enabled,
        });
        onSave?.(newPipeline);
      }
    } catch (error) {
      console.error("Failed to save pipeline", error);
    }
  }

  function handleCancel() {
    onCancel?.();
  }

  onMount(() => {
    validatePipeline();
  });
</script>

<div class="pipeline-builder">
  <Card>
    <CardHeader>
      <CardTitle>{pipeline ? "Edit Pipeline" : "Create Pipeline"}</CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <!-- Pipeline Info -->
      <div class="space-y-2">
        <Label for="pipeline-name">Pipeline Name</Label>
        <Input id="pipeline-name" bind:value={name} placeholder="My Pipeline" />
      </div>

      <div class="space-y-2">
        <Label for="pipeline-description">Description</Label>
        <Textarea
          id="pipeline-description"
          bind:value={description}
          placeholder="Pipeline description"
        />
      </div>

      <!-- Execution Context -->
      <div class="space-y-2">
        <Label for="execution-context">Execution Context</Label>
        <div class="flex gap-2">
          <Select
            bind:value={executionContext.type}
            options={[
              { value: "sdk", label: "SDK" },
              { value: "docker", label: "Docker" },
            ]}
            class="min-w-[120px]"
          />
          {#if executionContext.type === "sdk"}
            <Input
              bind:value={executionContext.sdkType}
              placeholder="node"
              class="flex-1"
            />
            <Input
              bind:value={executionContext.sdkVersion}
              placeholder="version (optional)"
            />
          {:else}
            <Input
              bind:value={executionContext.dockerImage}
              placeholder="docker image"
              class="flex-1"
            />
          {/if}
          <Input
            bind:value={executionContext.workingDirectory}
            placeholder="working directory"
          />
        </div>
      </div>

      <!-- Steps -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <Label>Pipeline Steps</Label>
          <Button onclick={() => (showBlockLibrary = true)}>Add Step</Button>
        </div>

        {#if validationErrors.length > 0}
          <div
            class="rounded border border-red-200 bg-red-50 p-3 dark:border-red-800 dark:bg-red-900/20"
          >
            <p class="text-sm font-medium text-red-800 dark:text-red-200">
              Validation Errors:
            </p>
            <ul
              class="list-inside list-disc text-sm text-red-700 dark:text-red-300"
            >
              {#each validationErrors as error}
                <li>{error}</li>
              {/each}
            </ul>
          </div>
        {/if}

        <div class="space-y-2">
          {#each steps as step (step.id)}
            <div
              class="cursor-pointer rounded-md border p-3 hover:bg-accent {selectedStep?.id ===
              step.id
                ? 'border-primary'
                : ''}"
              onclick={() => (selectedStep = step)}
            >
              <div class="flex items-center justify-between">
                <div>
                  <p class="font-medium">{step.name}</p>
                  <p class="text-sm text-muted-foreground">
                    Block: {step.blockId}
                  </p>
                  {#if step.dependsOn && step.dependsOn.length > 0}
                    <p class="text-xs text-muted-foreground">
                      Depends on: {step.dependsOn.join(", ")}
                    </p>
                  {/if}
                </div>
                <Button
                  variant="destructive"
                  size="sm"
                  onclick={(e) => {
                    e.stopPropagation();
                    removeStep(step.id);
                  }}
                >
                  Remove
                </Button>
              </div>
            </div>
          {:else}
            <p class="text-sm text-muted-foreground text-center py-4">
              No steps yet. Click "Add Step" to get started.
            </p>
          {/each}
        </div>
      </div>

      <!-- Actions -->
      <div class="flex justify-end gap-2 border-t pt-4">
        <Button variant="outline" onclick={handleCancel}>Cancel</Button>
        <Button onclick={handleSave} disabled={validationErrors.length > 0}>
          Save Pipeline
        </Button>
      </div>
    </CardContent>
  </Card>

  {#if showBlockLibrary}
    <BlockLibrary
      onSelect={async (blockId) => {
        await addStep(blockId);
        showBlockLibrary = false;
      }}
      onClose={() => (showBlockLibrary = false)}
    />
  {/if}
</div>

<style>
  .pipeline-builder {
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
  }
</style>
