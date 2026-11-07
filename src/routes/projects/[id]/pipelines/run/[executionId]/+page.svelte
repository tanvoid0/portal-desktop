<!--
	Pipeline Run Page - deep link to one execution; the detail itself is PipelineRunDetail,
	which also renders in the /pipeline-runs rail.
-->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import PipelineRunDetail from '$lib/domains/projects/pipelines/components/PipelineRunDetail.svelte';
  import { createProjectQuery } from '$lib/domains/projects';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';

  const projectId = $derived(page.params.id);
  const executionId = $derived(page.params.executionId ?? '');
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

  function handleBack() {
    goto(`/projects/${projectId}?tab=actions`);
  }
</script>

<svelte:head>
  <title>Pipeline Run - {projectName || 'Project'} - Portal Desktop</title>
</svelte:head>

<PipelineRunDetail {executionId} onClose={handleBack} backLabel="Back to Actions" />
