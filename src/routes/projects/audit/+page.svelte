<!--
	/projects/audit — bulk discovery and triage over the projects registry.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import ProjectAudit from '$lib/domains/projects/components/ProjectAudit.svelte';
  import { PageHeader } from '$lib/components/shell';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';

  onMount(() => {
    breadcrumbActions.setCustomBreadcrumbs([
      { label: 'Projects', href: '/projects' },
      { label: 'Audit', href: '/projects/audit' },
    ]);
  });
</script>

<div class="space-y-4 p-4">
  <PageHeader
    title="Project audit"
    description="Find every project on disk, see what is alive, and catch work that exists nowhere else."
  />
  <ProjectAudit />
</div>
