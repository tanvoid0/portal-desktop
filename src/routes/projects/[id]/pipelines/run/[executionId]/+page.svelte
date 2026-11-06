<!--
	Pipeline Run Page - Full-page execution monitor with Vercel-style logs
-->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import ExecutionMonitor from '$lib/domains/projects/pipelines/components/ExecutionMonitor.svelte';
  import { createProjectQuery } from '$lib/domains/projects';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';

  const projectId = $derived(page.params.id);
  const executionId = $derived(page.params.executionId);
  const projectQuery = createProjectQuery(() => projectId);

  const projectName = $derived(projectQuery.data?.name ?? '');

  $effect(() => {
    if (projectQuery.data) {
      breadcrumbActions.setProjectSubpageBreadcrumbs(
        projectQuery.data.name,
        projectQuery.data.id,
        'Pipeline Run'
      );
    }
    return () => breadcrumbActions.clearCustomBreadcrumbs();
  });

  // Opened from the global Action Runs list: return there with its filters
  const from = $derived(page.url.searchParams.get('from'));
  const backUrl = $derived.by(() => {
    if (!from) return null;
    const u = new URL(from, location.origin);
    return u.origin === location.origin && u.pathname === '/pipeline-runs'
      ? u.pathname + u.search
      : null;
  });
  const fromList = $derived(backUrl !== null);

  function handleBack() {
    goto(backUrl ?? `/projects/${projectId}?tab=actions`);
  }
</script>

<svelte:head>
  <title>Pipeline Run - {projectName || 'Project'} - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <Button variant="ghost" size="sm" onclick={handleBack} class="w-fit gap-2">
    <ArrowLeft class="h-4 w-4" />
    {fromList ? 'Back to Action Runs' : 'Back to Actions'}
  </Button>

  {#if executionId}
    <ExecutionMonitor {executionId} onClose={handleBack} />
  {:else}
    <p class="text-muted-foreground">Invalid execution ID</p>
  {/if}
</div>
