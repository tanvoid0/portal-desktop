<!--
	/projects/audit — bulk discovery and triage over the projects registry.
	Two views: the scan/triage audit, and a one-card-at-a-time review of projects
	whose folder is gone. Both stay mounted so a running scan survives a tab switch.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import * as Tabs from '$lib/components/ui/tabs';
  import ProjectAudit from '$lib/domains/projects/components/ProjectAudit.svelte';
  import AiTidy from '$lib/domains/projects/components/AiTidy.svelte';
  import UnavailableProjects from '$lib/domains/projects/components/UnavailableProjects.svelte';
  import { createUnavailableProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { PageHeader } from '$lib/components/shell';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';

  const unavailable = createUnavailableProjectsQuery();
  const missing = $derived(unavailable.data?.length ?? 0);

  let view = $state<'audit' | 'relink' | 'tidy'>('audit');
  // Nothing left to relink: don't leave the user on an empty tab.
  $effect(() => {
    if (view === 'relink' && unavailable.isSuccess && missing === 0) view = 'audit';
  });

  onMount(() => {
    breadcrumbActions.setCustomBreadcrumbs([
      { label: 'Projects', href: '/projects' },
      { label: 'Audit', href: '/projects/audit' },
    ]);
  });
</script>

<!-- No padding here: the shell's PageContainer already sets the content gutter,
     and /projects sits at that same inset. -->
<div class="space-y-6">
  <PageHeader
    title="Project audit"
    description="Find every project on disk, see what is alive, and catch work that exists nowhere else."
  />
  <Tabs.Root value={view} onValueChange={(v) => (view = v as typeof view)}>
    <Tabs.List>
      <Tabs.Trigger value="audit">Audit</Tabs.Trigger>
      {#if missing}
        <Tabs.Trigger value="relink">Unavailable ({missing})</Tabs.Trigger>
      {/if}
      <Tabs.Trigger value="tidy">AI tidy</Tabs.Trigger>
    </Tabs.List>
  </Tabs.Root>
  <div class:hidden={view !== 'audit'}><ProjectAudit /></div>
  <!-- One-column review views: on a wide window a full-width card strands its
       buttons a screen away from the text they act on. -->
  <div class="max-w-3xl" class:hidden={view !== 'relink'}>
    <UnavailableProjects active={view === 'relink'} />
  </div>
  <div class="max-w-3xl" class:hidden={view !== 'tidy'}><AiTidy /></div>
</div>
