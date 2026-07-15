<!--
	Pipeline Run Page - Full-page execution monitor with Vercel-style logs
-->
<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { ArrowLeft } from "@lucide/svelte";
  import ExecutionMonitor from "$lib/domains/projects/pipelines/components/ExecutionMonitor.svelte";
  import { createProjectQuery } from "$lib/domains/projects";
  import { breadcrumbActions } from "$lib/domains/shared/stores/breadcrumbStore";

  const projectId = $derived($page.params.id);
  const executionId = $derived($page.params.executionId);
  const projectQuery = createProjectQuery(() => projectId);

  const projectName = $derived(projectQuery.data?.name ?? "");

  $effect(() => {
    if (projectQuery.data) {
      breadcrumbActions.setProjectDetailsBreadcrumbs(projectQuery.data.name);
    }
  });

  function handleBack() {
    goto(`/projects/${projectId}?tab=actions`);
  }
</script>

<svelte:head>
  <title>Pipeline Run - {projectName || "Project"} - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto flex h-[calc(100vh-4rem)] flex-col space-y-4 p-6">
  <Button variant="ghost" size="sm" onclick={handleBack} class="w-fit gap-2">
    <ArrowLeft class="h-4 w-4" />
    Back to Actions
  </Button>

  {#if executionId}
    <ExecutionMonitor {executionId} onClose={handleBack} />
  {:else}
    <p class="text-muted-foreground">Invalid execution ID</p>
  {/if}
</div>
