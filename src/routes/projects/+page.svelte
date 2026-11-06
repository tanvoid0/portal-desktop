<!--
	Projects page - compact rows grouped by area
-->

<script module lang="ts">
  // Rescanning every project is slow, so it runs once per app session in the background.
  let modulesRefreshed = false;
</script>

<script lang="ts">
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Activity from '~icons/fa6-solid/wave-square';
  import AlertTriangle from '~icons/fa6-solid/triangle-exclamation';
  import Star from '~icons/fa6-solid/star';
  import Plus from '~icons/fa6-solid/plus';
  import ScanSearch from '~icons/fa6-solid/magnifying-glass';
  import ProjectRow from '$lib/domains/projects/components/ProjectRow.svelte';
  import Select from '$lib/components/ui/select.svelte';
  import { projectService } from '$lib/domains/projects';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { createAllModulesQuery } from '$lib/domains/projects/queries/moduleQueries';
  import { refreshAllModules } from '$lib/domains/projects/api/moduleApi';
  import { groupByProject } from '$lib/domains/projects/utils/moduleStacks';
  import { nestProjects, descendantIds } from '$lib/domains/projects/utils/nestProjects';
  import { groupByArea } from '$lib/domains/projects/utils/groupProjects';
  import { queryClient } from '$lib/domains/shared/query';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import type { Project } from '$lib/domains/projects/types';
  import { toggleProjectStar } from '$lib/domains/projects/api/projectApi';
  import { toast } from '$lib/utils/toast';
  import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
  import { logger } from '$lib/domains/shared/services/logger';
  import { confirmAction } from '$lib/utils/confirm';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
    PageStats,
    PageFilters,
  } from '$lib/components/shell';
  import type { PageStat } from '$lib/components/shell';

  const log = logger.createScoped('ProjectsPage');

  const projectsQuery = createProjectsQuery();
  const modulesQuery = createAllModulesQuery();

  let searchQuery = $state('');
  let filterStack = $state<string | null>(null);
  let filterActivity = $state<string | null>(null);
  let starredOnly = $state(false);
  let sortBy = $state<'recent' | 'name'>('recent');
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
      // The scan also fills the stack of projects that had none (and may add new
      // framework rows), so reload the icon registry and refetch both.
      .then(() => projectIconRegistry.reload())
      .then(() =>
        Promise.all([
          queryClient.invalidateQueries({ queryKey: queryKeys.projects.modules }),
          queryClient.invalidateQueries({ queryKey: queryKeys.projects.all }),
        ])
      )
      .catch((error) => log.error('Failed to refresh modules', error));
  });

  const stackNames = (project: Project): string[] =>
    [
      ...projectIconRegistry.resolveFrameworks(project),
      ...projectIconRegistry.resolveLanguages(project),
    ].map((item) => item.name);

  const lastSeen = (p: Project): number => {
    const d = p.last_activity ?? p.last_opened;
    return d ? new Date(d).getTime() : 0;
  };

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
        if (filterStack && !stackNames(project).includes(filterStack)) return false;
        if (filterActivity && project.activity_status !== filterActivity) return false;
        if (starredOnly && !project.starred) return false;
        return true;
      })
      .sort((a, b) =>
        sortBy === 'name'
          ? a.name.localeCompare(b.name)
          : lastSeen(b) - lastSeen(a) || a.name.localeCompare(b.name)
      )
  );

  // Projects nested inside another (by path or manual grouping) sit under their
  // root row. Computed over the filtered set so a nested project still surfaces
  // as a root when its parent is filtered out.
  const groups = $derived(groupByArea(nestProjects(filteredProjects)));

  const stats = $derived<PageStat[]>([
    { label: 'Total', value: projects.length, icon: FolderOpen },
    {
      label: 'Active this week',
      value: projects.filter((p) => p.activity_status === 'active').length,
      icon: Activity,
    },
    {
      label: 'Need attention',
      value: projects.filter((p) => p.has_uncommitted_changes || (p.unpushed_commits ?? 0) > 0)
        .length,
      icon: AlertTriangle,
    },
    { label: 'Starred', value: projects.filter((p) => p.starred).length, icon: Star },
  ]);

  const isFiltered = $derived(
    Boolean(searchQuery.trim() || filterStack || filterActivity || starredOnly)
  );

  const uniqueStacks = $derived.by(() => {
    if (!registryReady) return [];
    const names = new Set<string>();
    for (const project of projects) for (const name of stackNames(project)) names.add(name);
    return [...names].sort((a, b) => a.localeCompare(b));
  });

  // Manual grouping: a project can be grouped under any project except itself
  // and its own descendants (which would make a cycle).
  function eligibleParents(project: Project): Array<{ id: string; name: string }> {
    const blocked = descendantIds(projects, project.id);
    blocked.add(project.id);
    return projects
      .filter((p) => !blocked.has(p.id))
      .map((p) => ({ id: p.id, name: p.name }))
      .sort((a, b) => a.name.localeCompare(b.name));
  }

  async function handleToggleStar(project: Project) {
    try {
      await toggleProjectStar(Number(project.id));
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    } catch (error) {
      log.error('Failed to toggle star', error);
      toast.error('Could not update the star');
    }
  }

  const handleSetParent = async (projectId: string, parentId: number | null) => {
    try {
      await projectService.setProjectParent(projectId, parentId);
    } catch (error) {
      log.error('Failed to set project parent', error);
    }
  };

  const handleProjectDelete = async (project: Project) => {
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

  <PageStats {stats} />

  <PageFilters bind:searchQuery searchPlaceholder="Search projects...">
    {#snippet filters()}
      <Select
        value={filterStack ?? ''}
        placeholder="All stacks"
        options={[
          { value: '', label: 'All stacks' },
          ...uniqueStacks.map((n) => ({ value: n, label: n })),
        ]}
        onSelect={(value) => (filterStack = value || null)}
        class="w-[170px]"
      />

      <Select
        value={filterActivity ?? ''}
        placeholder="Any activity"
        options={[
          { value: '', label: 'Any activity' },
          { value: 'active', label: 'Active' },
          { value: 'recent', label: 'Recent' },
          { value: 'dormant', label: 'Dormant' },
          { value: 'stale', label: 'Stale' },
        ]}
        onSelect={(value) => (filterActivity = value || null)}
        class="w-[150px]"
      />

      <Select
        value={sortBy}
        options={[
          { value: 'recent', label: 'Recently active' },
          { value: 'name', label: 'Name' },
        ]}
        onSelect={(value) => (sortBy = value as typeof sortBy)}
        class="w-[170px]"
      />

      <div class="flex items-center gap-2">
        <Switch id="starred-only" bind:checked={starredOnly} />
        <Label for="starred-only" class="text-sm">Starred only</Label>
      </div>
    {/snippet}

    {#snippet actions()}
      {#if isFiltered}
        <Button
          variant="ghost"
          size="sm"
          onclick={() => {
            searchQuery = '';
            filterStack = null;
            filterActivity = null;
            starredOnly = false;
          }}
          class="text-muted-foreground"
        >
          Clear filters
        </Button>
      {/if}
    {/snippet}
  </PageFilters>

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
    <div class="space-y-6">
      {#each groups as group (group.key)}
        <section>
          <h2 class="mb-1 flex items-baseline gap-2 border-b px-3 pb-1 text-sm font-semibold">
            {group.title}
            <span class="text-xs font-normal text-muted-foreground">{group.items.length}</span>
          </h2>
          {#each group.items as item (item.project.id)}
            <ProjectRow
              project={item.project}
              {registryReady}
              modules={modulesByProject.get(Number(item.project.id)) ?? []}
              eligibleParents={eligibleParents(item.project)}
              onDelete={handleProjectDelete}
              onSetParent={handleSetParent}
              onToggleStar={handleToggleStar}
            />
            {#each item.children as child (child.id)}
              <ProjectRow
                project={child}
                nested
                {registryReady}
                modules={modulesByProject.get(Number(child.id)) ?? []}
                eligibleParents={eligibleParents(child)}
                onDelete={handleProjectDelete}
                onSetParent={handleSetParent}
                onToggleStar={handleToggleStar}
              />
            {/each}
          {/each}
        </section>
      {/each}
    </div>
  {/if}
</div>
