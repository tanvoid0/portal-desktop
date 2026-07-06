<script lang="ts">
  import { onMount } from "svelte";
  import { automationStore } from "../stores/automationStore";
  import type { AvailableWorkflow, WorkflowResult } from "../types";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Alert, AlertDescription } from "$lib/components/ui/alert";
  import { Play, Loader2, CheckCircle, XCircle } from "@lucide/svelte";

  export let project: {
    id: string;
    name: string;
    path: string;
    metadata?: { framework?: string };
    framework_ids?: number[];
    package_manager_ids?: number[];
    build_command?: string;
    start_command?: string;
    test_command?: string;
    output_directory?: string;
    dev_port?: number;
    prod_port?: number;
  };
  export let onWorkflowComplete: (result: WorkflowResult) => void = () => {};

  let suggestedWorkflows: AvailableWorkflow[] = [];
  let selectedWorkflow: AvailableWorkflow | null = null;
  let isTriggering = false;
  let lastResult: WorkflowResult | null = null;

  $: if (project) {
    loadSuggestedWorkflows();
  }

  async function loadSuggestedWorkflows() {
    if (!project) return;

    await automationStore.getSuggestedWorkflows(
      project.metadata?.framework,
      undefined,
    );

    suggestedWorkflows = $automationStore.suggestedWorkflows;
  }

  async function triggerWorkflow(workflow: AvailableWorkflow) {
    if (!project || isTriggering) return;

    selectedWorkflow = workflow;
    isTriggering = true;
    lastResult = null;

    try {
      const result = await automationStore.triggerWorkflow(workflow.id, {
        id: project.id,
        name: project.name,
        path: project.path,
        framework: project.metadata?.framework,
        package_manager: undefined,
        build_command: project.build_command,
        start_command: project.start_command,
        test_command: project.test_command,
        output_directory: project.output_directory,
        dev_port: project.dev_port,
        prod_port: project.prod_port,
      });

      lastResult = result;
      onWorkflowComplete(result);
    } catch (error) {
      console.error("Failed to trigger workflow:", error);
    } finally {
      isTriggering = false;
    }
  }

  onMount(() => {
    automationStore.checkHealth();
  });
</script>

<div class="flex flex-col gap-4">
  <div class="space-y-1">
    <h3 class="text-lg font-semibold">Automate Project</h3>
    {#if !$automationStore.isN8nHealthy}
      <p class="text-sm text-destructive">
        n8n is not running. Start it with: npm run n8n:start
      </p>
    {/if}
  </div>

  {#if $automationStore.loading}
    <div class="flex items-center justify-center p-4">
      <Loader2 class="h-5 w-5 animate-spin text-primary" />
      <span class="ml-2 text-sm text-muted-foreground">Loading workflows...</span>
    </div>
  {:else if suggestedWorkflows.length === 0}
    <p class="p-4 text-center text-muted-foreground">
      No automation workflows available for this project type.
    </p>
  {:else}
    <div class="space-y-2">
      {#each suggestedWorkflows as workflow (workflow.id)}
        <Button
          variant="outline"
          class="flex h-auto w-full items-center justify-between p-3"
          onclick={() => triggerWorkflow(workflow)}
          disabled={isTriggering}
        >
          <div class="flex-1 text-left">
            <div class="font-medium">{workflow.name}</div>
            {#if workflow.description}
              <div class="text-sm text-muted-foreground">{workflow.description}</div>
            {/if}
          </div>

          <div class="flex items-center space-x-2">
            {#if isTriggering && selectedWorkflow?.id === workflow.id}
              <Loader2 class="h-4 w-4 animate-spin text-primary" />
            {:else if lastResult && selectedWorkflow?.id === workflow.id}
              {#if lastResult.success}
                <CheckCircle class="h-4 w-4 text-green-600" />
              {:else}
                <XCircle class="h-4 w-4 text-destructive" />
              {/if}
            {:else}
              <Play class="h-4 w-4 text-muted-foreground" />
            {/if}
          </div>
        </Button>
      {/each}
    </div>
  {/if}

  {#if $automationStore.error}
    <Alert variant="destructive">
      <AlertDescription>{$automationStore.error}</AlertDescription>
    </Alert>
  {/if}
</div>
