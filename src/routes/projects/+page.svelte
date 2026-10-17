<!--
	Projects page - Project management interface
	Showcases the projects domain with full CRUD operations
-->

<script module lang="ts">
  // Rescanning every project is slow, so it runs once per app session in the background.
  let modulesRefreshed = false;
</script>

<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { FolderOpen, CheckCircle, Database, Code } from '@lucide/svelte';
  import ProjectCard from '$lib/domains/projects/components/ProjectCard.svelte';
  import Select from '$lib/components/ui/select.svelte';
  import { projectService } from '$lib/domains/projects';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { createAllModulesQuery } from '$lib/domains/projects/queries/moduleQueries';
  import { refreshAllModules } from '$lib/domains/projects/api/moduleApi';
  import { groupByProject } from '$lib/domains/projects/utils/moduleStacks';
  import { queryClient } from '$lib/domains/shared/query';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import type { Project } from '$lib/domains/projects/types';
  import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
  import { logger } from '$lib/domains/shared/services/logger';
  import { confirmAction } from '$lib/utils/confirm';
  import { formatBytes } from '$lib/domains/shared/utils';
  import { getProjectFramework } from '$lib/domains/projects/utils/display';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';
  import { WorkflowTrigger, WorkflowResults } from '$lib/domains/automation';
  import type { WorkflowResult } from '$lib/domains/automation/types';
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
    PageStats,
    PageFilters,
  } from '$lib/components/shell';
  import type { PageStat } from '$lib/components/shell';
  import { Plus, ScanSearch, ArrowUp } from '@lucide/svelte';

  const log = logger.createScoped('ProjectsPage');

  const projectsQuery = createProjectsQuery();
  const modulesQuery = createAllModulesQuery();

  let searchQuery = $state('');
  let selectedProject = $state<any>(null);
  let showAutomation = $state(false);
  let workflowResult = $state<WorkflowResult | null>(null);

  // Filter state
  let filterFramework = $state<string | null>(null);
  let filterStatus = $state<string | null>(null);
  let filterPackageManager = $state<string | null>(null);

  // Sort state
  type SortOption = 'name' | 'last_opened' | 'size' | 'created_at';
  let sortBy = $state<SortOption>('name');
  let sortDirection = $state<'asc' | 'desc'>('asc');
  let registryReady = $state(false);

  const projects = $derived(projectsQuery.data ?? []);
  const modulesByProject = $derived(groupByProject(modulesQuery.data ?? []));
  const isLoading = $derived(projectsQuery.isPending);
  const pageError = $derived(projectsQuery.isError ? 'Failed to load projects' : null);

  $effect(() => {
    if (projectsQuery.isSuccess) {
      void projectIconRegistry.ensureLoaded().then(() => {
        registryReady = true;
      });
      breadcrumbActions.setProjectBreadcrumbs();
    }
  });

  $effect(() => {
    if (modulesRefreshed) return;
    modulesRefreshed = true;
    refreshAllModules()
      .then(() => queryClient.invalidateQueries({ queryKey: queryKeys.projects.modules }))
      .catch((error) => log.error('Failed to refresh modules', error));
  });

  function projectHasFramework(project: Project, framework: string): boolean {
    return projectIconRegistry.resolveFrameworks(project).some((item) => item.name === framework);
  }

  function projectHasPackageManager(project: Project, packageManager: string): boolean {
    return projectIconRegistry
      .resolvePackageManagers(project)
      .some((item) => item.name === packageManager);
  }

  const filteredProjects = $derived(
    [...projects]
      .filter((project) => {
        if (searchQuery.trim()) {
          const query = searchQuery.toLowerCase();
          if (
            !project.name.toLowerCase().includes(query) &&
            !project.description?.toLowerCase().includes(query) &&
            !project.path.toLowerCase().includes(query)
          ) {
            return false;
          }
        }
        if (filterFramework && !projectHasFramework(project, filterFramework)) {
          return false;
        }
        if (filterStatus && project.status !== filterStatus) {
          return false;
        }
        if (filterPackageManager && !projectHasPackageManager(project, filterPackageManager)) {
          return false;
        }
        return true;
      })
      .sort((a, b) => {
        let comparison = 0;

        switch (sortBy) {
          case 'name':
            comparison = a.name.localeCompare(b.name);
            break;
          case 'last_opened':
            comparison =
              (a.last_opened ? new Date(a.last_opened).getTime() : 0) -
              (b.last_opened ? new Date(b.last_opened).getTime() : 0);
            break;
          case 'size':
            comparison = a.size - b.size;
            break;
          case 'created_at':
            comparison =
              (a.created_at ? new Date(a.created_at).getTime() : 0) -
              (b.created_at ? new Date(b.created_at).getTime() : 0);
            break;
        }

        return sortDirection === 'asc' ? comparison : -comparison;
      })
  );

  const stats = $derived<PageStat[]>([
    { label: 'Total Projects', value: projects.length, icon: FolderOpen },
    {
      label: 'Active Projects',
      value: projects.filter((p) => p.status === 'active').length,
      icon: CheckCircle,
    },
    {
      label: 'Total Size',
      value: formatBytes(projects.reduce((sum, p) => sum + p.size, 0)),
      icon: Database,
    },
    {
      label: 'Most Used Type',
      value:
        Object.entries(
          projects.reduce(
            (acc, p) => {
              const framework = getProjectFramework(p) || 'Unknown';
              acc[framework] = (acc[framework] || 0) + 1;
              return acc;
            },
            {} as Record<string, number>
          )
        ).sort(([, a], [, b]) => b - a)[0]?.[0] || 'none',
      icon: Code,
    },
  ]);

  const isFiltered = $derived(
    Boolean(searchQuery.trim() || filterFramework || filterStatus || filterPackageManager)
  );

  const uniqueFrameworks = $derived.by(() => {
    if (!registryReady) return [];

    const names = new Set<string>();
    for (const project of projects) {
      for (const framework of projectIconRegistry.resolveFrameworks(project)) {
        names.add(framework.name);
      }
    }
    return [...names].sort((a, b) => a.localeCompare(b));
  });

  const uniquePackageManagers = $derived.by(() => {
    if (!registryReady) return [];

    const names = new Set<string>();
    for (const project of projects) {
      for (const manager of projectIconRegistry.resolvePackageManagers(project)) {
        names.add(manager.name);
      }
    }
    return [...names].sort((a, b) => a.localeCompare(b));
  });

  // Handle project click - navigate to details page
  const handleProjectClick = (project: any) => {
    goto(`/projects/${project.id}`);
  };

  // Handle automation
  function handleAutomationClick(project: any, event: Event) {
    event.stopPropagation();
    selectedProject = project;
    showAutomation = true;
    workflowResult = null;
  }

  function handleWorkflowComplete(result: WorkflowResult) {
    workflowResult = result;
    log.info('Workflow completed', { result });
  }

  function closeAutomation() {
    showAutomation = false;
    selectedProject = null;
    workflowResult = null;
  }

  // Handle project delete
  const handleProjectDelete = async (project: any) => {
    const confirmed = await confirmAction(`Are you sure you want to delete "${project.name}"?`);
    if (confirmed) {
      try {
        await projectService.deleteProject(project.id);
        log.info('Project deleted', { id: project.id, name: project.name });
      } catch (error) {
        log.error('Failed to delete project', error);
      }
    }
  };
</script>

<svelte:head>
  <title>Projects - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <PageHeader title="Projects" description="Manage your development projects">
    {#snippet actions()}
      <Button variant="outline" onclick={() => goto('/projects/audit')}>
        <ScanSearch class="mr-2 h-4 w-4" />
        Audit
      </Button>
      <Button onclick={() => goto('/projects/create')}>
        <Plus class="mr-2 h-4 w-4" />
        New Project
      </Button>
    {/snippet}
  </PageHeader>

  <!-- Stats Cards -->
  <PageStats {stats} />

  <!-- Filters -->
  <PageFilters bind:searchQuery searchPlaceholder="Search projects...">
    {#snippet filters()}
      <Select
        defaultValue={filterFramework || ''}
        placeholder="All Frameworks"
        options={uniqueFrameworks.map((f) => ({ value: f, label: f }))}
        onSelect={(value) => (filterFramework = value || null)}
        class="min-w-[150px]"
      />

      <Select
        defaultValue={filterStatus || ''}
        placeholder="All Statuses"
        options={[
          { value: 'active', label: 'Active' },
          { value: 'archived', label: 'Archived' },
          { value: 'deleted', label: 'Deleted' },
        ]}
        onSelect={(value) => (filterStatus = value || null)}
        class="min-w-[130px]"
      />

      <Select
        defaultValue={filterPackageManager || ''}
        placeholder="All Package Managers"
        options={uniquePackageManagers.map((m) => ({ value: m, label: m }))}
        onSelect={(value) => (filterPackageManager = value || null)}
        class="min-w-[180px]"
      />

      <Select
        defaultValue={sortBy}
        placeholder="Sort by"
        options={[
          { value: 'name', label: 'Name' },
          { value: 'last_opened', label: 'Last Opened' },
          { value: 'size', label: 'Size' },
          { value: 'created_at', label: 'Created Date' },
        ]}
        onSelect={(value) => (sortBy = value as SortOption)}
        class="min-w-[140px]"
      />
    {/snippet}

    {#snippet actions()}
      <Button
        variant="outline"
        size="sm"
        onclick={() => (sortDirection = sortDirection === 'asc' ? 'desc' : 'asc')}
        class="flex items-center gap-1"
        title={`Sort ${sortDirection === 'asc' ? 'Ascending' : 'Descending'}`}
      >
        <ArrowUp class="h-4 w-4 {sortDirection === 'desc' ? 'rotate-180' : ''}" />
        {sortDirection === 'asc' ? 'Asc' : 'Desc'}
      </Button>

      <!-- Search counts as a filter: clearing everything but the search box left
           the empty state telling you to adjust criteria you could not reach. -->
      {#if isFiltered}
        <Button
          variant="ghost"
          size="sm"
          onclick={() => {
            searchQuery = '';
            filterFramework = null;
            filterStatus = null;
            filterPackageManager = null;
          }}
          class="text-muted-foreground"
        >
          Clear filters
        </Button>
      {/if}
    {/snippet}
  </PageFilters>

  <!-- Projects Grid -->
  {#if isLoading}
    <PageLoading message="Loading projects..." />
  {:else if pageError}
    <PageError
      title="Failed to Load Projects"
      message={pageError}
      onRetry={() => projectsQuery.refetch()}
    />
  {:else if filteredProjects.length === 0}
    <PageEmpty
      title="No projects found"
      description="Create your first project to get started"
      filteredDescription="Try adjusting your search criteria"
      {isFiltered}
      icon={FolderOpen}
      actionLabel="Create Project"
      onAction={() => goto('/projects/create')}
    />
  {:else}
    <div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
      {#each filteredProjects as project (project.id)}
        <ProjectCard
          {project}
          modules={modulesByProject.get(Number(project.id)) ?? []}
          onClick={(p) => handleProjectClick(p)}
          onEdit={(p) => goto(`/projects/${p.id}`)}
          onDelete={handleProjectDelete}
          showActions={true}
        />
      {/each}
    </div>
  {/if}

  <!-- Automation Modal -->
  <Dialog.Root
    open={showAutomation && !!selectedProject}
    onOpenChange={(isOpen) => {
      if (!isOpen) closeAutomation();
    }}
  >
    <Dialog.Content class="max-h-[90vh] max-w-2xl overflow-y-auto">
      <Dialog.Header>
        <Dialog.Title>Automate: {selectedProject?.name}</Dialog.Title>
      </Dialog.Header>

      {#if workflowResult}
        <WorkflowResults result={workflowResult} />
      {:else if selectedProject}
        <WorkflowTrigger project={selectedProject} onWorkflowComplete={handleWorkflowComplete} />
      {/if}
    </Dialog.Content>
  </Dialog.Root>
</div>
